use colored::Colorize;
use mongodb::bson::doc;
use mongodb::Client;
use tokio::fs::File;
use tokio::io::{AsyncReadExt};
use crate::schemas::{
    Config,
    ConfigAccount,
};
use crate::database::Connection;


pub async fn load_accounts(db: &Client) -> Vec<ConfigAccount> {
    let collection = db.database("clubcooee_trade").collection("accounts");
    let curd = Connection::new(collection);
    let filter = doc! {};
    let accounts = curd.read(filter).await.unwrap();

    accounts.iter().map(|account| {
        ConfigAccount {
            name: account.get("name").unwrap().as_str().unwrap().to_string(),
            udid: account.get("udid").unwrap().as_str().unwrap().to_string(),
            token: account.get("token").unwrap().as_str().unwrap().to_string(),
            role: account.get("role").unwrap().as_str().unwrap().to_string(),
        }
    }).collect()
}

pub async fn load_tokens(db: &Client) -> Vec<String> {
    let collection = db.database("clubcooee_trade").collection("tokens");
    let curd = Connection::new(collection);
    let filter = doc! {};
    let tokens = curd.read(filter).await.unwrap();

    tokens.iter().map(|token| {
        token.get("token").unwrap().as_str().unwrap().to_string()
    }).collect()
}

pub async fn load_servers(db: &Client) -> Vec<String> {
    let collection = db.database("clubcooee_trade").collection("servers");
    let curd = Connection::new(collection);
    let filter = doc! {};
    let servers = curd.read(filter).await.unwrap();

    servers.iter().map(|server| {
        server.get("host").unwrap().as_str().unwrap().to_string()
    }).collect()
}

pub async fn load_config(db: &Client) -> Config {
    let collection = db.database("clubcooee_trade").collection("config");
    let curd = Connection::new(collection);
    let filter = doc! {};
    let config = curd.read(filter).await.unwrap();

    let doc = config.get(0).ok_or("Config not found").unwrap();

    let hostname = match doc.get("hostname") {
        Some(endpoint) => endpoint.as_str().unwrap().to_string(),
        _ => "".to_string()
    };

    Config {
        hostname,
    }
}

pub async fn migration(db: &Client, name: &str) {
    let start = std::time::Instant::now();
    if name == "servers" {
        // load servers from file servers.json
        let mut file = File::open("servers.json").await.expect("File not found");
        let mut contents = vec![];
        file.read_to_end(&mut contents).await.expect("Error while reading file");

        // parse servers
        let servers: Vec<String> = serde_json::from_slice(&contents).expect("Error while reading file");

        // create collection
        let collection = db.database("clubcooee_trade").collection("servers");
        let curd = Connection::new(collection);

        // check if the collection is empty
        let filter = doc! {};
        let fetch = curd.read(filter).await.unwrap();
        if fetch.len() == 0 && servers.len() > 0 {
            // insert servers into the collection
            for server in servers {
                let doc = doc! {
                    "host": server,
                };
                curd.create(doc).await.unwrap();
            }
        }
    } else if name == "tokens" {
        // load tokens from file tokens.json
        let mut file = File::open("tokens.json").await.expect("File not found");
        let mut contents = vec![];
        file.read_to_end(&mut contents).await.expect("Error while reading file");

        // parse tokens
        let tokens: Vec<String> = serde_json::from_slice(&contents).expect("Error while reading file");

        // create collection
        let collection = db.database("clubcooee_trade").collection("tokens");
        let curd = Connection::new(collection);

        // check if the collection is empty
        let filter = doc! {};
        let fetch = curd.read(filter).await.unwrap();
        if fetch.len() == 0 && tokens.len() > 0 {
            // insert tokens into the collection
            for token in tokens {
                let doc = doc! {
                    "token": token,
                };
                curd.create(doc).await.unwrap();
            }
        }
    } else if name == "config" {
        // load config from file config.json
        let mut file = File::open("config.json").await.expect("File not found");
        let mut contents = vec![];
        file.read_to_end(&mut contents).await.expect("Error while reading file");
        let config: Config = serde_json::from_slice(&contents).expect("Error while reading file");

        // create collection
        let collection = db.database("clubcooee_trade").collection("config");
        let curd = Connection::new(collection);

        // check if the collection is empty
        let filter = doc! {};
        let fetch = curd.read(filter).await.unwrap();
        if fetch.len() == 0 {
            // insert config into the collection
            let doc = doc! {
                "hostname": config.hostname,
            };
            curd.create(doc).await.unwrap();
        }
    } else if name == "accounts" {
        // load accounts from file accounts.json
        let mut file = File::open("accounts.json").await.expect("File not found");
        let mut contents = vec![];
        file.read_to_end(&mut contents).await.expect("Error while reading file");

        // parse accounts
        let accounts: Vec<ConfigAccount> = serde_json::from_slice(&contents).expect("Error while reading file");

        // create collection
        let collection = db.database("clubcooee_trade").collection("accounts");
        let curd = Connection::new(collection);

        // check if the collection is empty
        let filter = doc! {};
        let fetch = curd.read(filter).await.unwrap();
        if fetch.len() == 0 && accounts.len() > 0 {
            // insert accounts into the collection
            for account in accounts {
                let doc = doc! {
                    "name": account.name,
                    "udid": account.udid,
                    "token": account.token,
                    "role": account.role,
                };
                curd.create(doc).await.unwrap();
            }
        }
    }
    eprintln!("{} Migration completed {}... {:?}", "[+]".white().bold(), name, start.elapsed());
}
