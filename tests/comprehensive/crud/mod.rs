//! Comprehensive CRUD operation tests
//!
//! Tests all Create, Read, Update, Delete operations across all backends
//! with thorough state verification before and after each operation.

#![cfg(not(target_arch = "wasm32"))]

use netabase_store::{netabase_definition_module, NetabaseModel, netabase};
use netabase_store::traits::tree::NetabaseTreeSync;
use netabase_store::traits::model::NetabaseModelTrait;
use netabase_store::error::NetabaseError;

use super::utils::{TestBackend, DatabaseState, verify_tree_contents, verify_clean_state};

// Test schema
#[netabase_definition_module(CrudTestDefinition, CrudTestKeys)]
mod test_models {
    use super::*;

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
    #[netabase(CrudTestDefinition)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub name: String,
        pub email: String,
        #[secondary_key]
        pub age: u32,
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
    #[netabase(CrudTestDefinition)]
    pub struct Post {
        #[primary_key]
        pub id: String,
        pub title: String,
        pub content: String,
        #[secondary_key]
        pub published: bool,
    }
}

use test_models::*;

/// Test basic CRUD operations with state verification
pub fn test_basic_crud<B>() -> Result<(), NetabaseError>
where
    B: TestBackend<CrudTestDefinition>
        + netabase_store::traits::store_ops::OpenTree<CrudTestDefinition, User>,
{
    println!("\n=== Testing CRUD operations on {} ===", B::backend_name());

    let store = B::create_temp()?;

    // Verify initial clean state
    verify_clean_state(&store)?;
    println!("✓ Initial state is clean");

    let tree = store.open_tree::<User>();

    // === CREATE ===
    println!("\n--- Testing CREATE ---");
    let state_before = DatabaseState::capture(&store)?;

    let user1 = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        age: 30,
    };

    tree.put(user1.clone())?;
    println!("✓ Inserted user1");

    let state_after = DatabaseState::capture(&store)?;
    let diff = state_before.diff(&state_after);
    assert_eq!(diff.entry_count_change, 2, "Should add 1 primary + 1 secondary entry");
    println!("✓ State verification passed: {} entries added", diff.entry_count_change);

    // Verify tree has exactly 1 entry
    verify_tree_contents::<CrudTestDefinition, _, User>(&store, 1)?;
    println!("✓ Primary tree has correct count");

    // === READ ===
    println!("\n--- Testing READ ---");
    let retrieved = tree.get(user1.primary_key())?;
    assert_eq!(retrieved, Some(user1.clone()), "Should retrieve inserted user");
    println!("✓ Retrieved user matches original");

    // Test read non-existent
    let non_existent = tree.get(UserPrimaryKey(999))?;
    assert_eq!(non_existent, None, "Non-existent user should return None");
    println!("✓ Non-existent read returns None");

    // === UPDATE ===
    println!("\n--- Testing UPDATE ---");
    let mut user1_updated = user1.clone();
    user1_updated.name = "Alice Updated".to_string();
    user1_updated.age = 31;

    tree.put(user1_updated.clone())?;
    println!("✓ Updated user1");

    let retrieved_updated = tree.get(user1.primary_key())?;
    assert_eq!(
        retrieved_updated,
        Some(user1_updated.clone()),
        "Should retrieve updated user"
    );
    println!("✓ Retrieved updated user matches");

    // Verify count didn't change (update, not insert)
    verify_tree_contents::<CrudTestDefinition, _, User>(&store, 1)?;
    println!("✓ Count unchanged after update");

    // === DELETE ===
    println!("\n--- Testing DELETE ---");
    let state_before_delete = DatabaseState::capture(&store)?;

    let removed = tree.remove(user1.primary_key())?;
    assert_eq!(removed, Some(user1_updated), "Should return removed user");
    println!("✓ Deleted user1, returned value matches");

    let state_after_delete = DatabaseState::capture(&store)?;
    let diff_delete = state_before_delete.diff(&state_after_delete);
    assert_eq!(
        diff_delete.entry_count_change, -2,
        "Should remove 1 primary + 1 secondary entry"
    );
    println!("✓ State verification passed: {} entries removed", diff_delete.entry_count_change.abs());

    // Verify tree is now empty
    verify_tree_contents::<CrudTestDefinition, _, User>(&store, 0)?;
    println!("✓ Tree is empty after delete");

    // Test delete non-existent
    let removed_none = tree.remove(UserPrimaryKey(999))?;
    assert_eq!(removed_none, None, "Deleting non-existent should return None");
    println!("✓ Deleting non-existent returns None");

    println!("\n✅ All CRUD tests passed for {}", B::backend_name());
    Ok(())
}

