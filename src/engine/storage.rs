use std::sync::{atomic::AtomicUsize, Arc};

use indexmap::IndexMap;
use indextree::{Arena, Node, NodeId};
use tokio::{runtime::Handle, sync::RwLock};

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
    pub async fn insert(&self, value: V) -> TreeStorageId {
        let mut data = self.data.write().await;
        data.new_node(value)
    }

    pub async fn get(&self, id: impl Into<TreeStorageId>) -> Option<TreeStorageNode<V>> {
        let id = id.into();
        let data = self.data.read().await;
        data.get(id).cloned()
    }

    pub fn data(&self) -> Arc<RwLock<TreeStorageData<V>>> {
        self.data.clone()
    }

    pub async fn remove_at(&self, id: &TreeStorageId) {
        let mut data = self.data.write().await;
        id.remove_subtree(&mut data);
    }

    pub async fn with_data<T>(&self, f: impl FnOnce(&TreeStorageData<V>) -> T) -> T {
        let guard = self.data.read().await;
        f(&guard)
    }

    pub async fn with_data_mut<T>(&self, f: impl FnOnce(&mut TreeStorageData<V>) -> T) -> T {
        let mut guard = self.data.write().await;
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
        self.insert_with_id(value, id);
        id
    }
    pub fn insert_with_id(&self, value: V, id: FlatStorageId) -> FlatStorageId {
        let handle = Handle::current();
        tokio::task::block_in_place(|| {
            let mut data = handle.block_on(self.data.write());
            data.insert(id, value);
        });
        id
    }
    pub fn get(&self, id: &FlatStorageId) -> Option<V> {
        let handle = Handle::current();
        tokio::task::block_in_place(|| {
            let data = handle.block_on(self.data.read());
            data.get(id).cloned()
        })
    }

    pub fn data(&self) -> Arc<RwLock<FlatStorageData<V>>> {
        self.data.clone()
    }

    pub fn remove_at(&self, id: &FlatStorageId) {
        let handle = Handle::current();
        tokio::task::block_in_place(|| {
            let mut data = handle.block_on(self.data.write());
            data.remove(id);
        });
    }

    pub fn with_data<T>(&self, f: impl FnOnce(&FlatStorageData<V>) -> T) -> T {
        let handle = Handle::current();
        tokio::task::block_in_place(|| {
            let guard = handle.block_on(self.data.read());
            f(&guard)
        })
    }

    pub fn with_data_mut<T>(&self, f: impl FnOnce(&mut FlatStorageData<V>) -> T) -> T {
        let handle = Handle::current();
        tokio::task::block_in_place(|| {
            let mut guard = handle.block_on(self.data.write());
            f(&mut guard)
        })
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

// #[test]
// pub fn test_flat_storage() {
//     let flat = FlatStorage::<usize>::new();
//     let id = flat.insert(1);
//     let id2 = flat.insert(2);
//     let id3 = flat.insert(3);

//     assert_eq!(flat.get(&id).unwrap(), 1);
//     assert_eq!(flat.get(&id2).unwrap(), 2);
//     assert_eq!(flat.get(&id3).unwrap(), 3);

//     flat.remove_at(&id);
//     assert_eq!(flat.get(&id), None);
// }

// #[test]
// pub fn test_tree_storage() {
//     let tree = TreeStorage::<usize>::new();
//     let id = tree.insert(1);
//     let id2 = tree.insert(2);
//     let id3 = tree.insert(3);
//     tree.with_data_mut(|arena| {
//         id.append(id2, arena);
//         id.append(id3, arena);
//     });

//     assert_eq!(*tree.get(id).unwrap().get(), 1);

//     let children = tree.with_data(|arena| {
//         id.children(arena)
//             .map(|child| *arena.get(child).unwrap().get())
//             .collect::<Vec<_>>()
//     });

//     assert_eq!(children, [2, 3]);
// }
