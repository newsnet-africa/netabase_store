//! Transaction types for the memory backend.
//!
//! This module provides read and write transaction types that operate on the
//! in-memory storage.

use std::marker::PhantomData;
use strum::IntoDiscriminant;

use crate::errors::NetabaseResult;
use crate::traits::registry::definition::NetabaseDefinition;

use super::storage::{Storage, StorageInner};

/// A read-only transaction on a memory store.
///
/// Read transactions operate on a snapshot of the data, providing isolation
/// from concurrent writes.
#[derive(Debug)]
pub struct MemoryReadTransaction<D>
where
    D: NetabaseDefinition,
    <D as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    snapshot: StorageInner,
    _phantom: PhantomData<D>,
}

impl<D> MemoryReadTransaction<D>
where
    D: NetabaseDefinition,
    <D as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    /// Create a new read transaction from a storage snapshot.
    pub(crate) fn new(snapshot: StorageInner) -> Self {
        Self {
            snapshot,
            _phantom: PhantomData,
        }
    }

    /// Read a value from a table by key.
    pub fn get(&self, table: &str, key: &[u8]) -> Option<Vec<u8>> {
        self.snapshot.get(table, key)
    }

    /// Read all values for a key from a multimap table.
    pub fn get_multimap(&self, table: &str, key: &[u8]) -> Vec<Vec<u8>> {
        self.snapshot.get_multimap(table, key)
    }

    /// Iterate over all entries in a table.
    pub fn iter_table(&self, table: &str) -> impl Iterator<Item = (&Vec<u8>, &Vec<u8>)> {
        self.snapshot.iter_table(table)
    }

    /// Count entries in a table.
    pub fn count_table(&self, table: &str) -> usize {
        self.snapshot.count_table(table)
    }

    /// Count entries in a multimap table.
    pub fn count_multimap(&self, table: &str) -> usize {
        self.snapshot.count_multimap(table)
    }
}

/// A read-write transaction on a memory store.
///
/// Write transactions accumulate mutations and apply them atomically on commit.
/// If the transaction is dropped without committing, all changes are discarded.
#[derive(Debug)]
pub struct MemoryWriteTransaction<D>
where
    D: NetabaseDefinition,
    <D as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    storage: Storage,
    mutations: Vec<Mutation>,
    committed: bool,
    _phantom: PhantomData<D>,
}

/// A mutation to be applied to storage.
#[derive(Debug, Clone)]
enum Mutation {
    Insert {
        table: String,
        key: Vec<u8>,
        value: Vec<u8>,
    },
    Remove {
        table: String,
        key: Vec<u8>,
    },
    InsertMultimap {
        table: String,
        key: Vec<u8>,
        value: Vec<u8>,
    },
    RemoveMultimap {
        table: String,
        key: Vec<u8>,
        value: Vec<u8>,
    },
    RemoveAllMultimap {
        table: String,
        key: Vec<u8>,
    },
}

impl<D> MemoryWriteTransaction<D>
where
    D: NetabaseDefinition,
    <D as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    /// Create a new write transaction.
    pub(crate) fn new(storage: Storage) -> Self {
        Self {
            storage,
            mutations: Vec::new(),
            committed: false,
            _phantom: PhantomData,
        }
    }

    /// Read a value from a table by key.
    ///
    /// This reads from the current committed state, not pending mutations.
    pub fn get(&self, table: &str, key: &[u8]) -> NetabaseResult<Option<Vec<u8>>> {
        let guard = self.storage.read()?;
        Ok(guard.get(table, key))
    }

    /// Read all values for a key from a multimap table.
    pub fn get_multimap(&self, table: &str, key: &[u8]) -> NetabaseResult<Vec<Vec<u8>>> {
        let guard = self.storage.read()?;
        Ok(guard.get_multimap(table, key))
    }

    /// Insert a value into a table.
    pub fn insert(&mut self, table: &str, key: Vec<u8>, value: Vec<u8>) {
        self.mutations.push(Mutation::Insert {
            table: table.to_string(),
            key,
            value,
        });
    }

    /// Remove a value from a table.
    pub fn remove(&mut self, table: &str, key: Vec<u8>) {
        self.mutations.push(Mutation::Remove {
            table: table.to_string(),
            key,
        });
    }

    /// Insert a value into a multimap table.
    pub fn insert_multimap(&mut self, table: &str, key: Vec<u8>, value: Vec<u8>) {
        self.mutations.push(Mutation::InsertMultimap {
            table: table.to_string(),
            key,
            value,
        });
    }

    /// Remove a specific value from a multimap table.
    pub fn remove_multimap(&mut self, table: &str, key: Vec<u8>, value: Vec<u8>) {
        self.mutations.push(Mutation::RemoveMultimap {
            table: table.to_string(),
            key,
            value,
        });
    }

    /// Remove all values for a key from a multimap table.
    pub fn remove_all_multimap(&mut self, table: &str, key: Vec<u8>) {
        self.mutations.push(Mutation::RemoveAllMultimap {
            table: table.to_string(),
            key,
        });
    }

    /// Commit all mutations atomically.
    pub fn commit(mut self) -> NetabaseResult<()> {
        let mut guard = self.storage.write()?;
        
        for mutation in self.mutations.drain(..) {
            match mutation {
                Mutation::Insert { table, key, value } => {
                    guard.insert(&table, key, value);
                }
                Mutation::Remove { table, key } => {
                    guard.remove(&table, &key);
                }
                Mutation::InsertMultimap { table, key, value } => {
                    guard.insert_multimap(&table, key, value);
                }
                Mutation::RemoveMultimap { table, key, value } => {
                    guard.remove_multimap(&table, &key, &value);
                }
                Mutation::RemoveAllMultimap { table, key } => {
                    guard.remove_all_multimap(&table, &key);
                }
            }
        }
        
        self.committed = true;
        Ok(())
    }

    /// Abort the transaction, discarding all mutations.
    pub fn abort(mut self) {
        self.mutations.clear();
        self.committed = true;
    }

    /// Iterate over all entries in a table.
    pub fn iter_table(&self, table: &str) -> NetabaseResult<Vec<(Vec<u8>, Vec<u8>)>> {
        let guard = self.storage.read()?;
        Ok(guard.iter_table(table)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect())
    }

    /// Count entries in a table.
    pub fn count_table(&self, table: &str) -> NetabaseResult<usize> {
        let guard = self.storage.read()?;
        Ok(guard.count_table(table))
    }
}

