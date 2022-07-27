use erender::engine::storage::Storage;

#[test]
pub fn test_storage() {
    let mut storage = Storage::new();
    let id = storage.insert(());
    assert_eq!(id, 1);
    let id = storage.insert(());
    assert_eq!(id, 2);
    let id = storage.insert(());
    assert_eq!(id, 3);

    storage.remove_at(id);

    assert_eq!(storage.get(id), None);
}
