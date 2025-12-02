//! Test the new introspection API

#![cfg(not(target_arch = "wasm32"))]

use netabase_store::{netabase_definition_module, NetabaseModel, netabase};
use netabase_store::traits::introspection::{DatabaseIntrospection, TreeType};
use netabase_store::traits::tree::NetabaseTreeSync;

#[netabase_definition_module(IntrospectionTestDef, IntrospectionTestKeys)]
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
    #[netabase(IntrospectionTestDef)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub name: String,
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
    #[netabase(IntrospectionTestDef)]
    pub struct Post {
        #[primary_key]
        pub id: String,
        pub title: String,
        #[secondary_key]
        pub published: bool,
    }
}

use test_models::*;

#[cfg(feature = "sled")]
#[test]
fn test_sled_introspection() {
    use netabase_store::databases::sled_store::SledStore;

    let store = SledStore::<IntrospectionTestDef>::temp().unwrap();

    // Initially, all trees should exist but be empty
    let all_trees = store.list_all_trees().unwrap();
    println!("\n=== Initial Sled Store State ===");
    for tree in &all_trees {
        println!("  Tree: {} ({:?}), Count: {:?}", tree.name, tree.tree_type, tree.entry_count);
    }

    // Should have 2 model trees + 2 secondary trees = 4 total
    assert!(all_trees.len() >= 4, "Should have at least 4 trees (2 models + 2 secondary)");

    // Check model trees exist
    let model_trees = store.list_model_trees().unwrap();
    assert_eq!(model_trees.len(), 2, "Should have 2 model trees");
    assert!(model_trees.iter().any(|t| t.name == "User"));
    assert!(model_trees.iter().any(|t| t.name == "Post"));

    // Check secondary trees exist
    let secondary_trees = store.list_secondary_trees().unwrap();
    assert_eq!(secondary_trees.len(), 2, "Should have 2 secondary index trees");
    assert!(secondary_trees.iter().any(|t| t.name == "User_secondary"));
    assert!(secondary_trees.iter().any(|t| t.name == "Post_secondary"));

    // Insert some data
    let user_tree = store.open_tree::<User>();
    user_tree.put(User { id: 1, name: "Alice".into(), age: 30 }).unwrap();
    user_tree.put(User { id: 2, name: "Bob".into(), age: 25 }).unwrap();

    // Verify counts updated
    let user_count = store.tree_entry_count("User").unwrap();
    assert_eq!(user_count, 2, "User tree should have 2 entries");

    let user_secondary_count = store.tree_entry_count("User_secondary").unwrap();
    assert_eq!(user_secondary_count, 2, "User_secondary should have 2 entries");

    // Test database stats
    let stats = store.database_stats().unwrap();
    println!("\n=== Sled Database Stats ===");
    println!("  Total trees: {}", stats.total_trees);
    println!("  Model trees: {}", stats.model_trees);
    println!("  Secondary trees: {}", stats.secondary_trees);
    println!("  Total entries: {}", stats.total_entries);
    assert_eq!(stats.model_trees, 2);
    assert_eq!(stats.secondary_trees, 2);
    assert!(stats.total_entries >= 4); // At least 2 primary + 2 secondary

    println!("\n✅ Sled introspection test passed!");
}

#[cfg(feature = "redb")]
#[test]
fn test_redb_introspection() {
    use netabase_store::databases::redb_store::RedbStore;

    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.redb");
    let store = RedbStore::<IntrospectionTestDef>::new(path).unwrap();

    // Initially, all trees should exist but be empty
    let all_trees = store.list_all_trees().unwrap();
    println!("\n=== Initial Redb Store State ===");
    for tree in &all_trees {
        println!("  Tree: {} ({:?}), Count: {:?}", tree.name, tree.tree_type, tree.entry_count);
    }

    // Should have 2 model trees + 2 secondary trees = 4 total
    assert!(all_trees.len() >= 4, "Should have at least 4 trees");

    // Check model trees exist
    let model_trees = store.list_model_trees().unwrap();
    assert_eq!(model_trees.len(), 2, "Should have 2 model trees");

    // Check secondary trees exist
    let secondary_trees = store.list_secondary_trees().unwrap();
    assert_eq!(secondary_trees.len(), 2, "Should have 2 secondary index trees");

    // Insert some data
    let user_tree = store.open_tree::<User>();
    user_tree.put(User { id: 1, name: "Alice".into(), age: 30 }).unwrap();
    user_tree.put(User { id: 2, name: "Bob".into(), age: 25 }).unwrap();
    user_tree.put(User { id: 3, name: "Charlie".into(), age: 35 }).unwrap();

    // Verify counts updated
    let user_count = store.tree_entry_count("User").unwrap();
    assert_eq!(user_count, 3, "User tree should have 3 entries");

    let user_secondary_count = store.tree_entry_count("User_secondary").unwrap();
    assert_eq!(user_secondary_count, 3, "User_secondary should have 3 entries");

    // Test tree_keys_raw
    let user_keys = store.tree_keys_raw("User").unwrap();
    assert_eq!(user_keys.len(), 3, "Should have 3 keys");

    // Test database stats
    let stats = store.database_stats().unwrap();
    println!("\n=== Redb Database Stats ===");
    println!("  Total trees: {}", stats.total_trees);
    println!("  Model trees: {}", stats.model_trees);
    println!("  Secondary trees: {}", stats.secondary_trees);
    println!("  Total entries: {}", stats.total_entries);
    assert_eq!(stats.model_trees, 2);
    assert_eq!(stats.secondary_trees, 2);
    assert!(stats.total_entries >= 6); // At least 3 primary + 3 secondary

    println!("\n✅ Redb introspection test passed!");
}
