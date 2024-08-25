use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct RoundRobin<T> {
    items: Arc<Mutex<Vec<T>>>,
    index: Arc<Mutex<usize>>,
}

impl<T> RoundRobin<T> where T: Clone {
    pub fn new(items: Vec<T>) -> Self {
        RoundRobin {
            items: Arc::new(Mutex::new(items)),
            index: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn next(&self) -> Option<T> {
        let mut index = self.index.lock().await;
        let items = self.items.lock().await;

        if items.is_empty() {
            return None;
        }

        let item = items[*index].clone();
        *index = (*index + 1) % items.len();

        Some(item)
    }
}