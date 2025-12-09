//! Redb table adapters
//!
//! Provides wrappers around redb table types for internal use.
//! These don't implement the backend traits directly, but are used by the transaction adapters.

use super::key::{RedbKeyType, RedbValueType};
use redb::{ReadableTable, ReadableTableMetadata};

/// Read-only table wrapper for redb
///
/// This is a simple wrapper that provides access to redb's table functionality.
pub struct RedbReadableTableAdapter<K, V>
where
    K: RedbKeyType,
    V: RedbValueType,
{
    pub(crate) table: redb::ReadOnlyTable<K, V>,
}

impl<K, V> RedbReadableTableAdapter<K, V>
where
    K: RedbKeyType,
    V: RedbValueType,
{
    pub fn new(table: redb::ReadOnlyTable<K, V>) -> Self {
        Self { table }
    }

    pub fn get<'a>(&'a self, key: K) -> Result<Option<redb::AccessGuard<'a, V>>, redb::StorageError>
    where
        K: std::borrow::Borrow<<K as redb::Value>::SelfType<'static>>,
        V: redb::Value,
    {
        self.table.get(key)
    }

    pub fn len(&self) -> Result<u64, redb::StorageError> {
        self.table.len()
    }
}

/// Writable table wrapper for redb
pub struct RedbWritableTableAdapter<'txn, K, V>
where
    K: RedbKeyType,
    V: RedbValueType,
{
    pub(crate) table: redb::Table<'txn, K, V>,
}

impl<'txn, K, V> RedbWritableTableAdapter<'txn, K, V>
where
    K: RedbKeyType,
    V: RedbValueType,
{
    pub fn new(table: redb::Table<'txn, K, V>) -> Self {
        Self { table }
    }

    pub fn get<'a>(&'a self, key: K) -> Result<Option<redb::AccessGuard<'a, V>>, redb::StorageError>
    where
        K: std::borrow::Borrow<<K as redb::Value>::SelfType<'static>>,
        V: redb::Value,
    {
        self.table.get(key)
    }

    pub fn insert(&mut self, key: K, value: V) -> Result<(), redb::StorageError>
    where
        K: std::borrow::Borrow<<K as redb::Value>::SelfType<'static>>,
        V: std::borrow::Borrow<<V as redb::Value>::SelfType<'static>>,
    {
        self.table.insert(key, value)?;
        Ok(())
    }

    pub fn remove(&mut self, key: K) -> Result<bool, redb::StorageError>
    where
        K: std::borrow::Borrow<<K as redb::Value>::SelfType<'static>>,
    {
        match self.table.remove(key)? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    pub fn len(&self) -> Result<u64, redb::StorageError> {
        self.table.len()
    }
}
