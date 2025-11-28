//! Simple test for streams functionality
//!
//! This is a minimal example to test the streams macro integration.

use netabase_store::{NetabaseModel, netabase, netabase_definition_module, streams};

// Define a simple blog schema with subscription streams
#[netabase_definition_module(SimpleBlogDefinition, SimpleBlogKeys)]
#[streams(UserTopic, PostTopic)]
mod simple_blog {
    use super::*;

    /// Simple user model
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
    #[netabase(SimpleBlogDefinition)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub username: String,
    }

    /// Simple post model
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
    #[netabase(SimpleBlogDefinition)]
    pub struct Post {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub author_id: u64,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Simple Netabase Streams Test");
    println!("================================\n");

    // Just test basic compilation and functionality
    println!("📋 Testing basic subscription functionality...");

    // Test subscription topics enum exists
    let user_topic = SimpleBlogDefinitionSubscriptions::UserTopic;
    let post_topic = SimpleBlogDefinitionSubscriptions::PostTopic;

    println!("  - UserTopic: {}", user_topic.as_str());
    println!("  - PostTopic: {}", post_topic.as_str());

    // Test subscription manager creation
    println!("\n🔧 Testing subscription manager...");
    let manager = SimpleBlogDefinitionSubscriptionManager::new();
    println!("  ✅ Manager created successfully");

    // Test accessing individual trees
    println!("\n🌲 Testing subscription tree access:");
    if let Some(_user_tree) = manager.get_tree(user_topic) {
        println!("  ✅ UserTopic tree accessible");
    }

    if let Some(_post_tree) = manager.get_tree(post_topic) {
        println!("  ✅ PostTopic tree accessible");
    }

    println!("\n✨ Simple streams test completed successfully!");
    println!("\n💡 This demonstrates:");
    println!("   - Streams macro integration with netabase_definition_module");
    println!("   - Generated subscription topics enum");
    println!("   - Subscription manager creation");
    println!("   - Topic-based tree access");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::simple_blog::*;

    #[test]
    fn test_subscription_enum_generation() {
        // Test that the subscription enum was generated correctly
        let user_topic = SimpleBlogDefinitionSubscriptions::UserTopic;
        let post_topic = SimpleBlogDefinitionSubscriptions::PostTopic;

        assert_eq!(user_topic.as_str(), "UserTopic");
        assert_eq!(post_topic.as_str(), "PostTopic");
    }

    #[test]
    fn test_subscription_manager() {
        let manager = SimpleBlogDefinitionSubscriptionManager::new();

        // Test that trees are accessible
        assert!(
            manager
                .get_tree(SimpleBlogDefinitionSubscriptions::UserTopic)
                .is_some()
        );
        assert!(
            manager
                .get_tree(SimpleBlogDefinitionSubscriptions::PostTopic)
                .is_some()
        );
    }

    #[test]
    fn test_topic_names() {
        // Test topic names
        let topic_names = SimpleBlogDefinitionSubscriptions::all_topics();
        assert!(topic_names.contains(&"UserTopic"));
        assert!(topic_names.contains(&"PostTopic"));
        assert_eq!(topic_names.len(), 2);
    }
}
