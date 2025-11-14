//! Basic example of using the redb backend
//!
//! This example demonstrates:
//! - Creating a database with the new config API
//! - Defining models with primary and secondary keys
//! - CRUD operations
//! - Secondary index queries

use netabase_store::config::FileConfig;
use netabase_store::databases::redb_store::RedbStore;
use netabase_store::traits::backend_store::BackendStore;
use netabase_store::{netabase, netabase_definition_module, NetabaseModel};

// Define the database schema
#[netabase_definition_module(AppDef, AppKeys)]
mod models {
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
    #[netabase(AppDef)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub name: String,
        #[secondary_key]
        pub email: String,
        pub age: u32,
    }
}

use models::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Redb Backend Example\n");

    // Create a database using the new config API
    let db_path = "/tmp/redb_example.db";
    println!("📁 Creating database at: {}", db_path);

    let config = FileConfig::builder()
        .path(db_path.into())
        .truncate(true)  // Start fresh each time
        .build();

    let store = <RedbStore<AppDef> as BackendStore<AppDef>>::new(config)?;
    let tree = store.open_tree::<User>();

    println!("✅ Database and tree created successfully\n");

    // Create some users
    println!("➕ Inserting users...");
    let users = vec![
        User {
            id: 1,
            name: "Alice Johnson".to_string(),
            email: "alice@example.com".to_string(),
            age: 30,
        },
        User {
            id: 2,
            name: "Bob Smith".to_string(),
            email: "bob@example.com".to_string(),
            age: 25,
        },
        User {
            id: 3,
            name: "Charlie Brown".to_string(),
            email: "charlie@example.com".to_string(),
            age: 35,
        },
    ];

    for user in &users {
        tree.put(user.clone())?;
        println!("  ✓ Inserted: {} ({})", user.name, user.email);
    }

    println!("\n🔍 Retrieving users by primary key...");

    // Retrieve by primary key using the Keys enum
    let user = tree.get(UserKey::Primary(UserPrimaryKey(1)))?;
    match user {
        Some(u) => println!("  ✓ Found user ID 1: {} - {}", u.name, u.email),
        None => println!("  ✗ User not found"),
    }

    println!("\n🔎 Querying by secondary key (email)...");

    // Query by secondary key (email)
    // Note: The macro generates newtype wrappers for secondary keys
    let results = tree.get_by_secondary_key(UserSecondaryKeys::Email(
        UserEmailSecondaryKey("bob@example.com".to_string()),
    ))?;

    for user in results {
        println!("  ✓ Found: {} (ID: {}, Age: {})", user.name, user.id, user.age);
    }

    println!("\n🗑️  Removing user...");

    // Remove a user
    let removed = tree.remove(UserKey::Primary(UserPrimaryKey(2)))?;
    match removed {
        Some(u) => println!("  ✓ Removed: {}", u.name),
        None => println!("  ✗ User not found"),
    }

    // Verify removal
    let user = tree.get(UserKey::Primary(UserPrimaryKey(2)))?;
    assert!(user.is_none(), "User should be removed");
    println!("  ✓ Verified: User 2 no longer exists");

    println!("\n✅ All operations completed successfully!");
    println!("\n💡 Database file: {}", db_path);

    Ok(())
}
