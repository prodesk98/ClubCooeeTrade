use base64::Engine;
use reqwest::Url;
use base64::prelude::BASE64_STANDARD;
use regex::Regex;
use crate::schemas::{Item, Proxy};

pub fn proxy(uri: &str) -> Proxy {
    let parsed = Url::parse(uri).unwrap();

    let username = parsed.username();
    let password = parsed.password().unwrap();
    let host = parsed.host_str().unwrap().to_string();
    let port = parsed.port().unwrap_or(8080);
    let credentials = BASE64_STANDARD.encode(format!("{}:{}", username, password).as_bytes());

    Proxy {
        credentials,
        host,
        port,
    }
}


pub fn item(data: &str) -> Vec<Item> {
    let re = Regex::new(r#""id":(\d+),"price":(\d+),.*?"itemt":\{"id":(\d+).*?"name":"([^"]+)".*?"image":"([^"]+)""#).unwrap();
    let mut items = Vec::new();

    for cap in re.captures_iter(&data) {
        if let (Some(id_match), Some(price_match), Some(itemt_id_match), Some(name_match), Some(image_match)) =
            (cap.get(1), cap.get(2), cap.get(3), cap.get(4), cap.get(5)) {
            let item = Item {
                id: id_match.as_str().parse().unwrap_or(0),
                name: name_match.as_str().to_string(),
                image: image_match.as_str().to_string(),
                itemt: itemt_id_match.as_str().parse().unwrap_or(0),
                price: price_match.as_str().parse().unwrap_or(0),
            };
            items.push(item);
        }
    }
    items
}
