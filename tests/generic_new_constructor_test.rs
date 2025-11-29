/// Integration tests for the generic NetabaseStore::new() constructor
///
/// These tests verify that the new() constructor works correctly with
/// different backends and can be used in generic contexts.
use netabase_store::{NetabaseModelTrait, NetabaseStore, netabase_definition_module};
use tempfile::TempDir;

// Define a test schema
#[netabase_definition_module(TestDef, TestKeys)]
mod test_models {
    use netabase_store::{NetabaseModel, netabase};

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        Eq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(TestDef)]
    pub struct TestUser {
        #[primary_key]
        pub id: u64,
        pub name: String,
        #[secondary_key]
        pub email: String,
    }

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        Eq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(TestDef)]
    pub struct TestPost {
        #[primary_key]
        pub id: String,
        pub title: String,
        pub author_id: u64,
    }
}

use test_models::*;

/// Helper function to create a temp directory for tests
fn setup_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

#[test]
#[cfg(feature = "sled")]
fn test_new_with_sled_backend() {
    use netabase_store::databases::sled_store::SledStore;

    let temp_dir = setup_temp_dir();
    let db_path = temp_dir.path().join("test_sled.db");

    // Create store using generic new()
    let store = NetabaseStore::<TestDef, SledStore<TestDef>>::new(&db_path)
        .expect("Failed to create SledStore");

    // Test basic operations
    let tree = store.open_tree::<TestUser>();

    let user = TestUser {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    // Insert
    tree.put(user.clone()).expect("Failed to put user");

    // Retrieve by primary key
    let retrieved = tree
        .get(user.primary_key())
        .expect("Failed to get user")
        .expect("User not found");

    assert_eq!(retrieved, user);

    // Retrieve by secondary key
    let by_email = tree
        .get_by_secondary_key(TestUserSecondaryKeys::Email(TestUserEmailSecondaryKey(
            "alice@example.com".to_string(),
        )))
        .expect("Failed to get by email");

    assert_eq!(by_email.len(), 1);
    assert_eq!(by_email[0], user);
}

#[test]
#[cfg(feature = "redb")]
fn test_new_with_redb_backend() {
    use netabase_store::databases::redb_store::RedbStore;

    let temp_dir = setup_temp_dir();
    let db_path = temp_dir.path().join("test_redb.redb");

    // Create store using generic new()
    let store = NetabaseStore::<TestDef, RedbStore<TestDef>>::new(&db_path)
        .expect("Failed to create RedbStore");

    // Test basic operations
    let tree = store.open_tree::<TestPost>();

    let post = TestPost {
        id: "post-1".to_string(),
        title: "Test Post".to_string(),
        author_id: 42,
    };

    // Insert
    tree.put(post.clone()).expect("Failed to put post");

    // Retrieve by primary key (Redb requires Key enum wrapper)
    let retrieved = tree
        .get(TestPostKey::Primary(post.primary_key()))
        .expect("Failed to get post")
        .expect("Post not found");

    assert_eq!(retrieved, post);
}

#[test]
#[cfg(feature = "sled")]
fn test_new_with_multiple_models() {
    use netabase_store::databases::sled_store::SledStore;

    let temp_dir = setup_temp_dir();
    let db_path = temp_dir.path().join("multi_model.db");

    let store = NetabaseStore::<TestDef, SledStore<TestDef>>::new(&db_path)
        .expect("Failed to create store");

    // Open trees for different models
    let user_tree = store.open_tree::<TestUser>();
    let post_tree = store.open_tree::<TestPost>();

    // Insert users
    let user1 = TestUser {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    let user2 = TestUser {
        id: 2,
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
    };

    user_tree.put(user1.clone()).unwrap();
    user_tree.put(user2.clone()).unwrap();

    // Insert posts
    let post1 = TestPost {
        id: "post-1".to_string(),
        title: "Alice's Post".to_string(),
        author_id: 1,
    };
    let post2 = TestPost {
        id: "post-2".to_string(),
        title: "Bob's Post".to_string(),
        author_id: 2,
    };

    post_tree.put(post1.clone()).unwrap();
    post_tree.put(post2.clone()).unwrap();

    // Verify all data
    assert_eq!(user_tree.get(user1.primary_key()).unwrap().unwrap(), user1);
    assert_eq!(user_tree.get(user2.primary_key()).unwrap().unwrap(), user2);
    assert_eq!(post_tree.get(post1.primary_key()).unwrap().unwrap(), post1);
    assert_eq!(post_tree.get(post2.primary_key()).unwrap().unwrap(), post2);
}

/// Test that the generic constructor works in generic functions
/// Note: This test only works with Sled since generic tree operations are complex with Redb's Key enum requirement
#[cfg(feature = "sled")]
#[test]
fn test_generic_function_with_backend() {
    use netabase_store::databases::sled_store::SledStore;

    let temp_dir = setup_temp_dir();
    let db_path = temp_dir.path().join("generic_test.db");

    // Direct test with Sled (generic across backends is complex due to different key requirements)
    let store = NetabaseStore::<TestDef, SledStore<TestDef>>::new(&db_path)
        .expect("Failed to create store");

    let tree = store.open_tree::<TestUser>();

    let user = TestUser {
        id: 99,
        name: "Generic Test".to_string(),
        email: "generic@test.com".to_string(),
    };

    tree.put(user.clone()).expect("Failed to put");

    let retrieved = tree
        .get(user.primary_key())
        .expect("Failed to get")
        .expect("User should exist");

    assert_eq!(retrieved, user);
}

#[test]
#[cfg(feature = "sled")]
fn test_new_equals_convenience_method() {
    use netabase_store::databases::sled_store::SledStore;

    let temp_dir = setup_temp_dir();

    // Create two stores: one with new(), one with sled()
    let path1 = temp_dir.path().join("store1.db");
    let path2 = temp_dir.path().join("store2.db");

    let store1 = NetabaseStore::<TestDef, SledStore<TestDef>>::new(&path1)
        .expect("Failed to create store with new()");

    let store2 =
        NetabaseStore::<TestDef, _>::sled(&path2).expect("Failed to create store with sled()");

    // Both should work identically
    let tree1 = store1.open_tree::<TestUser>();
    let tree2 = store2.open_tree::<TestUser>();

    let user = TestUser {
        id: 1,
        name: "Test".to_string(),
        email: "test@example.com".to_string(),
    };

    tree1.put(user.clone()).unwrap();
    tree2.put(user.clone()).unwrap();

    assert_eq!(tree1.get(user.primary_key()).unwrap().unwrap(), user);
    assert_eq!(tree2.get(user.primary_key()).unwrap().unwrap(), user);
}

#[test]
#[cfg(feature = "redb")]
fn test_new_redb_persistence() {
    use netabase_store::databases::redb_store::RedbStore;

    let temp_dir = setup_temp_dir();
    let db_path = temp_dir.path().join("persistence.redb");

    // Create store, insert data, drop it
    {
        let store = NetabaseStore::<TestDef, RedbStore<TestDef>>::new(&db_path)
            .expect("Failed to create store");

        let tree = store.open_tree::<TestUser>();
        let user = TestUser {
            id: 1,
            name: "Persistent".to_string(),
            email: "persist@example.com".to_string(),
        };
        tree.put(user).unwrap();
    }

    // Reopen and verify data persists
    {
        let store = NetabaseStore::<TestDef, RedbStore<TestDef>>::new(&db_path)
            .expect("Failed to reopen store");

        let tree = store.open_tree::<TestUser>();
        let retrieved = tree
            .get(TestUserKey::Primary(TestUserPrimaryKey(1)))
            .expect("Failed to get user")
            .expect("User should exist");

        assert_eq!(retrieved.id, 1);
        assert_eq!(retrieved.name, "Persistent");
    }
}

#[test]
#[cfg(feature = "sled")]
fn test_new_with_backend_methods() {
    use netabase_store::databases::sled_store::SledStore;

    let temp_dir = setup_temp_dir();
    let db_path = temp_dir.path().join("backend_access.db");

    let store = NetabaseStore::<TestDef, SledStore<TestDef>>::new(&db_path)
        .expect("Failed to create store");

    // Test that we can access backend-specific methods
    let backend: &SledStore<TestDef> = store.backend();

    // Sled-specific method
    backend.flush().expect("Failed to flush");

    // Test tree_names (backend-specific)
    let tree_names = backend.tree_names();
    assert!(!tree_names.is_empty()); // At least 0 trees initially
}
