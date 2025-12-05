//! Integration tests for subscription tree functionality
//!
//! These tests verify that the subscription trees work correctly with
//! different subscription types, insertion, removal, and queries.

#![cfg(all(feature = "redb", not(feature = "paxos")))]

use std::println;

use netabase_store::databases::redb_zerocopy::*;
use netabase_store::subscriptions;
use netabase_store::traits::definition::NetabaseDefinitionWithSubscription;
use netabase_store::{NetabaseModel, netabase, netabase_definition_module};

// Define subscription types with correct strum derives
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
    CommentReplies,
    SystemAlerts,
}

// Test definition and models
#[netabase_definition_module(TestDef, TestKeys)]
#[subscriptions(UserNotifications, PostUpdates, CommentReplies, SystemAlerts)]
mod test_models {
    use netabase_store::subscriptions;

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
        #[secondary_key]
        pub email: String,
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
        pub content: String,
        #[secondary_key]
        pub author_id: u64,
    }
}

use test_models::*;

// Implement subscription trait for our test definition
impl NetabaseDefinitionWithSubscription for TestDef {
    type Subscriptions = TestSubscriptions;
}

/// Helper function to create a temporary database
fn create_temp_store() -> RedbStoreZeroCopy<TestDef> {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test_subscription.redb");
    RedbStoreZeroCopy::<TestDef>::new(&db_path).unwrap()
}

/// Helper function to create test data hash
fn create_test_hash(value: u8) -> [u8; 32] {
    [value; 32]
}

#[test]
fn test_subscription_tree_basic_operations() {
    let store = create_temp_store();

    // Test inserting and retrieving subscriptions
    let mut txn = store.begin_write().unwrap();
    let mut sub_tree = txn
        .open_subscription_tree(TestSubscriptions::UserNotifications)
        .unwrap();

    // Test initial state
    assert_eq!(sub_tree.subscription_count().unwrap(), 0);

    // Insert some subscriptions using proper key construction
    let user_key = TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(1)));
    let post_key = TestKeys::PostKey(PostKey::Primary(PostPrimaryKey(2)));
    let hash1 = create_test_hash(1);
    let hash2 = create_test_hash(2);

    sub_tree.subscribe(user_key.clone(), hash1).unwrap();
    sub_tree.subscribe(post_key.clone(), hash2).unwrap();

    println!("Sub Tree: {:?}", sub_tree.subscription_count());

    // Verify subscriptions exist
    assert_eq!(sub_tree.get_subscription(&user_key).unwrap(), Some(hash1));
    assert_eq!(sub_tree.get_subscription(&post_key).unwrap(), Some(hash2));
    assert_eq!(sub_tree.subscription_count().unwrap(), 2);

    // Test non-existent subscription
    let non_existent_key = TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(999)));
    assert_eq!(sub_tree.get_subscription(&non_existent_key).unwrap(), None);

    drop(sub_tree);
    txn.commit().unwrap();
}

#[test]
fn test_subscription_tree_removal() {
    let store = create_temp_store();

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
fn test_subscription_tree_clear() {
    let store = create_temp_store();

    // Setup multiple subscriptions
    let mut txn = store.begin_write().unwrap();
    let mut sub_tree = txn
        .open_subscription_tree(TestSubscriptions::CommentReplies)
        .unwrap();

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

    drop(sub_tree);
    txn.commit().unwrap();
}

#[test]
fn test_multiple_subscription_types() {
    let store = create_temp_store();

    // Test that different subscription types have separate tables
    let mut txn = store.begin_write().unwrap();

    let key = TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(1)));
    let hash1 = create_test_hash(1);
    let hash2 = create_test_hash(2);
    let hash3 = create_test_hash(3);

    // Work with user notifications tree atomically
    {
        let mut user_notif_tree = txn
            .open_subscription_tree(TestSubscriptions::UserNotifications)
            .unwrap();

        user_notif_tree.subscribe(key.clone(), hash1).unwrap();
        assert_eq!(user_notif_tree.get_subscription(&key).unwrap(), Some(hash1));
        assert_eq!(user_notif_tree.subscription_count().unwrap(), 1);

        // Remove from user notifications and add back for final verification
        user_notif_tree.unsubscribe(&key).unwrap();
        assert_eq!(user_notif_tree.get_subscription(&key).unwrap(), None);
        user_notif_tree.subscribe(key.clone(), hash1).unwrap();
    } // user_notif_tree is dropped here

    // Work with post updates tree atomically
    {
        let mut post_update_tree = txn
            .open_subscription_tree(TestSubscriptions::PostUpdates)
            .unwrap();

        post_update_tree.subscribe(key.clone(), hash2).unwrap();
        assert_eq!(
            post_update_tree.get_subscription(&key).unwrap(),
            Some(hash2)
        );
        assert_eq!(post_update_tree.subscription_count().unwrap(), 1);
    } // post_update_tree is dropped here

    // Work with system alerts tree atomically
    {
        let mut system_alert_tree = txn
            .open_subscription_tree(TestSubscriptions::SystemAlerts)
            .unwrap();

        system_alert_tree.subscribe(key.clone(), hash3).unwrap();
        assert_eq!(
            system_alert_tree.get_subscription(&key).unwrap(),
            Some(hash3)
        );
        assert_eq!(system_alert_tree.subscription_count().unwrap(), 1);
    } // system_alert_tree is dropped here

    // Verify all subscription types work independently by checking final state
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
        assert_eq!(
            post_update_tree.get_subscription(&key).unwrap(),
            Some(hash2)
        );
    }

    {
        let system_alert_tree = txn
            .open_subscription_tree(TestSubscriptions::SystemAlerts)
            .unwrap();
        assert_eq!(
            system_alert_tree.get_subscription(&key).unwrap(),
            Some(hash3)
        );
    }

    txn.commit().unwrap();
}

