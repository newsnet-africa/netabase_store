// Models module - contains blob type definitions
pub mod blob_types;

use netabase_store::traits::database::hash::HashAlgorithm;
use std::hash::{Hash, Hasher};

/// A fast hasher wrapper using standard library DefaultHasher
#[derive(Debug, Clone, Copy)]
pub struct FastHasher;

impl HashAlgorithm for FastHasher {
    type Hasher = std::collections::hash_map::DefaultHasher;
}

/// Helper function to compute hash for a model
pub fn hash_model<T: serde::Serialize>(model: &T) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    // Serialize to bytes first to ensure consistent hashing across platforms/runs
    // (though DefaultHasher is not guaranteed to be consistent across runs, it's fine for testing)
    // For production, use a stable hasher like Sha256 or similar.
    // Here we just hash the debug string or similar for simplicity if T implements Hash?
    // But T is NetabaseModel which requires serde::Serialize.
    // Let's use postcard to bytes then hash bytes.
    let bytes = postcard::to_allocvec(model).unwrap_or_default();
    bytes.hash(&mut hasher);
    hasher.finish()
}