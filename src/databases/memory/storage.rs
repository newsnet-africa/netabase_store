//! Low-level storage implementation for the memory backend.
//!
//! This module provides the core storage primitives that mimic redb's behavior.

use std::collections::BTreeMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::errors::{NetabaseError, NetabaseResult};

/// A table name identifier.
pub type TableName = String;

/// A single table storing key-value pairs.
pub type Table = BTreeMap<Vec<u8>, Vec<u8>>;

/// A multimap table storing key to multiple values.
pub type MultimapTable = BTreeMap<Vec<u8>, Vec<Vec<u8>>>;

/// The internal storage structure.
#[derive(Debug, Default)]
pub struct StorageInner {
    /// Regular tables (key -> value)
    pub tables: BTreeMap<TableName, Table>,
    /// Multimap tables (key -> [value, value, ...])
    pub multimap_tables: BTreeMap<TableName, MultimapTable>,
}

/// Thread-safe storage wrapper.
#[derive(Debug, Clone, Default)]
pub struct Storage {
    inner: Arc<RwLock<StorageInner>>,
}

impl Storage {
    /// Create a new empty storage.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(StorageInner::default())),
        }
    }

    /// Get a read lock on the storage.
    pub fn read(&self) -> NetabaseResult<RwLockReadGuard<'_, StorageInner>> {
        self.inner
            .read()
            .map_err(|_| NetabaseError::Other)
    }

    /// Get a write lock on the storage.
    pub fn write(&self) -> NetabaseResult<RwLockWriteGuard<'_, StorageInner>> {
        self.inner
            .write()
            .map_err(|_| NetabaseError::Other)
    }

    /// Create a snapshot of the current storage state.
    pub fn snapshot(&self) -> NetabaseResult<StorageInner> {
        let guard = self.read()?;
        Ok(StorageInner {
            tables: guard.tables.clone(),
            multimap_tables: guard.multimap_tables.clone(),
        })
    }
}

impl StorageInner {
    /// Ensure a regular table exists.
    pub fn ensure_table(&mut self, name: &str) {
        self.tables.entry(name.to_string()).or_default();
    }

    /// Ensure a multimap table exists.
    pub fn ensure_multimap_table(&mut self, name: &str) {
        self.multimap_tables.entry(name.to_string()).or_default();
    }

    /// Get a value from a regular table.
    pub fn get(&self, table: &str, key: &[u8]) -> Option<Vec<u8>> {
        self.tables.get(table)?.get(key).cloned()
    }

    /// Insert a value into a regular table.
    pub fn insert(&mut self, table: &str, key: Vec<u8>, value: Vec<u8>) {
        self.ensure_table(table);
        self.tables.get_mut(table).unwrap().insert(key, value);
    }

    /// Remove a value from a regular table.
    pub fn remove(&mut self, table: &str, key: &[u8]) -> Option<Vec<u8>> {
        self.tables.get_mut(table)?.remove(key)
    }

    /// Get all values for a key from a multimap table.
    pub fn get_multimap(&self, table: &str, key: &[u8]) -> Vec<Vec<u8>> {
        self.multimap_tables
            .get(table)
            .and_then(|t| t.get(key))
            .cloned()
            .unwrap_or_default()
    }

    /// Insert a value into a multimap table.
    pub fn insert_multimap(&mut self, table: &str, key: Vec<u8>, value: Vec<u8>) {
        self.ensure_multimap_table(table);
        self.multimap_tables
            .get_mut(table)
            .unwrap()
            .entry(key)
            .or_default()
            .push(value);
    }

    /// Remove a specific value from a multimap table.
    pub fn remove_multimap(&mut self, table: &str, key: &[u8], value: &[u8]) -> bool {
        if let Some(table) = self.multimap_tables.get_mut(table)
            && let Some(values) = table.get_mut(key)
                && let Some(pos) = values.iter().position(|v| v == value) {
                    values.remove(pos);
                    return true;
                }
        false
    }

