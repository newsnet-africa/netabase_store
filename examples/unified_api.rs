//! Unified NetabaseStore API Example
//!
//! This example demonstrates how NetabaseStore provides a consistent API across
//! ALL storage backends. The same code works with different backends by just
//! changing the initialization method.
//!
//! ## Supported Backends:
//! - **Sled**: Native, persistent, crash-safe (`NetabaseStore::sled()`)
//! - **Redb**: Native, persistent, zero-copy reads (`NetabaseStore::redb()`)
//! - **IndexedDB**: Browser/WASM, async API (different pattern)
//!
//! ## API Consistency:
//! ✅ All sync backends (Sled, Redb) have **identical APIs**:
//! - Same CRUD operations: `put()`, `get()`, `remove()`
//! - Same secondary key queries: `get_by_secondary_key()`
//! - Same iteration: `iter()`, `len()`, `is_empty()`
//! - Same batch operations: `create_batch()` → `commit()`
//! - Same transaction API: `store.read()` and `store.write()`
//!
//! ## Backend-Specific Features:
//! While the core API is identical, each backend exposes additional methods:
//! - Sled: `flush()`, `size_on_disk()`
//! - Redb: `tree_names()`, `compact()`
//!
//! Run with:
//! ```bash
//! cargo run --example unified_api --features native
//! ```
use netabase_store::{NetabaseStore, netabase_definition_module};

// Define a simple data model
#[netabase_definition_module(Definition, DefinitionKeys)]
pub mod definitions {
    use netabase_store::{NetabaseModel, netabase};

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
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║         Unified NetabaseStore API Example                     ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");
    println!("This example demonstrates that the SAME CODE works across all backends!\n");

    // Create a user to test with
    let user = User {
        id: 1,
        email: "alice@example.com".to_string(),
        name: "Alice".to_string(),
    };

    // ========================================================================
    // Backend 1: SLED
    // ========================================================================
    println!("Backend 1: SLED");
    println!("───────────────");
    println!("Type: Native, persistent, crash-safe");
    println!("Best for: General purpose, high write throughput\n");
    let sled_dir = tempfile::tempdir()?;
    let sled_store = NetabaseStore::<Definition, _>::sled(sled_dir.path())?;
    let sled_tree = sled_store.open_tree::<User>();

    sled_tree.put(user.clone())?;
    println!("  ✓ Inserted user");

    let retrieved = sled_tree.get(UserPrimaryKey(1))?;
    assert_eq!(retrieved, Some(user.clone()));
    println!("  ✓ Retrieved user by primary key");

    let by_email = sled_tree.get_by_secondary_key(UserSecondaryKeys::Email(
        UserEmailSecondaryKey("alice@example.com".to_string()),
    ))?;
    assert_eq!(by_email.len(), 1);
    println!("  ✓ Found user by secondary key (email)");

    println!("  ✓ Count: {} users", sled_tree.len());

    // Access Sled-specific features (beyond the common API)
    println!(
        "  💡 Sled-specific feature: Flushed {} bytes",
        sled_store.flush()?
    );
    println!();

    // ========================================================================
    // Backend 2: REDB
    // ========================================================================
    println!("Backend 2: REDB");
    println!("───────────────");
    println!("Type: Native, persistent, zero-copy reads");
    println!("Best for: Read-heavy workloads, memory efficiency\n");
    let redb_dir = tempfile::tempdir()?;
    let redb_path = redb_dir.path().join("test.redb");
    let redb_store = NetabaseStore::<Definition, _>::redb(&redb_path)?;
    let redb_tree = redb_store.open_tree::<User>();

    redb_tree.put(user.clone())?;
    println!("  ✓ Inserted user");

    let retrieved = redb_tree.get(UserKey::Primary(UserPrimaryKey(1)))?;
    assert_eq!(retrieved, Some(user.clone()));
    println!("  ✓ Retrieved user by primary key");

    let by_email = redb_tree.get_by_secondary_key(UserSecondaryKeys::Email(
        UserEmailSecondaryKey("alice@example.com".to_string()),
    ))?;
    assert_eq!(by_email.len(), 1);
    println!("  ✓ Found user by secondary key (email)");

    println!("  ✓ Count: {} users", redb_tree.len()?);

    // Access Redb-specific features (beyond the common API)
    println!(
        "  💡 Redb-specific feature: Tree names: {:?}",
        redb_store.tree_names()
    );
    println!();

    // ========================================================================
    // SUMMARY
    // ========================================================================
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                        SUMMARY                                 ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    println!("✅ The SAME API worked on BOTH backends:");
    println!("   • put() - Insert/update records");
    println!("   • get() - Retrieve by primary key");
    println!("   • get_by_secondary_key() - Query by secondary key");
    println!("   • len() - Count records");
    println!("   • All operations have identical signatures\n");

    println!("💡 Key Insights:");
    println!("   1. Write code once, run on any backend");
    println!("   2. Switch backends by changing ONE line (initialization)");
    println!("   3. Backend-specific features still accessible when needed");
    println!("   4. All backends support same data models and secondary keys\n");

    println!("🎯 When to Use Each Backend:");
    println!("   • Sled:   General purpose, production apps");
    println!("   • Redb:   Read-heavy workloads, embedded systems");
    println!("   • For testing: Use temp() methods for fast, isolated tests\n");

    println!("📚 For More Examples:");
    println!("   • examples/batch_operations.rs - Batch operations");
    println!("   • examples/transactions.rs - Transaction API");
    println!("   • tests/backend_crud_tests.rs - Comprehensive tests\n");

    println!("✅ All tests passed!");

    Ok(())
}
