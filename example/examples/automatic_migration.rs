//! Example demonstrating automatic version detection and migration.
//!
//! This example shows how the generated `Family` enum enables transparent
//! migration when reading data from different schema versions.

use netabase_store::databases::redb::RedbStore;
use netabase_store::traits::database::store::NBStore;
use netabase_store::traits::migration::MigrateFrom;
use serde::{Deserialize, Serialize};

#[netabase_macros::netabase_definition(AutoMigrationDemo)]
mod demo {
    use super::*;

    // Version 1: Simple user with full name
    #[derive(
        netabase_macros::NetabaseModel,
        Debug,
        Clone,
        Serialize,
        Deserialize,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
    )]
    #[netabase_version(family = "User", version = 1)]
    pub struct UserV1 {
        #[primary_key]
        pub id: String,
        pub name: String,
    }

    // Version 2: Split name into first/last
    #[derive(
        netabase_macros::NetabaseModel,
        Debug,
        Clone,
        Serialize,
        Deserialize,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
    )]
    #[netabase_version(family = "User", version = 2, current)]
    pub struct User {
        #[primary_key]
        pub id: String,
        pub first_name: String,
        pub last_name: String,
    }

    // Migration implementation
    impl MigrateFrom<UserV1> for User {
        fn migrate_from(old: UserV1) -> Self {
            let parts: Vec<&str> = old.name.split_whitespace().collect();
            User {
                id: old.id,
                first_name: parts.first().copied().unwrap_or("").to_string(),
                last_name: parts.get(1).copied().unwrap_or("").to_string(),
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use demo::*;

    println!("=== Automatic Migration Demo ===\n");

    println!("Demonstrating UserFamily enum (auto-generated):\n");

    // Create some test data
    let v1_user = UserV1 {
        id: UserID("alice".into()),
        name: "Alice Wonderland".into(),
    };

    let v2_user = User {
        id: UserID("bob".into()),
        first_name: "Bob".into(),
        last_name: "Builder".into(),
    };

    println!("1. Serialize V1 data:");
    let v1_bytes = postcard::to_allocvec(&v1_user)?;
    println!("   UserV1 bytes: {} bytes", v1_bytes.len());

    println!("\n2. Try to detect version from V1 bytes:");
    match UserFamily::try_from_bytes(&v1_bytes) {
        Ok(family) => {
            println!("   ✓ Detected version: {}", family.version());
            println!("   Model: {}", family.model_name());
            
            println!("\n3. Migrate to current version:");
            let current = family.to_current();
            println!("   Migrated to: {:?}", current);
            println!("   first_name: {}", current.first_name);
            println!("   last_name: {}", current.last_name);
            
            assert_eq!(current.first_name, "Alice");
            assert_eq!(current.last_name, "Wonderland");
        }
        Err(e) => {
            println!("   ✗ Failed: {:?}", e);
        }
    }

    println!("\n4. Serialize V2 data:");
    let v2_bytes = postcard::to_allocvec(&v2_user)?;
    println!("   User (V2) bytes: {} bytes", v2_bytes.len());

    println!("\n5. Detect version from V2 bytes:");
    match UserFamily::try_from_bytes(&v2_bytes) {
        Ok(family) => {
            println!("   ✓ Detected version: {}", family.version());
            println!("   Model: {}", family.model_name());
            
            let current = family.to_current();
            println!("   Current: {:?}", current);
            
            assert_eq!(current.id, UserID("bob".into()));
        }
        Err(e) => {
            println!("   ✗ Failed: {:?}", e);
        }
    }

    println!("\n6. Demonstrating in database context:");
    {
        let (store, _temp) = RedbStore::<AutoMigrationDemo>::new_temporary()?;
        
        // Write current version
        let txn = store.begin_write()?;
        txn.create(&v2_user)?;
        txn.commit()?;
        
        // Read it back (uses family enum internally)
        let txn = store.begin_read()?;
        let read_user: User = txn.read(&UserID("bob".into()))?.expect("Should exist");
        println!("   ✓ Read from database: {} {}", read_user.first_name, read_user.last_name);
    }

    println!("\n✅ Automatic migration demonstration complete!");
    println!("\nHow it works:");
    println!("  1. Macro generates UserFamily enum with V1 and V2 variants");
    println!("  2. try_from_bytes() attempts deserialization with each version");
    println!("  3. to_current() migrates old versions to current");
    println!("  4. redb::Value::from_bytes() uses UserFamily internally");
    println!("\nBenefits:");
    println!("  - No manual version tracking needed");
    println!("  - Database can contain mixed versions");
    println!("  - Zero-downtime deployments possible");
    println!("  - Transparent to application code");

    Ok(())
}
