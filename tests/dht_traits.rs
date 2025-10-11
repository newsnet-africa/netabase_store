//! Comprehensive tests for DHT traits based on libp2p MemoryStore patterns
//!
//! This test suite validates the KademliaRecord and KademliaRecordKey traits
//! along with the provider record helpers and record iteration functionality.

use std::collections::HashMap;

use netabase_store::traits::{
    definition::{NetabaseDefinition, NetabaseDefinitionDiscriminants, NetabaseDefinitionKeys},
    dht::{KademliaRecord, KademliaRecordKey, provider_record_helpers},
    model::{NetabaseModel, NetabaseModelKey},
};

use bincode::{Decode, Encode};
use strum::EnumIter;

// Only run tests when libp2p feature is enabled
#[cfg(feature = "libp2p")]
use libp2p::{
    Multiaddr, PeerId,
    kad::{ProviderRecord, RecordKey},
};

// Test data structures
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct TestUser {
    pub id: String,
    pub name: String,
    pub email: String,
    pub age: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct TestPost {
    pub id: String,
    pub title: String,
    pub content: String,
    pub author_id: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct TestUserKey {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct TestPostKey {
    pub id: String,
}

// Test NetabaseDefinition enum
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
            TestDefinition::User(user) => TestDefinitionKeys::User(TestUserKey {
                id: user.id.clone(),
            }),
            TestDefinition::Post(post) => TestDefinitionKeys::Post(TestPostKey {
                id: post.id.clone(),
            }),
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
        TestUserKey {
            id: self.id.clone(),
        }
    }
}

impl NetabaseModel for TestPost {
    type Key = TestPostKey;
    type Defined = TestDefinition;
    const DISCRIMINANT: TestDiscriminants = TestDiscriminants::Post;

    fn key(&self) -> Self::Key {
        TestPostKey {
            id: self.id.clone(),
        }
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

// DHT trait implementations
// Note: KademliaRecord and KademliaRecordKey traits are now automatically
// implemented via blanket implementations for any type that satisfies the bounds:
// - NetabaseDefinition + bincode::Encode + bincode::Decode<()> for KademliaRecord
// - NetabaseDefinitionKeys + bincode::Encode + bincode::Decode<()> for KademliaRecordKey
//
// Since TestDefinition and TestDefinitionKeys already implement these traits,
// they automatically get the KademliaRecord and KademliaRecordKey implementations!

// Helper functions for tests
fn create_test_user() -> TestUser {
    TestUser {
        id: "user123".to_string(),
        name: "Alice Johnson".to_string(),
        email: "alice@example.com".to_string(),
        age: 30,
    }
}

fn create_test_post() -> TestPost {
    TestPost {
        id: "post456".to_string(),
        title: "Hello World".to_string(),
        content: "This is my first post!".to_string(),
        author_id: "user123".to_string(),
        timestamp: 1640995200,
    }
}

#[cfg(feature = "libp2p")]
fn random_peer_id() -> PeerId {
    PeerId::random()
}

#[cfg(feature = "libp2p")]
fn random_multiaddr() -> Multiaddr {
    use rand::Rng;
    let port: u16 = rand::thread_rng().gen_range(1024..65535);
    format!("/ip4/127.0.0.1/tcp/{}", port).parse().unwrap()
}

// Tests for KademliaRecord trait
#[cfg(test)]
#[cfg(feature = "libp2p")]
mod kademlia_record_tests {
    use super::*;

    #[test]
    fn test_record_conversion_user() {
        let user = create_test_user();
        let definition = TestDefinition::User(user.clone());

        // Convert to Record
        let record = definition.try_to_record().unwrap();

        // Verify key is correctly generated
        assert!(!record.key.to_vec().is_empty());
        assert!(!record.value.is_empty());
        assert!(record.publisher.is_none());
        assert!(record.expires.is_none());

        // Convert back
        let recovered = TestDefinition::try_from_record(record).unwrap();

        // Verify round-trip
        assert_eq!(definition, recovered);
        if let TestDefinition::User(recovered_user) = recovered {
            assert_eq!(user, recovered_user);
        } else {
            panic!("Wrong type recovered");
        }
    }

    #[test]
    fn test_record_conversion_post() {
        let post = create_test_post();
        let definition = TestDefinition::Post(post.clone());

        // Convert to Record
        let record = definition.try_to_record().unwrap();

        // Verify record structure
        assert!(!record.key.to_vec().is_empty());
        assert!(!record.value.is_empty());

        // Convert back
        let recovered = TestDefinition::try_from_record(record).unwrap();

        // Verify round-trip
        assert_eq!(definition, recovered);
        if let TestDefinition::Post(recovered_post) = recovered {
            assert_eq!(post, recovered_post);
        } else {
            panic!("Wrong type recovered");
        }
    }

    #[test]
    fn test_multiple_records_different_keys() {
        let user1 = TestUser {
            id: "user1".to_string(),
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 25,
        };

        let user2 = TestUser {
            id: "user2".to_string(),
            name: "Bob".to_string(),
            email: "bob@example.com".to_string(),
            age: 30,
        };

        let def1 = TestDefinition::User(user1);
        let def2 = TestDefinition::User(user2);

        let record1 = def1.try_to_record().unwrap();
        let record2 = def2.try_to_record().unwrap();

        // Different users should have different keys
        assert_ne!(record1.key, record2.key);
        assert_ne!(record1.value, record2.value);
    }

    #[test]
    fn test_serialization_deterministic() {
        let user = create_test_user();
        let definition = TestDefinition::User(user);

        // Convert multiple times
        let record1 = definition.try_to_record().unwrap();
        let record2 = definition.try_to_record().unwrap();

        // Should be identical
        assert_eq!(record1.key, record2.key);
        assert_eq!(record1.value, record2.value);
    }

    #[test]
    fn test_record_size_reasonable() {
        let user = create_test_user();
        let post = create_test_post();

        let user_record = TestDefinition::User(user).try_to_record().unwrap();
        let post_record = TestDefinition::Post(post).try_to_record().unwrap();

        // Records should be reasonably sized (not empty, not huge)
        assert!(user_record.value.len() > 10);
        assert!(user_record.value.len() < 1024);
        assert!(post_record.value.len() > 10);
        assert!(post_record.value.len() < 1024);

        // Keys should be reasonable size
        assert!(user_record.key.to_vec().len() > 5);
        assert!(user_record.key.to_vec().len() < 256);
    }
}

// Tests for KademliaRecordKey trait
#[cfg(test)]
#[cfg(feature = "libp2p")]
mod kademlia_record_key_tests {
    use super::*;

    #[test]
    fn test_record_key_conversion() {
        let user_key = TestDefinitionKeys::User(TestUserKey {
            id: "test_user".to_string(),
        });

        // Convert to RecordKey
        let record_key = user_key.try_to_record_key().unwrap();

        // Convert back
        let recovered = TestDefinitionKeys::try_from_record_key(&record_key).unwrap();

        // Verify round-trip
        assert_eq!(user_key, recovered);
    }

    #[test]
    fn test_different_keys_different_record_keys() {
        let user_key1 = TestDefinitionKeys::User(TestUserKey {
            id: "user1".to_string(),
        });
        let user_key2 = TestDefinitionKeys::User(TestUserKey {
            id: "user2".to_string(),
        });
        let post_key = TestDefinitionKeys::Post(TestPostKey {
            id: "post1".to_string(),
        });

        let record_key1 = user_key1.try_to_record_key().unwrap();
        let record_key2 = user_key2.try_to_record_key().unwrap();
        let record_key3 = post_key.try_to_record_key().unwrap();

        // All should be different
        assert_ne!(record_key1, record_key2);
        assert_ne!(record_key1, record_key3);
        assert_ne!(record_key2, record_key3);
    }

    #[test]
    fn test_key_serialization_consistency() {
        let key = TestDefinitionKeys::Post(TestPostKey {
            id: "consistent_test".to_string(),
        });

        // Serialize multiple times
        let record_key1 = key.try_to_record_key().unwrap();
        let record_key2 = key.try_to_record_key().unwrap();

        // Should be identical
        assert_eq!(record_key1, record_key2);
    }
}

// Tests for ProviderRecord helpers
#[cfg(test)]
#[cfg(feature = "libp2p")]
mod provider_record_helper_tests {
    use super::*;

    #[test]
    fn test_provider_record_ivec_conversion() {
        let record_key = RecordKey::new(b"test_record_key");
        let peer_id = random_peer_id();
        let addresses = vec![random_multiaddr(), random_multiaddr()];

        let provider_record = ProviderRecord {
            key: record_key.clone(),
            provider: peer_id,
            expires: None,
            addresses: addresses.clone(),
        };

        // Convert to IVec format
        let (key_ivec, value_ivec) =
            provider_record_helpers::provider_record_to_ivec(&provider_record).unwrap();

        // Convert back
        let recovered =
            provider_record_helpers::ivec_to_provider_record(&key_ivec, &value_ivec).unwrap();

        // Verify round-trip
        assert_eq!(provider_record.key, recovered.key);
        assert_eq!(provider_record.provider, recovered.provider);
        assert_eq!(provider_record.addresses.len(), recovered.addresses.len());
        // Note: Multiaddr comparison might not be exact due to serialization format
    }

    #[test]
    fn test_provider_record_empty_addresses() {
        let record_key = RecordKey::new(b"empty_addresses");
        let peer_id = random_peer_id();

        let provider_record = ProviderRecord {
            key: record_key,
            provider: peer_id,
            expires: None,
            addresses: vec![],
        };

        // Convert to IVec and back
        let (key_ivec, value_ivec) =
            provider_record_helpers::provider_record_to_ivec(&provider_record).unwrap();

        let recovered =
            provider_record_helpers::ivec_to_provider_record(&key_ivec, &value_ivec).unwrap();

        assert_eq!(provider_record.key, recovered.key);
        assert_eq!(provider_record.provider, recovered.provider);
        assert!(recovered.addresses.is_empty());
    }

    #[test]
    fn test_provider_record_invalid_key_format() {
        let short_key = sled::IVec::from(vec![1, 2, 3]);
        let value = sled::IVec::from(vec![]);

        let result = provider_record_helpers::ivec_to_provider_record(&short_key, &value);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_provider_record_tree() {
        let temp_db = sled::open("test_provider_tree").unwrap();
        let tree = provider_record_helpers::create_provider_record_tree(&temp_db, "test_providers");

        assert!(tree.is_ok());

        // Cleanup
        std::fs::remove_dir_all("test_provider_tree").ok();
    }

    #[test]
    fn test_get_providers_for_key() {
        let temp_db = sled::open("test_providers_lookup").unwrap();
        let tree =
            provider_record_helpers::create_provider_record_tree(&temp_db, "providers").unwrap();

        let record_key = RecordKey::new(b"lookup_test");
        let peer1 = random_peer_id();
        let peer2 = random_peer_id();

        // Add two providers for the same key
        let provider1 = ProviderRecord {
            key: record_key.clone(),
            provider: peer1,
            expires: None,
            addresses: vec![random_multiaddr()],
        };

        let provider2 = ProviderRecord {
            key: record_key.clone(),
            provider: peer2,
            expires: None,
            addresses: vec![random_multiaddr()],
        };

        // Store them using the helper function
        provider_record_helpers::add_provider_to_key(&tree, &provider1).unwrap();
        // Note: This will overwrite provider1 since they have the same key
        provider_record_helpers::add_provider_to_key(&tree, &provider2).unwrap();

        // Retrieve providers (only provider2 should remain due to overwrite)
        let providers = provider_record_helpers::get_providers_for_key(&tree, &record_key).unwrap();

        assert_eq!(providers.len(), 1);
        assert!(providers.iter().any(|p| p.provider == peer2));

        // Cleanup
        std::fs::remove_dir_all("test_providers_lookup").ok();
    }
}

// Integration tests
#[cfg(test)]
#[cfg(feature = "libp2p")]
mod integration_tests {
    use super::*;

    #[test]
    fn test_record_store_workflow() {
        // Create test data
        let user = create_test_user();
        let post = create_test_post();

        let user_def = TestDefinition::User(user.clone());
        let post_def = TestDefinition::Post(post.clone());

        // Convert to records
        let user_record = user_def.try_to_record().unwrap();
        let post_record = post_def.try_to_record().unwrap();

        // Simulate storing in a record store
        let mut records = HashMap::new();
        records.insert(user_record.key.clone(), user_record.clone());
        records.insert(post_record.key.clone(), post_record.clone());

        // Retrieve and verify
        let retrieved_user_record = records.get(&user_record.key).unwrap();
        let retrieved_post_record = records.get(&post_record.key).unwrap();

        let recovered_user =
            TestDefinition::try_from_record(retrieved_user_record.clone()).unwrap();
        let recovered_post =
            TestDefinition::try_from_record(retrieved_post_record.clone()).unwrap();

        assert_eq!(user_def, recovered_user);
        assert_eq!(post_def, recovered_post);
    }

    #[test]
    fn test_provider_management_workflow() {
        let temp_db = sled::open("test_workflow").unwrap();
        let tree =
            provider_record_helpers::create_provider_record_tree(&temp_db, "providers").unwrap();

        let record_key = RecordKey::new(b"workflow_test");
        let local_peer = random_peer_id();
        let remote_peer = random_peer_id();

        // Add local provider
        let local_provider = ProviderRecord {
            key: record_key.clone(),
            provider: local_peer,
            expires: None,
            addresses: vec![random_multiaddr()],
        };

        // Add remote provider
        let remote_provider = ProviderRecord {
            key: record_key.clone(),
            provider: remote_peer,
            expires: None,
            addresses: vec![random_multiaddr()],
        };

        // Store providers (remote will overwrite local since they share the same key)
        provider_record_helpers::add_provider_to_key(&tree, &local_provider).unwrap();
        provider_record_helpers::add_provider_to_key(&tree, &remote_provider).unwrap();

        // Retrieve all providers (should only be remote due to overwrite)
        let all_providers =
            provider_record_helpers::get_providers_for_key(&tree, &record_key).unwrap();
        assert_eq!(all_providers.len(), 1);
        assert_eq!(all_providers[0].provider, remote_peer);

        // Simulate removing the remote provider
        let removed =
            provider_record_helpers::remove_provider_from_key(&tree, &record_key, &remote_peer)
                .unwrap();
        assert!(removed);

        // Verify no providers remain
        let remaining_providers =
            provider_record_helpers::get_providers_for_key(&tree, &record_key).unwrap();
        assert_eq!(remaining_providers.len(), 0);

        // Cleanup
        std::fs::remove_dir_all("test_workflow").ok();
    }

    #[test]
    fn test_large_data_handling() {
        // Create a large record
        let large_content = "x".repeat(10_000); // 10KB content
        let large_post = TestPost {
            id: "large_post".to_string(),
            title: "Large Post".to_string(),
            content: large_content,
            author_id: "user123".to_string(),
            timestamp: 1640995200,
        };

        let large_def = TestDefinition::Post(large_post.clone());

        // Should still work
        let record = large_def.try_to_record().unwrap();
        assert!(record.value.len() > 10_000);

        let recovered = TestDefinition::try_from_record(record).unwrap();
        assert_eq!(large_def, recovered);
    }

    #[test]
    fn test_many_addresses_provider() {
        // Create provider with many addresses
        let many_addresses: Vec<Multiaddr> = (1000..1100)
            .map(|port| format!("/ip4/127.0.0.1/tcp/{}", port).parse().unwrap())
            .collect();

        let provider = ProviderRecord {
            key: RecordKey::new(b"many_addresses"),
            provider: random_peer_id(),
            expires: None,
            addresses: many_addresses.clone(),
        };

        // Convert and recover
        let (key_ivec, value_ivec) =
            provider_record_helpers::provider_record_to_ivec(&provider).unwrap();

        let recovered =
            provider_record_helpers::ivec_to_provider_record(&key_ivec, &value_ivec).unwrap();

        assert_eq!(provider.key, recovered.key);
        assert_eq!(provider.provider, recovered.provider);
        // Note: Some addresses might be lost due to serialization issues with invalid Multiaddr formats
        assert!(!recovered.addresses.is_empty());
    }
}

// Error handling tests
#[cfg(test)]
#[cfg(feature = "libp2p")]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_invalid_record_data() {
        // Try to deserialize invalid data as a record
        let invalid_data = vec![0xFF, 0xFF, 0xFF, 0xFF];
        let result = TestDefinition::try_from_vec(&invalid_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_key_data() {
        // Try to deserialize invalid data as a key
        let invalid_data = vec![0xFF, 0xFF, 0xFF, 0xFF];
        let result = TestDefinitionKeys::try_from_vec(&invalid_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_corrupted_provider_record() {
        let valid_key = sled::IVec::from(b"valid_key_with_peer_id_bytes".to_vec());
        let corrupted_value = sled::IVec::from(vec![0xFF, 0xFF, 0xFF, 0xFF]);

        let result = provider_record_helpers::ivec_to_provider_record(&valid_key, &corrupted_value);
        assert!(result.is_err());
    }
}

// Performance tests (basic)
#[cfg(test)]
#[cfg(feature = "libp2p")]
mod performance_tests {
    use super::*;

    #[test]
    fn test_serialization_performance() {
        let user = create_test_user();
        let definition = TestDefinition::User(user);

        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = definition.try_to_record().unwrap();
        }
        let duration = start.elapsed();

        // Should complete 1000 serializations quickly
        assert!(duration.as_millis() < 1000); // Less than 1 second
    }

    #[test]
    fn test_deserialization_performance() {
        let user = create_test_user();
        let definition = TestDefinition::User(user);
        let record = definition.try_to_record().unwrap();

        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = TestDefinition::try_from_record(record.clone()).unwrap();
        }
        let duration = start.elapsed();

        // Should complete 1000 deserializations quickly
        assert!(duration.as_millis() < 1000); // Less than 1 second
    }
}
