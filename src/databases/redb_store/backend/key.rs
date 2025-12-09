//! Redb key type utilities
//!
//! Provides helper types and utilities for working with redb keys in the backend.
//! We don't implement BackendKey for redb types directly due to trait coherence rules.
//! Instead, the redb backend works directly with redb::Key types.

use redb::{Key, Value};

/// Helper trait that combines redb's Key and Value traits
///
/// This is used internally by the redb backend to constrain types.
/// Most redb types implement both Key and Value.
pub trait RedbKeyType: Key + Value + Clone + Send + Sync + 'static {
    // Marker trait - just combines the requirements
}

// Blanket implementation for types that meet the requirements
impl<T> RedbKeyType for T
where
    T: Key + Value + Clone + Send + Sync + 'static,
{
}

/// Helper trait that combines requirements for redb values
pub trait RedbValueType: Value + Clone + Send + Sync + 'static {
    // Marker trait
}

// Blanket implementation
impl<T> RedbValueType for T
where
    T: Value + Clone + Send + Sync + 'static,
{
}
