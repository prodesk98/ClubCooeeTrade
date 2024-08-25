

#[derive(Clone)]
pub struct Telegram {
    token: String,
    chat_id: String,
}


impl Telegram {
    pub fn new(token: String, chat_id: String) -> Self {
        Self {
            token,
            chat_id,
        }
    }

    pub async fn send_text(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.token);
        let data = format!("chat_id={}&text={}", self.chat_id, message);
        let client = reqwest::Client::new();
        let _res = client.post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(data)
            .send()
            .await?;
        Ok(())
    }

    pub async fn send_image(&self, image: String, caption: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("https://api.telegram.org/bot{}/sendPhoto", self.token);
        let data = format!("chat_id={}&photo={}&caption={}", self.chat_id, image, caption);
        let client = reqwest::Client::new();
        let _res = client.post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(data)
            .send()
            .await?;
        Ok(())
    }
}

