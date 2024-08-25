use std::collections::{HashSet, VecDeque};
use redis::{Client, AsyncCommands};

pub struct ItemCache {
    set: HashSet<String>,
    queue: VecDeque<String>,
    capacity: usize,
}

#[derive(Clone)]
pub struct Redis {
    client: Client,
}

impl ItemCache {
    pub fn new(capacity: usize) -> Self {
        ItemCache {
            set: HashSet::new(),
            queue: VecDeque::new(),
            capacity,
        }
    }

    pub fn insert(&mut self, item: String) {
        if self.set.len() >= self.capacity {
            let removed = self.queue.pop_front().unwrap();
            self.set.remove(&removed);
        }

        self.set.insert(item.clone());
        self.queue.push_back(item);
    }

    pub fn contains(&self, item: &str) -> bool {
        self.set.contains(item)
    }
}

impl Redis {
    pub fn new(client: Client) -> Self {
        Redis { client }
    }

    pub async fn get(&self, key: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let value: String = conn.get(key).await?;
        if value.is_empty() {
            return Err("Key not found".into());
        }
        Ok(value)
    }

    pub async fn set(&self, key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        conn.set(key, value).await?;
        Ok(())
    }
}
