//! Backend Subscription Integration Example
//!
//! This example demonstrates how to integrate subscription tracking with database backends.
//! It shows automatic subscription updates when data is added or removed from stores.

use chrono::Utc;
use netabase_store::{
    NetabaseDateTime, NetabaseModel,
    databases::sled_store::SledStore,
    netabase, netabase_definition_module, streams,
    traits::subscription::{SubscriptionManager, Subscriptions},
};

// Define a blog schema with subscription streams
#[netabase_definition_module(BlogDefinition, BlogKeys)]
#[streams(Users, Posts)]
mod blog {
    use super::*;

    /// User model
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
        pub name: String,
        pub email: String,
        #[bincode(with_serde)]
        pub created_at: NetabaseDateTime,
    }

    /// Post model
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
        pub id: String,
        pub title: String,
        pub content: String,
        pub author_id: u64,
        #[bincode(with_serde)]
        pub created_at: NetabaseDateTime,
    }
}

use blog::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔌 Backend Subscription Integration Example");
    println!("============================================\n");

    // Create a temporary sled store for testing
    println!("📦 Creating temporary sled store...");
    let store = SledStore::<BlogDefinition>::temp()?;
    println!("  ✅ Store created\n");

    // Create a subscription manager
    println!("🔧 Creating subscription manager...");
    let mut manager = BlogDefinitionSubscriptionManager::new();
    println!("  ✅ Manager created");
    println!(
        "  Topics available: {:?}\n",
        BlogDefinition::all_subscriptions()
    );

    // Create sample data
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        created_at: Utc::now(),
    };

    let post = Post {
        id: "post-1".to_string(),
        title: "Getting Started with Netabase".to_string(),
        content: "This is an introduction to Netabase...".to_string(),
        author_id: 1,
        created_at: Utc::now(),
    };

    println!("📝 Sample data created:");
    println!("  User: {} ({})", user.name, user.email);
    println!("  Post: {}\n", post.title);

    // Manually integrate subscriptions with store operations
    println!("💾 Storing data and updating subscriptions...");

    // Store user in database
    let user_tree = store.open_tree::<User>();
    user_tree.put(user.clone())?;
    println!("  ✅ User stored in database");

    // Update subscription for user
    let user_key = bincode::encode_to_vec(&user.id, bincode::config::standard())?;
    let user_data = bincode::encode_to_vec(&user, bincode::config::standard())?;
    manager.subscribe_item(BlogDefinitionSubscriptions::Users, user_key, &user_data)?;
    println!("  ✅ User subscription updated");

    // Store post in database
    let post_tree = store.open_tree::<Post>();
    post_tree.put(post.clone())?;
    println!("  ✅ Post stored in database");

    // Update subscription for post
    let post_key = bincode::encode_to_vec(&post.id, bincode::config::standard())?;
    let post_data = bincode::encode_to_vec(&post, bincode::config::standard())?;
    manager.subscribe_item(BlogDefinitionSubscriptions::Posts, post_key, &post_data)?;
    println!("  ✅ Post subscription updated\n");

    // Check subscription stats
    println!("📊 Subscription Statistics:");
    let stats = manager.stats();
    println!("  Total items tracked: {}", stats.total_items);
    println!("  Active topics: {}\n", stats.active_topics);

    // Get merkle roots for synchronization
    println!("🌲 Merkle Tree Roots (for sync):");
    for topic in BlogDefinition::all_subscriptions() {
        if let Ok(Some(root)) = manager.topic_merkle_root(topic) {
            println!("  {}: {:x?}", topic, &root[..8]);
        }
    }
    println!();

    // Demonstrate synchronization scenario
    println!("🔄 Demonstrating Synchronization:");
    println!("  Creating a second manager (simulating remote node)...");
    let mut remote_manager = BlogDefinitionSubscriptionManager::new();

    // Remote has same post but different user
    let remote_user = User {
        id: 2,
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
        created_at: Utc::now(),
    };

    let remote_user_key = bincode::encode_to_vec(&remote_user.id, bincode::config::standard())?;
    let remote_user_data = bincode::encode_to_vec(&remote_user, bincode::config::standard())?;
    remote_manager.subscribe_item(
        BlogDefinitionSubscriptions::Users,
        remote_user_key,
        &remote_user_data,
    )?;

    // Add same post to remote
    let post_key2 = bincode::encode_to_vec(&post.id, bincode::config::standard())?;
    let post_data2 = bincode::encode_to_vec(&post, bincode::config::standard())?;
    remote_manager.subscribe_item(BlogDefinitionSubscriptions::Posts, post_key2, &post_data2)?;

    println!("  ✅ Remote manager populated\n");

    // Compare the two managers
    println!("🔍 Comparing local and remote subscriptions:");
    match manager.compare_with(&mut remote_manager) {
        Ok(diffs) => {
            if diffs.is_empty() {
                println!("  ✅ No differences found - nodes are in sync!");
            } else {
                println!("  🔄 Found {} topic(s) with differences:", diffs.len());
                for (topic, diff) in diffs {
                    println!("\n  Topic: {:?}", topic);
                    println!("    Missing in local: {} items", diff.missing_in_self.len());
                    println!(
                        "    Missing in remote: {} items",
                        diff.missing_in_other.len()
                    );
                    println!(
                        "    Different values: {} items",
                        diff.different_values.len()
                    );
                }
            }
        }
        Err(e) => println!("  ❌ Error comparing: {}", e),
    }
    println!();

    // Demonstrate removal with subscription update
    println!("🗑️  Demonstrating removal with subscription update:");
    let post_key3 = bincode::encode_to_vec(&post.id, bincode::config::standard())?;
    match manager.unsubscribe_item(BlogDefinitionSubscriptions::Posts, &post_key3) {
        Ok(Some(hash)) => {
            println!("  ✅ Post unsubscribed, hash: {:?}", hash);
            // In a real application, you would also remove from the database
            // post_tree.remove(post.id)?;
        }
        Ok(None) => println!("  ⚠️  Post was not subscribed"),
        Err(e) => println!("  ❌ Error: {}", e),
    }

    let updated_stats = manager.stats();
    println!(
        "  Updated stats - Total items: {}\n",
        updated_stats.total_items
    );

    println!("✨ Example completed successfully!");
    println!();
    println!("💡 Key Takeaways:");
    println!("   • Subscription managers track data changes using merkle trees");
    println!("   • Store operations can be integrated with subscription updates");
    println!("   • Merkle roots enable efficient synchronization checks");
    println!("   • Differences can be identified and resolved between nodes");
    println!("   • Manual integration gives full control over when subscriptions update");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_integration_basics() {
        let store = SledStore::<BlogDefinition>::temp().unwrap();
        let mut manager = BlogDefinitionSubscriptionManager::new();

        let user = User {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            created_at: Utc::now(),
        };

        // Store in database
        let user_tree = store.open_tree::<User>();
        user_tree.put(user.clone()).unwrap();

        // Update subscription
        let user_key = bincode::encode_to_vec(&user.id, bincode::config::standard()).unwrap();
        let user_data = bincode::encode_to_vec(&user, bincode::config::standard()).unwrap();
        manager
            .subscribe_item(BlogDefinitionSubscriptions::Users, user_key, &user_data)
            .unwrap();

        // Verify subscription stats
        let stats = manager.stats();
        assert_eq!(stats.total_items, 1);
        assert_eq!(stats.active_topics, 1);
    }

    #[test]
    fn test_subscription_removal() {
        let mut manager = BlogDefinitionSubscriptionManager::new();

        let user = User {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            created_at: Utc::now(),
        };

        // Subscribe
        let user_key = bincode::encode_to_vec(&user.id, bincode::config::standard()).unwrap();
        let user_data = bincode::encode_to_vec(&user, bincode::config::standard()).unwrap();
        manager
            .subscribe_item(
                BlogDefinitionSubscriptions::Users,
                user_key.clone(),
                &user_data,
            )
            .unwrap();

        // Verify it was added
        assert_eq!(manager.stats().total_items, 1);

        // Unsubscribe
        let result = manager.unsubscribe_item(BlogDefinitionSubscriptions::Users, &user_key);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());

        // Verify it was removed
        assert_eq!(manager.stats().total_items, 0);
    }

    #[test]
    fn test_merkle_root_consistency() {
        let mut manager1 = BlogDefinitionSubscriptionManager::new();
        let mut manager2 = BlogDefinitionSubscriptionManager::new();

        let user = User {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            created_at: Utc::now(),
        };

        let user_key = bincode::encode_to_vec(&user.id, bincode::config::standard()).unwrap();
        let user_data = bincode::encode_to_vec(&user, bincode::config::standard()).unwrap();

        // Add same user to both managers
        manager1
            .subscribe_item(
                BlogDefinitionSubscriptions::Users,
                user_key.clone(),
                &user_data,
            )
            .unwrap();
        manager2
            .subscribe_item(BlogDefinitionSubscriptions::Users, user_key, &user_data)
            .unwrap();

        // Merkle roots should be identical
        let root1 = manager1
            .topic_merkle_root(BlogDefinitionSubscriptions::Users)
            .unwrap();
        let root2 = manager2
            .topic_merkle_root(BlogDefinitionSubscriptions::Users)
            .unwrap();

        assert_eq!(root1, root2);
    }
}
