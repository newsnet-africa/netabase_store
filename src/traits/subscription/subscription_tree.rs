//! Enhanced subscription tree traits and implementations
//!
//! This module provides the core subscription tree functionality with proper
//! integration with the database backends and merkle tree comparison capabilities.

use std::hash::{Hash, Hasher};

use bincode::Encode;
use netabase_deps::blake3;
use serde::{Deserialize, Serialize};

/// Hash of a model for use in subscription trees
///
/// This is a wrapper around blake3::Hash that provides the necessary traits
/// for use in subscription trees and merkle tree operations.
#[derive(Clone)]
pub struct ModelHash(blake3::Hash);

impl Encode for ModelHash {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        bincode::Encode::encode(&self.0.as_bytes(), encoder)?;
        Ok(())
    }
}

impl bincode::Decode<()> for ModelHash {
    fn decode<D: bincode::de::Decoder<Context = ()>>(
        decoder: &mut D,
    ) -> core::result::Result<Self, bincode::error::DecodeError> {
        let blake_bytes: [u8; 32] = bincode::Decode::decode(decoder)?;
        Ok(Self(blake_bytes.into()))
    }
}

impl<'de> bincode::BorrowDecode<'de, ()> for ModelHash {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = ()>>(
        decoder: &mut D,
    ) -> core::result::Result<Self, bincode::error::DecodeError> {
        let blake_bytes: [u8; 32] = bincode::Decode::decode(decoder)?;
        Ok(Self(blake_bytes.into()))
    }
}

impl ModelHash {
    /// Create a new ModelHash from a blake3 hash
    pub fn new(hash: blake3::Hash) -> Self {
        Self(hash)
    }

    /// Create a ModelHash from raw data
    pub fn from_data<T: AsRef<[u8]>>(data: T) -> Self {
        let hash = blake3::hash(data.as_ref());
        Self(hash)
    }

    /// Create a ModelHash from key and data combined
    pub fn from_key_and_data<K: AsRef<[u8]>, D: AsRef<[u8]>>(key: K, data: D) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(key.as_ref());
        hasher.update(data.as_ref());
        Self(hasher.finalize())
    }

    /// Get the underlying blake3 hash
    pub fn inner(&self) -> &blake3::Hash {
        &self.0
    }

    /// Get the hash as bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }

    /// Convert to a hex string
    pub fn to_hex(&self) -> String {
        self.0.to_hex().to_string()
    }

    /// Create a zero hash (for testing or empty states)
    pub fn zero() -> Self {
        Self(blake3::Hash::from([0u8; 32]))
    }

    /// Check if this is a zero hash
    pub fn is_zero(&self) -> bool {
        self.0.as_bytes() == &[0u8; 32]
    }
}

impl From<blake3::Hash> for ModelHash {
    fn from(hash: blake3::Hash) -> Self {
        Self(hash)
    }
}

impl From<[u8; 32]> for ModelHash {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes.into())
    }
}

impl AsRef<[u8]> for ModelHash {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl Hash for ModelHash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.as_bytes().hash(state);
    }
}

impl PartialEq for ModelHash {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_bytes() == other.0.as_bytes()
    }
}

impl Eq for ModelHash {}

impl PartialOrd for ModelHash {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ModelHash {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.as_bytes().cmp(other.0.as_bytes())
    }
}

impl std::fmt::Display for ModelHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl std::fmt::Debug for ModelHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ModelHash").field(&self.to_hex()).finish()
    }
}

impl Serialize for ModelHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_hex().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ModelHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let hex_string = String::deserialize(deserializer)?;
        let bytes = blake3::Hash::from_hex(&hex_string)
            .map_err(|e| serde::de::Error::custom(format!("Invalid hash hex: {}", e)))?;
        Ok(Self(bytes))
    }
}

// Re-export the main implementations from the subscription module
pub use crate::subscription::subscription_tree::{
    DefaultSubscriptionManager, MerkleSubscriptionTree, SubscriptionDiff,
};

// Tests are moved to the main subscription module