#[test]
fn test_read_only_subscription_tree() {
    let store = create_temp_store();

    // Setup some data
    {
        let mut txn = store.begin_write().unwrap();
        let mut sub_tree = txn
            .open_subscription_tree(TestSubscriptions::UserNotifications)
            .unwrap();

        let key1 = TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(1)));
        let key2 = TestKeys::PostKey(PostKey::Primary(PostPrimaryKey(1)));
        let hash1 = create_test_hash(1);
        let hash2 = create_test_hash(2);

        sub_tree.subscribe(key1, hash1).unwrap();
        sub_tree.subscribe(key2, hash2).unwrap();

        drop(sub_tree);
        txn.commit().unwrap();
    }

    // Test read-only access
    let txn = store.begin_read().unwrap();
    let sub_tree = txn
        .open_subscription_tree(TestSubscriptions::UserNotifications)
        .unwrap();

    // Verify we can read the data
    assert_eq!(sub_tree.subscription_count().unwrap(), 2);
    assert_eq!(
        sub_tree
            .get_subscription(&TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(1))))
            .unwrap(),
        Some(create_test_hash(1))
    );
    assert_eq!(
        sub_tree
            .get_subscription(&TestKeys::PostKey(PostKey::Primary(PostPrimaryKey(1))))
            .unwrap(),
        Some(create_test_hash(2))
    );

    // Verify non-existent key returns None
    assert_eq!(
        sub_tree
            .get_subscription(&TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(999))))
            .unwrap(),
        None
    );
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
fn test_subscription_tree_bulk_operations() {
    let store = create_temp_store();

    let mut txn = store.begin_write().unwrap();
    let mut sub_tree = txn
        .open_subscription_tree(TestSubscriptions::PostUpdates)
        .unwrap();

    // Insert many subscriptions in a single transaction
    const NUM_SUBSCRIPTIONS: u64 = 1000;

    for i in 1..=NUM_SUBSCRIPTIONS {
        let key = if i % 2 == 0 {
            TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(i)))
        } else {
            TestKeys::PostKey(PostKey::Primary(PostPrimaryKey(i)))
        };
        let hash = create_test_hash((i % 256) as u8);
        sub_tree.subscribe(key, hash).unwrap();
    }

    assert_eq!(
        sub_tree.subscription_count().unwrap(),
        NUM_SUBSCRIPTIONS as usize
    );

    // Verify random samples
    for i in [1, 100, 500, 999].iter() {
        let key = if i % 2 == 0 {
            TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(*i)))
        } else {
            TestKeys::PostKey(PostKey::Primary(PostPrimaryKey(*i)))
        };
        let expected_hash = create_test_hash((i % 256) as u8);
        assert_eq!(
            sub_tree.get_subscription(&key).unwrap(),
            Some(expected_hash)
        );
    }

    // Remove half of the subscriptions
    for i in (1..=NUM_SUBSCRIPTIONS).step_by(2) {
        let key = TestKeys::PostKey(PostKey::Primary(PostPrimaryKey(i)));
        sub_tree.unsubscribe(&key).unwrap();
    }

    assert_eq!(
        sub_tree.subscription_count().unwrap(),
        (NUM_SUBSCRIPTIONS / 2) as usize
    );

    drop(sub_tree);
    txn.commit().unwrap();
}

