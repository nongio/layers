use std::sync::{Arc, RwLock};

use indexmap::IndexMap;

pub struct Storage<V: Clone> {
    pub map: Arc<RwLock<IndexMap<usize, V>>>,
    index: RwLock<u32>,
}
impl<V: Clone> Storage<V> {
    pub fn new() -> Self {
        Self {
            map: Arc::new(RwLock::new(IndexMap::<usize, V>::new())),
            index: RwLock::new(0),
        }
    }
    pub fn insert(&mut self, value: V) -> usize {
        let mut index = self.index.write().unwrap();
        *index = *index + 1;
        let id = *index as usize;
        self.map.write().unwrap().insert(id, value);

        id
    }
    pub fn insert_with_id(&mut self, value: V, id: usize) -> usize {
        self.map.write().unwrap().insert(id, value);
        id
    }
    pub fn get(&self, id: usize) -> Option<V> {
        match self.map.read().unwrap().get(&id) {
            Some(value) => Some(value.clone()),
            None => None,
        }
    }
}
