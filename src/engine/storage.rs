use std::sync::{atomic::AtomicUsize, Arc, RwLock};

use indexmap::IndexMap;
use indextree::{Arena, Node, NodeId};

/// The implementation utilizes the indexmap and indextree libraries for data storage,
/// while keeping these dependencies internal and not exposed to the user. The typedefs
/// specified herein facilitate the substitution of underlying data structures as needed.
///
/// Tree storage data structure.
pub type TreeStorageData<T> = Arena<T>;
/// Tree storage node.
pub type TreeStorageNode<T> = Node<T>;
/// Tree storage node id.
pub type TreeStorageId = NodeId;
/// Flat storage node id.
pub type FlatStorageId = usize;
/// Flat storage data structure.
pub type FlatStorageData<T> = IndexMap<FlatStorageId, T>;

/// Storage class. Allows to store and retrieve objects using their unique id.
/// Supports arena storage for tree based structures and hasmap storage for flat structures.
///
pub struct TreeStorage<V: Clone + Send + Sync> {
    data: Arc<RwLock<TreeStorageData<V>>>,
}

impl<V: Clone + Send + Sync> TreeStorage<V> {
    /// Creates new empty tree storage.
    pub fn new() -> Self {
        Default::default()
    }
    pub fn insert(&self, value: V) -> TreeStorageId {
        self.data.write().unwrap().new_node(value)
    }

    pub fn get(&self, id: impl Into<TreeStorageId>) -> Option<TreeStorageNode<V>> {
        let id = id.into();
        let data = self.data.read().unwrap();
        // this is equivalent to Some(obj.clone())
        data.get(id).cloned()
    }

    pub fn data(&self) -> Arc<RwLock<TreeStorageData<V>>> {
        self.data.clone()
    }

    pub fn remove_at(&self, id: &TreeStorageId) {
        id.remove_subtree(&mut self.data.write().unwrap());
    }

    pub fn with_data<T>(&self, f: impl FnOnce(&TreeStorageData<V>) -> T) -> T {
        let guard = self.data.read().unwrap();
        f(&guard)
    }

    pub fn with_data_mut<T>(&self, f: impl FnOnce(&mut TreeStorageData<V>) -> T) -> T {
        let mut guard = self.data.write().unwrap();
        f(&mut guard)
    }
}

impl<V: Clone + Send + Sync> Default for TreeStorage<V> {
    fn default() -> Self {
        Self {
            data: Arc::new(RwLock::new(TreeStorageData::<V>::new())),
        }
    }
}

pub struct FlatStorage<V: Clone> {
    data: Arc<RwLock<FlatStorageData<V>>>,
    index: AtomicUsize,
}

impl<V: Clone + Send + Sync> FlatStorage<V> {
    /// Creates new empty tree storage.
    pub fn new() -> Self {
        Default::default()
    }
    pub fn insert(&self, value: V) -> FlatStorageId {
        let id = self.index.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.data.write().unwrap().insert(id, value);
        id
    }
    pub fn insert_with_id(&self, value: V, id: FlatStorageId) -> FlatStorageId {
        self.data.write().unwrap().insert(id, value);
        id
    }
    pub fn get(&self, id: &FlatStorageId) -> Option<V> {
        let data = self.data.read().unwrap();
        // this is like Some(obj.clone())
        data.get(id).cloned()
    }

    pub fn data(&self) -> Arc<RwLock<FlatStorageData<V>>> {
        self.data.clone()
    }

    pub fn remove_at(&self, id: &FlatStorageId) {
        self.data.write().unwrap().remove(id);
    }

    pub fn with_data<T>(&self, f: impl FnOnce(&FlatStorageData<V>) -> T) -> T {
        let guard = self.data.read().unwrap();
        f(&guard)
    }

    pub fn with_data_mut<T>(&self, f: impl FnOnce(&mut FlatStorageData<V>) -> T) -> T {
        let mut guard = self.data.write().unwrap();
        f(&mut guard)
    }
}

impl<V: Clone + Send + Sync> Default for FlatStorage<V> {
    fn default() -> Self {
        Self {
            data: Arc::new(RwLock::new(FlatStorageData::<V>::new())),
            index: 0.into(),
        }
    }
}
