mod database;
mod round_robin;
mod cache;
mod settings;
mod schemas;
mod bot;
mod sessions;
mod socket;
mod parse;
mod market;
mod license;
mod logger;
mod telegram;
mod trade;
mod proxies;

use mongodb::Client;
use dotenvy::dotenv;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use colored::Colorize;
use rand::prelude::SliceRandom;
use tokio::sync::RwLock;
use crate::cache::ItemCache;
use crate::database::Connection;
use crate::settings::{load_accounts, load_config, load_servers, load_tokens, migration};
use crate::schemas::{ConfigAccount, Proxy};
use crate::telegram::Telegram;

async fn start(
    ip: String,
    token: String,
    seller: ConfigAccount,
    buyer: ConfigAccount,
    telegram: Telegram,
    hostname: String,
    proxy: Proxy,
    traders: Arc<RwLock<Connection>>,
    sold: Arc<RwLock<Connection>>,
    cache: Arc<RwLock<ItemCache>>
) -> Result<(), Box<dyn std::error::Error>> {
    // create a new session
    // socket
    let socket = socket::Socket::new(
        hostname,
        ip,
        token,
        443,
        proxy
    );
    // session
    let session = bot::Bot::new(
        socket,
        seller,
        buyer,
        telegram,
        traders,
        sold,
        cache,
    );
    // start checking
    session.start().await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    // license
    // TODO: Uncomment this code to enable license checking
    // let host = hostname::get();
    // let hostname = host.unwrap().to_string_lossy().to_string();
    // license::check(hostname.as_str()).await;

    // MongoDB connection
    let mongo_dsn = "mongodb://protons:c2f193e26f960f1b3649cbd3e31d5255@localhost:27013";
    let mongo_client = Client::with_uri_str(mongo_dsn).await.unwrap();

    match mongo_client.database("clubcooee_trade").list_collection_names().await {
        Ok(_) => eprintln!("{} MongoDB connected", "[*]".white().bold()),
        Err(e) => {
            eprintln!("{} MongoDB connection error: {:?}", "[-]".red().bold(), e);
            return;
        }
    }

    // migration
    migration(&mongo_client, "servers").await;
    migration(&mongo_client, "tokens").await;
    migration(&mongo_client, "config").await;
    migration(&mongo_client, "accounts").await;

    // Load configurations
    let servers = load_servers(&mongo_client).await;
    let config = load_config(&mongo_client).await;
    let accounts = load_accounts(&mongo_client).await;

    // random tokens
    let tokens = load_tokens(&mongo_client).await;
    let mut tokens_random = tokens.clone();
    tokens_random.shuffle(&mut rand::thread_rng());

    // Algorithm Round Robin
    let rr_tokens = round_robin::RoundRobin::new(tokens_random);
    let rr_servers = round_robin::RoundRobin::new(servers);
    let rr_accounts_seller = round_robin::RoundRobin::new(
        accounts
            .iter()
            .filter(|x| x.role == "seller")
            .collect()
    );
    let rr_accounts_buyer = round_robin::RoundRobin::new(
        accounts
            .iter()
            .filter(|x| x.role == "buyer")
            .collect()
    );

    // Connection trades
    let connection_trades = Arc::new(
        RwLock::new(
            Connection::new(
                mongo_client
                    .database("clubcooee_trade")
                    .collection("trades")
            )
        )
    );

    // Connection sold
    let connection_sold = Arc::new(
        RwLock::new(
            Connection::new(
                mongo_client
                    .database("clubcooee_trade")
                    .collection("sold")
            )
        )
    );

    // Cache
    let cache = Arc::new(RwLock::new(ItemCache::new(500)));

    // Telegram
    let telegram = Telegram::new(
        env::var("BOT_TELEGRAM_ID").unwrap(),
        env::var("BOT_TELEGRAM_CHAT_ID").unwrap()
    );

    // Proxy Manager
    let mut proxy_manager = proxies::ProxyManager::new(
        Arc::new(
            RwLock::new(
                rr_tokens.clone()
            )
        ),
        Arc::new(
            RwLock::new(
                rr_servers.clone()
            )
        )
    )
    .await;
    proxy_manager.load().await;

    loop {
        // clone variables
        let hostname = config.hostname.clone();
        let ip_clone = rr_servers.next().await.clone().unwrap();
        let token_clone = rr_tokens.next().await.clone().unwrap();
        let account_seller_clone = rr_accounts_seller.next().await.clone().unwrap().clone();
        let account_buyer_clone = rr_accounts_buyer.next().await.clone().unwrap().clone();
        let telegram_clone = telegram.clone();
        let proxy = proxy_manager.next().await;
        let connection_trades_clone = Arc::clone(&connection_trades);
        let connection_sold_clone = Arc::clone(&connection_sold);
        let cache_clone = Arc::clone(&cache);
        //

        match start(
            ip_clone,
            token_clone,
            account_seller_clone,
            account_buyer_clone,
            telegram_clone,
            hostname,
            proxy,
            connection_trades_clone,
            connection_sold_clone,
            cache_clone,
        ).await {
            Ok(_) => {},
            Err(e) => eprintln!("{} session error: {:?}", "[-]".red().bold(), e),
        };

        // sleep 60 seconds
        tokio::time::sleep(Duration::from_secs(60)).await;
        proxy_manager.load().await;
    }
}
