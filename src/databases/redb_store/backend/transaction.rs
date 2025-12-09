//! Redb transaction adapters
//!
//! Wraps redb transaction types to implement BackendReadTransaction and BackendWriteTransaction.

use crate::backend::{
    BackendError, BackendKey, BackendReadTransaction, BackendReadableTable, BackendValue,
    BackendWritableTable, BackendWriteTransaction,
};
use redb::{Key, ReadTransaction, TableDefinition, Value, WriteTransaction};

/// Read transaction adapter for redb
pub struct RedbReadTransactionAdapter<'db> {
    #[allow(dead_code)]
    txn: ReadTransaction,
    _phantom: std::marker::PhantomData<&'db ()>,
}

impl<'db> RedbReadTransactionAdapter<'db> {
    pub fn new(txn: ReadTransaction) -> Self {
        Self {
            txn,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'db> BackendReadTransaction for RedbReadTransactionAdapter<'db> {
    fn open_table<K: BackendKey, V: BackendValue>(
        &self,
        table_name: &str,
    ) -> Result<Box<dyn BackendReadableTable<K, V>>, Box<dyn BackendError>> {
        // This requires K and V to also implement redb::Key and redb::Value
        // Due to trait constraints, we need to use a different approach
        // We'll create a type-erased wrapper that can handle this

        // For now, we'll return an error indicating this needs the concrete types
        // In practice, this will be called from code that knows the concrete types
        Err(Box::new(
            crate::databases::redb_store::backend::error::RedbBackendError::from_table(
                redb::TableError::TableDoesNotExist(table_name.to_string()),
            ),
        ))
    }

    fn table_exists(&self, _table_name: &str) -> Result<bool, Box<dyn BackendError>> {
        // Redb doesn't have a direct "table exists" check
        // We'd need to maintain a registry or try to open the table
        // For now, return false - this will be handled at a higher level
        Ok(false)
    }
}

/// Write transaction adapter for redb
pub struct RedbWriteTransactionAdapter {
    txn: WriteTransaction,
}

impl RedbWriteTransactionAdapter {
    pub fn new(txn: WriteTransaction) -> Self {
        Self { txn }
    }

    /// Internal method to open a redb table with concrete types
    ///
    /// This is used by the higher-level Netabase code that knows the concrete types.
    /// Note: Write transactions in redb don't provide read-only table access
    /// Use open_redb_table_mut for mutable access
    pub fn open_redb_table<K, V>(
        &self,
        table_name: &str,
    ) -> Result<(), Box<dyn BackendError>>
    where
        K: Key + 'static,
        V: Value + 'static,
    {
        // Write transactions in redb don't support opening read-only tables
        // Callers should use open_redb_table_mut instead
        Err(Box::new(crate::databases::redb_store::backend::error::RedbBackendError::from_table(
            redb::TableError::TableDoesNotExist(table_name.to_string())
        )))
    }

    /// Internal method to open a mutable redb table with concrete types
    pub fn open_redb_table_mut<K, V>(
        &mut self,
        table_name: &str,
    ) -> Result<redb::Table<'_, K, V>, Box<dyn BackendError>>
    where
        K: Key + 'static,
        V: Value + 'static,
    {
        let table_def: TableDefinition<K, V> = TableDefinition::new(table_name);
        self.txn
            .open_table(table_def)
            .map_err(|e| Box::new(crate::databases::redb_store::backend::error::RedbBackendError::from_table(e)) as Box<dyn BackendError>)
    }

    /// Get access to the underlying redb transaction
    ///
    /// This is used by Netabase internals that need direct redb access.
    pub fn redb_transaction(&self) -> &WriteTransaction {
        &self.txn
    }

    /// Get mutable access to the underlying redb transaction
    pub fn redb_transaction_mut(&mut self) -> &mut WriteTransaction {
        &mut self.txn
    }
}

impl BackendReadTransaction for RedbWriteTransactionAdapter {
    fn open_table<K: BackendKey, V: BackendValue>(
        &self,
        table_name: &str,
    ) -> Result<Box<dyn BackendReadableTable<K, V>>, Box<dyn BackendError>> {
        // Same limitation as read transaction
        Err(Box::new(
            crate::databases::redb_store::backend::error::RedbBackendError::from_table(
                redb::TableError::TableDoesNotExist(table_name.to_string()),
            ),
        ))
    }

    fn table_exists(&self, _table_name: &str) -> Result<bool, Box<dyn BackendError>> {
        Ok(false)
    }
}

impl BackendWriteTransaction for RedbWriteTransactionAdapter {
    fn open_table_mut<K: BackendKey, V: BackendValue>(
        &mut self,
        _table_name: &str,
    ) -> Result<Box<dyn BackendWritableTable<K, V>>, Box<dyn BackendError>> {
        // Same limitation as read operations
        // The actual implementation will use the internal methods that know concrete types
        todo!("Use internal open_redb_table_mut with concrete types")
    }

    fn commit(self) -> Result<(), Box<dyn BackendError>> {
        self.txn
            .commit()
            .map_err(|e| Box::new(crate::databases::redb_store::backend::error::RedbBackendError::from_commit(e)) as Box<dyn BackendError>)
    }

    fn abort(self) -> Result<(), Box<dyn BackendError>> {
        self.txn.abort().map_err(|e| -> Box<dyn BackendError> {
            Box::new(crate::databases::redb_store::backend::error::RedbBackendError::from(e))
        })
    }
}
