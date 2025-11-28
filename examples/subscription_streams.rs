//! Example demonstrating the streams subscription functionality
//!
//! This example shows how to use the streams macro to define subscription topics
//! and create a subscription system for efficient data synchronization.

use netabase_store::{
    NetabaseDateTime, NetabaseModel, netabase, netabase_definition_module, streams,
};

// Define a blog schema with subscription streams
#[netabase_definition_module(BlogDefinition, BlogKeys)]
#[streams(UserTopic, PostTopic, CommentTopic)]
mod blog {
    use super::*;

    /// User model with subscription tracking
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

    /// Post model with subscription tracking
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
        #[secondary_key]
        pub author_id: u64,
        #[bincode(with_serde)]
        pub created_at: NetabaseDateTime,
    }

    /// Comment model with subscription tracking
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
    pub struct Comment {
        #[primary_key]
        pub id: u64,
        pub post_id: String,
        pub author_id: u64,
        pub content: String,
        #[bincode(with_serde)]
        pub created_at: NetabaseDateTime,
    }
}

use blog::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use chrono::Utc;
    use netabase_store::traits::subscription::Subscriptions;

    println!("🚀 Netabase Subscription Streams Example");
    println!("=========================================\n");

    // Demonstrate subscription topics
    println!("📋 Available subscription topics:");
    for topic in BlogDefinition::subscriptions() {
        println!("  - {:?}", topic);
    }
    println!();

    // Create sample data
    let user = User {
        id: 1,
        name: "alice".to_string(),
        email: "alice@example.com".to_string(),
        created_at: Utc::now(),
    };

    let post = Post {
        id: "post-1".to_string(),
        title: "Hello, World!".to_string(),
        content: "This is my first blog post.".to_string(),
        author_id: 1,
        created_at: Utc::now(),
    };

    let comment = Comment {
        id: 1,
        post_id: "post-1".to_string(),
        author_id: 1,
        content: "Great post!".to_string(),
        created_at: Utc::now(),
    };

    println!("📝 Sample data created:");
    println!("  User: {} ({})", user.name, user.email);
    println!("  Post: {}", post.title);
    println!("  Comment: {}", comment.content);
    println!();

    // Create subscription manager
    let mut subscription_manager = BlogDefinitionSubscriptionManager::new();
    println!("✅ Subscription manager created");

    // Demonstrate adding items to subscription trees
    println!("\n📊 Adding items to subscription trees...");

    // Add user to UserTopic
    let user_key = bincode::encode_to_vec(&user.id, bincode::config::standard())?;
    let user_data = bincode::encode_to_vec(&user, bincode::config::standard())?;
    match subscription_manager.subscribe_item(
        BlogDefinitionSubscriptions::UserTopic,
        user_key,
        &user_data,
    ) {
        Ok(()) => println!("  ✅ User added to UserTopic subscription"),
        Err(e) => println!("  ❌ Failed to add user: {}", e),
    }

    // Add post to PostTopic
    let post_key = bincode::encode_to_vec(&post.id, bincode::config::standard())?;
    let post_data = bincode::encode_to_vec(&post, bincode::config::standard())?;
    match subscription_manager.subscribe_item(
        BlogDefinitionSubscriptions::PostTopic,
        post_key,
        &post_data,
    ) {
        Ok(()) => println!("  ✅ Post added to PostTopic subscription"),
        Err(e) => println!("  ❌ Failed to add post: {}", e),
    }

    // Add comment to CommentTopic
    let comment_key = bincode::encode_to_vec(&comment.id, bincode::config::standard())?;
    let comment_data = bincode::encode_to_vec(&comment, bincode::config::standard())?;
    match subscription_manager.subscribe_item(
        BlogDefinitionSubscriptions::CommentTopic,
        comment_key,
        &comment_data,
    ) {
        Ok(()) => println!("  ✅ Comment added to CommentTopic subscription"),
        Err(e) => println!("  ❌ Failed to add comment: {}", e),
    }

    println!("\n🔍 Subscription tree information:");

    // Get subscription trees for each topic
    if let Some(user_tree) = subscription_manager.get_tree(BlogDefinitionSubscriptions::UserTopic) {
        println!("  UserTopic tree: {} items", user_tree.len());
    }

    if let Some(post_tree) = subscription_manager.get_tree(BlogDefinitionSubscriptions::PostTopic) {
        println!("  PostTopic tree: {} items", post_tree.len());
    }

    if let Some(comment_tree) =
        subscription_manager.get_tree(BlogDefinitionSubscriptions::CommentTopic)
    {
        println!("  CommentTopic tree: {} items", comment_tree.len());
    }

    println!("\n🌲 Demonstrating merkle tree comparison...");

    // Create a second subscription manager for comparison
    let mut other_manager = BlogDefinitionSubscriptionManager::new();

    // Add some different data to show differences
    let other_user = User {
        id: 2,
        name: "bob".to_string(),
        email: "bob@example.com".to_string(),
        created_at: Utc::now(),
    };

    let other_user_key = bincode::encode_to_vec(&other_user.id, bincode::config::standard())?;
    let other_user_data = bincode::encode_to_vec(&other_user, bincode::config::standard())?;
    let _ = other_manager.subscribe_item(
        BlogDefinitionSubscriptions::UserTopic,
        other_user_key,
        &other_user_data,
    );

    let post_key2 = bincode::encode_to_vec(&post.id, bincode::config::standard())?;
    let post_data2 = bincode::encode_to_vec(&post, bincode::config::standard())?;
    let _ = other_manager.subscribe_item(
        BlogDefinitionSubscriptions::PostTopic,
        post_key2,
        &post_data2,
    ); // Same post

    // Compare the two managers
    match subscription_manager.compare_with(&mut other_manager) {
        Ok(diffs) => {
            if diffs.is_empty() {
                println!("  🟰 No differences found between subscription trees");
            } else {
                println!("  🔄 Found {} topic(s) with differences:", diffs.len());
                for (topic, diff) in diffs {
                    println!(
                        "    {:?}: {} total differences",
                        topic,
                        diff.total_differences()
                    );
                    println!("      - Missing in first: {}", diff.missing_in_self.len());
                    println!("      - Missing in second: {}", diff.missing_in_other.len());
                    println!("      - Different values: {}", diff.different_values.len());
                }
            }
        }
        Err(e) => println!("  ❌ Failed to compare trees: {}", e),
    }

    println!("\n✨ Example completed successfully!");
    println!("\n💡 This demonstrates how subscription streams enable:");
    println!("   - Efficient tracking of data changes by topic");
    println!("   - Merkle tree-based comparison for synchronization");
    println!("   - Type-safe subscription management");
    println!("   - Scalable data organization by topic");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use blog::*;

    #[test]
    fn test_subscription_topics() {
        use netabase_store::traits::subscription::Subscriptions;

        // Test that we can iterate over subscription topics
        let topics: Vec<_> = BlogDefinition::subscriptions().collect();
        assert_eq!(topics.len(), 3);

        // Test that topics have expected names
        let topic_names = BlogDefinitionSubscriptions::all_topics();
        assert!(topic_names.contains(&"UserTopic"));
        assert!(topic_names.contains(&"PostTopic"));
        assert!(topic_names.contains(&"CommentTopic"));
    }

    #[test]
    fn test_subscription_manager_creation() {
        let manager = BlogDefinitionSubscriptionManager::new();

        // Test that all trees are initially empty
        assert_eq!(
            manager
                .get_tree(BlogDefinitionSubscriptions::UserTopic)
                .unwrap()
                .len(),
            0
        );
        assert_eq!(
            manager
                .get_tree(BlogDefinitionSubscriptions::PostTopic)
                .unwrap()
                .len(),
            0
        );
        assert_eq!(
            manager
                .get_tree(BlogDefinitionSubscriptions::CommentTopic)
                .unwrap()
                .len(),
            0
        );
    }

    #[test]
    fn test_topic_string_conversion() {
        let user_topic = BlogDefinitionSubscriptions::UserTopic;
        assert_eq!(user_topic.as_str(), "UserTopic");

        let post_topic = BlogDefinitionSubscriptions::PostTopic;
        assert_eq!(post_topic.as_str(), "PostTopic");

        let comment_topic = BlogDefinitionSubscriptions::CommentTopic;
        assert_eq!(comment_topic.as_str(), "CommentTopic");
    }
}
