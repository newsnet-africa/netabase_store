//! In-memory store implementation.
//!
//! This module provides the main `MemoryStore` type that implements the `NBStore` trait.

use std::marker::PhantomData;
use std::path::Path;
use strum::IntoDiscriminant;

use crate::errors::NetabaseResult;
use crate::traits::database::store::NBStore;
use crate::traits::registry::definition::NetabaseDefinition;

use super::storage::Storage;
use super::transaction::{MemoryReadTransaction, MemoryWriteTransaction};

/// An in-memory database store.
///
/// This provides a fast, ephemeral database suitable for testing and development.
/// All data is stored in memory and lost when the store is dropped.
///
/// # Example
///
/// ```rust,no_run
/// use netabase_store::databases::memory::MemoryStore;
/// use netabase_store::traits::database::store::NBStore;
///
/// let store = MemoryStore::<MyDefinition>::new();
///
/// // Use like any other store
/// let txn = store.begin_write()?;
/// txn.create(&model)?;
/// txn.commit()?;
///
/// let txn = store.begin_read()?;
/// let result = txn.read(&key)?;
/// ```
#[derive(Debug)]
pub struct MemoryStore<D>
where
    D: NetabaseDefinition,
    <D as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    storage: Storage,
    _phantom: PhantomData<D>,
}

impl<D> Clone for MemoryStore<D>
where
    D: NetabaseDefinition,
    <D as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<D> Default for MemoryStore<D>
where
    D: NetabaseDefinition,
    <D as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<D> MemoryStore<D>
where
    D: NetabaseDefinition,
    <D as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    /// Create a new empty in-memory store.
    pub fn new() -> Self {
        Self {
            storage: Storage::new(),
            _phantom: PhantomData,
        }
    }

    /// Get a reference to the underlying storage.
    ///
    /// This is mainly useful for testing and debugging.
    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    /// Begin a read-only transaction.
    pub fn begin_read(&self) -> NetabaseResult<MemoryReadTransaction<D>> {
        let snapshot = self.storage.snapshot()?;
        Ok(MemoryReadTransaction::new(snapshot))
    }

    /// Begin a read-write transaction.
    pub fn begin_write(&self) -> NetabaseResult<MemoryWriteTransaction<D>> {
        Ok(MemoryWriteTransaction::new(self.storage.clone()))
    }
}

impl<D> NBStore<D> for MemoryStore<D>
where
    D: NetabaseDefinition,
    <D as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    /// Create a new store.
    ///
    /// For `MemoryStore`, the path is ignored since data is stored in memory.
    fn new<P: AsRef<Path>>(_path: P) -> NetabaseResult<Self>
    where
        Self: Sized,
    {
        Ok(Self::new())
    }

    fn execute_transaction<F: Fn()>(f: F) {
        f()
    }
}

// TODO: Re-enable tests when doc_example module is fixed
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::doc_example::ExampleDef;
//
//     #[test]
//     fn test_memory_store_creation() {
//         let store = MemoryStore::<ExampleDef>::new();
//         assert!(store.begin_read().is_ok());
//         assert!(store.begin_write().is_ok());
//     }
//
//     #[test]
//     fn test_memory_store_clone() {
//         let store1 = MemoryStore::<ExampleDef>::new();
//         let store2 = store1.clone();
//         
//         // Both should share the same underlying storage
//         // (writes through one should be visible through the other)
//         // This is intentional for the testing use case
//         assert!(store1.begin_read().is_ok());
//         assert!(store2.begin_read().is_ok());
//     }
// }
