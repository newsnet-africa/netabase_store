//! libp2p Kademlia RecordStore implementation support
//!
//! This module provides the necessary types, configuration, and implementations
//! for using Netabase stores as libp2p Kademlia record stores.
//!
//! # Storage Guarantee
//!
//! **All records stored are guaranteed to be `NetabaseModelTrait` types, never wrapped in `NetabaseDefinitionTrait`.**
//!
//! - Keys use ModelRecordKey format: `<discriminant>:<key_bytes>`
//! - Values are serialized models directly (not Definition wrappers)
//! - No backward compatibility with old formats
//!
//! # Recommended Usage
//!
#![allow(dead_code)] // Items used only in specific feature configurations
//! Use the `ModelRecordStore` trait for type-safe operations:
//!
//! ```ignore
//! use netabase_store::databases::record_store::model_store::ModelRecordStore;
//!
//! // Store a model
//! store.put_model::<MyDefinition, _>(&user)?;
//!
//! // Retrieve by key
//! let user: User = store.get_model::<MyDefinition, User, _>(&user_id)?;
//! ```
//!
//! # Module Structure
//!
//! - `model_store`: Model-aware RecordStore extension (recommended API)
//! - `sled_impl`: SledStore RecordStore implementation
//! - `redb_impl`: RedbStore RecordStore implementation

#[cfg(feature = "libp2p")]
use libp2p::kad::store::{Error, Result};
#[cfg(feature = "libp2p")]
use libp2p::kad::{ProviderRecord, Record, RecordKey as Key};
#[cfg(feature = "libp2p")]
use libp2p::PeerId;

// Model-aware extension
#[cfg(feature = "libp2p")]
pub mod model_store;

// Re-export backend implementations
#[cfg(all(feature = "libp2p", feature = "sled"))]
pub mod sled_impl;

#[cfg(all(feature = "libp2p", feature = "redb"))]
pub mod redb_impl;

// Generic RecordStore implementations (use macro-generated helper methods)
#[cfg(feature = "libp2p")]
pub mod generic_impl;

/// Serializable version of libp2p::kad::Record
/// Note: expires is always None since Instant is not serializable
#[cfg(feature = "libp2p")]
#[derive(Debug, Clone, bincode::Encode, bincode::Decode)]
pub struct SerializableRecord {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub publisher: Option<Vec<u8>>,
    // expires is always None - we don't persist expiration times
}

/// Serializable version of libp2p::kad::ProviderRecord
/// Note: expires is always None since Instant is not serializable
#[cfg(feature = "libp2p")]
#[derive(Debug, Clone, bincode::Encode, bincode::Decode)]
pub struct SerializableProviderRecord {
    pub key: Vec<u8>,
    pub provider: Vec<u8>,
    // expires is always None - we don't persist expiration times
    pub addresses: Vec<Vec<u8>>,
}

// Constants for RecordStore implementation
#[cfg(feature = "libp2p")]
pub const PROVIDER_TREE_NAME: &str = "__libp2p_providers";
#[cfg(feature = "libp2p")]
pub const PROVIDED_TREE_NAME: &str = "__libp2p_provided";
#[cfg(feature = "libp2p")]
const MAX_RECORDS: usize = 1024;
#[cfg(feature = "libp2p")]
const MAX_VALUE_BYTES: usize = 65 * 1024; // 65 KB
#[cfg(feature = "libp2p")]
const MAX_PROVIDERS_PER_KEY: usize = 20;
#[cfg(feature = "libp2p")]
const MAX_PROVIDED_KEYS: usize = 1024;

/// Configuration for the RecordStore implementation
#[cfg(feature = "libp2p")]
pub struct RecordStoreConfig {
    pub max_records: usize,
    pub max_value_bytes: usize,
    pub max_providers_per_key: usize,
    pub max_provided_keys: usize,
}

#[cfg(feature = "libp2p")]
impl Default for RecordStoreConfig {
    fn default() -> Self {
        Self {
            max_records: MAX_RECORDS,
            max_value_bytes: MAX_VALUE_BYTES,
            max_providers_per_key: MAX_PROVIDERS_PER_KEY,
            max_provided_keys: MAX_PROVIDED_KEYS,
        }
    }
}

/// Utility functions for encoding/decoding libp2p types
#[cfg(feature = "libp2p")]
pub mod utils {
    use super::*;

    /// Encode a Key to bytes
    pub fn encode_key(key: &Key) -> Vec<u8> {
        key.to_vec()
    }

    /// Encode a Record to bytes using SerializableRecord
    pub fn encode_record(record: &Record) -> Result<Vec<u8>> {
        let serializable = SerializableRecord {
            key: record.key.to_vec(),
            value: record.value.clone(),
            publisher: record.publisher.as_ref().map(|p| p.to_bytes()),
        };
        bincode::encode_to_vec(&serializable, bincode::config::standard())
            .map_err(|_| Error::ValueTooLarge)
    }

    /// Decode a Record from bytes using SerializableRecord
    pub fn decode_record(bytes: &[u8]) -> Result<Record> {
        let (serializable, _): (SerializableRecord, _) =
            bincode::decode_from_slice(bytes, bincode::config::standard())
                .map_err(|_| Error::MaxRecords)?;

        let publisher = match serializable.publisher {
            Some(bytes) => Some(PeerId::from_bytes(&bytes).map_err(|_| Error::MaxRecords)?),
            None => None,
        };

        Ok(Record {
            key: Key::from(serializable.key),
            value: serializable.value,
            publisher,
            expires: None, // Always None - we don't persist expiration times
        })
    }

    /// Encode a ProviderRecord to bytes using SerializableProviderRecord
    pub fn encode_provider(provider: &ProviderRecord) -> Result<Vec<u8>> {
        let serializable = SerializableProviderRecord {
            key: provider.key.to_vec(),
            provider: provider.provider.to_bytes(),
            addresses: provider.addresses.iter().map(|a| a.to_vec()).collect(),
        };
        bincode::encode_to_vec(&serializable, bincode::config::standard())
            .map_err(|_| Error::ValueTooLarge)
    }

    /// Decode a ProviderRecord from bytes using SerializableProviderRecord
    pub fn decode_provider(bytes: &[u8]) -> Result<ProviderRecord> {
        let (serializable, _): (SerializableProviderRecord, _) =
            bincode::decode_from_slice(bytes, bincode::config::standard())
                .map_err(|_| Error::MaxRecords)?;

        let provider = PeerId::from_bytes(&serializable.provider).map_err(|_| Error::MaxRecords)?;

        let addresses = serializable
            .addresses
            .iter()
            .filter_map(|bytes| libp2p::Multiaddr::try_from(bytes.clone()).ok())
            .collect();

        Ok(ProviderRecord {
            key: Key::from(serializable.key),
            provider,
            expires: None, // Always None - we don't persist expiration times
            addresses,
        })
    }
}
