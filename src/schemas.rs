use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Proxy {
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize, Clone)]
pub struct Redis {
    pub scheme: String,
    pub host: String,
    pub port: u16,
    pub password: String,
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub hostname: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ConfigAccount {
    pub name: String,
    pub udid: String,
    pub token: String,
    pub role: String,
}

#[derive(Deserialize)]
pub struct Items {
    pub idtemplate: u32,
    pub name: String,
    pub price: u32,
}

#[derive(Deserialize, Clone)]
pub struct Item {
    pub id: u32,
    pub name: String,
    pub image: String,
    pub price: u32,
    pub itemt: u32,
}

#[derive(Deserialize)]
pub struct ItemMarket {
    pub history: Vec<f64>,
}

#[derive(Deserialize)]
pub struct Peer {
    id: u16,
    name: String,
}