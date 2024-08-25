use colored::Colorize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::parse;
use crate::schemas::{ConfigAccount, Item, ItemMarket};
use crate::socket::Socket;
use crate::telegram::Telegram;

#[derive(Clone)]
pub struct Market {
    pub socket: Socket,
    pub seller: ConfigAccount,
    pub buyer: ConfigAccount,
    telegram: Telegram,
}

impl Market {
    pub fn new(
        socket: Socket,
        seller: ConfigAccount,
        buyer: ConfigAccount,
        telegram: Telegram
    ) -> Self {
        Self {
            socket,
            seller,
            buyer,
            telegram,
        }
    }

    pub async fn search(&self, itemt: u32) -> Result<ItemMarket, Box<dyn std::error::Error>> {
        // HTTP request
        let request = self.socket.http(
            "/api3/trade_search",
            format!("template_id={}&seller_id=0&token={}", itemt, self.socket.token)
        );

        // TCP connection
        let stream = self.socket.connect().await?;
        let mut tls_stream = self.socket.tls_connect(stream).await?;

        tls_stream.write_all(request.as_bytes()).await?;

        let mut buffer = [0; 1024*2];
        let n = tls_stream.read(&mut buffer).await?;
        let response = String::from_utf8_lossy(&buffer[..n]);

        if !response.contains("prices") {
            return Err("Failed to search item".into());
        }

        Ok(ItemMarket {
            history: parse::prices(&response),
        })
    }

    pub async fn fetch(&self) -> Result<Vec<Item>, Box<dyn std::error::Error>> {
        // HTTP request
        let request = self.socket.http(
            "/api3/trade_get_items",
            format!("sort=2&token={}&fp4=74d8ca5a4c539ac6d2dcc22f6591cf8f&build=aj.24.726.1455", self.socket.token)
        );

        // TCP connection
        let stream = self.socket.connect().await?;
        let mut tls_stream = self.socket.tls_connect(stream).await?;

        // Send TLS request
        tls_stream.write_all(request.as_bytes()).await?;

        // Read response
        let mut buffer = [0; 1024*12];
        let mut response = Vec::new();
        let n = tls_stream.read(&mut buffer).await?;
        response.extend_from_slice(&buffer[..n]);

        let body = String::from_utf8_lossy(&response);

        // blocking
        if body.contains("misuse quota") {
            return Err("Misuse quota".into());
        }

        // extract items from regex
        let items = parse::item(&body);

        Ok(items)
    }

    pub async fn sell(&self, id: u32, price: f64) -> Result<bool, Box<dyn std::error::Error>> {
        // HTTP request
        let request = self.socket.http(
            "/api3/trade_sell",
            format!(
                "id={}&price={}&duration_days={}&token={}&udid={}",
                id,
                price,
                1,
                self.seller.token,
                self.seller.udid
            )
        );

        // TCP connection
        let stream = self.socket.connect().await?;
        let mut tls_stream = self.socket.tls_connect(stream).await?;

        // time start
        let _s = std::time::Instant::now();

        // Send TLS request
        tls_stream.write_all(request.as_bytes()).await?;
        tls_stream.flush().await?;

        // read response
        let mut buffer = [0; 1024];
        let n = tls_stream.read(&mut buffer).await?;
        let response = String::from_utf8_lossy(&buffer[..n]);
        eprintln!("{} Sent Selling item: {:?}... {:?}", "[*]".blue().bold(), id, _s.elapsed());

        let selling = response.contains("market_item_sell_form_success");

        if selling {
            Ok(true)
        } else {
            Err("Failed to sell item".into())
        }
    }

    pub async fn buy(&self, item: Item) -> Result<bool, Box<dyn std::error::Error>> {
        // HTTP request
        let request = self.socket.http(
            "/api3/trade_buy",
            format!(
                "id={}&token={}&udid={}",
                item.id,
                self.buyer.token,
                self.buyer.udid
            )
        );

        // TCP connection
        let stream = self.socket.connect().await?;
        let mut tls_stream = self.socket.tls_connect(stream).await?;

        // time start
        let _s = std::time::Instant::now();

        // Send TLS request
        tls_stream.write_all(request.as_bytes()).await?;
        tls_stream.flush().await?;

        // read response
        let mut buffer = [0; 1024*4];
        let n = tls_stream.read(&mut buffer).await?;
        let response = String::from_utf8_lossy(&buffer[..n]);
        eprintln!("{} Sent Buying item: {:?}... {:?}", "[*]".blue().bold(), item.id, _s.elapsed());

        let buying = response.contains("market_item_buy_form_success");

        if buying {
            let message = format!(
                "{} ({})\nid: {}, price: {}cc",
                item.name, item.itemt, item.id, item.price
            );
            self.telegram.send_image(item.image.to_string().replace("\\/", "/"), &message).await?;
            Ok(true)
        } else {
            Err("Failed to buy item".into())
        }
    }
}
