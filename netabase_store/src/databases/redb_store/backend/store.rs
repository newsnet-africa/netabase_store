//! Redb store adapter
//!
//! Implements BackendStore for redb, providing the main database interface.

use crate::backend::{BackendError, BackendStore};
use super::transaction::{RedbReadTransactionAdapter, RedbWriteTransactionAdapter};
use redb::ReadableDatabase;
use std::path::Path;

/// Redb backend store implementation
///
/// This wraps a redb::Database and implements the BackendStore trait,
/// allowing it to be used as a storage backend for Netabase.
pub struct RedbBackendStore {
    db: redb::Database,
}

impl RedbBackendStore {
    /// Create or open a redb database at the given path
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn BackendError>> {
        let db = redb::Database::create(path).map_err(|e| {
            Box::new(crate::databases::redb_store::backend::error::RedbBackendError::from_database(e))
                as Box<dyn BackendError>
        })?;

        Ok(Self { db })
    }

    /// Get a reference to the underlying redb database
    ///
    /// This is used by Netabase internals that need direct redb access.
    pub fn redb_database(&self) -> &redb::Database {
        &self.db
    }
}

impl BackendStore for RedbBackendStore {
    type ReadTransaction<'a> = RedbReadTransactionAdapter<'a>;
    type WriteTransaction = RedbWriteTransactionAdapter;

    fn begin_read(&self) -> Result<Self::ReadTransaction<'_>, Box<dyn BackendError>> {
        let txn = self.db.begin_read().map_err(|e| -> Box<dyn BackendError> {
            Box::new(super::error::RedbBackendError::from(e))
        })?;
        Ok(RedbReadTransactionAdapter::new(txn))
    }

    fn begin_write(&self) -> Result<Self::WriteTransaction, Box<dyn BackendError>> {
        let txn = self.db.begin_write().map_err(|e| -> Box<dyn BackendError> {
            Box::new(super::error::RedbBackendError::from(e))
        })?;
        Ok(RedbWriteTransactionAdapter::new(txn))
    }

    fn flush(&self) -> Result<(), Box<dyn BackendError>> {
        // Redb doesn't require explicit flushing
        Ok(())
    }

    fn close(self) -> Result<(), Box<dyn BackendError>> {
        // Redb closes automatically when dropped
        Ok(())
    }
}
