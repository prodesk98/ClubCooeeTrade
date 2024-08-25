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
    connection: Arc<RwLock<Connection>>,
    cache: Arc<RwLock<ItemCache>>,
    market: Market,
}

impl Bot {
    pub fn new(
        socket: Socket,
        seller: ConfigAccount,
        buyer: ConfigAccount,
        telegram: Telegram,
        connection: Arc<RwLock<Connection>>,
        cache: Arc<RwLock<ItemCache>>,
    ) -> Self {
        Self {
            connection,
            cache,
            market: Market::new(
                socket,
                seller,
                buyer,
                telegram
            ),
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

        let mut tasks = Vec::new();
        for item in items {
            let cache = Arc::clone(&self.cache);
            let connection = Arc::clone(&self.connection);
            let market = self.market.clone();

            let task = tokio::spawn(async move {
                match timeout(
                    Duration::from_secs(5),
                    Self::verify(item, cache, connection, market)).await {
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
        connection: Arc<RwLock<Connection>>,
        market: Market
    ) -> Result<(), Box<dyn std::error::Error>> {
        // time start
        let _s = std::time::Instant::now();

        // cache guard read
        let cache_guard_read = cache.write().await;

        // check if item is already cached
        if cache_guard_read.contains(&item.id.to_string()) {
            return Err("Item already cached".into());
        }
        drop(cache_guard_read);

        let search = market.search(item.itemt).await?;
        let mut trade = Trade::new(search.history, item.price as f64);
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
        let connection_guard = connection.write().await;

        let item = item.clone();
        connection_guard.create(
            doc! {
                    "id": item.id,
                    "price": item.price.to_string(),
                    "resale": resale.to_string(),
                    "timestamp": Bson::String(chrono::Utc::now().to_rfc3339()),
                }
        ).await?;
        drop(connection_guard);

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