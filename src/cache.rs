use std::collections::{HashSet, VecDeque};

pub struct ItemCache {
    set: HashSet<String>,
    queue: VecDeque<String>,
    capacity: usize,
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