//! Comprehensive validation test for the subscription system
//!
//! This test validates that the subscription system is fully operational
//! after the RelationalLinks API implementation.

use chrono::Utc;
use netabase_store::{
    NetabaseDateTime, NetabaseModel, databases::sled_store::SledStore, netabase,
    netabase_definition_module, streams, traits::subscription::Subscriptions,
};

// Define test schema with subscription streams
#[netabase_definition_module(ValidationDefinition, ValidationKeys)]
#[streams(UserStream, PostStream, CommentStream)]
mod validation_schema {
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
    #[netabase(ValidationDefinition)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub name: String,
        #[secondary_key]
        pub email: String,
        #[bincode(with_serde)]
        pub created_at: NetabaseDateTime,
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
    #[netabase(ValidationDefinition)]
    pub struct Post {
        #[primary_key]
        pub id: String,
        pub title: String,
        pub content: String,
        #[secondary_key]
        pub author_id: u64,
        #[bincode(with_serde)]
        pub published_at: NetabaseDateTime,
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
    #[netabase(ValidationDefinition)]
    pub struct Comment {
        #[primary_key]
        pub id: u64,
        pub post_id: String,
        #[secondary_key]
        pub author_id: u64,
        pub content: String,
        #[bincode(with_serde)]
        pub created_at: NetabaseDateTime,
    }
}

use validation_schema::*;

#[test]
fn test_subscription_system_is_fully_operational() {
    // Test 1: Verify subscription topics are available
    let topics = ValidationDefinition::all_subscriptions();
    assert_eq!(topics.len(), 3);
    assert!(topics.contains(&ValidationDefinitionSubscriptions::UserStream));
    assert!(topics.contains(&ValidationDefinitionSubscriptions::PostStream));
    assert!(topics.contains(&ValidationDefinitionSubscriptions::CommentStream));

    // Test 2: Create subscription manager
    let mut manager = ValidationDefinitionSubscriptionManager::new();
    assert_eq!(manager.stats().total_items, 0);
    assert_eq!(manager.stats().active_topics, 0);

    // Test 3: Create test data
    let now = Utc::now();

    let user = User {
        id: 1,
        name: "Alice Johnson".to_string(),
        email: "alice@example.com".to_string(),
        created_at: now,
    };

    let post = Post {
        id: "post-123".to_string(),
        title: "Test Post".to_string(),
        content: "This is a test post content.".to_string(),
        author_id: 1,
        published_at: now,
    };

    let comment = Comment {
        id: 101,
        post_id: "post-123".to_string(),
        author_id: 1,
        content: "Great post!".to_string(),
        created_at: now,
    };

    // Test 4: Add items to subscription trees
    let user_key = bincode::encode_to_vec(&user.id, bincode::config::standard()).unwrap();
    let user_data = bincode::encode_to_vec(&user, bincode::config::standard()).unwrap();

    let post_key = bincode::encode_to_vec(&post.id, bincode::config::standard()).unwrap();
    let post_data = bincode::encode_to_vec(&post, bincode::config::standard()).unwrap();

    let comment_key = bincode::encode_to_vec(&comment.id, bincode::config::standard()).unwrap();
    let comment_data = bincode::encode_to_vec(&comment, bincode::config::standard()).unwrap();

    // Subscribe items
    manager
        .subscribe_item(
            ValidationDefinitionSubscriptions::UserStream,
            user_key.clone(),
            &user_data,
        )
        .unwrap();

    manager
        .subscribe_item(
            ValidationDefinitionSubscriptions::PostStream,
            post_key.clone(),
            &post_data,
        )
        .unwrap();

    manager
        .subscribe_item(
            ValidationDefinitionSubscriptions::CommentStream,
            comment_key.clone(),
            &comment_data,
        )
        .unwrap();

    // Test 5: Verify subscription statistics
    let stats = manager.stats();
    assert_eq!(stats.total_items, 3);
    assert_eq!(stats.active_topics, 3);

    // Test 6: Verify merkle roots are generated
    let user_root = manager
        .topic_merkle_root(ValidationDefinitionSubscriptions::UserStream)
        .unwrap();
    let post_root = manager
        .topic_merkle_root(ValidationDefinitionSubscriptions::PostStream)
        .unwrap();
    let comment_root = manager
        .topic_merkle_root(ValidationDefinitionSubscriptions::CommentStream)
        .unwrap();

    assert!(user_root.is_some());
    assert!(post_root.is_some());
    assert!(comment_root.is_some());

    // Test 7: Test synchronization scenarios
    let mut remote_manager = ValidationDefinitionSubscriptionManager::new();

    // Remote has same user and post, but different comment
    manager
        .subscribe_item(
            ValidationDefinitionSubscriptions::UserStream,
            user_key.clone(),
            &user_data,
        )
        .unwrap();

    manager
        .subscribe_item(
            ValidationDefinitionSubscriptions::PostStream,
            post_key.clone(),
            &post_data,
        )
        .unwrap();

    let different_comment = Comment {
        id: 102,
        post_id: "post-123".to_string(),
        author_id: 1,
        content: "Different comment".to_string(),
        created_at: now,
    };

    let diff_comment_key =
        bincode::encode_to_vec(&different_comment.id, bincode::config::standard()).unwrap();
    let diff_comment_data =
        bincode::encode_to_vec(&different_comment, bincode::config::standard()).unwrap();

    remote_manager
        .subscribe_item(
            ValidationDefinitionSubscriptions::CommentStream,
            diff_comment_key,
            &diff_comment_data,
        )
        .unwrap();

    // Compare managers
    let diffs = manager.compare_with(&mut remote_manager).unwrap();
    assert!(
        !diffs.is_empty(),
        "Should have differences between managers"
    );

    // Test 8: Test removal functionality
    let removed_hash = manager
        .unsubscribe_item(
            ValidationDefinitionSubscriptions::CommentStream,
            &comment_key,
        )
        .unwrap();
    assert!(removed_hash.is_some());

    let updated_stats = manager.stats();
    assert_eq!(updated_stats.total_items, 2);

    // Test 9: Verify tree access
    assert!(
        manager
            .get_tree(ValidationDefinitionSubscriptions::UserStream)
            .is_some()
    );
    assert!(
        manager
            .get_tree(ValidationDefinitionSubscriptions::PostStream)
            .is_some()
    );
    assert!(
        manager
            .get_tree(ValidationDefinitionSubscriptions::CommentStream)
            .is_some()
    );
}

#[test]
fn test_database_integration_with_subscriptions() {
    // Create temporary sled store
    let store = SledStore::<ValidationDefinition>::temp().unwrap();
    let mut manager = ValidationDefinitionSubscriptionManager::new();

    let now = Utc::now();
    let user = User {
        id: 1,
        name: "Bob Smith".to_string(),
        email: "bob@example.com".to_string(),
        created_at: now,
    };

    // Store in database
    let user_tree = store.open_tree::<User>();
    user_tree.put(user.clone()).unwrap();

    // Manually update subscription (in real implementation this would be automatic)
    let user_key = bincode::encode_to_vec(&user.id, bincode::config::standard()).unwrap();
    let user_data = bincode::encode_to_vec(&user, bincode::config::standard()).unwrap();

    manager
        .subscribe_item(
            ValidationDefinitionSubscriptions::UserStream,
            user_key,
            &user_data,
        )
        .unwrap();

    // Verify subscription was updated
    assert_eq!(manager.stats().total_items, 1);
    assert_eq!(manager.stats().active_topics, 1);

    // Verify data can be retrieved from database
    let retrieved_user = user_tree.get(validation_schema::UserPrimaryKey(user.id)).unwrap();
    assert!(retrieved_user.is_some());
    assert_eq!(retrieved_user.unwrap(), user);
}

#[test]
fn test_topic_string_conversions() {
    // Test topic name conversions
    assert_eq!(
        ValidationDefinition::topic_name(ValidationDefinitionSubscriptions::UserStream),
        "UserStream"
    );
    assert_eq!(
        ValidationDefinition::topic_name(ValidationDefinitionSubscriptions::PostStream),
        "PostStream"
    );
    assert_eq!(
        ValidationDefinition::topic_name(ValidationDefinitionSubscriptions::CommentStream),
        "CommentStream"
    );
}

#[test]
fn test_subscription_tree_individual_operations() {
    let mut manager = ValidationDefinitionSubscriptionManager::new();

    // Test empty state
    assert_eq!(
        manager
            .get_tree(ValidationDefinitionSubscriptions::UserStream)
            .unwrap()
            .len(),
        0
    );

    let now = Utc::now();
    let user = User {
        id: 1,
        name: "Test User".to_string(),
        email: "test@example.com".to_string(),
        created_at: now,
    };

    let user_key = bincode::encode_to_vec(&user.id, bincode::config::standard()).unwrap();
    let user_data = bincode::encode_to_vec(&user, bincode::config::standard()).unwrap();

    // Test subscription
    manager
        .subscribe_item(
            ValidationDefinitionSubscriptions::UserStream,
            user_key.clone(),
            &user_data,
        )
        .unwrap();

    assert_eq!(
        manager
            .get_tree(ValidationDefinitionSubscriptions::UserStream)
            .unwrap()
            .len(),
        1
    );

    // Test contains key
    assert!(manager.contains_key_in_any_topic(&user_key));

    // Test topics containing key
    let topics = manager.topics_containing_key(&user_key);
    assert_eq!(topics.len(), 1);
    assert_eq!(topics[0], ValidationDefinitionSubscriptions::UserStream);

    // Test unsubscription
    let removed = manager
        .unsubscribe_item(ValidationDefinitionSubscriptions::UserStream, &user_key)
        .unwrap();
    assert!(removed.is_some());

    assert_eq!(
        manager
            .get_tree(ValidationDefinitionSubscriptions::UserStream)
            .unwrap()
            .len(),
        0
    );
    assert!(!manager.contains_key_in_any_topic(&user_key));
}

#[test]
fn test_subscription_merkle_tree_consistency() {
    let mut manager1 = ValidationDefinitionSubscriptionManager::new();
    let mut manager2 = ValidationDefinitionSubscriptionManager::new();

    let now = Utc::now();
    let user = User {
        id: 42,
        name: "Consistency Test".to_string(),
        email: "consistency@example.com".to_string(),
        created_at: now,
    };

    let user_key = bincode::encode_to_vec(&user.id, bincode::config::standard()).unwrap();
    let user_data = bincode::encode_to_vec(&user, bincode::config::standard()).unwrap();

    // Add same data to both managers
    manager1
        .subscribe_item(
            ValidationDefinitionSubscriptions::UserStream,
            user_key.clone(),
            &user_data,
        )
        .unwrap();

    manager2
        .subscribe_item(
            ValidationDefinitionSubscriptions::UserStream,
            user_key,
            &user_data,
        )
        .unwrap();

    // Merkle roots should be identical
    let root1 = manager1
        .topic_merkle_root(ValidationDefinitionSubscriptions::UserStream)
        .unwrap();
    let root2 = manager2
        .topic_merkle_root(ValidationDefinitionSubscriptions::UserStream)
        .unwrap();

    assert_eq!(root1, root2);

    // Comparison should show no differences
    let diffs = manager1.compare_with(&mut manager2).unwrap();
    assert!(diffs.is_empty());
}

#[test]
fn test_subscription_system_clear_operations() {
    let mut manager = ValidationDefinitionSubscriptionManager::new();

    let now = Utc::now();
    let user = User {
        id: 1,
        name: "Clear Test".to_string(),
        email: "clear@example.com".to_string(),
        created_at: now,
    };

    let user_key = bincode::encode_to_vec(&user.id, bincode::config::standard()).unwrap();
    let user_data = bincode::encode_to_vec(&user, bincode::config::standard()).unwrap();

    // Add data to multiple topics
    manager
        .subscribe_item(
            ValidationDefinitionSubscriptions::UserStream,
            user_key.clone(),
            &user_data,
        )
        .unwrap();

    // Verify data was added
    assert_eq!(manager.stats().total_items, 1);

    // Test clear all
    manager.clear_all().unwrap();
    assert_eq!(manager.stats().total_items, 0);
    assert_eq!(manager.stats().active_topics, 0);

    // Verify all trees are empty
    assert_eq!(
        manager
            .get_tree(ValidationDefinitionSubscriptions::UserStream)
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        manager
            .get_tree(ValidationDefinitionSubscriptions::PostStream)
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        manager
            .get_tree(ValidationDefinitionSubscriptions::CommentStream)
            .unwrap()
            .len(),
        0
    );
}
