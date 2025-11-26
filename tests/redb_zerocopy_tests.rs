//! Integration tests for zero-copy redb implementation
//!
//! These tests verify the RedbStoreZeroCopy API works correctly with
//! explicit transaction management and bulk operations.

#![cfg(feature = "redb")]

use netabase_store::databases::redb_zerocopy::*;
use netabase_store::{NetabaseModel, netabase, netabase_definition_module};

// Test definition and models
#[netabase_definition_module(TestDef, TestKeys)]
mod test_models {
    use super::*;

    #[derive(
        NetabaseModel,
        Debug,
        Clone,
        PartialEq,
        Eq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(TestDef)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub name: String,
        #[secondary_key]
        pub email: String,
    }

    #[derive(
        NetabaseModel,
        Debug,
        Clone,
        PartialEq,
        Eq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(TestDef)]
    pub struct Product {
        #[primary_key]
        pub sku: String,
        pub name: String,
        pub price_cents: u64,
    }
}

use test_models::*;

#[test]
fn test_basic_crud() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("basic_crud.redb");
    let store = RedbStoreZeroCopy::<TestDef>::new(&db_path).unwrap();

    // Write
    let mut txn = store.begin_write().unwrap();
    let mut tree = txn.open_tree::<User>().unwrap();
    tree.put(User {
        id: 1,
        name: "Alice".into(),
        email: "alice@example.com".into(),
    })
    .unwrap();
    drop(tree);
    txn.commit().unwrap();

    // Read (owned)
    let txn = store.begin_read().unwrap();
    let tree = txn.open_tree::<User>().unwrap();
    let user = tree.get(&UserPrimaryKey(1)).unwrap().unwrap();
    assert_eq!(user.name, "Alice");
    assert_eq!(user.id, 1);
}

#[test]
fn test_put_many() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("bulk.redb");
    let store = RedbStoreZeroCopy::<TestDef>::new(&db_path).unwrap();

    let users: Vec<User> = (0..100)
        .map(|i| User {
            id: i,
            name: format!("User{}", i),
            email: format!("user{}@example.com", i),
        })
        .collect();

    let mut txn = store.begin_write().unwrap();
    let mut tree = txn.open_tree::<User>().unwrap();
    tree.put_many(users).unwrap();
    drop(tree);
    txn.commit().unwrap();

    // Verify count
    let txn = store.begin_read().unwrap();
    let tree = txn.open_tree::<User>().unwrap();
    assert_eq!(tree.len().unwrap(), 100);
}

#[test]
fn test_transaction_isolation() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("isolation.redb");
    let store = RedbStoreZeroCopy::<TestDef>::new(&db_path).unwrap();

    // Start read transaction
    let read_txn = store.begin_read().unwrap();
    let read_tree = read_txn.open_tree::<User>().unwrap();
    assert_eq!(read_tree.len().unwrap(), 0);

    // Write in separate transaction
    let mut write_txn = store.begin_write().unwrap();
    let mut write_tree = write_txn.open_tree::<User>().unwrap();
    write_tree
        .put(User {
            id: 1,
            name: "Alice".into(),
            email: "alice@example.com".into(),
        })
        .unwrap();
    drop(write_tree);
    write_txn.commit().unwrap();

    // Read transaction still sees old state (MVCC)
    assert_eq!(read_tree.len().unwrap(), 0);

    // New read transaction sees new state
    drop(read_tree);
    drop(read_txn);
    let new_txn = store.begin_read().unwrap();
    let new_tree = new_txn.open_tree::<User>().unwrap();
    assert_eq!(new_tree.len().unwrap(), 1);
}

