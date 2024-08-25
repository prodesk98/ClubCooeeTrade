use colored::Colorize;

struct Logger {}

impl Logger {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn debug(&self, message: &str) {
        println!("{} {}", "[+]".blue().bold(), message);
    }

    pub async fn success(&self, message: &str) {
        println!("{} {}", "[+]".green().bold(), message);
    }

    pub async fn info(&self, message: &str) {
        println!("{} {}", "[+]".white().bold(), message);
    }

    pub async fn error(&self, message: &str) {
        println!("{} {}", "[-]".red().bold(), message);
    }
}