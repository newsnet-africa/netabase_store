//! Database store trait for backend abstraction.
//!
//! This module defines the `NBStore` trait, which provides the primary interface
//! for database backends. Implementations provide database lifecycle management
//! and transaction creation.
//!
//! # Trait Overview
//!
//! The `NBStore` trait abstracts over different storage backends, allowing
//! applications to be backend-agnostic. Currently implemented by:
//!
//! - [`RedbStore`](crate::databases::redb::RedbStore) - Production backend using redb
//! - [`MemoryStore`](crate::databases::memory::MemoryStore) - In-memory backend for testing
//!
//! # Example
//!
//! ```rust,no_run
//! use netabase_store::prelude::*;
//! use netabase_store::traits::database::store::NBStore;
//! # use serde::{Serialize, Deserialize};
//! # #[netabase_macros::netabase_definition(MyApp)]
//! # mod models {
//! #     use super::*;
//! #     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
//! #     pub struct User { #[primary_key] pub id: String }
//! # }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use models::*;
//!
//! // Create store - any backend implementing NBStore
//! let (store, _temp) = RedbStore::<MyApp>::new_temporary()?;
//! # Ok(())
//! # }
//! ```

use std::path::Path;
use crate::traits::registry::definition::NetabaseDefinition;
use crate::errors::NetabaseResult;

/// Core trait for database store backends.
///
/// This trait defines the interface that all database backends must implement.
/// It provides methods for creating and managing database instances, as well as
/// beginning read and write transactions.
///
/// # Type Parameters
///
/// - `D`: The [`NetabaseDefinition`] containing all models in this database
///
/// # Implementors
///
/// - [`RedbStore<D>`](crate::databases::redb::RedbStore) - Persistent embedded database
/// - [`MemoryStore<D>`](crate::databases::memory::MemoryStore) - In-memory for testing
///
/// # Example
///
/// Implementing a custom backend:
///
/// ```rust,no_run
/// use netabase_store::traits::database::store::NBStore;
/// use netabase_store::traits::registry::definition::NetabaseDefinition;
///
/// struct MyCustomStore<D: NetabaseDefinition> {
///     // Your implementation
/// }
///
/// impl<D: NetabaseDefinition> NBStore<D> for MyCustomStore<D>
/// where
///     D::Discriminant: 'static + std::fmt::Debug,
/// {
///     fn new<P: AsRef<Path>>(path: P) -> NetabaseResult<Self> {
///         // Initialize your backend
///     }
///     
///     fn execute_transaction<F: Fn()>(f: F) {
///         // Execute in transaction context
///     }
/// }
/// ```
pub trait NBStore<D: NetabaseDefinition>
where
    D::Discriminant: 'static + std::fmt::Debug,
{
    /// Create a new database store at the specified path.
    ///
    /// Opens or creates a database at the given file path. If the database doesn't
    /// exist, it will be created with the schema defined by definition `D`.
    ///
    /// # Arguments
    ///
    /// - `path`: File system path for the database
    ///
    /// # Returns
    ///
    /// A new store instance, or an error if creation fails
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use netabase_store::databases::redb::RedbStore;
    /// use netabase_store::traits::database::store::NBStore;
    ///
    /// let store = RedbStore::<MyApp>::new("./my_database.redb")?;
    /// ```
    fn new<P: AsRef<Path>>(path: P) -> NetabaseResult<Self>
    where
        Self: Sized,
        D::TreeNames: Default;

    /// Execute a function within a transaction context.
    ///
    /// This method provides a way to execute operations within a transactional
    /// boundary. The exact semantics depend on the backend implementation.
    ///
    /// # Arguments
    ///
    /// - `f`: Closure to execute within transaction context
    ///
    /// # Note
    ///
    /// This is a low-level method. Most users should use `begin_read()` and
    /// `begin_write()` instead for explicit transaction management.
    fn execute_transaction<F: Fn()>(f: F);
}
