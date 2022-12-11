use layers::engine::storage::{FlatStorage, TreeStorage};

#[test]
pub fn test_flat_storage() {
    let mut flat = FlatStorage::<usize>::new();
    let id = flat.insert(1);
    let id2 = flat.insert(2);
    let id3 = flat.insert(3);

    assert_eq!(flat.get(&id).unwrap(), 1);
    assert_eq!(flat.get(&id2).unwrap(), 2);
    assert_eq!(flat.get(&id3).unwrap(), 3);

    flat.remove_at(&id);
    assert_eq!(flat.get(&id), None);
}

#[test]
pub fn test_tree_storage() {
    let mut tree = TreeStorage::<usize>::new();
    let id = tree.insert(1);
    let id2 = tree.insert(2);
    let id3 = tree.insert(3);
    let data = tree.data();
    let mut arena = data.write().unwrap();
    id.append(id2, &mut arena);
    id.append(id3, &mut arena);

    assert_eq!(*tree.get(id).unwrap().get(), 1);

    let children = id
        .children(&arena)
        .map(|child| *arena.get(child).unwrap().get())
        .collect::<Vec<_>>();

    assert_eq!(children, [2, 3]);
}
