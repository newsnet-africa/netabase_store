/// Configuration API Showcase
///
/// This example demonstrates the unified configuration system for netabase_store,
/// showing how to use FileConfig, BackendStore trait, and switch between backends.
///
/// Run with different features to try different backends:
/// ```bash
/// cargo run --example config_api_showcase --features "native,sled"
/// cargo run --example config_api_showcase --features "native,redb"
/// cargo run --example config_api_showcase --features "native,redb-zerocopy"
/// ```
use netabase_store::error::NetabaseError;
use netabase_store::netabase_definition_module;
use netabase_store::traits::backend_store::BackendStore;
use netabase_store::traits::model::NetabaseModelTrait;

// Define a simple schema for demonstration
#[netabase_definition_module(AppDefinition, AppKeys)]
mod app_models {
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
    #[netabase(AppDefinition)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub username: String,
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
    #[netabase(AppDefinition)]
    pub struct Post {
        #[primary_key]
        pub id: u64,
        pub title: String,
        #[secondary_key]
        pub author_id: u64,
    }
}

use app_models::*;

fn main() -> Result<(), NetabaseError> {
    println!("=== Configuration API Showcase ===\n");

    // Demonstration 1: Builder Pattern (Recommended)
    println!("1. Builder Pattern Configuration");
    println!("   Most ergonomic with IDE autocomplete");
    builder_pattern_demo()?;

    // Demonstration 2: Simple Constructor
    println!("\n2. Simple Constructor");
    println!("   Quick setup with defaults");
    simple_constructor_demo()?;

    // Demonstration 3: Temporary Databases
    println!("\n3. Temporary Database (Testing)");
    println!("   Perfect for unit tests");
    temporary_database_demo()?;

    // Demonstration 4: Backend Switching
    println!("\n4. Backend Portability");
    println!("   Switch backends with same config");
    backend_switching_demo()?;

    // Demonstration 5: Advanced Configuration
    println!("\n5. Advanced Configuration Options");
    println!("   Fine-tuning performance and behavior");
    advanced_config_demo()?;

    println!("\n=== All Demonstrations Completed Successfully ===");
    Ok(())
}

#[cfg(feature = "sled")]
fn builder_pattern_demo() -> Result<(), NetabaseError> {
    use netabase_store::config::FileConfig;
    use netabase_store::databases::sled_store::SledStore;

    // Create config using builder pattern
    let config = FileConfig::builder()
        .path(std::env::temp_dir().join("config_demo_builder.db"))
        .cache_size_mb(512)
        .truncate(true)
        .build();

    println!("   Config: path={:?}, cache=512MB", config.path);

    // Initialize store with BackendStore trait
    let store = <SledStore<AppDefinition> as BackendStore<AppDefinition>>::new(config)?;

    // Use the store
    let tree = store.open_tree::<User>();
    let user = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    tree.put(user.clone())?;
    let retrieved = tree.get(user.primary_key())?;

    println!("   ✓ Stored and retrieved: {:?}", retrieved);

    Ok(())
}

#[cfg(not(feature = "sled"))]
fn builder_pattern_demo() -> Result<(), NetabaseError> {
    println!("   (Skipped: sled feature not enabled)");
    Ok(())
}

#[cfg(feature = "redb")]
fn simple_constructor_demo() -> Result<(), NetabaseError> {
    use netabase_store::config::FileConfig;
    use netabase_store::databases::redb_store::RedbStore;

    // Simple constructor with defaults
    let config = FileConfig::new(std::env::temp_dir().join("config_demo_simple.redb"));

    println!(
        "   Config: path={:?}, using defaults (cache=256MB)",
        config.path
    );

    // Initialize store
    let store = <RedbStore<AppDefinition> as BackendStore<AppDefinition>>::new(config)?;

    // Use the store
    let tree = store.open_tree::<Post>();
    let post = Post {
        id: 1,
        title: "Configuration API Guide".to_string(),
        author_id: 1,
    };

    tree.put(post.clone())?;
    let retrieved = tree.get(PostKey::Primary(post.primary_key()))?;

    println!("   ✓ Stored and retrieved: {:?}", retrieved);

    Ok(())
}

#[cfg(not(feature = "redb"))]
fn simple_constructor_demo() -> Result<(), NetabaseError> {
    println!("   (Skipped: redb feature not enabled)");
    Ok(())
}

#[cfg(feature = "sled")]
fn temporary_database_demo() -> Result<(), NetabaseError> {
    use netabase_store::databases::sled_store::SledStore;

    println!("   Creating temporary database (no path needed)");

    // Create temporary database - perfect for testing
    let store = <SledStore<AppDefinition> as BackendStore<AppDefinition>>::temp()?;

    // Use it like any other store
    let tree = store.open_tree::<User>();
    let user = User {
        id: 100,
        username: "temp_user".to_string(),
        email: "temp@example.com".to_string(),
    };

    tree.put(user.clone())?;
    let retrieved = tree.get(user.primary_key())?;

    println!("   ✓ Temporary store works: {:?}", retrieved);
    println!("   (Database will be deleted on program exit)");

    Ok(())
}