impl<D> Drop for MemoryWriteTransaction<D>
where
    D: NetabaseDefinition,
    <D as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    fn drop(&mut self) {
        if !self.committed && !self.mutations.is_empty() {
            // Log warning about uncommitted transaction
            #[cfg(debug_assertions)]
            eprintln!(
                "Warning: MemoryWriteTransaction dropped with {} uncommitted mutations",
                self.mutations.len()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc_examples::ExampleDef;
    use super::super::storage::Storage;

    #[test]
    fn test_read_transaction() {
        let storage = Storage::new();
        
        // Insert data
        {
            let mut guard = storage.write().unwrap();
            guard.insert("users", b"alice".to_vec(), b"Alice".to_vec());
        }
        
        // Create read transaction
        let snapshot = storage.snapshot().unwrap();
        let txn = MemoryReadTransaction::<ExampleDef>::new(snapshot);
        
        assert_eq!(txn.get("users", b"alice"), Some(b"Alice".to_vec()));
        assert_eq!(txn.get("users", b"bob"), None);
    }

    #[test]
    fn test_write_transaction_commit() {
        let storage = Storage::new();
        
        // Create and commit write transaction
        {
            let mut txn = MemoryWriteTransaction::<ExampleDef>::new(storage.clone());
            txn.insert("users", b"alice".to_vec(), b"Alice".to_vec());
            txn.insert("users", b"bob".to_vec(), b"Bob".to_vec());
            txn.commit().unwrap();
        }
        
        // Verify data was committed
        let guard = storage.read().unwrap();
        assert_eq!(guard.get("users", b"alice"), Some(b"Alice".to_vec()));
        assert_eq!(guard.get("users", b"bob"), Some(b"Bob".to_vec()));
    }

    #[test]
    fn test_write_transaction_abort() {
        let storage = Storage::new();
        
        // Insert initial data
        {
            let mut guard = storage.write().unwrap();
            guard.insert("users", b"alice".to_vec(), b"Alice".to_vec());
        }
        
        // Create and abort write transaction
        {
            let mut txn = MemoryWriteTransaction::<ExampleDef>::new(storage.clone());
            txn.insert("users", b"bob".to_vec(), b"Bob".to_vec());
            txn.remove("users", b"alice".to_vec());
            txn.abort();
        }
        
        // Verify data was NOT changed
        let guard = storage.read().unwrap();
        assert_eq!(guard.get("users", b"alice"), Some(b"Alice".to_vec()));
        assert_eq!(guard.get("users", b"bob"), None);
    }

    #[test]
    fn test_multimap_operations() {
        let storage = Storage::new();
        
        // Create write transaction with multimap operations
        {
            let mut txn = MemoryWriteTransaction::<ExampleDef>::new(storage.clone());
            txn.insert_multimap("index", b"category".to_vec(), b"item1".to_vec());
            txn.insert_multimap("index", b"category".to_vec(), b"item2".to_vec());
            txn.commit().unwrap();
        }
        
        // Verify multimap data
        let guard = storage.read().unwrap();
        let values = guard.get_multimap("index", b"category");
        assert_eq!(values.len(), 2);
        assert!(values.contains(&b"item1".to_vec()));
        assert!(values.contains(&b"item2".to_vec()));
    }
}
