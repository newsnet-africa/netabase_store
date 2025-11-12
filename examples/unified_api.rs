///! Example demonstrating the unified NetabaseStore API
///!
///! This example shows how to use NetabaseStore<D, Backend> to write
///! backend-agnostic code that works with any storage backend.

use netabase_store::{netabase_definition_module, NetabaseStore};

// Define a simple data model
#[netabase_definition_module(Definition, DefinitionKeys)]
pub mod definitions {
    use netabase_store::{netabase, NetabaseModel};

    #[derive(
        NetabaseModel,
        bincode::Encode,
        bincode::Decode,
        Clone,
        Debug,
        PartialEq,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(Definition)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        #[secondary_key]
        pub email: String,
        pub name: String,
    }
}

use definitions::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Unified NetabaseStore API Example\n");
    println!("==================================\n");

    // Create a user to test with
    let user = User {
        id: 1,
        email: "alice@example.com".to_string(),
        name: "Alice".to_string(),
    };

    // Test with SledStore using NetabaseStore wrapper
    println!("1. Testing with NetabaseStore<Sled>:");
    let sled_dir = tempfile::tempdir()?;
    let sled_store = NetabaseStore::<Definition, _>::sled(sled_dir.path())?;
    let sled_tree = sled_store.open_tree::<User>();

    sled_tree.put(user.clone())?;
    println!("  ✓ Inserted user");

    let retrieved = sled_tree.get(UserPrimaryKey(1))?;
    assert_eq!(retrieved, Some(user.clone()));
    println!("  ✓ Retrieved user: {:?}", retrieved.unwrap());

    let by_email = sled_tree.get_by_secondary_key(UserSecondaryKeys::Email(
        UserEmailSecondaryKey("alice@example.com".to_string()),
    ))?;
    assert_eq!(by_email.len(), 1);
    println!("  ✓ Found user by email");

    // Access Sled-specific features
    println!("  ℹ Sled-specific: Flushed {} bytes", sled_store.flush()?);
    println!();

    // Test with RedbStore using NetabaseStore wrapper
    println!("2. Testing with NetabaseStore<Redb>:");
    let redb_dir = tempfile::tempdir()?;
    let redb_path = redb_dir.path().join("test.redb");
    let redb_store = NetabaseStore::<Definition, _>::redb(&redb_path)?;
    let redb_tree = redb_store.open_tree::<User>();

    redb_tree.put(user.clone())?;
    println!("  ✓ Inserted user");

    let retrieved = redb_tree.get(UserPrimaryKey(1))?;
    assert_eq!(retrieved, Some(user.clone()));
    println!("  ✓ Retrieved user: {:?}", retrieved.unwrap());

    let by_email = redb_tree.get_by_secondary_key(UserSecondaryKeys::Email(
        UserEmailSecondaryKey("alice@example.com".to_string()),
    ))?;
    assert_eq!(by_email.len(), 1);
    println!("  ✓ Found user by email");

    // Access Redb-specific features
    println!("  ℹ Redb-specific: Tree names: {:?}", redb_store.tree_names());
    println!();

    println!("✓ All tests passed!");
    println!("\nThe same API worked with both backends using NetabaseStore!");
    println!("Backend-specific features are still accessible when needed.");

    Ok(())
}
