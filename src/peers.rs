use crate::cache::Redis;
use crate::schemas::Peer;

#[derive(Clone)]
pub struct Peers {
    pub hostname: String,
    pub redis: Redis,
}

impl Peers {
    pub fn new(hostname: String, redis: Redis) -> Self {
        Self {
            hostname,
            redis,
        }
    }

    pub async fn get(&self, key: &str) -> Result<Peer, Box<dyn std::error::Error>> {
        let result = self.redis.get(key).await?;
        let peer: Peer = serde_json::from_str(&result)?;
        Ok(peer)
    }
}