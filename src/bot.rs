use std::sync::Arc;
use std::time::Duration;
use colored::Colorize;
use futures::future::join_all;
use tokio::sync::RwLock;
use tokio::time::timeout;
use crate::cache::ItemCache;
use crate::database::Connection;
use crate::market::Market;
use crate::schemas::{ConfigAccount, Item};
use crate::socket::Socket;
use crate::telegram::Telegram;

#[derive(Clone)]
pub struct Bot {
    connection: Arc<RwLock<Connection>>,
    cache: Arc<RwLock<ItemCache>>,
    market: Market,
}

impl Bot {
    pub fn new(
        socket: Socket,
        account: ConfigAccount,
        telegram: Telegram,
        connection: Arc<RwLock<Connection>>,
        cache: Arc<RwLock<ItemCache>>,
    ) -> Self {
        Self {
            connection,
            cache,
            market: Market::new(
                socket,
                account,
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
                    Duration::from_secs(3),
                    Self::verify(item, connection, cache, market)).await {
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
        item: Item, connection: Arc<RwLock<Connection>>,
        cache: Arc<RwLock<ItemCache>>,
        market: Market
    ) -> Result<(), Box<dyn std::error::Error>> {
        // time start
        let _s = std::time::Instant::now();

        // connection guard
        let connection_guard = connection.write().await;

        // cache guard
        let mut cache_guard = cache.write().await;

        // check if item is already cached
        if cache_guard.contains(&item.id.to_string()) {
            return Err("Item already cached".into());
        }

        // check if item is qualified
        let filter = mongodb::bson::doc! {
            "idtemplate": item.itemt,
            "price": {
                "$gte": item.price
            }
        };

        let qualified = connection_guard.read(filter).await.unwrap();
        eprintln!("{} Verifying item: i:{} t:{}... {:?}", "[?]".white().bold(), item.id, item.itemt, _s.elapsed());
        if qualified.len() == 0 {
            cache_guard.insert(item.id.to_string());
            return Err("Item not qualified".into());
        }

        eprintln!("{} Qualified item: {:?}... {}|{}cc",
                  "[!]".green().bold(), item.id, item.itemt, item.price);

        let item_id = item.id.to_string();
        // sent buy request
        match market.buy(item).await {
            Ok(_) => {
                eprintln!("{} Bought item: {:?}... {:?}", "[+]".green().bold(), item_id, _s.elapsed());
            },
            Err(e) => {
                eprintln!("{} Error: {:?}", "[-]".red().bold(), e);
                cache_guard.insert(item_id);
                return Err(e);
            }
        }
        Ok(())
    }
}