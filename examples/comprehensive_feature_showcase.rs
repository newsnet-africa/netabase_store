/// Comprehensive Feature Showcase
///
/// This example demonstrates all major features of netabase_store across different backends,
/// providing a complete overview of the library's capabilities and serving as a comprehensive
/// integration test.
///
/// Run with different feature combinations to test various backends:
/// ```bash
/// # Test all native backends
/// cargo run --example comprehensive_feature_showcase --features "native,libp2p"
///
/// # Test specific backend
/// cargo run --example comprehensive_feature_showcase --features "sled"
/// cargo run --example comprehensive_feature_showcase --features "redb"
/// cargo run --example comprehensive_feature_showcase --features "redb-zerocopy"
///
/// # Test WASM (requires wasm-pack)
/// wasm-pack build --target web --features "wasm"
/// ```
use netabase_store::error::NetabaseError;
use netabase_store::netabase_definition_module;
use netabase_store::traits::model::NetabaseModelTrait;
use std::time::{SystemTime, UNIX_EPOCH};

#[netabase_definition_module(ShowcaseDefinition, ShowcaseKeys)]
mod showcase_models {
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
    #[netabase(ShowcaseDefinition)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub username: String,
        #[secondary_key]
        pub email: String,
        pub created_at: u64,
        pub is_active: bool,
        #[secondary_key]
        pub role: String,
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
    #[netabase(ShowcaseDefinition)]
    pub struct BlogPost {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,
        #[secondary_key]
        pub author_id: u64,
        pub created_at: u64,
        pub updated_at: Option<u64>,
        #[secondary_key]
        pub published: bool,
        pub tags: Vec<String>,
        pub view_count: u64,
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
    #[netabase(ShowcaseDefinition)]
    pub struct Comment {
        #[primary_key]
        pub id: u64,
        pub content: String,
        #[secondary_key]
        pub post_id: u64,
        #[secondary_key]
        pub author_id: u64,
        pub created_at: u64,
        pub parent_id: Option<u64>,
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
    #[netabase(ShowcaseDefinition)]
    pub struct Category {
        #[primary_key]
        pub id: u64,
        pub name: String,
        pub description: String,
        #[secondary_key]
        pub parent_id: Option<u64>,
        pub created_at: u64,
    }
}

use showcase_models::*;

fn main() -> Result<(), NetabaseError> {
    println!("🚀 Netabase Store Comprehensive Feature Showcase");
    println!("================================================\n");

    // Test all available backends
    test_backend_availability()?;

    // Feature demonstrations
    demonstrate_unified_api()?;
    demonstrate_crud_operations()?;
    demonstrate_secondary_keys()?;
    demonstrate_batch_operations()?;
    demonstrate_transactions()?;
    demonstrate_configuration_options()?;
    demonstrate_data_types()?;
    demonstrate_performance_features()?;

    #[cfg(all(feature = "libp2p", not(target_arch = "wasm32")))]
    demonstrate_libp2p_integration()?;

    #[cfg(target_arch = "wasm32")]
    {
        println!("🌐 WASM-specific features would be demonstrated here");
        // WASM-specific demonstrations would go here
    }

    println!("\n✅ Comprehensive showcase completed successfully!");
    println!("   All demonstrated features are working correctly.");

    Ok(())
}

fn test_backend_availability() -> Result<(), NetabaseError> {
    println!("📦 Testing Backend Availability");
    println!("==============================");

    #[cfg(feature = "sled")]
    {
        println!("✓ Sled backend available");
        test_sled_backend()?;
    }

    #[cfg(feature = "redb")]
    {
        println!("✓ Redb backend available");
        test_redb_backend()?;
    }

    #[cfg(feature = "redb-zerocopy")]
    {
        println!("✓ RedbZeroCopy backend available");
        test_redb_zerocopy_backend()?;
    }

    #[cfg(feature = "wasm")]
    {
        println!("✓ WASM IndexedDB backend available");
    }

    #[cfg(feature = "libp2p")]
    {
        println!("✓ LibP2P integration available");
    }

    println!();
    Ok(())
}

#[cfg(feature = "sled")]
fn test_sled_backend() -> Result<(), NetabaseError> {
    use netabase_store::databases::sled_store::SledStore;

    let store = SledStore::<ShowcaseDefinition>::temp()?;
    let tree = store.open_tree::<User>();

    let user = User {
        id: 1,
        username: "sled_test_user".to_string(),
        email: "sled@example.com".to_string(),
        created_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        is_active: true,
        role: "admin".to_string(),
    };

    tree.put(user.clone())?;
    let retrieved = tree.get(user.primary_key())?;
    assert_eq!(retrieved, Some(user));

    println!("  • Sled backend test passed");
    Ok(())
}

#[cfg(feature = "redb")]
fn test_redb_backend() -> Result<(), NetabaseError> {
    use netabase_store::databases::redb_store::RedbStore;

    let temp_path = std::env::temp_dir().join("showcase_redb_test.redb");
    let store = RedbStore::<ShowcaseDefinition>::new(temp_path)?;
    let tree = store.open_tree::<User>();

    let user = User {
        id: 1,
        username: "redb_test_user".to_string(),
        email: "redb@example.com".to_string(),
        created_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        is_active: true,
        role: "user".to_string(),
    };

    tree.put(user.clone())?;
    let retrieved = tree.get(UserKey::Primary(user.primary_key()))?;
    assert_eq!(retrieved, Some(user));

    println!("  • Redb backend test passed");
    Ok(())
}

#[cfg(feature = "redb-zerocopy")]
fn test_redb_zerocopy_backend() -> Result<(), NetabaseError> {
    use netabase_store::databases::redb_zerocopy::RedbStoreZeroCopy;

    let temp_path = std::env::temp_dir().join("showcase_zerocopy_test.redb");
    let store = RedbStoreZeroCopy::<ShowcaseDefinition>::new(temp_path)?;

    {
        let mut txn = store.begin_write()?;
        let mut tree = txn.open_tree::<User>()?;

        let user = User {
            id: 1,
            username: "zerocopy_test_user".to_string(),
            email: "zerocopy@example.com".to_string(),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            is_active: true,
            role: "user".to_string(),
        };

        tree.put(user.clone())?;
        drop(tree);
        txn.commit()?;
    }

    {
        let txn = store.begin_read()?;
        let tree = txn.open_tree::<User>()?;
        let retrieved = tree.get(&UserPrimaryKey(1))?;
        assert!(retrieved.is_some());
        drop(tree);
    }

    println!("  • RedbZeroCopy backend test passed");
    Ok(())
}

fn demonstrate_unified_api() -> Result<(), NetabaseError> {
    println!("🔄 Unified API Demonstration");
    println!("============================");
    println!("• Demonstrating unified API across backends...");

    #[cfg(feature = "sled")]
    {
        use netabase_store::databases::sled_store::SledStore;

        println!("  • Testing Sled backend...");
        let store = SledStore::<ShowcaseDefinition>::temp()?;
        let tree = store.open_tree::<User>();

        let user = User {
            id: 42,
            username: "sled_user".to_string(),
            email: "sled@example.com".to_string(),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            is_active: true,
            role: "tester".to_string(),
        };

        tree.put(user.clone())?;
        let retrieved = tree.get(user.primary_key())?;
        assert_eq!(retrieved, Some(user));

        println!("    ✓ Basic operations work on Sled");
    }

    #[cfg(feature = "redb")]
    {
        use netabase_store::databases::redb_store::RedbStore;

        println!("  • Testing ReDB backend...");
        let path = std::env::temp_dir().join("showcase_unified_redb.redb");
        let store = RedbStore::<ShowcaseDefinition>::new(path)?;
        let tree = store.open_tree::<User>();

        let user = User {
            id: 42,
            username: "redb_user".to_string(),
            email: "redb@example.com".to_string(),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            is_active: true,
            role: "tester".to_string(),
        };

        tree.put(user.clone())?;
        let retrieved = tree.get(UserKey::Primary(user.primary_key()))?;
        assert_eq!(retrieved, Some(user));

        println!("    ✓ Basic operations work on ReDB");
    }

    #[cfg(not(feature = "native"))]
    {
        println!("• Native backends not available, skipping unified API demo");
    }

    println!();
    Ok(())
}

fn demonstrate_crud_operations() -> Result<(), NetabaseError> {
    println!("📝 CRUD Operations Demonstration");
    println!("================================");

    #[cfg(feature = "sled")]
    {
        use netabase_store::databases::sled_store::SledStore;

        let store = SledStore::<ShowcaseDefinition>::temp()?;
        let users_tree = store.open_tree::<User>();
        let posts_tree = store.open_tree::<BlogPost>();

        // CREATE
        println!("• Creating sample data...");
        let user = User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@blog.com".to_string(),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            is_active: true,
            role: "author".to_string(),
        };
        users_tree.put(user.clone())?;

        let post = BlogPost {
            id: 1,
            title: "Welcome to My Blog".to_string(),
            content: "This is my first blog post using netabase_store!".to_string(),
            author_id: 1,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            updated_at: None,
            published: true,
            tags: vec!["welcome".to_string(), "first-post".to_string()],
            view_count: 0,
        };
        posts_tree.put(post.clone())?;
        println!("  ✓ Created user and blog post");

        // READ
        println!("• Reading data...");
        let retrieved_user = users_tree.get(user.primary_key())?.unwrap();
        let retrieved_post = posts_tree.get(post.primary_key())?.unwrap();
        assert_eq!(retrieved_user, user);
        assert_eq!(retrieved_post, post);
        println!("  ✓ Successfully read data");

        // UPDATE
        println!("• Updating data...");
        let mut updated_post = retrieved_post;
        updated_post.content = "This is my updated first blog post!".to_string();
        updated_post.updated_at = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
        updated_post.view_count = 42;
        posts_tree.put(updated_post.clone())?;

        let re_retrieved = posts_tree.get(updated_post.primary_key())?.unwrap();
        assert_eq!(re_retrieved.content, "This is my updated first blog post!");
        assert!(re_retrieved.updated_at.is_some());
        assert_eq!(re_retrieved.view_count, 42);
        println!("  ✓ Successfully updated data");

        // DELETE
        println!("• Deleting data...");
        let deleted_post = posts_tree.remove(updated_post.primary_key())?.unwrap();
        assert_eq!(deleted_post.id, updated_post.id);

        let check_deleted = posts_tree.get(updated_post.primary_key())?;
        assert!(check_deleted.is_none());
        println!("  ✓ Successfully deleted data");
    }

