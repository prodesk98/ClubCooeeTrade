use colored::Colorize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::parse;
use crate::schemas::{ConfigAccount, Item};
use crate::socket::Socket;
use crate::telegram::Telegram;

#[derive(Clone)]
pub struct Market {
    pub socket: Socket,
    pub account: ConfigAccount,
    telegram: Telegram,
}

impl Market {
    pub fn new(socket: Socket, account: ConfigAccount, telegram: Telegram) -> Self {
        Self {
            socket,
            account,
            telegram,
        }
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

    pub async fn buy(&self, item: Item) -> Result<bool, Box<dyn std::error::Error>> {
        // HTTP request
        let request = self.socket.http(
            "/api3/trade_buy",
            format!(
                "id={}&token={}&udid={}",
                item.id,
                self.account.token,
                self.account.udid
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
            self.telegram.send_image(item.image, &message).await?;
            Ok(true)
        } else {
            Err("Failed to buy item".into())
        }
    }
}