#[test]
fn test_secondary_index() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("secondary.redb");
    let store = RedbStoreZeroCopy::<TestDef>::new(&db_path).unwrap();

    let mut txn = store.begin_write().unwrap();
    let mut tree = txn.open_tree::<User>().unwrap();
    tree.put(User {
        id: 1,
        name: "Alice".into(),
        email: "alice@example.com".into(),
    })
    .unwrap();
    tree.put(User {
        id: 2,
        name: "Bob".into(),
        email: "bob@example.com".into(),
    })
    .unwrap();
    drop(tree);
    txn.commit().unwrap();

    // Query by secondary key
    let txn = store.begin_read().unwrap();
    let tree = txn.open_tree::<User>().unwrap();
    let result = tree
        .get_by_secondary_key(&UserSecondaryKeys::Email(UserEmailSecondaryKey(
            "alice@example.com".to_string(),
        )))
        .unwrap();

    assert_eq!(result.len(), 1);
    // The result contains guards for primary keys - MultimapValue is an iterator
    let prim_keys: Vec<_> = result.into_iter().map(|res| res.unwrap().value()).collect();
    assert_eq!(prim_keys.len(), 1);
}

#[test]
fn test_remove() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("remove.redb");
    let store = RedbStoreZeroCopy::<TestDef>::new(&db_path).unwrap();

    // Insert
    let mut txn = store.begin_write().unwrap();
    let mut tree = txn.open_tree::<User>().unwrap();
    tree.put(User {
        id: 1,
        name: "Alice".into(),
        email: "alice@example.com".into(),
    })
    .unwrap();
    drop(tree);
    txn.commit().unwrap();

    // Remove
    let mut txn = store.begin_write().unwrap();
    let mut tree = txn.open_tree::<User>().unwrap();
    let removed = tree.remove(UserPrimaryKey(1)).unwrap();
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().name, "Alice");
    drop(tree);
    txn.commit().unwrap();

    // Verify removed
    let txn = store.begin_read().unwrap();
    let tree = txn.open_tree::<User>().unwrap();
    assert_eq!(tree.len().unwrap(), 0);
}

#[test]
fn test_remove_many() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("remove_many.redb");
    let store = RedbStoreZeroCopy::<TestDef>::new(&db_path).unwrap();

    // Insert multiple
    let mut txn = store.begin_write().unwrap();
    let mut tree = txn.open_tree::<User>().unwrap();
    for i in 1..=5 {
        tree.put(User {
            id: i,
            name: format!("User{}", i),
            email: format!("user{}@example.com", i),
        })
        .unwrap();
    }
    drop(tree);
    txn.commit().unwrap();

    // Remove multiple
    let mut txn = store.begin_write().unwrap();
    let mut tree = txn.open_tree::<User>().unwrap();
    let removed = tree
        .remove_many(vec![
            UserPrimaryKey(1),
            UserPrimaryKey(3),
            UserPrimaryKey(5),
        ])
        .unwrap();
    assert_eq!(removed.len(), 3);
    drop(tree);
    txn.commit().unwrap();

    // Verify
    let txn = store.begin_read().unwrap();
    let tree = txn.open_tree::<User>().unwrap();
    assert_eq!(tree.len().unwrap(), 2);
    assert!(tree.get(&UserPrimaryKey(1)).unwrap().is_none());
    assert!(tree.get(&UserPrimaryKey(2)).unwrap().is_some());
    assert!(tree.get(&UserPrimaryKey(3)).unwrap().is_none());
}

#[test]
fn test_clear() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("clear.redb");
    let store = RedbStoreZeroCopy::<TestDef>::new(&db_path).unwrap();

    // Insert
    let mut txn = store.begin_write().unwrap();
    let mut tree = txn.open_tree::<User>().unwrap();
    for i in 1..=10 {
        tree.put(User {
            id: i,
            name: format!("User{}", i),
            email: format!("user{}@example.com", i),
        })
        .unwrap();
    }
    assert_eq!(tree.len().unwrap(), 10);
    tree.clear().unwrap();
    assert_eq!(tree.len().unwrap(), 0);
    drop(tree);
    txn.commit().unwrap();

    // Verify cleared
    let txn = store.begin_read().unwrap();
    let tree = txn.open_tree::<User>().unwrap();
    assert_eq!(tree.len().unwrap(), 0);
}

#[test]
fn test_is_empty() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("empty.redb");
    let store = RedbStoreZeroCopy::<TestDef>::new(&db_path).unwrap();

    let txn = store.begin_read().unwrap();
    let tree = txn.open_tree::<User>().unwrap();
    assert!(tree.is_empty().unwrap());
    drop(tree);
    drop(txn);

    // Add item
    let mut txn = store.begin_write().unwrap();
    let mut tree = txn.open_tree::<User>().unwrap();
    tree.put(User {
        id: 1,
        name: "Alice".into(),
        email: "alice@example.com".into(),
    })
    .unwrap();
    assert!(!tree.is_empty().unwrap());
    drop(tree);
    txn.commit().unwrap();
}

