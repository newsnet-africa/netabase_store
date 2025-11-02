///! Example demonstrating the unified trait API for netabase_store
///!
///! This example shows how to use the NetabaseTreeSync trait to write
///! backend-agnostic code that works with both SledStore and RedbStore.

use netabase_store::databases::sled_store::SledStore;
use netabase_store::databases::redb_store::RedbStore;
use netabase_store::traits::tree::NetabaseTreeSync;
use netabase_store::netabase_definition_module;
use crate::definitions::*;

// Define a simple data model
#[netabase_definition_module(Definition, DefinitionKeys)]
pub mod definitions {
    use netabase_store::{NetabaseModel, netabase};

    #[derive(NetabaseModel, bincode::Encode, bincode::Decode, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    #[netabase(Definition)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        #[secondary_key]
        pub email: String,
        pub name: String,
    }
}

// Generic function that works with any backend implementing NetabaseTreeSync
fn perform_crud_operations<T>(tree: &T) -> Result<(), Box<dyn std::error::Error>>
where
    T: NetabaseTreeSync<Definition, User, PrimaryKey = UserPrimaryKey, SecondaryKeys = UserSecondaryKeys>,
{
    println!("Testing unified API with generic function...");

    // Insert a user
    let user = User {
        id: 1,
        email: "alice@example.com".to_string(),
        name: "Alice".to_string(),
    };
    tree.put(user.clone())?;
    println!("  ✓ Inserted user: {:?}", user);

    // Retrieve the user
    let retrieved = tree.get(UserPrimaryKey(1))?;
    assert_eq!(retrieved, Some(user.clone()));
    println!("  ✓ Retrieved user: {:?}", retrieved.unwrap());

    // Query by secondary key (email)
    let by_email = tree.get_by_secondary_key(UserSecondaryKeys::Email(
        UserEmailSecondaryKey("alice@example.com".to_string()),
    ))?;
    assert_eq!(by_email.len(), 1);
    println!("  ✓ Found user by email: {:?}", by_email[0]);

    // Check length
    let len = tree.len()?;
    assert_eq!(len, 1);
    println!("  ✓ Tree length: {}", len);

    // Remove the user
    let removed = tree.remove(UserPrimaryKey(1))?;
    assert_eq!(removed, Some(user));
    println!("  ✓ Removed user");

    // Verify it's empty
    assert!(tree.is_empty()?);
    println!("  ✓ Tree is empty");

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Unified API Example\n");
    println!("===================\n");

    // Test with SledStore
    println!("1. Testing with SledStore:");
    let sled_dir = tempfile::tempdir()?;
    let sled_store = SledStore::<Definition>::new(sled_dir.path())?;
    let sled_tree = sled_store.open_tree::<User>();
    perform_crud_operations(&sled_tree)?;
    println!();

    // Test with RedbStore
    println!("2. Testing with RedbStore:");
    let redb_dir = tempfile::tempdir()?;
    let redb_path = redb_dir.path().join("test.redb");
    let redb_store = RedbStore::<Definition>::new(&redb_path)?;
    let redb_tree = redb_store.open_tree::<User>();
    perform_crud_operations(&redb_tree)?;
    println!();

    println!("✓ All tests passed!");
    println!("\nThe same generic code worked with both SledStore and RedbStore!");

    Ok(())
}