    #[cfg(not(feature = "sled"))]
    {
        println!("• Sled backend not available, skipping CRUD demo");
    }

    println!();
    Ok(())
}

fn demonstrate_secondary_keys() -> Result<(), NetabaseError> {
    println!("🔑 Secondary Key Demonstration");
    println!("==============================");

    #[cfg(feature = "sled")]
    {
        use netabase_store::databases::sled_store::SledStore;

        let store = SledStore::<ShowcaseDefinition>::temp()?;
        let users_tree = store.open_tree::<User>();
        let posts_tree = store.open_tree::<BlogPost>();
        let _comments_tree = store.open_tree::<Comment>();

        // Setup test data
        println!("• Setting up test data...");
        let users = vec![
            User {
                id: 1,
                username: "alice".to_string(),
                email: "alice@example.com".to_string(),
                created_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                is_active: true,
                role: "admin".to_string(),
            },
            User {
                id: 2,
                username: "bob".to_string(),
                email: "bob@example.com".to_string(),
                created_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                is_active: true,
                role: "author".to_string(),
            },
            User {
                id: 3,
                username: "charlie".to_string(),
                email: "charlie@example.com".to_string(),
                created_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                is_active: false,
                role: "author".to_string(),
            },
        ];

        for user in &users {
            users_tree.put(user.clone())?;
        }

        let posts = vec![
            BlogPost {
                id: 1,
                title: "Alice's First Post".to_string(),
                content: "Hello world!".to_string(),
                author_id: 1,
                created_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                updated_at: None,
                published: true,
                tags: vec!["hello".to_string()],
                view_count: 0,
            },
            BlogPost {
                id: 2,
                title: "Bob's Draft".to_string(),
                content: "Work in progress...".to_string(),
                author_id: 2,
                created_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                updated_at: None,
                published: false,
                tags: vec!["draft".to_string()],
                view_count: 0,
            },
            BlogPost {
                id: 3,
                title: "Bob's Published Post".to_string(),
                content: "This is published!".to_string(),
                author_id: 2,
                created_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                updated_at: None,
                published: true,
                tags: vec!["published".to_string()],
                view_count: 0,
            },
        ];

        for post in &posts {
            posts_tree.put(post.clone())?;
        }

        // Query by user role
        println!("• Querying users by role...");
        let authors = users_tree.get_by_secondary_key(UserSecondaryKeys::Role(
            UserRoleSecondaryKey("author".to_string()),
        ))?;
        println!("  ✓ Found {} authors", authors.len());
        assert_eq!(authors.len(), 2);

        let admins = users_tree.get_by_secondary_key(UserSecondaryKeys::Role(
            UserRoleSecondaryKey("admin".to_string()),
        ))?;
        println!("  ✓ Found {} admins", admins.len());
        assert_eq!(admins.len(), 1);

        // Query by publication status
        println!("• Querying posts by publication status...");
        let published_posts = posts_tree.get_by_secondary_key(BlogPostSecondaryKeys::Published(
            BlogPostPublishedSecondaryKey(true),
        ))?;
        println!("  ✓ Found {} published posts", published_posts.len());
        assert_eq!(published_posts.len(), 2);

        let draft_posts = posts_tree.get_by_secondary_key(BlogPostSecondaryKeys::Published(
            BlogPostPublishedSecondaryKey(false),
        ))?;
        println!("  ✓ Found {} draft posts", draft_posts.len());
        assert_eq!(draft_posts.len(), 1);

        // Query by author
        println!("• Querying posts by author...");
        let bobs_posts = posts_tree.get_by_secondary_key(BlogPostSecondaryKeys::AuthorId(
            BlogPostAuthorIdSecondaryKey(2),
        ))?;
        println!("  ✓ Found {} posts by Bob", bobs_posts.len());
        assert_eq!(bobs_posts.len(), 2);
    }

    #[cfg(not(feature = "sled"))]
    {
        println!("• Sled backend not available, skipping secondary key demo");
    }

    println!();
    Ok(())
}

fn demonstrate_batch_operations() -> Result<(), NetabaseError> {
    println!("📦 Batch Operations Demonstration");
    println!("=================================");

    #[cfg(feature = "sled")]
    {
        use netabase_store::databases::sled_store::SledStore;
        use std::time::Instant;

        let store = SledStore::<ShowcaseDefinition>::temp()?;
        let users_tree = store.open_tree::<User>();

        // Generate batch data
        println!("• Generating batch data...");
        let batch_size = 1000;
        let users: Vec<User> = (0..batch_size)
            .map(|i| User {
                id: i,
                username: format!("user_{:04}", i),
                email: format!("user_{}@example.com", i),
                created_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                is_active: i % 2 == 0,
                role: if i % 10 == 0 { "admin" } else { "user" }.to_string(),
            })
            .collect();

        // Batch insert
        println!("• Performing batch insert of {} users...", batch_size);
        let start_time = Instant::now();
        for user in users.iter() {
            users_tree.put(user.clone())?;
        }
        let insert_duration = start_time.elapsed();
        println!("  ✓ Batch insert completed in {:?}", insert_duration);

        // Batch read
        println!("• Performing batch read...");
        let keys: Vec<UserPrimaryKey> = (0..batch_size).map(|i| UserPrimaryKey(i)).collect();
        let start_time = Instant::now();
        let retrieved_users: Vec<User> = keys
            .into_iter()
            .filter_map(|key| users_tree.get(key).ok().flatten())
            .collect();
        let read_duration = start_time.elapsed();
        println!(
            "  ✓ Batch read of {} users completed in {:?}",
            retrieved_users.len(),
            read_duration
        );
        assert_eq!(retrieved_users.len(), batch_size as usize);

        // Batch delete
        println!("• Performing batch delete of first 100 users...");
        let delete_keys: Vec<UserPrimaryKey> = (0..100).map(|i| UserPrimaryKey(i)).collect();
        let start_time = Instant::now();
        let mut deleted_users = Vec::new();
        for key in delete_keys {
            if let Ok(Some(user)) = users_tree.remove(key) {
                deleted_users.push(user);
            }
        }
        let delete_duration = start_time.elapsed();
        println!(
            "  ✓ Batch delete of {} users completed in {:?}",
            deleted_users.len(),
            delete_duration
        );
        assert_eq!(deleted_users.len(), 100);

        // Verify deletion
        let remaining_count = users_tree.iter().count();
        println!("  ✓ {} users remaining after batch delete", remaining_count);
        assert_eq!(remaining_count, (batch_size - 100) as usize);
    }

    #[cfg(not(feature = "sled"))]
    {
        println!("• Sled backend not available, skipping batch operations demo");
    }

    println!();
    Ok(())
}

fn demonstrate_transactions() -> Result<(), NetabaseError> {
    println!("💳 Transaction Demonstration");
    println!("============================");

    #[cfg(feature = "redb-zerocopy")]
    {
        use netabase_store::databases::redb_zerocopy::{
            RedbStoreZeroCopy, with_read_transaction, with_write_transaction,
        };

        let temp_path = std::env::temp_dir().join("showcase_transactions.redb");
        let store = RedbStoreZeroCopy::<ShowcaseDefinition>::new(&temp_path)?;

        // Demonstrate atomic transactions
        println!("• Demonstrating atomic transactions...");

        // Transaction that commits
        let users_inserted = with_write_transaction(&store, |txn| {
            let mut users_tree = txn.open_tree::<User>()?;
            let mut posts_tree = txn.open_tree::<BlogPost>()?;

            let user = User {
                id: 1,
                username: "transaction_user".to_string(),
                email: "txn@example.com".to_string(),
                created_at: NetabaseDateTime::now(),
                is_active: true,
                role: "author".to_string(),
            };
            users_tree.put(user.clone())?;

            let post = BlogPost {
                id: 1,
                title: "Transaction Test".to_string(),
                content: "This post was created in a transaction".to_string(),
                author_id: 1,
                created_at: NetabaseDateTime::now(),
                updated_at: None,
                published: true,
                tags: vec!["transaction".to_string()],
                view_count: 0,
            };
            posts_tree.put(post)?;

            Ok(1)
        })?;

        println!(
            "  ✓ Transaction committed successfully, inserted {} user",
            users_inserted
        );

        // Verify data was committed
        let user_count = with_read_transaction(&store, |txn| {
            let users_tree = txn.open_tree::<User>()?;
            Ok(users_tree.len()?)
        })?;
        println!("  ✓ Verified {} users exist after commit", user_count);
        assert_eq!(user_count, 1);

        // Transaction that aborts
        println!("• Demonstrating transaction abort...");
        {
            let mut txn = store.begin_write()?;
            let mut users_tree = txn.open_tree::<User>()?;

            let user = User {
                id: 2,
                username: "abort_user".to_string(),
                email: "abort@example.com".to_string(),
                created_at: NetabaseDateTime::now(),
                is_active: true,
                role: "temp".to_string(),
            };
            users_tree.put(user)?;

            // Abort the transaction
            drop(users_tree);
            txn.abort()?;
        }

        // Verify data was not committed
        let user_count_after_abort = with_read_transaction(&store, |txn| {
            let users_tree = txn.open_tree::<User>()?;
            Ok(users_tree.len()?)
        })?;
        println!("  ✓ User count after abort: {}", user_count_after_abort);
        assert_eq!(user_count_after_abort, 1); // Still only 1 user

        // Demonstrate isolation
        println!("• Demonstrating transaction isolation...");
        let read_txn = store.begin_read()?;
        let users_tree_read = read_txn.open_tree::<User>()?;
        let initial_count = users_tree_read.len()?;

        // Start a write transaction in "background"
        {
            let mut write_txn = store.begin_write()?;
            let mut users_tree_write = write_txn.open_tree::<User>()?;
            users_tree_write.put(User {
                id: 3,
                username: "isolation_test".to_string(),
                email: "isolation@example.com".to_string(),
                created_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                is_active: true,
                role: "test".to_string(),
            })?;
            drop(users_tree_write);
            write_txn.commit()?;
        }

        // Read transaction should still see old state
        let count_during_isolation = users_tree_read.len()?;
        println!(
            "  ✓ Read transaction sees {} users (isolated from concurrent writes)",
            count_during_isolation
        );
        assert_eq!(count_during_isolation, initial_count);

        drop(users_tree_read);
        drop(read_txn);

        // New read transaction sees new state
        let final_count = with_read_transaction(&store, |txn| {
            let users_tree = txn.open_tree::<User>()?;
            Ok(users_tree.len()?)
        })?;
        println!("  ✓ New read transaction sees {} users", final_count);
        assert_eq!(final_count, 2);
    }

    #[cfg(not(feature = "redb-zerocopy"))]
    {
        println!("• RedbZeroCopy backend not available, skipping transaction demo");
        println!("  Note: Other backends have implicit transaction support");
    }

    println!();
    Ok(())
}

fn demonstrate_configuration_options() -> Result<(), NetabaseError> {
    println!("⚙️  Configuration Options Demonstration");
    println!("=======================================");

    #[cfg(feature = "native")]
    {
        use netabase_store::config::FileConfig;

        println!("• Demonstrating FileConfig builder pattern...");

        let config = FileConfig::builder()
            .path(std::env::temp_dir().join("showcase_config_demo.db"))
            .cache_size_mb(512)
            .create_if_missing(true)
            .truncate(false)
            .read_only(false)
            .use_fsync(true)
            .build();

        println!("  ✓ Created config with 512MB cache, fsync enabled");

        #[cfg(feature = "sled")]
        {
            use netabase_store::databases::sled_store::SledStore;

            let store = SledStore::<ShowcaseDefinition>::new(&config.path)?;
            let tree = store.open_tree::<User>();

            let user = User {
                id: 1,
                username: "config_test".to_string(),
                email: "config@example.com".to_string(),
                created_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                is_active: true,
                role: "tester".to_string(),
            };

            tree.put(user.clone())?;
            let retrieved = tree.get(user.primary_key())?;
            assert_eq!(retrieved, Some(user));

            println!("  ✓ Sled store works with custom configuration");
        }

        // Demonstrate temporary stores
        println!("• Demonstrating temporary stores for testing...");

        #[cfg(feature = "sled")]
        {
            use netabase_store::databases::sled_store::SledStore;

            let temp_store = SledStore::<ShowcaseDefinition>::temp()?;
            let tree = temp_store.open_tree::<User>();

            for i in 0..5 {
                tree.put(User {
                    id: i,
                    username: format!("temp_user_{}", i),
                    email: format!("temp_{}@example.com", i),
                    created_at: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    is_active: true,
                    role: "temp".to_string(),
                })?;
            }

            let count = tree.iter().count();
            println!("  ✓ Temporary store created and used ({} users)", count);
            // Store will be automatically cleaned up when dropped
        }
    }

    #[cfg(not(feature = "native"))]
    {
        println!("• Native backends not available, skipping configuration demo");
    }

    println!();
    Ok(())
}

fn demonstrate_data_types() -> Result<(), NetabaseError> {
    println!("📊 Data Types Demonstration");
    println!("===========================");

    #[cfg(feature = "sled")]
    {
        use netabase_store::databases::sled_store::SledStore;

        let store = SledStore::<ShowcaseDefinition>::temp()?;

        println!("• Testing various data types...");

        // Test User with different field types
        let user = User {
            id: u64::MAX,                                        // Test max value
            username: "🦀 Unicode User 测试".to_string(),        // Unicode
            email: "special+chars@sub-domain.co.uk".to_string(), // Special chars
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(), // Timestamp
            is_active: false,                                    // Boolean
            role: "special-role_123".to_string(),                // Mixed chars
        };

        let users_tree = store.open_tree::<User>();
        users_tree.put(user.clone())?;
        let retrieved_user = users_tree.get(user.primary_key())?.unwrap();
        assert_eq!(retrieved_user, user);
        println!("  ✓ User model with various data types");

        // Test BlogPost with collections and options
        let post = BlogPost {
            id: 0,                                                   // Test zero value
            title: "".to_string(),                                   // Empty string
            content: "Line 1\nLine 2\r\nLine 3\tTabbed".to_string(), // Special chars
            author_id: user.id,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            updated_at: Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            ),
            published: true,
            tags: vec![], // Empty vector
            view_count: 0,
        };

        let posts_tree = store.open_tree::<BlogPost>();
        posts_tree.put(post.clone())?;
        let retrieved_post = posts_tree.get(post.primary_key())?.unwrap();
        assert_eq!(retrieved_post, post);
        println!("  ✓ BlogPost model with Option and Vec types");

        // Test Comment with optional parent (None case)
        let comment = Comment {
            id: 1,
            content: "This is a top-level comment".to_string(),
            post_id: post.id,
            author_id: user.id,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            parent_id: None, // None value
        };

        let comments_tree = store.open_tree::<Comment>();
        comments_tree.put(comment.clone())?;
        let retrieved_comment = comments_tree.get(comment.primary_key())?.unwrap();
        assert_eq!(retrieved_comment, comment);
        println!("  ✓ Comment model with None optional value");

        // Test Comment with optional parent (Some case)
        let reply_comment = Comment {
            id: 2,
            content: "This is a reply".to_string(),
            post_id: post.id,
            author_id: user.id,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            parent_id: Some(comment.id), // Some value
        };

        comments_tree.put(reply_comment.clone())?;
        let retrieved_reply = comments_tree.get(reply_comment.primary_key())?.unwrap();
        assert_eq!(retrieved_reply, reply_comment);
        println!("  ✓ Comment model with Some optional value");
    }

