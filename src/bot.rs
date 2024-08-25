use std::sync::Arc;
use std::time::Duration;
use colored::Colorize;
use futures::future::join_all;
use mongodb::bson::{doc, Bson};
use tokio::sync::RwLock;
use tokio::time::timeout;
use crate::cache::ItemCache;
use crate::database::Connection;
use crate::market::Market;
use crate::schemas::{ConfigAccount, Item};
use crate::socket::Socket;
use crate::telegram::Telegram;
use crate::trade::Trade;

#[derive(Clone)]
pub struct Bot {
    traders: Arc<RwLock<Connection>>,
    sold: Arc<RwLock<Connection>>,
    cache: Arc<RwLock<ItemCache>>,
    market: Market,
    telegram: Telegram,
}

impl Bot {
    pub fn new(
        socket: Socket,
        seller: ConfigAccount,
        buyer: ConfigAccount,
        telegram: Telegram,
        traders: Arc<RwLock<Connection>>,
        sold: Arc<RwLock<Connection>>,
        cache: Arc<RwLock<ItemCache>>,
    ) -> Self {
        Self {
            traders,
            sold,
            cache,
            market: Market::new(
                socket,
                seller,
                buyer,
                telegram.clone(),
            ),
            telegram,
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        // time start
        let _s = std::time::Instant::now();

        let items = match self.market.fetch().await {
            Ok(items) => items,
            Err(e) => {
                eprintln!("{} Error: {:?}", "[-]".red().bold(), e);
                return Err(e);
            }
        };

        eprintln!("{} found {} items... {:?}", "[*]".blue().bold(), items.len(), _s.elapsed());

        let market = self.market.clone();
        let items_solid = match timeout(
            Duration::from_secs(10),
            market.sold()
        ).await {
            Ok(sold) => sold?,
            Err(_) => {
                return Err("Failed to fetch sold items".into());
            }
        };
        let sold = Arc::clone(&self.sold);
        let sold_guard = sold.write().await;
        for it in items_solid {
            let it_clone = it.clone();
            if sold_guard.read(doc! {"id": it.id}).await?.len() == 0 {
                sold_guard.create(
                    doc! {
                        "id": it.id,
                        "itemt": it.itemt,
                        "name": String::from_utf8_lossy(it.name.as_bytes()).to_string(),
                        "price": it.price,
                        "created_at": Bson::String(chrono::Utc::now().to_rfc3339()),
                    }
                ).await?;
                self.telegram.send_image(it.image.to_string().replace("\\/", "/"), &format!(
                    "🔄 {} ({})\nid: {}, price: {}cc",
                    String::from_utf8_lossy(it.name.as_bytes()).to_string(), it_clone.itemt, it_clone.id, it_clone.price
                )).await?;
            }
        }
        drop(sold_guard);

        let mut tasks = Vec::new();
        for item in items {
            let cache = Arc::clone(&self.cache);
            let traders = Arc::clone(&self.traders);
            let sold = Arc::clone(&self.sold);
            let market = self.market.clone();

            let task = tokio::spawn(async move {
                match timeout(
                    Duration::from_secs(15),
                    Self::verify(item, cache, traders, sold, market)).await {
                        Ok(_) => {},
                        Err(e) => eprintln!("{} Error: {:?}", "[-]".red().bold(), e),
                    }
                }
            );
            tasks.push(task);
        }
        join_all(tasks).await;
        Ok(())
    }

    async fn verify(
        item: Item,
        cache: Arc<RwLock<ItemCache>>,
        traders: Arc<RwLock<Connection>>,
        sold: Arc<RwLock<Connection>>,
        market: Market
    ) -> Result<(), Box<dyn std::error::Error>> {
        // time start
        let _s = std::time::Instant::now();

        // cache guard read
        let cache_guard_read = cache.write().await;

        let item_id = item.id.to_string();
        let item_template = item.itemt.clone();

        // check if item is already cached
        if cache_guard_read.contains(&item_id) {
            return Err("Item already cached".into());
        }
        drop(cache_guard_read);

        let sold_guard = sold.write().await;
        let item_sold = sold_guard.read(doc! {"itemt": item_template}).await?;
        drop(sold_guard);

        if item_sold.len() < 5 {
            return Err("Item not qualified. min 5 items".into());
        }
        // select prices history
        let history = item_sold.iter().map(|it| it.get_i32("price").unwrap_or(0)).collect::<Vec<i32>>();
        // i32 to f64
        let history = history.iter().map(|&x| x as f64).collect::<Vec<f64>>();

        // trade verification
        let mut trade = Trade::new(history, item.price as f64);
        let qualified = trade.strategy(10.0);

        if !qualified {
            cache.write().await.insert(item.id.to_string());
            return Err("Item not qualified".into());
        }

        let resale = trade.resale(10.0);
        let item_clone = item.clone();
        let item_id = item_clone.id.clone().to_string();
        let item_template = item_clone.itemt.clone().to_string();

        match market.buy(item_clone).await {
            Ok(_) => {
                eprintln!("{} Bought item: {:?} {:?}... {:?}", "[+]".green().bold(),
                          item_id, item_template, _s.elapsed());
            },
            Err(e) => {
                eprintln!("{} Error: {:?} {:?} {:?}...", "[-]".red().bold(),
                          e, item_id, item_template);
                return Err(e);
            }
        }

        // connection guard
        let traders_guard = traders.write().await;

        let item = item.clone();
        traders_guard.create(
            doc! {
                    "id": item.id,
                    "price": item.price.to_string(),
                    "resale": resale.to_string(),
                    "created_at": Bson::String(chrono::Utc::now().to_rfc3339()),
                }
        ).await?;
        drop(traders_guard);

        match market.sell(item.id, resale).await {
            Ok(_) => {
                eprintln!("{} Sold item: {:?}... {:?}", "[+]".green().bold(), item_id, _s.elapsed());
            },
            Err(e) => {
                eprintln!("{} Error: {:?}", "[-]".red().bold(), e);
                return Err(e);
            }
        };

        Ok(())
    }
}