    /// Remove all values for a key from a multimap table.
    pub fn remove_all_multimap(&mut self, table: &str, key: &[u8]) -> Vec<Vec<u8>> {
        self.multimap_tables
            .get_mut(table)
            .and_then(|t| t.remove(key))
            .unwrap_or_default()
    }

    /// Iterate over all entries in a regular table.
    pub fn iter_table(&self, table: &str) -> impl Iterator<Item = (&Vec<u8>, &Vec<u8>)> {
        self.tables.get(table).into_iter().flatten()
    }

    /// Iterate over a range in a regular table.
    pub fn range_table<'a>(
        &'a self,
        table: &str,
        start: &[u8],
        end: &[u8],
    ) -> impl Iterator<Item = (&'a Vec<u8>, &'a Vec<u8>)> {
        self.tables
            .get(table)
            .into_iter()
            .flat_map(move |t| t.range(start.to_vec()..end.to_vec()))
    }

    /// Count entries in a regular table.
    pub fn count_table(&self, table: &str) -> usize {
        self.tables.get(table).map(|t| t.len()).unwrap_or(0)
    }

    /// Count entries in a multimap table.
    pub fn count_multimap(&self, table: &str) -> usize {
        self.multimap_tables
            .get(table)
            .map(|t| t.values().map(|v| v.len()).sum())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_operations() {
        let storage = Storage::new();
        
        // Insert
        {
            let mut guard = storage.write().unwrap();
            guard.insert("test", b"key1".to_vec(), b"value1".to_vec());
            guard.insert("test", b"key2".to_vec(), b"value2".to_vec());
        }
        
        // Read
        {
            let guard = storage.read().unwrap();
            assert_eq!(guard.get("test", b"key1"), Some(b"value1".to_vec()));
            assert_eq!(guard.get("test", b"key2"), Some(b"value2".to_vec()));
            assert_eq!(guard.get("test", b"key3"), None);
        }
        
        // Remove
        {
            let mut guard = storage.write().unwrap();
            assert_eq!(guard.remove("test", b"key1"), Some(b"value1".to_vec()));
            assert_eq!(guard.get("test", b"key1"), None);
        }
    }

    #[test]
    fn test_multimap_operations() {
        let storage = Storage::new();
        
        // Insert multiple values for same key
        {
            let mut guard = storage.write().unwrap();
            guard.insert_multimap("index", b"category".to_vec(), b"item1".to_vec());
            guard.insert_multimap("index", b"category".to_vec(), b"item2".to_vec());
            guard.insert_multimap("index", b"category".to_vec(), b"item3".to_vec());
        }
        
        // Read all values
        {
            let guard = storage.read().unwrap();
            let values = guard.get_multimap("index", b"category");
            assert_eq!(values.len(), 3);
            assert!(values.contains(&b"item1".to_vec()));
            assert!(values.contains(&b"item2".to_vec()));
            assert!(values.contains(&b"item3".to_vec()));
        }
        
        // Remove specific value
        {
            let mut guard = storage.write().unwrap();
            assert!(guard.remove_multimap("index", b"category", b"item2"));
            let values = guard.get_multimap("index", b"category");
            assert_eq!(values.len(), 2);
            assert!(!values.contains(&b"item2".to_vec()));
        }
    }

    #[test]
    fn test_snapshot() {
        let storage = Storage::new();
        
        // Insert data
        {
            let mut guard = storage.write().unwrap();
            guard.insert("test", b"key".to_vec(), b"value".to_vec());
        }
        
        // Take snapshot
        let snapshot = storage.snapshot().unwrap();
        
        // Modify original
        {
            let mut guard = storage.write().unwrap();
            guard.insert("test", b"key".to_vec(), b"new_value".to_vec());
        }
        
        // Snapshot should have old value
        assert_eq!(snapshot.get("test", b"key"), Some(b"value".to_vec()));
        
        // Original should have new value
        let guard = storage.read().unwrap();
        assert_eq!(guard.get("test", b"key"), Some(b"new_value".to_vec()));
    }
}