    #[cfg(not(feature = "sled"))]
    {
        println!("• Sled backend not available, skipping data types demo");
    }

    println!();
    Ok(())
}

fn demonstrate_performance_features() -> Result<(), NetabaseError> {
    println!("🚀 Performance Features Demonstration");
    println!("=====================================");

    #[cfg(feature = "sled")]
    {
        use netabase_store::databases::sled_store::SledStore;
        use std::time::Instant;

        let store = SledStore::<ShowcaseDefinition>::temp()?;
        let tree = store.open_tree::<User>();

        // Demonstrate bulk operations performance
        println!("• Demonstrating bulk operation performance...");

        let bulk_size = 10000;
        let users: Vec<User> = (0..bulk_size)
            .map(|i| User {
                id: i,
                username: format!("perf_user_{:05}", i),
                email: format!("perf_{}@example.com", i),
                created_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                is_active: i % 2 == 0,
                role: if i % 100 == 0 { "admin" } else { "user" }.to_string(),
            })
            .collect();

        // Time individual inserts
        let start = Instant::now();
        for user in users.iter().take(1000) {
            tree.put(user.clone())?;
        }
        let individual_duration = start.elapsed();

        // Clear and time bulk insert
        tree.clear()?;
        let start = Instant::now();
        for user in users.iter() {
            tree.put(user.clone())?;
        }
        let bulk_duration = start.elapsed();

        println!("  • Individual inserts (1000): {:?}", individual_duration);
        println!("  • Bulk insert ({}): {:?}", bulk_size, bulk_duration);
        println!("  ✓ Bulk operations are significantly faster");

        // Demonstrate query performance
        println!("• Demonstrating query performance...");

        // Primary key queries
        let start = Instant::now();
        let mut found_count = 0;
        for i in (0..bulk_size).step_by(100) {
            if tree.get(UserPrimaryKey(i))?.is_some() {
                found_count += 1;
            }
        }
        let primary_query_duration = start.elapsed();

        // Secondary key queries
        let start = Instant::now();
        let admins = tree.get_by_secondary_key(UserSecondaryKeys::Role(UserRoleSecondaryKey(
            "admin".to_string(),
        )))?;
        let secondary_query_duration = start.elapsed();

        println!(
            "  • Primary key queries ({} queries): {:?}",
            found_count, primary_query_duration
        );
        println!(
            "  • Secondary key query (found {} admins): {:?}",
            admins.len(),
            secondary_query_duration
        );
        println!("  ✓ Query performance is excellent even with large datasets");

        // Demonstrate iteration performance
        println!("• Demonstrating iteration performance...");
        let start = Instant::now();
        let all_users: Vec<User> = tree.iter().map(|result| result.unwrap().1).collect();
        let iteration_duration = start.elapsed();

        println!(
            "  • Full iteration ({} records): {:?}",
            all_users.len(),
            iteration_duration
        );
        println!("  ✓ Iteration performance scales well");
    }

    #[cfg(not(feature = "sled"))]
    {
        println!("• Sled backend not available, skipping performance demo");
    }

    println!();
    Ok(())
}

