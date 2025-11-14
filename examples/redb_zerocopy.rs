//! Zero-copy redb backend example
//!
//! This example demonstrates:
//! - Creating a database with the new config API
//! - Explicit transaction management
//! - Bulk operations
//! - Transaction isolation
//! - Helper functions for common patterns

use netabase_store::config::FileConfig;
use netabase_store::databases::redb_zerocopy::*;
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
    println!("🚀 Redb Zero-Copy Backend Example\n");

    // Create a database using the new config API
    let db_path = "/tmp/redb_zerocopy_example.db";
    println!("📁 Creating database at: {}", db_path);

    let config = FileConfig::builder()
        .path(db_path.into())
        .truncate(true)  // Start fresh each time
        .build();

    let store = <RedbStoreZeroCopy<AppDef> as BackendStore<AppDef>>::new(config)?;
    println!("✅ Database created successfully\n");

    // Example 1: Basic write transaction
    println!("📝 Example 1: Basic Write Transaction");
    {
        let mut txn = store.begin_write()?;
        let mut tree = txn.open_tree::<User>()?;

        tree.put(User {
            id: 1,
            name: "Alice Johnson".to_string(),
            email: "alice@example.com".to_string(),
            age: 30,
        })?;

        println!("  ✓ Inserted Alice");
        drop(tree);
        txn.commit()?;
        println!("  ✓ Transaction committed\n");
    }

    // Example 2: Bulk insert with single transaction
    println!("📝 Example 2: Bulk Insert (Single Transaction)");
    {
        let mut txn = store.begin_write()?;
        let mut tree = txn.open_tree::<User>()?;

        let users = vec![
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

        tree.put_many(users)?;
        println!("  ✓ Inserted 2 users in single transaction");
        drop(tree);
        txn.commit()?;
        println!("  ✓ Transaction committed\n");
    }

    // Example 3: Read transaction
    println!("📝 Example 3: Read Transaction");
    {
        let txn = store.begin_read()?;
        let tree = txn.open_tree::<User>()?;

        if let Some(user) = tree.get(&UserPrimaryKey(1))? {
            println!("  ✓ Found user: {} ({})", user.name, user.email);
        }

        println!("  ✓ Total users: {}\n", tree.len()?);
    }

    // Example 4: Secondary key query
    println!("📝 Example 4: Secondary Key Query");
    {
        let txn = store.begin_read()?;
        let tree = txn.open_tree::<User>()?;

        let results = tree.get_by_secondary_key(&UserSecondaryKeys::Email(
            UserEmailSecondaryKey("bob@example.com".to_string()),
        ))?;

        println!("  ✓ Query results: {} matches", results.len());
        for guard_result in results {
            let prim_key = guard_result?.value();
            if let Some(user) = tree.get(&prim_key)? {
                println!("    - {} (ID: {}, Age: {})", user.name, user.id, user.age);
            }
        }
        println!();
    }

    // Example 5: Using helper functions
    println!("📝 Example 5: Helper Functions");
    {
        // with_write_transaction automatically commits on success
        let count = with_write_transaction(&store, |txn| {
            let mut tree = txn.open_tree::<User>()?;
            tree.put(User {
                id: 4,
                name: "Diana Prince".to_string(),
                email: "diana@example.com".to_string(),
                age: 28,
            })?;
            Ok(tree.len()?)
        })?;

        println!("  ✓ Inserted Diana, total users: {}", count);

        // with_read_transaction for read-only operations
        let alice_age = with_read_transaction(&store, |txn| {
            let tree = txn.open_tree::<User>()?;
            let user = tree.get(&UserPrimaryKey(1))?.unwrap();
            Ok(user.age)
        })?;

        println!("  ✓ Alice's age: {}\n", alice_age);
    }

    // Example 6: Bulk removal
    println!("📝 Example 6: Bulk Removal");
    {
        let mut txn = store.begin_write()?;
        let mut tree = txn.open_tree::<User>()?;

        let removed = tree.remove_many(vec![UserPrimaryKey(2), UserPrimaryKey(3)])?;
        println!("  ✓ Removed {} users", removed.len());
        println!("  ✓ Remaining users: {}", tree.len()?);
        drop(tree);
        txn.commit()?;
        println!("  ✓ Transaction committed\n");
    }

    println!("✅ All examples completed successfully!");
    println!("💡 Database file: {}", db_path);

    Ok(())
}
