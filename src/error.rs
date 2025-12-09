//! Error types for Netabase operations.
//!
//! This module defines the error types that can occur during database operations.
//! All public APIs in Netabase return `NetabaseResult<T>`, which is an alias for
//! `Result<T, NetabaseError>`.
//!
//! # Error Handling Example
//!
//! ```
//! use netabase_store::error::{NetabaseError, NetabaseResult};
//!
//! fn example_operation() -> NetabaseResult<()> {
//!     // Operations that might fail return NetabaseResult
//!     // You can use ? operator for error propagation
//!     Ok(())
//! }
//!
//! // Handle errors with pattern matching
//! match example_operation() {
//!     Ok(()) => println!("Success!"),
//!     Err(NetabaseError::RedbError(e)) => {
//!         eprintln!("Database error: {}", e);
//!     }
//!     Err(NetabaseError::SledError(e)) => {
//!         eprintln!("Sled error: {}", e);
//!     }
//!     Err(NetabaseError::DecodeError(e)) => {
//!         eprintln!("Deserialization error: {}", e);
//!     }
//!     Err(NetabaseError::EncodeError(e)) => {
//!         eprintln!("Serialization error: {}", e);
//!     }
//!     Err(NetabaseError::Other(msg)) => {
//!         eprintln!("Other error: {}", msg);
//!     }
//! }
//! ```
//!
//! # Error Conversion
//!
//! Errors from the underlying `redb` database and `bincode` serialization library
//! are automatically converted to `NetabaseError` using the `From` trait:
//!
//! ```
//! use netabase_store::error::{NetabaseError, RedbError};
//! use netabase_store::NetabaseResult;
//!
//! fn propagate_redb_error() -> NetabaseResult<()> {
//!     // redb errors are automatically converted to NetabaseError
//!     // when using the ? operator
//!     Ok(())
//! }
//! ```

use thiserror::Error;

/// Result type alias for Netabase operations.
///
/// Most functions in this crate return `NetabaseResult<T>`, which is shorthand
/// for `Result<T, NetabaseError>`. This allows for convenient error handling
/// using the `?` operator.
///
/// # Example
///
/// ```
/// use netabase_store::NetabaseResult;
///
/// fn do_something() -> NetabaseResult<String> {
///     Ok("success".to_string())
/// }
///
/// fn caller() -> NetabaseResult<()> {
///     let result = do_something()?;
///     assert_eq!(result, "success");
///     Ok(())
/// }
/// ```
pub type NetabaseResult<T> = Result<T, NetabaseError>;

/// The main error type for Netabase operations.
///
/// This enum wraps all possible errors that can occur during database operations,
/// including errors from the underlying database backends (`redb` or `sled`) and
/// `bincode` serialization.
///
/// # Variants
///
/// - `RedbError`: Errors from the redb database backend
/// - `SledError`: Errors from the sled database backend
/// - `DecodeError`: Deserialization errors from bincode
/// - `EncodeError`: Serialization errors from bincode
/// - `Other`: Custom error messages
///
/// # Example
///
/// ```
/// use netabase_store::error::NetabaseError;
///
/// // Create a custom error
/// let error = NetabaseError::Other("Custom error message".to_string());
/// assert_eq!(error.to_string(), "Custom error message");
/// ```
#[derive(Error, Debug)]
pub enum NetabaseError {
    /// Wraps errors from the redb database
    #[error(transparent)]
    RedbError(#[from] RedbError),

    /// Wraps errors from the sled database
    #[error(transparent)]
    SledError(#[from] sled::Error),

    /// Wraps deserialization errors from bincode
    #[error(transparent)]
    DecodeError(#[from] bincode::error::DecodeError),

    /// Wraps serialization errors from bincode
    #[error(transparent)]
    EncodeError(#[from] bincode::error::EncodeError),

    /// Custom error with a message
    #[error("{0}")]
    Other(String),
}

/// Errors that can occur when interacting with the redb database.
///
/// This enum wraps all possible errors from the `redb` crate, providing
/// a unified error type for database operations.
///
/// # Variants
///
/// - `DatabaseError`: Errors during database creation or opening
/// - `TransactionError`: Errors during transaction operations
/// - `TableError`: Errors during table operations
/// - `CommitError`: Errors during transaction commit
/// - `StorageError`: Errors related to storage operations
/// - `CompactionError`: Errors during database compaction
#[derive(Error, Debug)]
pub enum RedbError {
    /// Errors from database creation or opening
    #[error(transparent)]
    DatabaseError(#[from] redb::DatabaseError),

    /// Errors from transaction operations
    #[error(transparent)]
    TransactionError(#[from] redb::TransactionError),

    /// Errors from table operations
    #[error(transparent)]
    TableError(#[from] redb::TableError),

    /// Errors from committing transactions
    #[error(transparent)]
    CommitError(#[from] redb::CommitError),

    /// Errors from storage operations
    #[error(transparent)]
    StorageError(#[from] redb::StorageError),

    /// Errors from database compaction
    #[error("Compaction error")]
    CompactionError,
}

macro_rules! impl_from_redb {
    ($($err:ty => $variant:ident),*) => {
        $(
            impl From<$err> for NetabaseError {
                fn from(err: $err) -> Self {
                    NetabaseError::RedbError(RedbError::$variant(err))
                }
            }
        )*
    };
}

impl_from_redb!(
    redb::DatabaseError => DatabaseError,
    redb::TransactionError => TransactionError,
    redb::TableError => TableError,
    redb::CommitError => CommitError,
    redb::StorageError => StorageError
);