#[cfg(not(feature = "sled"))]
fn temporary_database_demo() -> Result<(), NetabaseError> {
    println!("   (Skipped: sled feature not enabled)");
    Ok(())
}

fn backend_switching_demo() -> Result<(), NetabaseError> {
    use netabase_store::config::FileConfig;

    // Create ONE configuration
    let config = FileConfig::builder()
        .path(std::env::temp_dir().join("config_demo_switch.db"))
        .cache_size_mb(256)
        .truncate(true)
        .build();

    println!("   Using single config across all backends");

    // Try with Sled
    #[cfg(feature = "sled")]
    {
        use netabase_store::databases::sled_store::SledStore;
        let store = <SledStore<AppDefinition> as BackendStore<AppDefinition>>::new(config.clone())?;
        let tree = store.open_tree::<User>();
        tree.put(User {
            id: 1,
            username: "sled_user".to_string(),
            email: "sled@example.com".to_string(),
        })?;
        println!("   ✓ Sled backend works");
    }

    // Try with Redb (same config!)
    #[cfg(feature = "redb")]
    {
        use netabase_store::databases::redb_store::RedbStore;
        let config_redb = FileConfig::builder()
            .path(std::env::temp_dir().join("config_demo_switch.redb"))
            .cache_size_mb(256)
            .truncate(true)
            .build();
        let store = <RedbStore<AppDefinition> as BackendStore<AppDefinition>>::new(config_redb)?;
        let tree = store.open_tree::<User>();
        tree.put(User {
            id: 1,
            username: "redb_user".to_string(),
            email: "redb@example.com".to_string(),
        })?;
        println!("   ✓ Redb backend works");
    }

    // Try with RedbZeroCopy (uses transaction API)
    #[cfg(feature = "redb-zerocopy")]
    {
        use netabase_store::databases::redb_zerocopy::RedbStoreZeroCopy;
        let config_zerocopy = FileConfig::builder()
            .path(std::env::temp_dir().join("config_demo_switch_zc.redb"))
            .cache_size_mb(256)
            .truncate(true)
            .build();
        let store = <RedbStoreZeroCopy<AppDefinition> as BackendStore<AppDefinition>>::new(
            config_zerocopy,
        )?;

        // RedbZeroCopy uses explicit transactions
        let mut txn = store.begin_write()?;
        let mut tree = txn.open_tree::<User>()?;
        tree.put(User {
            id: 1,
            username: "zerocopy_user".to_string(),
            email: "zerocopy@example.com".to_string(),
        })?;
        drop(tree);
        txn.commit()?;
        println!("   ✓ RedbZeroCopy backend works");
    }

    println!("   → Code is identical, only backend type changes!");

    Ok(())
}

#[cfg(feature = "sled")]
fn advanced_config_demo() -> Result<(), NetabaseError> {
    use netabase_store::config::FileConfig;
    use netabase_store::databases::sled_store::SledStore;

    // Demonstrate all configuration options
    let config = FileConfig::builder()
        .path(std::env::temp_dir().join("config_demo_advanced.db"))
        .cache_size_mb(1024) // 1GB cache for high performance
        .create_if_missing(true) // Create if doesn't exist
        .truncate(false) // Keep existing data
        .read_only(false) // Allow writes
        .use_fsync(true) // Durability guarantee
        .build();

    println!("   Advanced config:");
    println!("     - cache_size_mb: 1024 (1GB)");
    println!("     - create_if_missing: true");
    println!("     - truncate: false");
    println!("     - read_only: false");
    println!("     - use_fsync: true");

    let store = <SledStore<AppDefinition> as BackendStore<AppDefinition>>::new(config)?;

    // Demonstrate batch operations
    let tree = store.open_tree::<User>();

    // Insert multiple users
    for i in 1..=10 {
        tree.put(User {
            id: i,
            username: format!("user_{}", i),
            email: format!("user_{}@example.com", i),
        })?;
    }

    println!("   ✓ Inserted 10 users with optimized settings");

    // Query by secondary key
    let users = tree.get_by_secondary_key(UserSecondaryKeys::Email(UserEmailSecondaryKey(
        "user_5@example.com".to_string(),
    )))?;

    println!("   ✓ Secondary key query found: {:?}", users.len());

    Ok(())
}

#[cfg(not(feature = "sled"))]
fn advanced_config_demo() -> Result<(), NetabaseError> {
    println!("   (Skipped: sled feature not enabled)");
    Ok(())
}
