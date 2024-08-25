use colored::Colorize;
use crate::bot::Bot;


pub struct Session {
    bot: Bot
}


impl Session {
    pub fn new(bot: Bot) -> Self {
        Self {
            bot,
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self.bot.start().await {
            Ok(_) => {},
            Err(e) => {
                eprintln!("{} Bot session failed: {:?}", "[-]".red().bold(), e);
                return Err(e);
            }
        }
        Ok(())
    }
}
