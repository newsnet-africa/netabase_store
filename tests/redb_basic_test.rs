//! Basic integration test for redb backend
//!
//! Tests the standard redb_store.rs implementation (not zerocopy)

#![cfg(all(feature = "redb", not(feature = "paxos")))]

use netabase_store::databases::redb_store::RedbStore;
use netabase_store::{NetabaseModel, netabase, netabase_definition_module};
use tempfile::TempDir;

// Define test schema
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
}

use test_models::*;

#[test]
fn test_redb_basic_put_get() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.redb");

    let store = RedbStore::<TestDef>::new(&db_path).unwrap();
    let tree = store.open_tree::<User>();

    // Create a user
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    // Put the user
    tree.put(user.clone()).unwrap();

    // Get the user back using the full Keys enum
    let retrieved = tree.get(UserKey::Primary(UserPrimaryKey(1))).unwrap();
    assert_eq!(Some(user), retrieved);
}

#[test]
fn test_redb_secondary_key_query() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.redb");

    let store = RedbStore::<TestDef>::new(&db_path).unwrap();
    let tree = store.open_tree::<User>();

    // Create users
    let user1 = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    let user2 = User {
        id: 2,
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
    };

    tree.put(user1.clone()).unwrap();
    tree.put(user2.clone()).unwrap();

    // Query by secondary key (email)
    let results = tree
        .get_by_secondary_key(UserSecondaryKeys::Email(UserEmailSecondaryKey(
            "alice@example.com".to_string(),
        )))
        .unwrap();

    assert_eq!(1, results.len());
    assert_eq!(user1, results[0]);
}

#[test]
fn test_redb_remove() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.redb");

    let store = RedbStore::<TestDef>::new(&db_path).unwrap();
    let tree = store.open_tree::<User>();

    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    tree.put(user.clone()).unwrap();

    // Remove using Keys enum
    let removed = tree.remove(UserKey::Primary(UserPrimaryKey(1))).unwrap();
    assert_eq!(Some(user), removed);

    // Verify it's gone
    let retrieved = tree.get(UserKey::Primary(UserPrimaryKey(1))).unwrap();
    assert_eq!(None, retrieved);
}
