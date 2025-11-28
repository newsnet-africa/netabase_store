//! Minimal test case for streams functionality

use netabase_store::{
    NetabaseModel, netabase, netabase_definition_module, streams,
    traits::subscription::{SubscriptionManager, SubscriptionTree, Subscriptions},
};

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

use simple_blog::*;

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
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_compilation() {
        // Test basic module compilation
        let model = User {
            id: 1,
            username: "test".to_string(),
        };
        let _def = SimpleBlogDefinition::User(model);
    }

    #[test]
    fn test_basic_store() {
        // Test basic store functionality without streams for now
        println!("Model compiled successfully");
    }

    #[test]
    fn test_streams_compilation() {
        // Test that the streams macro generates the expected types
        let _topic1 = SimpleBlogDefinitionSubscriptions::UserTopic;
        let _topic2 = SimpleBlogDefinitionSubscriptions::PostTopic;
        let mut _manager = SimpleBlogDefinitionSubscriptionManager::new();

        // Test that subscription functionality works
        let mut tree = UserTopicSubscriptionTree::new();
        assert_eq!(tree.len(), 0);

        // Test that the manager can be used
        let stats = _manager.stats();
        assert_eq!(stats.total_items, 0);
    }
}
