//! Redb error adapter
//!
//! Implements BackendError for redb error types, providing a clean
//! abstraction layer between redb and the Netabase core.

use crate::backend::error::BackendError;
use std::fmt;

/// Wrapper for redb errors that implements BackendError
#[derive(Debug)]
pub struct RedbBackendError {
    inner: RedbErrorKind,
}

#[derive(Debug)]
enum RedbErrorKind {
    Database(redb::DatabaseError),
    Transaction(redb::TransactionError),
    Table(redb::TableError),
    Storage(redb::StorageError),
    Commit(redb::CommitError),
}

impl RedbBackendError {
    pub fn from_database(err: redb::DatabaseError) -> Self {
        Self {
            inner: RedbErrorKind::Database(err),
        }
    }

    pub fn from_transaction(err: redb::TransactionError) -> Self {
        Self {
            inner: RedbErrorKind::Transaction(err),
        }
    }

    pub fn from_table(err: redb::TableError) -> Self {
        Self {
            inner: RedbErrorKind::Table(err),
        }
    }

    pub fn from_storage(err: redb::StorageError) -> Self {
        Self {
            inner: RedbErrorKind::Storage(err),
        }
    }

    pub fn from_commit(err: redb::CommitError) -> Self {
        Self {
            inner: RedbErrorKind::Commit(err),
        }
    }
}

impl fmt::Display for RedbBackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner {
            RedbErrorKind::Database(e) => write!(f, "Redb database error: {}", e),
            RedbErrorKind::Transaction(e) => write!(f, "Redb transaction error: {}", e),
            RedbErrorKind::Table(e) => write!(f, "Redb table error: {}", e),
            RedbErrorKind::Storage(e) => write!(f, "Redb storage error: {}", e),
            RedbErrorKind::Commit(e) => write!(f, "Redb commit error: {}", e),
        }
    }
}

impl std::error::Error for RedbBackendError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.inner {
            RedbErrorKind::Database(e) => Some(e),
            RedbErrorKind::Transaction(e) => Some(e),
            RedbErrorKind::Table(e) => Some(e),
            RedbErrorKind::Storage(e) => Some(e),
            RedbErrorKind::Commit(e) => Some(e),
        }
    }
}

impl BackendError for RedbBackendError {
    fn is_table_not_found(&self) -> bool {
        matches!(
            &self.inner,
            RedbErrorKind::Table(redb::TableError::TableDoesNotExist(_))
        )
    }

    fn is_transaction_conflict(&self) -> bool {
        // Redb doesn't have explicit transaction conflicts in the same way
        // as some databases, but we can check for relevant error types
        false
    }
}

// Conversion implementations
impl From<redb::DatabaseError> for RedbBackendError {
    fn from(err: redb::DatabaseError) -> Self {
        Self::from_database(err)
    }
}

impl From<redb::TransactionError> for RedbBackendError {
    fn from(err: redb::TransactionError) -> Self {
        Self::from_transaction(err)
    }
}

impl From<redb::TableError> for RedbBackendError {
    fn from(err: redb::TableError) -> Self {
        Self::from_table(err)
    }
}

impl From<redb::StorageError> for RedbBackendError {
    fn from(err: redb::StorageError) -> Self {
        Self::from_storage(err)
    }
}

impl From<redb::CommitError> for RedbBackendError {
    fn from(err: redb::CommitError) -> Self {
        Self::from_commit(err)
    }
}

// Conversions to Box<dyn BackendError>
impl From<redb::DatabaseError> for Box<dyn BackendError> {
    fn from(err: redb::DatabaseError) -> Self {
        Box::new(RedbBackendError::from(err))
    }
}

impl From<redb::TransactionError> for Box<dyn BackendError> {
    fn from(err: redb::TransactionError) -> Self {
        Box::new(RedbBackendError::from(err))
    }
}

impl From<redb::TableError> for Box<dyn BackendError> {
    fn from(err: redb::TableError) -> Self {
        Box::new(RedbBackendError::from(err))
    }
}

impl From<redb::StorageError> for Box<dyn BackendError> {
    fn from(err: redb::StorageError) -> Self {
        Box::new(RedbBackendError::from(err))
    }
}

impl From<redb::CommitError> for Box<dyn BackendError> {
    fn from(err: redb::CommitError) -> Self {
        Box::new(RedbBackendError::from(err))
    }
}
