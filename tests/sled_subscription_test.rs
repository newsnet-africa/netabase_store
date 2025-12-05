//! Sled subscription tree test
//!
//! This test verifies that subscription trees work correctly with the Sled store.

#![cfg(all(feature = "sled", not(feature = "paxos")))]

use netabase_store::databases::sled_store::SledStore;
use netabase_store::traits::definition::NetabaseDefinitionWithSubscription;
use netabase_store::{NetabaseModel, netabase, netabase_definition_module};

// Define subscription types
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum::EnumIter,
    strum::EnumDiscriminants,
    bincode::Encode,
    bincode::Decode,
    serde::Serialize,
    serde::Deserialize,
)]
#[strum_discriminants(derive(strum::AsRefStr, strum::Display, Hash))]
pub enum TestSubscriptions {
    UserNotifications,
    PostUpdates,
}

// Test definition and models
#[netabase_definition_module(TestDef, TestKeys)]
mod test_models {
    use super::*;

    #[derive(
        NetabaseModel,
        Debug,
        Clone,
        PartialEq,
        Eq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(TestDef)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub name: String,
    }
}

use test_models::*;

// Implement subscription trait for our test definition
impl NetabaseDefinitionWithSubscription for TestDef {
    type Subscriptions = TestSubscriptions;
}

/// Helper function to create test data hash
fn create_test_hash(value: u8) -> [u8; 32] {
    [value; 32]
}

#[test]
fn test_sled_subscription_basic_operations() {
    let store = SledStore::<TestDef>::temp().unwrap();

    // Test inserting and retrieving subscriptions
    let mut sub_tree = store.open_subscription_tree(TestSubscriptions::UserNotifications);

    // Test initial state
    assert_eq!(sub_tree.subscription_count().unwrap(), 0);

    // Insert a subscription using the primary key directly
    let user_key = TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(1)));
    let hash1 = create_test_hash(1);

    sub_tree.subscribe(user_key.clone(), hash1).unwrap();

    // Verify subscription exists
    assert_eq!(sub_tree.get_subscription(&user_key).unwrap(), Some(hash1));
    assert_eq!(sub_tree.subscription_count().unwrap(), 1);

    // Test non-existent subscription
    let non_existent_key = TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(999)));
    assert_eq!(sub_tree.get_subscription(&non_existent_key).unwrap(), None);

    // Test removal
    let removed = sub_tree.unsubscribe(&user_key).unwrap();
    assert_eq!(removed, Some(hash1));
    assert_eq!(sub_tree.subscription_count().unwrap(), 0);
}

#[test]
fn test_sled_subscription_multiple_types() {
    let store = SledStore::<TestDef>::temp().unwrap();

    // Test that different subscription types have separate trees
    let key = TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(1)));
    let hash1 = create_test_hash(1);
    let hash2 = create_test_hash(2);

    // Work with user notifications tree
    {
        let mut user_notif_tree = store.open_subscription_tree(TestSubscriptions::UserNotifications);
        user_notif_tree.subscribe(key.clone(), hash1).unwrap();
        assert_eq!(user_notif_tree.get_subscription(&key).unwrap(), Some(hash1));
        assert_eq!(user_notif_tree.subscription_count().unwrap(), 1);
    }

    // Work with post updates tree
    {
        let mut post_update_tree = store.open_subscription_tree(TestSubscriptions::PostUpdates);
        post_update_tree.subscribe(key.clone(), hash2).unwrap();
        assert_eq!(post_update_tree.get_subscription(&key).unwrap(), Some(hash2));
        assert_eq!(post_update_tree.subscription_count().unwrap(), 1);
    }

    // Verify both subscription types work independently
    {
        let user_notif_tree = store.open_subscription_tree(TestSubscriptions::UserNotifications);
        assert_eq!(user_notif_tree.get_subscription(&key).unwrap(), Some(hash1));
    }
    
    {
        let post_update_tree = store.open_subscription_tree(TestSubscriptions::PostUpdates);
        assert_eq!(post_update_tree.get_subscription(&key).unwrap(), Some(hash2));
    }
}

#[test]
fn test_sled_subscription_persistence() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("persistent_subscription.sled");

    // Create and populate database
    {
        let store = SledStore::<TestDef>::new(&db_path).unwrap();
        let mut sub_tree = store.open_subscription_tree(TestSubscriptions::UserNotifications);

        let key = TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(42)));
        let hash = create_test_hash(42);

        sub_tree.subscribe(key, hash).unwrap();
        store.flush().unwrap(); // Ensure data is written to disk
    }

    // Reopen database and verify persistence
    {
        let store = SledStore::<TestDef>::new(&db_path).unwrap();
        let sub_tree = store.open_subscription_tree(TestSubscriptions::UserNotifications);

        assert_eq!(sub_tree.subscription_count().unwrap(), 1);
        assert_eq!(
            sub_tree
                .get_subscription(&TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(42))))
                .unwrap(),
            Some(create_test_hash(42))
        );
    }
}

#[test]
fn test_sled_subscription_clear() {
    let store = SledStore::<TestDef>::temp().unwrap();
    let mut sub_tree = store.open_subscription_tree(TestSubscriptions::UserNotifications);

    // Add multiple subscriptions
    for i in 1..=5 {
        let key = TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(i)));
        let hash = create_test_hash(i as u8);
        sub_tree.subscribe(key, hash).unwrap();
    }

    assert_eq!(sub_tree.subscription_count().unwrap(), 5);

    // Clear all subscriptions
    sub_tree.clear_subscriptions().unwrap();

    // Verify all subscriptions are gone
    assert_eq!(sub_tree.subscription_count().unwrap(), 0);
    for i in 1..=5 {
        let key = TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(i)));
        assert_eq!(sub_tree.get_subscription(&key).unwrap(), None);
    }
}