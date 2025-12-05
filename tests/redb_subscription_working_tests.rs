//! Working subscription tree tests
//!
//! These tests verify that subscription trees work correctly using the correct key types.

#![cfg(all(feature = "redb", not(feature = "paxos")))]

use netabase_store::databases::redb_zerocopy::*;
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
    pub struct Post {
        #[primary_key]
        pub id: u64,
        pub title: String,
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
fn test_subscription_tree_basic_operations() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_subscription.redb");
    let store = RedbStoreZeroCopy::<TestDef>::new(&db_path).unwrap();

    // Test inserting and retrieving subscriptions
    let mut txn = store.begin_write().unwrap();
    let mut sub_tree = txn
        .open_subscription_tree(TestSubscriptions::UserNotifications)
        .unwrap();

    // Test initial state
    assert_eq!(sub_tree.subscription_count().unwrap(), 0);

    // For this test, let's use the TestKeys directly - they are the D::Keys type
    // We need to create keys that are compatible with D::Keys
    let user_key = TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(1)));
    let hash1 = create_test_hash(1);

    sub_tree.subscribe(user_key.clone(), hash1).unwrap();

    // Verify subscription exists
    assert_eq!(sub_tree.get_subscription(&user_key).unwrap(), Some(hash1));
    assert_eq!(sub_tree.subscription_count().unwrap(), 1);

    // Test non-existent subscription
    let non_existent_key = TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(999)));
    assert_eq!(sub_tree.get_subscription(&non_existent_key).unwrap(), None);

    drop(sub_tree);
    txn.commit().unwrap();
}

#[test]
fn test_subscription_tree_removal() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_subscription_removal.redb");
    let store = RedbStoreZeroCopy::<TestDef>::new(&db_path).unwrap();

    // Setup some subscriptions
    let mut txn = store.begin_write().unwrap();
    let mut sub_tree = txn
        .open_subscription_tree(TestSubscriptions::PostUpdates)
        .unwrap();

    let key1 = TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(1)));
    let key2 = TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(2)));
    let key3 = TestKeys::PostKey(PostKey::Primary(PostPrimaryKey(1)));
    let hash1 = create_test_hash(1);
    let hash2 = create_test_hash(2);
    let hash3 = create_test_hash(3);

    sub_tree.subscribe(key1.clone(), hash1).unwrap();
    sub_tree.subscribe(key2.clone(), hash2).unwrap();
    sub_tree.subscribe(key3.clone(), hash3).unwrap();

    assert_eq!(sub_tree.subscription_count().unwrap(), 3);

    // Test removing existing subscription
    let removed = sub_tree.unsubscribe(&key1).unwrap();
    assert_eq!(removed, Some(hash1));
    assert_eq!(sub_tree.subscription_count().unwrap(), 2);
    assert_eq!(sub_tree.get_subscription(&key1).unwrap(), None);

    // Test removing non-existent subscription
    let not_removed = sub_tree
        .unsubscribe(&TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(999))))
        .unwrap();
    assert_eq!(not_removed, None);
    assert_eq!(sub_tree.subscription_count().unwrap(), 2);

    // Verify remaining subscriptions still exist
    assert_eq!(sub_tree.get_subscription(&key2).unwrap(), Some(hash2));
    assert_eq!(sub_tree.get_subscription(&key3).unwrap(), Some(hash3));

    drop(sub_tree);
    txn.commit().unwrap();
}

#[test]
fn test_multiple_subscription_types() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_multiple_subscriptions.redb");
    let store = RedbStoreZeroCopy::<TestDef>::new(&db_path).unwrap();

    // Test that different subscription types have separate tables
    let mut txn = store.begin_write().unwrap();

    let key = TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(1)));
    let hash1 = create_test_hash(1);
    let hash2 = create_test_hash(2);

    // Work with user notifications tree atomically
    {
        let mut user_notif_tree = txn
            .open_subscription_tree(TestSubscriptions::UserNotifications)
            .unwrap();

        user_notif_tree.subscribe(key.clone(), hash1).unwrap();
        assert_eq!(user_notif_tree.get_subscription(&key).unwrap(), Some(hash1));
        assert_eq!(user_notif_tree.subscription_count().unwrap(), 1);
        
        // Remove from user notifications
        user_notif_tree.unsubscribe(&key).unwrap();
        assert_eq!(user_notif_tree.get_subscription(&key).unwrap(), None);
        
        // Add it back for later verification
        user_notif_tree.subscribe(key.clone(), hash1).unwrap();
    } // user_notif_tree is dropped here, releasing the borrow

    // Work with post updates tree atomically
    {
        let mut post_update_tree = txn
            .open_subscription_tree(TestSubscriptions::PostUpdates)
            .unwrap();

        post_update_tree.subscribe(key.clone(), hash2).unwrap();
        assert_eq!(post_update_tree.get_subscription(&key).unwrap(), Some(hash2));
        assert_eq!(post_update_tree.subscription_count().unwrap(), 1);
    } // post_update_tree is dropped here

    // Verify both subscription types work independently by checking final state
    {
        let user_notif_tree = txn
            .open_subscription_tree(TestSubscriptions::UserNotifications)
            .unwrap();
        assert_eq!(user_notif_tree.get_subscription(&key).unwrap(), Some(hash1));
    }
    
    {
        let post_update_tree = txn
            .open_subscription_tree(TestSubscriptions::PostUpdates)
            .unwrap();
        assert_eq!(post_update_tree.get_subscription(&key).unwrap(), Some(hash2));
    }

    txn.commit().unwrap();
}

#[test]
fn test_subscription_tree_persistence() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("persistent_subscription.redb");

    // Create and populate database
    {
        let store = RedbStoreZeroCopy::<TestDef>::new(&db_path).unwrap();
        let mut txn = store.begin_write().unwrap();
        let mut sub_tree = txn
            .open_subscription_tree(TestSubscriptions::UserNotifications)
            .unwrap();

        let key = TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(42)));
        let hash = create_test_hash(42);

        sub_tree.subscribe(key, hash).unwrap();
        drop(sub_tree);
        txn.commit().unwrap();
    }

    // Reopen database and verify persistence
    {
        let store = RedbStoreZeroCopy::<TestDef>::open(&db_path).unwrap();
        let txn = store.begin_read().unwrap();
        let sub_tree = txn
            .open_subscription_tree(TestSubscriptions::UserNotifications)
            .unwrap();

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
fn test_subscription_type_enumeration() {
    // Test that we can enumerate all subscription types
    let all_subscriptions = get_all_subscription_types::<TestDef>();

    assert_eq!(all_subscriptions.len(), 2);
    assert!(all_subscriptions.contains(&TestSubscriptions::UserNotifications));
    assert!(all_subscriptions.contains(&TestSubscriptions::PostUpdates));
}
