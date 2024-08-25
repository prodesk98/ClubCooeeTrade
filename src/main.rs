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
mod peers;
mod license;
mod logger;
mod telegram;
use mongodb::Client;
use dotenvy::dotenv;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use colored::Colorize;
use tokio::spawn;
use tokio::sync::RwLock;
use crate::cache::ItemCache;
use crate::database::Connection;
use crate::settings::{load_accounts, load_config, load_servers, load_tokens, migration};
use crate::schemas::{ConfigAccount, Proxy};
use crate::telegram::Telegram;

async fn start(
    ip: String,
    token: String,
    account: ConfigAccount,
    telegram: Telegram,
    hostname: String,
    proxy: Proxy,
    connection: Arc<RwLock<Connection>>,
    cache: Arc<RwLock<ItemCache>>
) -> Result<(), Box<dyn std::error::Error>> {
    // start bot session
    let session = bot::Bot::new(
        socket::Socket::new(
            hostname,
            ip,
            token,
            443,
            proxy
        ),
        account,
        telegram,
        connection,
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
    let host = hostname::get();
    let hostname = host.unwrap().to_string_lossy().to_string();
    license::check(hostname.as_str()).await;

    // MongoDB connection
    let mongo_dsn = "mongodb://mtz:7b36a5a4350a563bf6ca158386ebe3e7@localhost:27012";
    let mongo_client = Client::with_uri_str(mongo_dsn).await.unwrap();

    match mongo_client.database("cooee_bot").list_collection_names().await {
        Ok(_) => eprintln!("{} MongoDB connected", "[*]".white().bold()),
        Err(e) => {
            eprintln!("{} MongoDB connection error: {:?}", "[-]".red().bold(), e);
            return;
        }
    }

    // Proxy
    let proxy = parse::proxy(&env::var("PROXY").unwrap());

    // migration
    migration(&mongo_client, "items").await;
    migration(&mongo_client, "servers").await;
    migration(&mongo_client, "tokens").await;
    migration(&mongo_client, "config").await;
    migration(&mongo_client, "accounts").await;

    // Load configurations
    let tokens = load_tokens(&mongo_client).await;
    let servers = load_servers(&mongo_client).await;
    let config = load_config(&mongo_client).await;
    let accounts = load_accounts(&mongo_client).await;

    // Algorithm Round Robin
    let rr_tokens = round_robin::RoundRobin::new(tokens);
    let rr_servers = round_robin::RoundRobin::new(servers);
    let rr_accounts = round_robin::RoundRobin::new(accounts);

    // Connection items
    let connection = Arc::new(
        RwLock::new(
            Connection::new(
                mongo_client
                    .database("cooee_bot")
                    .collection("items")
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

    loop {
        // clone variables
        let hostname = config.hostname.clone();
        let ip_clone = rr_servers.next().await.clone().unwrap();
        let token_clone = rr_tokens.next().await.clone().unwrap();
        let account_clone = rr_accounts.next().await.clone().unwrap();
        let telegram_clone = telegram.clone();
        let proxy_clone = proxy.clone();
        let connection_clone = Arc::clone(&connection);
        let cache_clone = Arc::clone(&cache);
        //

        spawn(async move {
            let result = tokio::time::timeout(
                Duration::from_millis(1700),
                start(
                    ip_clone,
                    token_clone,
                    account_clone,
                    telegram_clone,
                    hostname,
                    proxy_clone,
                    connection_clone,
                    cache_clone,
                ),
            ).await;

            // Handle the result of `start`
            if let Err(e) = result {
                eprintln!("{} {}", "[-]".red().bold(), e);
            }
        });

        // sleep 200 milliseconds
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}
