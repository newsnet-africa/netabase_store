//! Model-aware RecordStore extension (Primary API)
//!
//! This module provides the **recommended API** for working with Netabase RecordStore.
//! It ensures type safety and guarantees that stored data is always a NetabaseModelTrait type.
//!
//! # Storage Guarantee
//!
//! When using this API:
//! - **Values are always NetabaseModelTrait types** (never Definition wrappers)
//! - **Keys use ModelRecordKey format** (`<discriminant>:<key_bytes>`)
//! - **Type safety is enforced** at compile time
//! - **Zero overhead** - no extra encoding/decoding layers
//!
//! # Why This Module Exists
//!
//! The libp2p `RecordStore` trait works with opaque byte vectors, but Netabase stores
//! are designed to work with typed models. This module bridges the gap by:
//!
//! 1. **Embedding model type info in keys** - enables efficient routing without value decoding
//! 2. **Providing typed operations** - work with models directly, not byte vectors
//! 3. **Enforcing storage guarantees** - ensures models (not Definitions) are stored
//!
//! # Usage
//!
//! Requires `libp2p` feature:
//! ```text
//! use netabase_store::databases::record_store::model_store::ModelRecordStore;
//!
//! // Store a model (guaranteed to store User type, not Definition wrapper)
//! let user = User { id: 1, name: "Alice".to_string() };
//! store.put_model::<BlogSchema, _>(&user)?;
//!
//! // Retrieve by primary key (type-safe)
//! let user: User = store.get_model::<BlogSchema, User, _>(&1)?;
//! ```

use crate::traits::definition::NetabaseDefinitionTrait;
use crate::traits::model::NetabaseModelTrait;
use std::str::FromStr;
use strum::IntoDiscriminant;

#[cfg(feature = "libp2p")]
use libp2p::kad::store::{Error, Result};
#[cfg(feature = "libp2p")]
use libp2p::kad::{Record, RecordKey as Key};

/// A record key that includes the model discriminant
///
/// This allows us to route records to the correct tree without decoding the value.
/// Format: `<discriminant_name>:<key_bytes>`
#[cfg(feature = "libp2p")]
#[derive(Debug, Clone)]
pub struct ModelRecordKey {
    /// The model discriminant as a string
    pub discriminant: String,
    /// The actual key bytes
    pub key_bytes: Vec<u8>,
}

#[cfg(feature = "libp2p")]
impl ModelRecordKey {
    /// Create a new ModelRecordKey from a discriminant and key bytes
    pub fn new(discriminant: String, key_bytes: Vec<u8>) -> Self {
        Self {
            discriminant,
            key_bytes,
        }
    }

    /// Create from a model instance
    pub fn from_model<D, M>(model: &M) -> Self
    where
        D: NetabaseDefinitionTrait,
        M: NetabaseModelTrait<D>,
        <D as IntoDiscriminant>::Discriminant: AsRef<str>
            + Clone
            + Copy
            + std::fmt::Debug
            + std::fmt::Display
            + PartialEq
            + Eq
            + std::hash::Hash
            + strum::IntoEnumIterator
            + Send
            + Sync
            + 'static
            + FromStr,
    {
        let discriminant = M::discriminant_name().to_string();
        let primary_key = model.primary_key();
        let key_bytes = bincode::encode_to_vec(&primary_key, bincode::config::standard())
            .expect("Key encoding should not fail");

        Self::new(discriminant, key_bytes)
    }

    /// Encode to a libp2p RecordKey
    pub fn to_record_key(&self) -> Key {
        let mut encoded = self.discriminant.as_bytes().to_vec();
        encoded.push(b':');
        encoded.extend_from_slice(&self.key_bytes);
        Key::from(encoded)
    }

    /// Decode from a libp2p RecordKey
    pub fn from_record_key(key: &Key) -> Result<Self> {
        let bytes = key.to_vec();
        let separator_pos = bytes
            .iter()
            .position(|&b| b == b':')
            .ok_or(Error::MaxRecords)?;

        let discriminant =
            String::from_utf8(bytes[..separator_pos].to_vec()).map_err(|_| Error::MaxRecords)?;
        let key_bytes = bytes[separator_pos + 1..].to_vec();

        Ok(Self {
            discriminant,
            key_bytes,
        })
    }

    /// Get the tree name for this key
    pub fn tree_name(&self) -> &str {
        &self.discriminant
    }
}

/// Extension trait for model-aware RecordStore operations
///
/// This trait provides methods that work directly with NetabaseModelTrait types,
/// ensuring that data stored in the database is always a model (not a Definition wrapper).
#[cfg(feature = "libp2p")]
pub trait ModelRecordStore {
    /// Put a model into the store
    ///
    /// This encodes the model DIRECTLY (not wrapped in Definition) and stores it with
    /// a key that includes the model discriminant, allowing efficient routing without decoding.
    ///
    /// # Guarantee
    ///
    /// The data stored in the database is guaranteed to be a NetabaseModelTrait type,
    /// serialized directly without the Definition wrapper.
    fn put_model<D, M>(&mut self, model: &M) -> Result<()>
    where
        D: NetabaseDefinitionTrait,
        M: NetabaseModelTrait<D> + bincode::Encode + Clone,
        <D as IntoDiscriminant>::Discriminant: AsRef<str>
            + Clone
            + Copy
            + std::fmt::Debug
            + std::fmt::Display
            + PartialEq
            + Eq
            + std::hash::Hash
            + strum::IntoEnumIterator
            + Send
            + Sync
            + 'static
            + FromStr;

