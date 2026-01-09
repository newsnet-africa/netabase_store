// Models module - contains blob type definitions
pub mod blob_types;

use netabase_store::traits::database::hash::{CryptoHash, DefaultHash, FastHash, HashAlgorithm};
use std::hash::{Hash, Hasher};

/// A fast hasher wrapper using standard library DefaultHasher
#[derive(Debug, Clone, Copy)]
pub struct FastHasher;

impl HashAlgorithm for FastHasher {
    type Hasher = std::collections::hash_map::DefaultHasher;
}

/// Helper function to compute hash for a model (DefaultHasher)
pub fn hash_model<T: serde::Serialize>(model: &T) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    let bytes = postcard::to_allocvec(model).unwrap_or_default();
    bytes.hash(&mut hasher);
    hasher.finish()
}

/// Helper function to compute hash for a model using FxHash (Fast)
pub fn hash_model_fast<T: serde::Serialize>(model: &T) -> u64 {
    let mut hasher = <FastHash as HashAlgorithm>::new_hasher();
    let bytes = postcard::to_allocvec(model).unwrap_or_default();
    bytes.hash(&mut hasher);
    hasher.finish()
}

/// Helper function to compute hash for a model using SHA256 (Crypto - truncated to u64)
pub fn hash_model_crypto<T: serde::Serialize>(model: &T) -> u64 {
    let mut hasher = <CryptoHash as HashAlgorithm>::new_hasher();
    let bytes = postcard::to_allocvec(model).unwrap_or_default();
    bytes.hash(&mut hasher);
    hasher.finish()
}
