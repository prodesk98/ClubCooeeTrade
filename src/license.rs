use std::error::Error;
use colored::Colorize;
use mongodb::bson::doc;
use mongodb::Client;
use serde_json::Value;
use crate::database::Connection;


async fn address() -> Result<String, Box<dyn Error>> {
    match reqwest::get("http://ip-api.com/json").await {
        Ok(response) => {
            match response.json::<Value>().await {
                Ok(data) => {
                    if let Some(ip) = data["query"].as_str() {
                        Ok(ip.to_string())
                    } else {
                        Err("Failed to parse JSON.".into())
                    }
                },
                Err(_) => {
                    Err("Failed to parse JSON.".into())
                }
            }
        },
        Err(e) => {
            Err(format!("Failed to get IP address: {:?}", e).into())
        }
    }
}

pub async fn check(hostname: &str) {
    let ip = address().await.unwrap();
    let atlas = "mongodb+srv://desk24:3v6QB2nXgXPfHcaH@clubcooee.c0jj8.mongodb.net/?retryWrites=true&w=majority&appName=clubcooee";
    let client = Client::with_uri_str(atlas).await.unwrap();
    let crud = Connection::new(client.database("clubcooee").collection("licenses"));
    let filter = doc! { "hostname": hostname, "ip": ip };
    let licensed = crud.read(filter).await.unwrap();
    if licensed.len() == 0 {
        eprintln!("{} License not found for: {:?}", "[-]".red().bold(), hostname);
        std::process::exit(1);
    }
    let license = &licensed[0];
    let active: bool = license.get("active").unwrap().as_bool().unwrap();
    if !active {
        eprintln!("{} License not active for: {:?}", "[-]".red().bold(), hostname);
        std::process::exit(1);
    }
    println!("{} Running on: {:?}", "[*]".green().bold(), hostname);
}

