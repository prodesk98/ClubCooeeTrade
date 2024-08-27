use colored::Colorize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::parse;
use crate::schemas::{ConfigAccount, Item};
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

    fn fp4(&self) -> String {
        // generate fp4
        let mut fp4 = String::new();
        for _ in 0..32 {
            let random = rand::random::<u8>();
            fp4.push_str(&format!("{:x}", random));
        }
        fp4
    }

    pub async fn fetch(&self) -> Result<Vec<Item>, Box<dyn std::error::Error>> {
        let url = format!(
            "https://{}/api3/trade_get_items",
            self.socket.hostname,
        );

        let fp4 = self.fp4();
        let form = [
            ("sort", "2"),
            ("token", self.socket.token.as_str()),
            ("fp4", fp4.as_str()),
            ("build", "aj.24.726.1455"),
        ];

        let client = reqwest::Client::new()
            .post(&url)
            .form(&form)
            .send()
            .await?;

        let body = client.text().await?;
        let items = parse::item(&body);

        Ok(items)
    }

    pub async fn sell(&self, id: u32, price: f64) -> Result<bool, Box<dyn std::error::Error>> {
        // HTTP request
        let request = self.socket.http(
            "/api3/trade_sell",
            format!(
                "id={}&price={}&duration_days={}&token={}&fp4={}&udid={}&build=aj.24.726.1455",
                id,
                price,
                1,
                self.buyer.token,
                self.fp4(),
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
                "id={}&token={}&fp4={}&udid={}&build=aj.24.726.1455",
                item.id,
                self.buyer.token,
                self.fp4(),
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
                "✅ {} ({})\nid: {}, price: {}cc",
                item.name, item.itemt, item.id, item.price
            );
            self.telegram.send_image(item.image.to_string().replace("\\/", "/"), &message).await?;
            Ok(true)
        } else {
            Err("Failed to buy item".into())
        }
    }

    pub async fn sold(&self) -> Result<Vec<Item>, Box<dyn std::error::Error>> {
        // endpoint
        let url = format!(
            "https://{}/api3/trade_get_items",
            self.socket.hostname,
        );
        let fp4 = self.fp4();
        let form = [
            ("sort", "5"),
            ("token", self.socket.token.as_str()),
            ("fp4", fp4.as_str()),
            ("build", "aj.24.726.1455"),
        ];

        let client = reqwest::Client::new()
            .post(&url)
            .form(&form)
            .send()
            .await?;

        let body = client.text().await?;
        let items = parse::item(&body);

        Ok(items)
    }
}
