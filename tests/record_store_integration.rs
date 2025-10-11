//! Integration tests for RecordStore implementation with SledStore
//!
//! This test suite validates:
//! 1. The blanket implementations of KademliaRecord and KademliaRecordKey traits
//! 2. The RecordStore implementation for SledStore
//! 3. Provider record management
//! 4. Round-trip serialization and deserialization
//! 5. Integration between NetabaseStore and libp2p Records

use tempfile::TempDir;

#[cfg(feature = "libp2p")]
use libp2p::{
    Multiaddr, PeerId,
    kad::{ProviderRecord, Record, RecordKey, store::RecordStore},
};

use netabase_store::{
    databases::sled_store::SledStore,
    traits::{
        definition::{NetabaseDefinition, NetabaseDefinitionDiscriminants, NetabaseDefinitionKeys},
        dht::{KademliaRecord, KademliaRecordKey},
        model::{NetabaseModel, NetabaseModelKey},
        store::Store,
    },
};

use bincode::{Decode, Encode};
use strum::EnumIter;

// Test data structures
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct TestUser {
    pub id: u64,
    pub username: String,
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct TestPost {
    pub id: u64,
    pub title: String,
    pub content: String,
    pub author_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct TestUserKey {
    pub id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct TestPostKey {
    pub id: u64,
}

// Test NetabaseDefinition implementation
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, strum::EnumDiscriminants)]
#[strum_discriminants(derive(EnumIter, Encode, Decode, Hash))]
#[strum_discriminants(name(TestDiscriminants))]
pub enum TestDefinition {
    User(TestUser),
    Post(TestPost),
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub enum TestDefinitionKeys {
    User(TestUserKey),
    Post(TestPostKey),
}

// Trait implementations
impl NetabaseDefinitionDiscriminants for TestDiscriminants {}
impl NetabaseDefinitionKeys for TestDefinitionKeys {
    type Discriminants = TestDiscriminants;

    fn discriminant(&self) -> Self::Discriminants {
        match self {
            TestDefinitionKeys::User(_) => TestDiscriminants::User,
            TestDefinitionKeys::Post(_) => TestDiscriminants::Post,
        }
    }
}

impl NetabaseDefinition for TestDefinition {
    type Keys = TestDefinitionKeys;
    type Discriminants = TestDiscriminants;

    fn keys(&self) -> Self::Keys {
        match self {
            TestDefinition::User(user) => TestDefinitionKeys::User(TestUserKey { id: user.id }),
            TestDefinition::Post(post) => TestDefinitionKeys::Post(TestPostKey { id: post.id }),
        }
    }
}

impl NetabaseModelKey for TestUserKey {
    type Model = TestUser;
}

impl NetabaseModelKey for TestPostKey {
    type Model = TestPost;
}

impl NetabaseModel for TestUser {
    type Key = TestUserKey;
    type Defined = TestDefinition;
    const DISCRIMINANT: TestDiscriminants = TestDiscriminants::User;

    fn key(&self) -> Self::Key {
        TestUserKey { id: self.id }
    }
}

impl NetabaseModel for TestPost {
    type Key = TestPostKey;
    type Defined = TestDefinition;
    const DISCRIMINANT: TestDiscriminants = TestDiscriminants::Post;

    fn key(&self) -> Self::Key {
        TestPostKey { id: self.id }
    }
}

impl From<TestUser> for TestDefinition {
    fn from(user: TestUser) -> Self {
        TestDefinition::User(user)
    }
}

impl From<TestPost> for TestDefinition {
    fn from(post: TestPost) -> Self {
        TestDefinition::Post(post)
    }
}

#[cfg(feature = "libp2p")]
mod libp2p_tests {
    use super::*;

    #[test]
    fn test_blanket_implementations_automatically_available() {
        // Test that types implementing the required traits automatically get DHT traits

        let user = TestUser {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        };

        let definition = TestDefinition::User(user);

        // These methods are available thanks to blanket implementations
        let _keys = definition.record_keys();
        let _serialized = definition.try_to_vec().unwrap();
        let _record = definition.try_to_record().unwrap();

        // Test that the keys also have the trait
        let keys = definition.keys();
        let _key_serialized = keys.try_to_vec().unwrap();
        let _record_key = keys.try_to_record_key().unwrap();

        // Test compile-time trait availability
        fn requires_kademlia_record<T: KademliaRecord>(_: &T) {}
        fn requires_kademlia_record_key<T: KademliaRecordKey>(_: &T) {}

        requires_kademlia_record(&definition);
        requires_kademlia_record_key(&keys);
    }

    #[test]
    fn test_record_conversion_round_trip() {
        let user = TestUser {
            id: 42,
            username: "test_user".to_string(),
            email: "test@example.com".to_string(),
        };

        let original_definition = TestDefinition::User(user);

        // Convert to Record
        let record = original_definition.try_to_record().unwrap();

        // Verify record structure
        assert!(!record.value.is_empty());
        assert!(record.publisher.is_none());
        assert!(record.expires.is_none());

        // Convert back to definition
        let recovered_definition = TestDefinition::try_from_record(record).unwrap();

        // Verify round-trip
        assert_eq!(original_definition, recovered_definition);
    }

    #[test]
    fn test_record_key_conversion() {
        let keys = TestDefinitionKeys::User(TestUserKey { id: 123 });

        // Convert to RecordKey
        let record_key = keys.try_to_record_key().unwrap();

        // Convert back
        let recovered_keys = TestDefinitionKeys::try_from_record_key(&record_key).unwrap();

        // Verify round-trip
        assert_eq!(keys, recovered_keys);
    }

    #[test]
    fn test_sled_store_record_store_implementation() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = SledStore::<TestDefinition>::new(temp_dir.path()).unwrap();

        // Create test data
        let user = TestUser {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        };

        let post = TestPost {
            id: 1,
            title: "Test Post".to_string(),
            content: "This is a test post".to_string(),
            author_id: 1,
        };

        let user_definition = TestDefinition::User(user.clone());
        let post_definition = TestDefinition::Post(post.clone());

        // Convert to records
        let user_record = user_definition.try_to_record().unwrap();
        let post_record = post_definition.try_to_record().unwrap();

        // Test putting records using RecordStore trait
        RecordStore::put(&mut store, user_record.clone()).unwrap();
        RecordStore::put(&mut store, post_record.clone()).unwrap();

        // Test getting records using RecordStore trait
        let retrieved_user = RecordStore::get(&store, &user_record.key);
        let retrieved_post = RecordStore::get(&store, &post_record.key);

        assert!(retrieved_user.is_some());
        assert!(retrieved_post.is_some());

        // Verify content
        let retrieved_user_record = retrieved_user.unwrap().into_owned();
        let retrieved_post_record = retrieved_post.unwrap().into_owned();

        assert_eq!(user_record.key, retrieved_user_record.key);
        assert_eq!(post_record.key, retrieved_post_record.key);

        // Test removal
        RecordStore::remove(&mut store, &user_record.key);
        let removed_user = RecordStore::get(&store, &user_record.key);
        assert!(removed_user.is_none());

        // Post should still be there
        let still_there_post = RecordStore::get(&store, &post_record.key);
        assert!(still_there_post.is_some());
    }

    #[test]
    fn test_provider_record_management() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = SledStore::<TestDefinition>::new(temp_dir.path()).unwrap();

        // Create test provider record
        let record_key = RecordKey::new(b"test_key");
        let peer_id = PeerId::random();
        let addresses = vec![
            "/ip4/127.0.0.1/tcp/8080".parse::<Multiaddr>().unwrap(),
            "/ip6/::1/tcp/8080".parse::<Multiaddr>().unwrap(),
        ];

        let provider_record = ProviderRecord {
            key: record_key.clone(),
            provider: peer_id,
            expires: None,
            addresses: addresses.clone(),
        };

        // Test adding provider
        store.add_provider(provider_record.clone()).unwrap();

        // Test getting providers
        let providers = store.providers(&record_key);
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].provider, peer_id);
        assert_eq!(providers[0].addresses.len(), addresses.len());

        // Test removing provider
        store.remove_provider(&record_key, &peer_id);
        let providers_after_removal = store.providers(&record_key);
        assert_eq!(providers_after_removal.len(), 0);
    }

    #[test]
    fn test_records_iterator() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = SledStore::<TestDefinition>::new(temp_dir.path()).unwrap();

        // Add multiple records
        let users = vec![
            TestUser {
                id: 1,
                username: "alice".to_string(),
                email: "alice@example.com".to_string(),
            },
            TestUser {
                id: 2,
                username: "bob".to_string(),
                email: "bob@example.com".to_string(),
            },
            TestUser {
                id: 3,
                username: "charlie".to_string(),
                email: "charlie@example.com".to_string(),
            },
        ];

        for user in &users {
            let definition = TestDefinition::User(user.clone());
            let record = definition.try_to_record().unwrap();
            RecordStore::put(&mut store, record).unwrap();
        }

        // Test iterator
        let records: Vec<_> = RecordStore::records(&store).collect();
        assert_eq!(records.len(), users.len());

        // Verify all records can be converted back
        for record_cow in records {
            let record = record_cow.into_owned();
            let recovered = TestDefinition::try_from_record(record).unwrap();

            match recovered {
                TestDefinition::User(user) => {
                    assert!(users.iter().any(|u| u == &user));
                }
                _ => panic!("Unexpected record type"),
            }
        }
    }

    #[test]
    fn test_provided_iterator() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = SledStore::<TestDefinition>::new(temp_dir.path()).unwrap();

        let peer_id = PeerId::random();
        let record_keys = vec![
            RecordKey::new(b"key1"),
            RecordKey::new(b"key2"),
            RecordKey::new(b"key3"),
        ];

        // Add provider records
        for key in &record_keys {
            let provider_record = ProviderRecord {
                key: key.clone(),
                provider: peer_id,
                expires: None,
                addresses: vec!["/ip4/127.0.0.1/tcp/8080".parse().unwrap()],
            };
            RecordStore::add_provider(&mut store, provider_record).unwrap();
        }

        // Test provided iterator
        let provided: Vec<_> = RecordStore::provided(&store).collect();
        assert_eq!(provided.len(), record_keys.len());

        // Verify all provided records have the correct peer ID
        for provider_cow in provided {
            let provider = provider_cow.into_owned();
            assert_eq!(provider.provider, peer_id);
        }
    }

    #[test]
    fn test_error_handling() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = SledStore::<TestDefinition>::new(temp_dir.path()).unwrap();

        // Test putting invalid record (this should still work as we accept any Record)
        let invalid_record = Record {
            key: RecordKey::new(b"invalid"),
            value: vec![0xFF; 10], // Invalid bincode data
            publisher: None,
            expires: None,
        };

        // Should not panic, but might not be retrievable as a valid TestDefinition
        let _ = RecordStore::put(&mut store, invalid_record.clone());

        // Getting it back should return None or the raw record
        let retrieved = RecordStore::get(&store, &invalid_record.key);
        // Note: The implementation might return the raw record even if it can't be deserialized
        // This is implementation-dependent behavior
    }

    #[test]
    fn test_multiple_types_in_store() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = SledStore::<TestDefinition>::new(temp_dir.path()).unwrap();

        // Add different types of records
        let user = TestUser {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        };

        let post = TestPost {
            id: 1,
            title: "Alice's Post".to_string(),
            content: "Hello world!".to_string(),
            author_id: 1,
        };

        let user_def = TestDefinition::User(user);
        let post_def = TestDefinition::Post(post);

        let user_record = user_def.try_to_record().unwrap();
        let post_record = post_def.try_to_record().unwrap();

        RecordStore::put(&mut store, user_record.clone()).unwrap();
        RecordStore::put(&mut store, post_record.clone()).unwrap();

        // Verify both can be retrieved
        let retrieved_user = RecordStore::get(&store, &user_record.key);
        let retrieved_post = RecordStore::get(&store, &post_record.key);

        assert!(retrieved_user.is_some());
        assert!(retrieved_post.is_some());

        // Verify iterator returns both
        let all_records: Vec<_> = RecordStore::records(&store).collect();
        assert_eq!(all_records.len(), 2);
    }
}

#[cfg(not(feature = "libp2p"))]
mod non_libp2p_tests {
    use super::*;

    #[test]
    fn test_basic_store_functionality() {
        // Test that the basic store functionality works without libp2p
        let temp_dir = TempDir::new().unwrap();
        let store = SledStore::<TestDefinition>::new(temp_dir.path()).unwrap();

        let user = TestUser {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        };

        // Test basic store operations
        let result = store.put(user.clone());
        assert!(result.is_ok());

        let retrieved = store.get(user.key());
        assert!(retrieved.is_ok());
        assert!(retrieved.unwrap().is_some());
    }
}
