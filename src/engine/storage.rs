use std::cell::RefCell;
use std::ops::Deref;
use std::sync::{atomic::AtomicUsize, Arc};

use indexmap::IndexMap;
use indextree::{Arena, Node, NodeId};
use std::sync::{RwLock, RwLockReadGuard};

thread_local! {
    /// The storage read locks this thread holds: (lock address, data address).
    static HELD_READS: RefCell<Vec<(usize, usize)>> = const { RefCell::new(Vec::new()) };
}

/// Shared access to a storage's data that a thread can take again while it
/// already holds it.
///
/// A second `RwLock::read()` on a lock the same thread is reading deadlocks as
/// soon as another thread queues a `write()` between the two: a waiting writer
/// holds back new readers, the writer waits for the outer read to end, and the
/// outer read is waiting for the inner one. Drawing reads the scene's arenas
/// and reads them again inside, whenever a mirror (`Layer::as_content`)
/// replays the subtree it follows, so any thread adding a layer at that moment
/// froze the drawing thread for good. A nested read here borrows the data the
/// outer read already holds instead of queueing for the lock.
pub(crate) enum SharedRead<'a, D> {
    Held(RwLockReadGuard<'a, D>, usize),
    Nested(&'a D),
}

impl<D> Deref for SharedRead<'_, D> {
    type Target = D;
    fn deref(&self) -> &D {
        match self {
            SharedRead::Held(guard, _) => guard,
            SharedRead::Nested(data) => data,
        }
    }
}

impl<D> Drop for SharedRead<'_, D> {
    fn drop(&mut self) {
        if let SharedRead::Held(_, key) = self {
            let key = *key;
            HELD_READS.with(|held| {
                let mut held = held.borrow_mut();
                if let Some(at) = held.iter().rposition(|(k, _)| *k == key) {
                    held.remove(at);
                }
            });
        }
    }
}

fn held_read<D>(lock: &RwLock<D>) -> Option<&D> {
    let key = lock as *const RwLock<D> as usize;
    let data = HELD_READS.with(|held| {
        held.borrow()
            .iter()
            .rev()
            .find(|(k, _)| *k == key)
            .map(|(_, data)| *data)
    })?;
    // SAFETY: the entry belongs to a `SharedRead::Held` of this thread that is
    // still alive (its drop removes the entry before the guard is released),
    // so the lock stays read-held while this borrow is in use on this thread
    // and no thread can write the data meanwhile.
    Some(unsafe { &*(data as *const D) })
}

fn hold<'a, D>(guard: RwLockReadGuard<'a, D>, lock: &RwLock<D>) -> SharedRead<'a, D> {
    let key = lock as *const RwLock<D> as usize;
    let data = &*guard as *const D as usize;
    HELD_READS.with(|held| held.borrow_mut().push((key, data)));
    SharedRead::Held(guard, key)
}

/// Read `lock`, re-using a read this thread already holds on it.
pub(crate) fn read_shared<D>(lock: &RwLock<D>) -> SharedRead<'_, D> {
    if let Some(data) = held_read(lock) {
        return SharedRead::Nested(data);
    }
    let guard = lock.read().unwrap_or_else(|e| e.into_inner());
    hold(guard, lock)
}

/// Like [`read_shared`], but `None` instead of waiting when the lock is taken
/// or a writer is queued for it. A read this thread already holds is re-used.
pub(crate) fn try_read_shared<D>(lock: &RwLock<D>) -> Option<SharedRead<'_, D>> {
    if let Some(data) = held_read(lock) {
        return Some(SharedRead::Nested(data));
    }
    let guard = lock.try_read().ok()?;
    Some(hold(guard, lock))
}

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
pub struct TreeStorage<V: Send + Sync> {
    data: Arc<RwLock<TreeStorageData<V>>>,
}

impl<V: Send + Sync> TreeStorage<V> {
    /// Creates new empty tree storage.
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert_sync(&self, value: V) -> TreeStorageId {
        let mut data = self.data.write().unwrap();
        data.new_node(value)
    }

    pub fn data(&self) -> Arc<RwLock<TreeStorageData<V>>> {
        self.data.clone()
    }

    pub fn remove_at_sync(&self, id: &TreeStorageId) {
        let mut data = self.data.write().unwrap();
        id.remove_subtree(&mut data);
    }

    pub fn with_data<T>(&self, f: impl FnOnce(&TreeStorageData<V>) -> T) -> T {
        let guard = read_shared(&self.data);
        f(&guard)
    }

    pub fn with_data_mut<T>(&self, f: impl FnOnce(&mut TreeStorageData<V>) -> T) -> T {
        let mut guard = self.data.write().unwrap();
        f(&mut guard)
    }
}

impl<V: Send + Sync> Default for TreeStorage<V> {
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
        let mut data = self.data.write().unwrap();
        data.insert(id, value);
        id
    }
    pub fn get(&self, id: &FlatStorageId) -> Option<V> {
        let data = read_shared(&self.data);
        data.get(id).cloned()
    }

    pub fn data(&self) -> Arc<RwLock<FlatStorageData<V>>> {
        self.data.clone()
    }

    pub fn remove_at(&self, id: &FlatStorageId) {
        let mut data = self.data.write().unwrap();
        data.remove(id);
    }

    pub fn with_data<T>(&self, f: impl FnOnce(&FlatStorageData<V>) -> T) -> T {
        let data = read_shared(&self.data);
        f(&data)
    }

    pub fn with_data_cloned<T>(&self, f: impl FnOnce(&FlatStorageData<V>) -> T) -> T {
        let data = self.with_data(|data| data.clone());
        f(&data)
    }

    pub fn with_data_mut<T>(&self, f: impl FnOnce(&mut FlatStorageData<V>) -> T) -> T {
        let mut data = self.data.write().unwrap();
        f(&mut data)
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