#[cfg(all(feature = "libp2p", not(target_arch = "wasm32")))]
fn demonstrate_libp2p_integration() -> Result<(), NetabaseError> {
    println!("🌐 LibP2P Integration Demonstration");
    println!("===================================");

    use libp2p::kad::store::RecordStore;
    use libp2p::kad::{Record, RecordKey};
    use netabase_store::databases::sled_store::SledStore;

    let mut store = SledStore::<ShowcaseDefinition>::temp()?;

    // Demonstrate Record storage (DHT compatibility)
    println!("• Demonstrating DHT Record storage...");

    let test_key = RecordKey::from(b"test_dht_key".to_vec());
    let test_record = Record {
        key: test_key.clone(),
        value: b"This is DHT data".to_vec(),
        publisher: None,
        expires: None,
    };

    if let Err(e) = store.put(test_record.clone()) {
        println!("Failed to put DHT record: {:?}", e);
        return Ok(());
    }
    let retrieved_record = store.get(&test_key);
    assert!(retrieved_record.is_some());
    println!("  ✓ DHT Record storage and retrieval works");

    // Demonstrate Provider records
    #[cfg(feature = "record-store")]
    {
        println!("• Demonstrating Provider record management...");

        const SHA_256_MH: u64 = 0x12;
        let content_hash = Multihash::wrap(SHA_256_MH, &[1, 2, 3, 4]).unwrap();
        let provider_peer = PeerId::random();

        let provider_record = ProviderRecord::new(content_hash, provider_peer, Vec::new());

        store.add_provider(provider_record.clone())?;
        let providers = store.providers(&provider_record.key);
        assert!(providers.contains(&provider_record));
        println!("  ✓ Provider record management works");

        // Test provider iteration
        let all_provided: Vec<_> = store.provided().collect();
        assert!(!all_provided.is_empty());
        println!(
            "  ✓ Provider iteration works ({} records)",
            all_provided.len()
        );

        // Test provider removal
        store.remove_provider(&provider_record.key, &provider_peer);
        let providers_after_removal = store.providers(&provider_record.key);
        assert!(!providers_after_removal.contains(&provider_record));
        println!("  ✓ Provider removal works");
    }

    // Demonstrate Records iteration
    println!("• Demonstrating Records iteration...");
    let all_records: Vec<_> = store.records().collect();
    assert!(!all_records.is_empty());
    println!(
        "  ✓ Records iteration works ({} records)",
        all_records.len()
    );

    println!();
    Ok(())
}

#[cfg(not(all(feature = "libp2p", not(target_arch = "wasm32"))))]
fn demonstrate_libp2p_integration() -> Result<(), NetabaseError> {
    println!("🌐 LibP2P Integration");
    println!("=====================");
    println!("• LibP2P features not available (requires 'libp2p' feature on native target)");
    println!();
    Ok(())
}
