use std::error::Error;
use std::fmt::Pointer;
use std::sync::Arc;
use std::time::Duration;
use colored::Colorize;
use futures::future::join_all;
use reqwest::Client;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::spawn;
use tokio::sync::RwLock;
use tokio::time::timeout;
use crate::parse;
use crate::round_robin::RoundRobin;
use crate::schemas::Proxy;
use crate::socket::Socket;

pub struct ProxyManager {
    pub rr: RoundRobin<Proxy>,
    pub blacklist: Vec<String>,
    proxies: Arc<RwLock<Vec<Proxy>>>,
    tokens: Arc<RwLock<RoundRobin<String>>>,
    servers: Arc<RwLock<RoundRobin<String>>>,
}


impl ProxyManager {
    pub async fn new(tokens: Arc<RwLock<RoundRobin<String>>>, servers: Arc<RwLock<RoundRobin<String>>>) -> Self {
        Self {
            rr: RoundRobin::new(vec![]),
            blacklist: vec![],
            proxies: Arc::new(RwLock::new(vec![])),
            tokens,
            servers,
        }
    }

    pub async fn load(&mut self) -> RoundRobin<Proxy> {
        let response = Client::new()
            .get("https://api.proxyscrape.com/v3/free-proxy-list/get?request=displayproxies&protocol=http&proxy_format=protocolipport&format=text&timeout=20000")
            .send()
            .await
            .unwrap();

        let text = response.text().await.unwrap();

        let mut proxies = vec![];
        let reader = BufReader::new(text.as_bytes());

        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let proxy = parse::proxy(format!("{}", line.trim()).as_str());
            proxies.push(proxy);
        }

        eprintln!("{} found {} proxies...", "[*]".blue().bold(), proxies.len());

        let mut tasks = Vec::new();
        for proxy in proxies {
            let proxies = Arc::clone(&self.proxies);
            let token = self.tokens.read().await.next().await.unwrap().clone();
            let server = self.servers.read().await.next().await.unwrap().clone();

            tasks.push(
                spawn(
                    async move {
                        let proxies = Arc::clone(&proxies);
                        let _ = timeout(
                            Duration::from_secs(5),
                            Self::filter(proxy, proxies, token, server),
                        ).await;
                    }
                )
            );
        }
        join_all(tasks).await;

        self.rr = RoundRobin::new(self.proxies.read().await.clone());
        self.rr.clone()
    }

    pub async fn filter(proxy: Proxy, proxies: Arc<RwLock<Vec<Proxy>>>, token: String, server: String) -> Result<(), Box<dyn Error>> {
        let hostname = "en.clubcooee.com".to_string();
        let s = Socket::new(
            hostname,
            server,
            token,
            443,
            proxy.clone()
        );
        let stream = s.connect_with_proxy().await?;
        let mut tls = s.tls_connect(stream).await?;

        let request = s.http(
            "/api3/trade_get_items",
            format!("sort=2&token={}&build=aj.24.726.1455", s.token)
        );
        tls.write_all(request.as_bytes()).await?;

        let mut buffer = [0; 1024];
        let mut response = Vec::new();
        let n = tls.read(&mut buffer).await?;
        response.extend_from_slice(&buffer[..n]);
        let body = String::from_utf8_lossy(&response);

        if body.contains("200") {
            proxies.write().await.push(proxy);
        }
        drop(tls);
        Ok(())
    }

    pub async fn next(&self) -> Proxy {
        while let Some(proxy) = self.rr.next().await {
            if !self.blacklist.contains(&proxy.host) {
                return proxy;
            }
        }
        self.rr.next().await.unwrap()
    }

    pub async fn add_blacklist(&mut self, proxy: Proxy) {
        self.blacklist.insert(0, proxy.host.clone());
    }
}