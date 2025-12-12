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
//!     Err(_) => {
//!         eprintln!("An error occurred");
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

    /// Configuration error (e.g., invalid TOML schema)
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Definition store is not loaded
    ///
    /// This error occurs when attempting to access a definition store that
    /// has not been loaded yet. The store needs to be loaded before use.
    #[error("Definition store not loaded: {0}")]
    StoreNotLoaded(String),

    /// Tree not found
    ///
    /// This error occurs when attempting to access a tree (table) that doesn't exist.
    #[error("Tree not found")]
    TreeNotFound,

    /// Permission denied
    ///
    /// This error occurs when a permission check fails at runtime.
    /// For compile-time permission checks, use the const generic parameter.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Definition not found
    ///
    /// This error occurs when attempting to access a definition that
    /// doesn't exist in the manager.
    #[error("Definition not found: {0}")]
    DefinitionNotFound(String),

    /// Manager error
    ///
    /// Generic error for definition manager operations.
    #[error("Manager error: {0}")]
    ManagerError(String),

    /// I/O error
    ///
    /// Wraps std::io::Error for file system operations.
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// TOML serialization error
    ///
    /// This error occurs when serializing data structures to TOML format.
    #[error(transparent)]
    TomlSerError(#[from] toml::ser::Error),

    /// TOML deserialization error
    ///
    /// This error occurs when parsing TOML files.
    #[error(transparent)]
    TomlDeError(#[from] toml::de::Error),
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
