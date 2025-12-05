//! Minimal subscription tree test to verify basic functionality
//!
//! This is a simplified test to verify the subscription tree works correctly.

#![cfg(all(feature = "redb", not(feature = "paxos")))]

use netabase_store::databases::redb_zerocopy::*;
use netabase_store::traits::definition::NetabaseDefinitionWithSubscription;
use netabase_store::{NetabaseModel, netabase, netabase_definition_module, subscriptions};

// Test definition and models
#[netabase_definition_module(TestDef, TestKeys)]
#[subscriptions(Hi, There)]
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
}

use test_models::*;

// Implement subscription trait for our test definition
impl NetabaseDefinitionWithSubscription for TestDef {
    type Subscriptions = TestModelsSubscriptions;
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
fn test_basic_subscription_operations() {
    let store = create_temp_store();

    // Test inserting and retrieving subscriptions
    let mut txn = store.begin_write().unwrap();
    let mut sub_tree = txn
        .open_subscription_tree(TestModelsSubscriptions::Hi)
        .unwrap();

    // Test initial state
    assert_eq!(sub_tree.subscription_count().unwrap(), 0);

    // Insert a subscription using the primary key directly
    let user_key = UserPrimaryKey(1);
    let hash1 = create_test_hash(1);

    sub_tree
        .subscribe(TestKeys::UserKey(UserKey::Primary(user_key.clone())), hash1)
        .unwrap();

    // Verify subscription exists
    assert_eq!(
        sub_tree
            .get_subscription(&TestKeys::UserKey(UserKey::Primary(user_key.clone())))
            .unwrap(),
        Some(hash1)
    );
    assert_eq!(sub_tree.subscription_count().unwrap(), 1);

    // Test non-existent subscription
    let non_existent_key = TestKeys::UserKey(UserKey::Primary(UserPrimaryKey(999)));
    assert_eq!(sub_tree.get_subscription(&non_existent_key).unwrap(), None);

    drop(sub_tree);
    txn.commit().unwrap();
}

#[test]
fn test_model_vec_u8_conversions() {
    // Test model to/from Vec<u8>
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    // Test From<User> for Vec<u8>
    let user_bytes: Vec<u8> = user.clone().into();
    assert!(!user_bytes.is_empty());

    // Test TryFrom<Vec<u8>> for User
    let user_recovered: User = user_bytes.try_into().unwrap();
    assert_eq!(user, user_recovered);
}

#[test]
fn test_primary_key_vec_u8_conversions() {
    let primary_key = UserPrimaryKey(42);

    // Test From<UserPrimaryKey> for Vec<u8>
    let key_bytes: Vec<u8> = primary_key.clone().into();
    assert!(!key_bytes.is_empty());

    // Test TryFrom<Vec<u8>> for UserPrimaryKey
    let key_recovered: UserPrimaryKey = key_bytes.try_into().unwrap();
    assert_eq!(primary_key, key_recovered);
}

#[test]
fn test_secondary_key_vec_u8_conversions() {
    let secondary_key = UserEmailSecondaryKey("test@example.com".to_string());

    // Test From<UserEmailSecondaryKey> for Vec<u8>
    let key_bytes: Vec<u8> = secondary_key.clone().into();
    assert!(!key_bytes.is_empty());

    // Test TryFrom<Vec<u8>> for UserEmailSecondaryKey
    let key_recovered: UserEmailSecondaryKey = key_bytes.try_into().unwrap();
    assert_eq!(secondary_key, key_recovered);
}

#[test]
fn test_secondary_keys_enum_vec_u8_conversions() {
    let secondary_keys = UserSecondaryKeys::Email(UserEmailSecondaryKey("test@example.com".to_string()));

    // Test From<UserSecondaryKeys> for Vec<u8>
    let keys_bytes: Vec<u8> = secondary_keys.clone().into();
    assert!(!keys_bytes.is_empty());

    // Test TryFrom<Vec<u8>> for UserSecondaryKeys
    let keys_recovered: UserSecondaryKeys = keys_bytes.try_into().unwrap();
    assert_eq!(secondary_keys, keys_recovered);
}

#[test]
fn test_keys_enum_vec_u8_conversions() {
    // Test primary key variant
    let primary_key = UserKey::Primary(UserPrimaryKey(123));

    let key_bytes: Vec<u8> = primary_key.clone().into();
    assert!(!key_bytes.is_empty());

    let key_recovered: UserKey = key_bytes.try_into().unwrap();
    assert_eq!(primary_key, key_recovered);

    // Test secondary key variant
    let secondary_key = UserKey::Secondary(UserSecondaryKeys::Email(
        UserEmailSecondaryKey("test@example.com".to_string())
    ));

    let key_bytes: Vec<u8> = secondary_key.clone().into();
    assert!(!key_bytes.is_empty());

    let key_recovered: UserKey = key_bytes.try_into().unwrap();
    assert_eq!(secondary_key, key_recovered);
}

#[test]
fn test_invalid_vec_u8_conversion() {
    // Test error handling for invalid Vec<u8>
    let invalid_bytes = vec![1, 2, 3]; // Random bytes that won't decode to a User

    let result: Result<User, _> = invalid_bytes.try_into();
    assert!(result.is_err());
}

#[test]
fn test_round_trip_conversion() {
    // Test that multiple conversions don't lose data
    let original_user = User {
        id: 999,
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
    };

    // Convert multiple times
    let bytes1: Vec<u8> = original_user.clone().into();
    let user1: User = bytes1.try_into().unwrap();
    let bytes2: Vec<u8> = user1.clone().into();
    let user2: User = bytes2.try_into().unwrap();

    assert_eq!(original_user, user1);
    assert_eq!(user1, user2);
    assert_eq!(original_user, user2);
}