#[test]
fn test_multi_tree_transaction() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("multi_tree.redb");
    let store = RedbStoreZeroCopy::<TestDef>::new(&db_path).unwrap();

    // Write to multiple trees in one transaction
    let mut txn = store.begin_write().unwrap();

    let mut user_tree = txn.open_tree::<User>().unwrap();
    user_tree
        .put(User {
            id: 1,
            name: "Alice".into(),
            email: "alice@example.com".into(),
        })
        .unwrap();
    drop(user_tree);

    let mut product_tree = txn.open_tree::<Product>().unwrap();
    product_tree
        .put(Product {
            sku: "ABC123".into(),
            name: "Widget".into(),
            price_cents: 1999,
        })
        .unwrap();
    drop(product_tree);

    txn.commit().unwrap();

    // Verify both
    let txn = store.begin_read().unwrap();
    let user_tree = txn.open_tree::<User>().unwrap();
    let product_tree = txn.open_tree::<Product>().unwrap();

    assert_eq!(user_tree.len().unwrap(), 1);
    assert_eq!(product_tree.len().unwrap(), 1);
}

#[test]
fn test_transaction_abort() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("abort.redb");
    let store = RedbStoreZeroCopy::<TestDef>::new(&db_path).unwrap();

    // Successful transaction
    let mut txn = store.begin_write().unwrap();
    let mut tree = txn.open_tree::<User>().unwrap();
    tree.put(User {
        id: 1,
        name: "Alice".into(),
        email: "alice@example.com".into(),
    })
    .unwrap();
    drop(tree);
    txn.commit().unwrap();

    // Aborted transaction
    let mut txn = store.begin_write().unwrap();
    let mut tree = txn.open_tree::<User>().unwrap();
    tree.put(User {
        id: 2,
        name: "Bob".into(),
        email: "bob@example.com".into(),
    })
    .unwrap();
    drop(tree);
    txn.abort().unwrap();

    // Verify only Alice exists
    let txn = store.begin_read().unwrap();
    let tree = txn.open_tree::<User>().unwrap();
    assert_eq!(tree.len().unwrap(), 1);
    assert!(tree.get(&UserPrimaryKey(1)).unwrap().is_some());
    assert!(tree.get(&UserPrimaryKey(2)).unwrap().is_none());
}

#[test]
fn test_with_write_transaction_helper() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("helper_write.redb");
    let store = RedbStoreZeroCopy::<TestDef>::new(&db_path).unwrap();

    // Use helper
    let count = with_write_transaction(&store, |txn| {
        let mut tree = txn.open_tree::<User>()?;
        for i in 1..=5 {
            tree.put(User {
                id: i,
                name: format!("User{}", i),
                email: format!("user{}@example.com", i),
            })?;
        }
        Ok(tree.len()?)
    })
    .unwrap();

    assert_eq!(count, 5);

    // Verify
    let txn = store.begin_read().unwrap();
    let tree = txn.open_tree::<User>().unwrap();
    assert_eq!(tree.len().unwrap(), 5);
}

#[test]
fn test_with_read_transaction_helper() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("helper_read.redb");
    let store = RedbStoreZeroCopy::<TestDef>::new(&db_path).unwrap();

    // Setup
    let mut txn = store.begin_write().unwrap();
    let mut tree = txn.open_tree::<User>().unwrap();
    tree.put(User {
        id: 42,
        name: "Answer".into(),
        email: "answer@example.com".into(),
    })
    .unwrap();
    drop(tree);
    txn.commit().unwrap();

    // Use helper
    let name = with_read_transaction(&store, |txn| {
        let tree = txn.open_tree::<User>()?;
        let user = tree.get(&UserPrimaryKey(42))?.unwrap();
        Ok(user.name.clone())
    })
    .unwrap();

    assert_eq!(name, "Answer");
}