/// Test CRUD operations with multiple models
pub fn test_multi_model_crud<B>() -> Result<(), NetabaseError>
where
    B: TestBackend<CrudTestDefinition>
        + netabase_store::traits::store_ops::OpenTree<CrudTestDefinition, User>
        + netabase_store::traits::store_ops::OpenTree<CrudTestDefinition, Post>,
{
    println!("\n=== Testing multi-model CRUD on {} ===", B::backend_name());

    let store = B::create_temp()?;
    verify_clean_state(&store)?;

    let user_tree = store.open_tree::<User>();
    let post_tree = store.open_tree::<Post>();

    // Insert users
    for i in 1..=10 {
        let user = User {
            id: i,
            name: format!("User {}", i),
            email: format!("user{}@example.com", i),
            age: 20 + (i as u32 % 50),
        };
        user_tree.put(user)?;
    }
    println!("✓ Inserted 10 users");

    // Insert posts
    for i in 1..=5 {
        let post = Post {
            id: format!("post-{}", i),
            title: format!("Post {}", i),
            content: format!("Content of post {}", i),
            published: i % 2 == 0,
        };
        post_tree.put(post)?;
    }
    println!("✓ Inserted 5 posts");

    // Verify counts
    verify_tree_contents::<CrudTestDefinition, _, User>(&store, 10)?;
    verify_tree_contents::<CrudTestDefinition, _, Post>(&store, 5)?;
    println!("✓ Both trees have correct counts");

    // Verify total entries (10 users + 10 secondary + 5 posts + 5 secondary = 30)
    let state = DatabaseState::capture(&store)?;
    assert_eq!(state.total_entries, 30, "Total entries should be 30");
    println!("✓ Total entry count is correct");

    // Clear one tree
    user_tree.clear()?;
    verify_tree_contents::<CrudTestDefinition, _, User>(&store, 0)?;
    verify_tree_contents::<CrudTestDefinition, _, Post>(&store, 5)?;
    println!("✓ Cleared user tree, post tree unaffected");

    println!("\n✅ All multi-model CRUD tests passed for {}", B::backend_name());
    Ok(())
}

/// Test edge cases and boundary conditions
pub fn test_crud_edge_cases<B>() -> Result<(), NetabaseError>
where
    B: TestBackend<CrudTestDefinition>
        + netabase_store::traits::store_ops::OpenTree<CrudTestDefinition, User>,
{
    println!("\n=== Testing CRUD edge cases on {} ===", B::backend_name());

    let store = B::create_temp()?;
    let tree = store.open_tree::<User>();

    // Test empty strings
    let user_empty = User {
        id: 1,
        name: String::new(),
        email: String::new(),
        age: 0,
    };
    tree.put(user_empty.clone())?;
    let retrieved = tree.get(UserPrimaryKey(1))?;
    assert_eq!(retrieved, Some(user_empty.clone()));
    println!("✓ Empty strings handled correctly");

    // Test very long strings
    let user_long = User {
        id: 2,
        name: "A".repeat(10000),
        email: "B".repeat(10000),
        age: u32::MAX,
    };
    tree.put(user_long.clone())?;
    let retrieved = tree.get(UserPrimaryKey(2))?;
    assert_eq!(retrieved, Some(user_long.clone()));
    println!("✓ Long strings handled correctly");

    // Test boundary values
    let user_boundary = User {
        id: u64::MAX,
        name: "Boundary".to_string(),
        email: "boundary@test.com".to_string(),
        age: u32::MAX,
    };
    tree.put(user_boundary.clone())?;
    let retrieved = tree.get(UserPrimaryKey(u64::MAX))?;
    assert_eq!(retrieved, Some(user_boundary));
    println!("✓ Boundary values handled correctly");

    verify_tree_contents::<CrudTestDefinition, _, User>(&store, 3)?;

    println!("\n✅ All edge case tests passed for {}", B::backend_name());
    Ok(())
}

// Backend-specific test implementations
#[cfg(feature = "sled")]
mod sled_tests {
    use super::*;
    use netabase_store::databases::sled_store::SledStore;
    use netabase_store::traits::introspection::DatabaseIntrospection;

    impl TestBackend<CrudTestDefinition> for SledStore<CrudTestDefinition> {
        fn create_temp() -> Result<Self, NetabaseError> {
            SledStore::temp()
        }

        fn backend_name() -> &'static str {
            "SledStore"
        }
    }

    #[test]
    fn test_sled_basic_crud() {
        test_basic_crud::<SledStore<CrudTestDefinition>>().unwrap();
    }

    #[test]
    fn test_sled_multi_model_crud() {
        test_multi_model_crud::<SledStore<CrudTestDefinition>>().unwrap();
    }

    #[test]
    fn test_sled_crud_edge_cases() {
        test_crud_edge_cases::<SledStore<CrudTestDefinition>>().unwrap();
    }
}

#[cfg(feature = "redb")]
mod redb_tests {
    use super::*;
    use netabase_store::databases::redb_store::RedbStore;
    use netabase_store::traits::introspection::DatabaseIntrospection;

    impl TestBackend<CrudTestDefinition> for RedbStore<CrudTestDefinition> {
        fn create_temp() -> Result<Self, NetabaseError> {
            let temp_dir = tempfile::tempdir()
                .map_err(|e| NetabaseError::Storage(e.to_string()))?;
            let path = temp_dir.path().join("test.redb");
            RedbStore::new(path)
        }

        fn backend_name() -> &'static str {
            "RedbStore"
        }
    }

    #[test]
    fn test_redb_basic_crud() {
        test_basic_crud::<RedbStore<CrudTestDefinition>>().unwrap();
    }

    #[test]
    fn test_redb_multi_model_crud() {
        test_multi_model_crud::<RedbStore<CrudTestDefinition>>().unwrap();
    }

    #[test]
    fn test_redb_crud_edge_cases() {
        test_crud_edge_cases::<RedbStore<CrudTestDefinition>>().unwrap();
    }
}