    /// Get a model from the store by its primary key
    ///
    /// Returns None if the model doesn't exist.
    ///
    /// # Type Safety
    ///
    /// The model type is decoded directly from storage, ensuring type safety.
    /// The discriminant in the key ensures we're retrieving the correct model type.
    fn get_model<D, M, K>(&self, key: &K) -> Option<M>
    where
        D: NetabaseDefinitionTrait,
        M: NetabaseModelTrait<D> + bincode::Decode<()>,
        K: bincode::Encode,
        <D as IntoDiscriminant>::Discriminant: AsRef<str>
            + Clone
            + Copy
            + std::fmt::Debug
            + std::fmt::Display
            + PartialEq
            + Eq
            + std::hash::Hash
            + strum::IntoEnumIterator
            + Send
            + Sync
            + 'static
            + FromStr;

    /// Remove a model from the store
    fn remove_model<D, M, K>(&mut self, key: &K)
    where
        D: NetabaseDefinitionTrait,
        M: NetabaseModelTrait<D>,
        K: bincode::Encode,
        <D as IntoDiscriminant>::Discriminant: AsRef<str>
            + Clone
            + Copy
            + std::fmt::Debug
            + std::fmt::Display
            + PartialEq
            + Eq
            + std::hash::Hash
            + strum::IntoEnumIterator
            + Send
            + Sync
            + 'static
            + FromStr;
}

/// Helper functions for working with model records
#[cfg(feature = "libp2p")]
pub mod utils {
    use super::*;

    /// Encode a model as a Record (stores the model directly, not wrapped in Definition)
    ///
    /// The model is serialized directly to ensure the database stores NetabaseModelTrait types.
    /// The model type information is preserved in the key via the discriminant.
    pub fn model_to_record<D, M>(model: &M) -> Result<Record>
    where
        D: NetabaseDefinitionTrait,
        M: NetabaseModelTrait<D> + bincode::Encode + Clone,
        <D as IntoDiscriminant>::Discriminant: AsRef<str>
            + Clone
            + Copy
            + std::fmt::Debug
            + std::fmt::Display
            + PartialEq
            + Eq
            + std::hash::Hash
            + strum::IntoEnumIterator
            + Send
            + Sync
            + 'static
            + FromStr,
    {
        // Create model record key with discriminant
        let model_key = ModelRecordKey::from_model::<D, M>(model);
        let record_key = model_key.to_record_key();

        // Encode the model DIRECTLY (not wrapped in Definition)
        // This ensures the database stores NetabaseModelTrait types
        let value_bytes = bincode::encode_to_vec(model, bincode::config::standard())
            .map_err(|_| Error::ValueTooLarge)?;

        Ok(Record {
            key: record_key,
            value: value_bytes,
            publisher: None,
            expires: None,
        })
    }

    /// Decode a Record to a model (expecting the model to be stored directly)
    ///
    /// This decodes directly to the model type, not through a Definition wrapper.
    pub fn record_to_model<D, M>(record: &Record) -> Result<M>
    where
        D: NetabaseDefinitionTrait,
        M: NetabaseModelTrait<D> + bincode::Decode<()>,
        <D as IntoDiscriminant>::Discriminant: AsRef<str>
            + Clone
            + Copy
            + std::fmt::Debug
            + std::fmt::Display
            + PartialEq
            + Eq
            + std::hash::Hash
            + strum::IntoEnumIterator
            + Send
            + Sync
            + 'static
            + FromStr,
    {
        // Decode the value DIRECTLY as the model type
        let (model, _): (M, _) =
            bincode::decode_from_slice(&record.value, bincode::config::standard())
                .map_err(|_| Error::MaxRecords)?;

        Ok(model)
    }

    /// Create a ModelRecordKey from a typed key
    pub fn key_to_model_key<D, M, K>(key: &K) -> Result<ModelRecordKey>
    where
        D: NetabaseDefinitionTrait,
        M: NetabaseModelTrait<D>,
        K: bincode::Encode,
        <D as IntoDiscriminant>::Discriminant: AsRef<str>
            + Clone
            + Copy
            + std::fmt::Debug
            + std::fmt::Display
            + PartialEq
            + Eq
            + std::hash::Hash
            + strum::IntoEnumIterator
            + Send
            + Sync
            + 'static
            + FromStr,
    {
        let discriminant = M::discriminant_name().to_string();
        let key_bytes = bincode::encode_to_vec(key, bincode::config::standard())
            .map_err(|_| Error::MaxRecords)?;

        Ok(ModelRecordKey::new(discriminant, key_bytes))
    }
}
