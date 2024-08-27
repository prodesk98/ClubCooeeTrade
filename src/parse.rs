use reqwest::Url;
use regex::Regex;
use crate::schemas::{Item, Proxy};

pub fn proxy(uri: &str) -> Proxy {
    let parsed = Url::parse(uri).unwrap();

    let host = parsed.host_str().unwrap().to_string();
    let port = parsed.port().unwrap_or(8080);

    Proxy {
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

pub fn prices(data: &str) -> Vec<f64> {
    let re = Regex::new(r#""prices":\[(.*?)]"#).unwrap();
    let mut prices = Vec::new();

    if let Some(captures) = re.captures(data) {
        if let Some(price_match) = captures.get(1) {
            let prices_str = price_match.as_str();
            for price in prices_str.split(',') {
                if let Ok(parsed_price) = price.trim().parse::<f64>() {
                    prices.push(parsed_price);
                }
            }
        }
    }

    prices
}
