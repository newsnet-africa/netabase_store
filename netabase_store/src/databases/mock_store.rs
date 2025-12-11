//! Mock Store Implementation for Phase 8 Demo
//! 
//! This provides a simple in-memory store implementation to demonstrate
//! the cross-definition linking functionality without requiring the full
//! netabase store implementation.

use crate::error::{NetabaseError, NetabaseResult};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock store for demonstration purposes
#[derive(Clone)]
pub struct MockStore {
    data: Arc<Mutex<HashMap<String, String>>>,
}

impl MockStore {
    /// Create a new in-memory mock store
    pub fn new_in_memory() -> NetabaseResult<Self> {
        Ok(Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    /// Store a key-value pair
    pub fn put(&self, key: &str, value: &str) -> NetabaseResult<()> {
        let mut data = self.data.lock().map_err(|_| NetabaseError::StoreNotLoaded("Lock error".to_string()))?;
        data.insert(key.to_string(), value.to_string());
        Ok(())
    }
    
    /// Retrieve a value by key
    pub fn get(&self, key: &str) -> NetabaseResult<Option<String>> {
        let data = self.data.lock().map_err(|_| NetabaseError::StoreNotLoaded("Lock error".to_string()))?;
        Ok(data.get(key).cloned())
    }
    
    /// Check if the store is operational
    pub fn is_some(&self) -> bool {
        true
    }
}

/// Placeholder RedbStore type for compatibility
/// Uses PhantomData to handle the generic parameter
pub struct RedbStore<D> {
    store: MockStore,
    _marker: std::marker::PhantomData<D>,
}

impl<D> RedbStore<D> {
    /// Create a new in-memory RedbStore (mock implementation)
    pub fn new_redb_in_memory() -> Result<Self, String> {
        let store = MockStore::new_in_memory().map_err(|e| format!("Failed to create store: {}", e))?;
        Ok(RedbStore {
            store,
            _marker: std::marker::PhantomData,
        })
    }
}