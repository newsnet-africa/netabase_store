//! Transaction API Example
//!
//! This example demonstrates the new type-safe transaction API with compile-time
//! guarantees for read-only vs read-write access.
//!
//! Run with:
//! ```bash
//! cargo run --example transactions --features "native sled"
//! ```

use netabase_store::{netabase_definition_module, NetabaseModel, NetabaseStore, netabase};
use anyhow::Result;

// Define our schema
#[netabase_definition_module(BlogDefinition, BlogKeys)]
mod schema {
    use super::*;

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(BlogDefinition)]
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
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(BlogDefinition)]
    pub struct Post {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,
        #[secondary_key]
        pub author_id: u64,
    }
}

use schema::*;

fn main() -> Result<()> {
    println!("🔥 Transaction API Example\n");

    // Create a temporary store for this example
    let store = NetabaseStore::<BlogDefinition, _>::temp()?;
    println!("✅ Created temporary Sled store\n");

    // ========================================================================
    // Example 1: Read-Only Transaction
    // ========================================================================
    println!("📖 Example 1: Read-Only Transaction");
    println!("   Multiple concurrent reads without blocking\n");

    {
        let mut txn = store.read();

        // Get user count
        let user_tree = txn.open_tree::<User>();
        let user_count = user_tree.len()?;
        drop(user_tree);  // Drop before opening next tree

        // Get post count
        let post_tree = txn.open_tree::<Post>();
        let post_count = post_tree.len()?;
        drop(post_tree);

        println!("   Users: {}, Posts: {}", user_count, post_count);
        println!("   Transaction auto-closes on drop\n");
    }

    // ========================================================================
    // Example 2: Read-Write Transaction with Single Insert
    // ========================================================================
    println!("✍️  Example 2: Read-Write Transaction (Single Insert)");

    {
        let mut txn = store.write();
        let mut user_tree = txn.open_tree::<User>();

        let user = User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        };

        user_tree.put(user.clone())?;
        println!("   Inserted: {}", user.username);

        txn.commit()?;
        println!("   ✅ Transaction committed\n");
    }

    // ========================================================================
    // Example 3: Bulk Inserts with Transaction (Performance Optimization)
    // ========================================================================
    println!("⚡ Example 3: Bulk Inserts with Transaction");
    println!("   Single transaction for 1000 inserts (10-100x faster!)\n");

    {
        let start = std::time::Instant::now();

        let mut txn = store.write();
        let mut user_tree = txn.open_tree::<User>();

        // All inserts use the same transaction
        for i in 2..1002 {
            let user = User {
                id: i,
                username: format!("user{}", i),
                email: format!("user{}@example.com", i),
            };
            user_tree.put(user)?;
        }

        txn.commit()?;

        let elapsed = start.elapsed();
        println!("   Inserted 1000 users in {:?}", elapsed);
        println!("   Average: {:?} per insert", elapsed / 1000);
        println!("   ✅ All changes committed atomically\n");
    }

    // ========================================================================
    // Example 4: Bulk Operation Helpers
    // ========================================================================
    println!("📦 Example 4: Bulk Operation Helpers");
    println!("   Using put_many() for even more convenience\n");

    {
        let users: Vec<User> = (1002..1102)
            .map(|i| User {
                id: i,
                username: format!("bulk_user{}", i),
                email: format!("bulk{}@example.com", i),
            })
            .collect();

        let start = std::time::Instant::now();

        let mut txn = store.write();
        let mut user_tree = txn.open_tree::<User>();

        user_tree.put_many(users)?;

        txn.commit()?;

        let elapsed = start.elapsed();
        println!("   Inserted 100 users with put_many() in {:?}", elapsed);
        println!("   ✅ Bulk operation committed\n");
    }

    // ========================================================================
    // Example 5: Multi-Tree Operations in Single Transaction
    // ========================================================================
    println!("🌳 Example 5: Multi-Tree Operations");
    println!("   Working with multiple models in one transaction\n");

    {
        let mut txn = store.write();

        // Insert user first
        let user = User {
            id: 2000,
            username: "blogger".to_string(),
            email: "blogger@example.com".to_string(),
        };
        {
            let mut user_tree = txn.open_tree::<User>();
            user_tree.put(user.clone())?;
            println!("   Inserted user: {}", user.username);
        }  // Drop user_tree

        // Insert posts for that user
        let mut post_tree = txn.open_tree::<Post>();
        for i in 0..5 {
            let post = Post {
                id: 100 + i,
                title: format!("Post #{}", i + 1),
                content: format!("Content for post {}", i + 1),
                author_id: user.id,
            };
            post_tree.put(post.clone())?;
            println!("   Inserted post: {}", post.title);
        }

        txn.commit()?;
        println!("   ✅ User and 5 posts committed atomically\n");
    }

    // ========================================================================
    // Example 6: Query by Secondary Key
    // ========================================================================
    println!("🔍 Example 6: Query by Secondary Key");

    {
        let mut txn = store.read();
        let post_tree = txn.open_tree::<Post>();

        let posts = post_tree.get_by_secondary_key(
            PostSecondaryKeys::AuthorId(PostAuthorIdSecondaryKey(2000))
        )?;

        println!("   Found {} posts for user 2000", posts.len());
        for post in posts {
            println!("     - {}", post.title);
        }
        println!();
    }

    // ========================================================================
    // Example 7: Bulk Read Operations
    // ========================================================================
    println!("📚 Example 7: Bulk Read Operations");

    {
        let mut txn = store.read();
        let user_tree = txn.open_tree::<User>();

        let keys: Vec<_> = (1..11).map(UserPrimaryKey).collect();
        let users = user_tree.get_many(keys)?;

        let found = users.iter().filter(|u| u.is_some()).count();
        println!("   Fetched 10 users: {} found", found);
        println!();
    }

    // ========================================================================
    // Example 8: Transaction Rollback
    // ========================================================================
    println!("↩️  Example 8: Transaction Rollback");
    println!("   Changes not committed = automatic rollback\n");

    {
        let count_before = {
            let mut txn = store.read();
            let user_tree = txn.open_tree::<User>();
            user_tree.len()?
        };

        {
            let mut txn = store.write();
            let mut user_tree = txn.open_tree::<User>();

            user_tree.put(User {
                id: 9999,
                username: "temporary".to_string(),
                email: "temp@example.com".to_string(),
            })?;

            println!("   Inserted user 9999 (but not committing...)");
            // Transaction drops here without commit = rollback
        }

        let count_after = {
            let mut txn = store.read();
            let user_tree = txn.open_tree::<User>();
            user_tree.len()?
        };

        println!("   Count before: {}, after: {}", count_before, count_after);
        println!("   ✅ Changes rolled back automatically\n");
    }

    // ========================================================================
    // Example 9: Compile-Time Safety Demonstration
    // ========================================================================
    println!("🔒 Example 9: Compile-Time Safety");
    println!("   Read-only transactions cannot modify data\n");

    {
        let mut txn = store.read();  // ReadOnly transaction
        let tree = txn.open_tree::<User>();

        // ✅ Read operations work
        let _user = tree.get(UserPrimaryKey(1))?;
        let _count = tree.len()?;

        // ❌ This would be a compile error:
        // tree.put(user)?;
        // Error: no method named `put` found for struct `TreeView<ReadOnly>`

        println!("   ✅ Compile-time enforcement: put() not available on ReadOnly");
        println!();
    }

    // ========================================================================
    // Final Stats
    // ========================================================================
    println!("📊 Final Statistics\n");

    {
        let mut txn = store.read();

        let user_tree = txn.open_tree::<User>();
        let user_count = user_tree.len()?;
        drop(user_tree);

        let post_tree = txn.open_tree::<Post>();
        let post_count = post_tree.len()?;

        println!("   Total users: {}", user_count);
        println!("   Total posts: {}", post_count);
    }

    println!("\n✨ Example complete!");
    println!("\n💡 Key Takeaways:");
    println!("   • Transactions eliminate per-operation overhead (10-100x faster)");
    println!("   • Type-state pattern provides compile-time safety");
    println!("   • Zero-cost abstractions (phantom types compile away)");
    println!("   • Automatic rollback on drop if not committed");
    println!("   • Works with both read-only and read-write operations");

    Ok(())
}
