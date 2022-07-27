use std::sync::{Arc, RwLock};

use indexmap::IndexMap;

trait Command {
    fn execute(&self);
}

/// Generic storage class. Allows to store and retrieve objects using their unique id.
/// On insert, the id can be optionally provided (otherwise will be generated automatically).
pub struct Storage<V: Clone + Send + Sync> {
    data: Arc<RwLock<IndexMap<usize, V>>>,
    index: RwLock<u32>,
}

impl<V: Clone + Send + Sync> Storage<V> {
    /// Creates new empty storage.
    pub fn new() -> Self {
        Default::default()
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
    pub fn get(&self, id: &usize) -> Option<V> {
        self.data.read().unwrap().get(id).cloned()
    }

    pub fn data(&self) -> Arc<RwLock<IndexMap<usize, V>>> {
        self.data.clone()
    }

    pub fn remove_at(&mut self, id: &usize) {
        self.data.write().unwrap().remove(id);
    }
}

impl<V: Clone + Send + Sync> Default for Storage<V> {
    fn default() -> Self {
        Self {
            data: Arc::new(RwLock::new(IndexMap::<usize, V>::new())),
            index: RwLock::new(0),
        }
    }
}
