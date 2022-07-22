use std::sync::{Arc, RwLock};

use indexmap::IndexMap;

trait Command {
    fn execute(&self);
}

pub struct Storage<V: Clone + Send + Sync> {
    pub data: Arc<RwLock<IndexMap<usize, V>>>,
    index: RwLock<u32>,
}
impl<V: Clone + Send + Sync> Storage<V> {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(IndexMap::<usize, V>::new())),
            index: RwLock::new(0),
        }
    }
    pub fn insert(&mut self, value: V) -> usize {
        let mut index = self.index.write().unwrap();
        *index += 1;
        let id = *index as usize;
        self.data.write().unwrap().insert(id, value);

        id
    }
    pub fn insert_with_id(&mut self, value: V, id: usize) -> usize {
        self.data.write().unwrap().insert(id, value);
        id
    }
    pub fn get(&self, id: usize) -> Option<V> {
        self.data.read().unwrap().get(&id).cloned()
    }
}
