//! Prelude module for NetabaseStore
//! 
//! This module re-exports the most commonly used types and traits
//! to make it easier to use NetabaseStore.

// Error handling
pub use crate::error::{NetabaseError, NetabaseResult};

// Backend types  
pub use crate::backend::{
    BackendKey, BackendValue, BackendStore, BackendReadTransaction, BackendWriteTransaction,
    BackendError, BackendTable, BackendReadableTable, BackendWritableTable,
};

// Core traits
pub use crate::traits::{
    model::NetabaseModelTrait,
    definition::NetabaseDefinition,
};

// Database implementations
pub use crate::databases::{
    redb_store::{RedbStore, RedbDefinitionManager},
    sled_store::{SledStore, SledDefinitionManager}, 
    manager::DefinitionManager,
};

// Commonly used external dependencies
pub use serde::{Serialize, Deserialize};

// In-memory backend for testing/examples
#[derive(Debug, Clone)]
pub struct InMemoryBackend {
    data: std::collections::HashMap<Vec<u8>, Vec<u8>>,
}

impl InMemoryBackend {
    pub fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
        }
    }
}

impl Default for InMemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}