#[test]
fn test_subscription_type_enumeration() {
    // Test that we can enumerate all subscription types
    let all_subscriptions = get_all_subscription_types::<TestDef>();

    assert_eq!(all_subscriptions.len(), 4);
    assert!(all_subscriptions.contains(&TestSubscriptions::UserNotifications));
    assert!(all_subscriptions.contains(&TestSubscriptions::PostUpdates));
    assert!(all_subscriptions.contains(&TestSubscriptions::CommentReplies));
    assert!(all_subscriptions.contains(&TestSubscriptions::SystemAlerts));
}

#[test]
fn test_subscription_tree_update_subscription() {
    let store = create_temp_store();

    let mut txn = store.begin_write().unwrap();
    let mut sub_tree = txn
        .open_subscription_tree(TestSubscriptions::UserNotifications)
        .unwrap();

    let key = TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(1)));
    let hash1 = create_test_hash(1);
    let hash2 = create_test_hash(2);

    // Initial subscription
    sub_tree.subscribe(key.clone(), hash1).unwrap();
    assert_eq!(sub_tree.get_subscription(&key).unwrap(), Some(hash1));
    assert_eq!(sub_tree.subscription_count().unwrap(), 1);

    // Update subscription (overwrites previous)
    sub_tree.subscribe(key.clone(), hash2).unwrap();
    assert_eq!(sub_tree.get_subscription(&key).unwrap(), Some(hash2));
    assert_eq!(sub_tree.subscription_count().unwrap(), 1); // Count should remain the same

    drop(sub_tree);
    txn.commit().unwrap();
}

#[test]
fn test_subscription_tree_error_handling() {
    let store = create_temp_store();

    // Test accessing subscription tree for empty data
    // First create the table by actually adding then removing data
    {
        let mut txn = store.begin_write().unwrap();
        let mut sub_tree = txn
            .open_subscription_tree(TestSubscriptions::UserNotifications)
            .unwrap();

        // Add a subscription to create the table, then remove it
        let temp_key = TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(999)));
        let temp_hash = create_test_hash(255);
        sub_tree.subscribe(temp_key.clone(), temp_hash).unwrap();
        sub_tree.unsubscribe(&temp_key).unwrap();

        drop(sub_tree);
        txn.commit().unwrap();
    }

    // Now test read access to empty table
    let txn = store.begin_read().unwrap();
    let sub_tree = txn
        .open_subscription_tree(TestSubscriptions::UserNotifications)
        .unwrap();

    // Should handle empty table gracefully
    assert_eq!(sub_tree.subscription_count().unwrap(), 0);
    assert_eq!(
        sub_tree
            .get_subscription(&TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(1))))
            .unwrap(),
        None
    );
}

#[test]
fn test_concurrent_subscription_operations() {
    let store = create_temp_store();

    // Setup initial data
    {
        let mut txn = store.begin_write().unwrap();
        let mut sub_tree = txn
            .open_subscription_tree(TestSubscriptions::UserNotifications)
            .unwrap();

        for i in 1..=10 {
            let key = TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(i)));
            let hash = create_test_hash(i as u8);
            sub_tree.subscribe(key, hash).unwrap();
        }

        drop(sub_tree);
        txn.commit().unwrap();
    }

    // Multiple read transactions can access the same data
    let txn1 = store.begin_read().unwrap();
    let txn2 = store.begin_read().unwrap();

    let sub_tree1 = txn1
        .open_subscription_tree(TestSubscriptions::UserNotifications)
        .unwrap();
    let sub_tree2 = txn2
        .open_subscription_tree(TestSubscriptions::UserNotifications)
        .unwrap();

    // Both should see the same data
    assert_eq!(sub_tree1.subscription_count().unwrap(), 10);
    assert_eq!(sub_tree2.subscription_count().unwrap(), 10);

    assert_eq!(
        sub_tree1
            .get_subscription(&TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(5))))
            .unwrap(),
        Some(create_test_hash(5))
    );
    assert_eq!(
        sub_tree2
            .get_subscription(&TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(5))))
            .unwrap(),
        Some(create_test_hash(5))
    );
}
