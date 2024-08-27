use std::sync::Arc;
use colored::Colorize;
use rustls::{ClientConfig, RootCertStore};
use rustls::pki_types::ServerName;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use tokio_rustls::TlsConnector;
use webpki_roots::TLS_SERVER_ROOTS;
use crate::schemas::Proxy;

#[derive(Clone)]
pub struct Socket {
    pub hostname: String,
    pub ip: String,
    pub token: String,
    pub port: u16,
    proxy: Proxy,
}


impl Socket {
    pub fn new(hostname: String, ip: String, token: String, port: u16, proxy: Proxy) -> Self {
        Self {
            hostname,
            ip,
            token,
            port,
            proxy,
        }
    }

    pub fn http(&self, path: &str, data: String) -> String {
        let headers = format!(
            "POST {} HTTP/1.1\r\n\
            Host: {}\r\n\
            Accept: */*\r\n\
            Accept-Language: pt-PT,pt;q=0.9,en-US;q=0.8,en;q=0.7\r\n\
            Connection: keep-alive\r\n\
            Content-type: application/x-www-form-urlencoded\r\n\
            Origin: https://cc-app.s3.amazonaws.com\r\n\
            Referer: https://cc-app.s3.amazonaws.com\r\n\
            Sec-Fetch-Dest: empty\r\n\
            Sec-Fetch-Mode: cors\r\n\
            Sec-Fetch-Site: cross-site\r\n\
            User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36\r\n\
            sec-ch-ua: \"Not)A;Brand\";v=\"99\", \"Google Chrome\";v=\"127\", \"Chromium\";v=\"127\"\r\n\
            sec-ch-ua-mobile: ?0\r\n\
            sec-ch-ua-platform: \"Windows\"\r\n\
            Content-Length: {}\r\n\r\n\
            {}",
            path,
            self.hostname,
            data.len(),
            data
        );
        headers
    }

    pub async fn connect_with_proxy(&self) -> Result<TcpStream, Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", self.ip, self.port);
        let _s = std::time::Instant::now();

        // Proxy connection
        let addr_proxy = format!("{}:{}", self.proxy.host, self.proxy.port);
        let mut stream = TcpStream::connect(addr_proxy).await?;

        let request = format!(
            "CONNECT {} HTTP/1.1\r\nHost: {}\r\n\r\n",
            addr,
            addr,
        );

        stream.write_all(request.as_bytes()).await?;
        stream.flush().await?;

        let mut proxy_response = vec![0; 1024];
        let n = stream.read(&mut proxy_response).await?;

        let response = String::from_utf8_lossy(&proxy_response[..n]);
        if !response.contains("200 Connection established") {
            return Err("Proxy connection failed".into());
        }
        eprintln!("{} Proxy connection established... {:?}", "[*]".blue().bold(), _s.elapsed());
        Ok(stream)
    }

    pub async fn connect(&self) -> Result<TcpStream, Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", self.ip, self.port);
        let stream = TcpStream::connect(addr).await?;
        Ok(stream)
    }

    pub async fn tls_connect(&self, stream: TcpStream) -> Result<TlsStream<TcpStream>, Box<dyn std::error::Error>> {
        let connector = TlsConnector::from(Arc::new(self.client_config()));
        let domain = ServerName::try_from(self.hostname.clone()).unwrap();

        let tls_stream = match connector.connect(domain, stream).await {
            Ok(tls_stream) => tls_stream,
            Err(e) => {
                return Err(Box::new(e));
            }
        };
        Ok(TlsStream::from(tls_stream))
    }

    fn client_config(&self) -> ClientConfig {
        // SSL/TLS configuration
        let mut root_cert_store = RootCertStore::empty();
        root_cert_store.extend(TLS_SERVER_ROOTS.iter().cloned());

        let config = ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();
        config
    }
}