//! Transaction trait definitions and configuration.
//!
//! This module defines the core transaction abstractions used by database backends.
//! Transactions provide ACID guarantees (where supported by the backend) and serve
//! as the primary interface for all CRUD operations.
//!
//! # Transaction Types
//!
//! - **Read Transactions**: Allow concurrent reads without blocking
//! - **Write Transactions**: Provide exclusive write access
//!
//! # Configuration
//!
//! [`TransactionConfig`] allows customization of transaction behavior:
//! - Table caching for circular relationships
//! - Cache size limits and eviction strategies
//! - Future extensibility for advanced scenarios
//!
//! # Example
//!
//! ```rust
//! use netabase_store::traits::database::transaction::TransactionConfig;
//!
//! // Default configuration
//! let config = TransactionConfig::default();
//!
//! // Optimized for circular relationships
//! let config = TransactionConfig::for_circular_relationships();
//! ```

use crate::{
    errors::NetabaseResult,
    traits::registry::{definition::NetabaseDefinition, repository::NetabaseRepository},
};

/// Configuration options for database transactions.
///
/// This struct allows customization of transaction behavior, including
/// table caching for circular relationship support and future extensibility.
///
/// # Example
///
/// ```rust
/// use netabase_store::traits::database::transaction::TransactionConfig;
///
/// let config = TransactionConfig::default();
/// // Configuration options can be customized for advanced use cases
/// ```
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct TransactionConfig {
    /// Whether to enable table handle caching.
    ///
    /// When enabled, opened table handles are cached to prevent
    /// double-opening in circular relationship scenarios (redb
    /// does not allow a table to be opened twice simultaneously).
    pub enable_table_cache: bool,

    /// Maximum number of table handles to cache.
    ///
    /// When `None`, all opened tables are cached for the transaction lifetime.
    /// When `Some(n)`, uses LRU eviction when cache exceeds `n` entries.
    pub max_cache_size: Option<usize>,

    /// Cache eviction strategy when max size is reached.
    pub cache_strategy: CacheStrategy,
}

/// Strategy for cache eviction when max size is reached.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CacheStrategy {
    /// Least Recently Used eviction (default).
    #[default]
    Lru,
    /// First In First Out eviction.
    Fifo,
    /// No eviction - return error when cache is full.
    NoEviction,
}


impl TransactionConfig {
    /// Create a new transaction config with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable table caching.
    #[inline]
    pub fn with_table_cache(mut self, enable: bool) -> Self {
        self.enable_table_cache = enable;
        self
    }

    /// Set the maximum cache size.
    #[inline]
    pub fn with_max_cache_size(mut self, size: usize) -> Self {
        self.max_cache_size = Some(size);
        self
    }

    /// Set the cache eviction strategy.
    #[inline]
    pub fn with_cache_strategy(mut self, strategy: CacheStrategy) -> Self {
        self.cache_strategy = strategy;
        self
    }

    /// Create a config optimized for circular relationships.
    ///
    /// This enables table caching with no size limit to prevent
    /// double-open errors when traversing circular references.
    pub fn for_circular_relationships() -> Self {
        Self {
            enable_table_cache: true,
            max_cache_size: None,
            cache_strategy: CacheStrategy::Lru,
        }
    }
}

/// Core transaction trait for CRUD operations on definitions.
///
/// This trait defines the interface that transactions must implement to support
/// all basic database operations. Implementations are provided by each backend
/// (redb, memory, etc.).
///
/// # Type Parameters
///
/// - `'db`: Lifetime of the database the transaction is attached to
/// - `D`: The [`NetabaseDefinition`] this transaction operates on
///
/// # Operations
///
/// - **Create**: Insert new records
/// - **Read**: Fetch records by key or predicate
/// - **Update**: Modify existing records
/// - **Delete**: Remove records
///
/// # Notes
///
/// Most users will interact with higher-level transaction types like
/// [`RedbTransaction`](crate::databases::redb::transaction::RedbTransaction)
/// rather than implementing this trait directly.
pub trait NBTransaction<'db, D: NetabaseDefinition>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    /// Type for read-only transactions.
    type ReadTransaction;
    /// Type for read-write transactions.
    type WriteTransaction;

    /// Create a new record in the database.
    ///
    /// # Arguments
    ///
    /// - `definition`: The model instance to create
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A record with the same primary key already exists
    /// - Serialization fails
    /// - Backend I/O fails
    fn create(&self, definition: &D) -> NetabaseResult<()>;

    /// Read a record by its primary key.
    ///
    /// # Arguments
    ///
    /// - `key`: The primary key to look up
    ///
    /// # Returns
    ///
    /// - `Some(D)` if a record with that key exists
    /// - `None` if no record exists
    fn read(&self, key: &D::DefKeys) -> NetabaseResult<Option<D>>;

    /// Update an existing record.
    ///
    /// # Arguments
    ///
    /// - `definition`: The updated model instance
    ///
    /// # Errors
    ///
    /// Returns an error if the record doesn't exist or update fails.
    fn update(&self, definition: &D) -> NetabaseResult<()>;

    /// Delete a record by its primary key.
    ///
    /// # Arguments
    ///
    /// - `key`: The primary key of the record to delete
    fn delete(&self, key: &D::DefKeys) -> NetabaseResult<()>;

    /// Create multiple records in a batch.
    ///
    /// This may be more efficient than calling `create()` repeatedly.
    fn create_many(&self, definitions: &[D]) -> NetabaseResult<()>;

    /// Read all records matching a predicate.
    ///
    /// # Arguments
    ///
    /// - `predicate`: Function that returns `true` for records to include
    fn read_if<F>(&self, predicate: F) -> NetabaseResult<Vec<D>>
    where
        F: Fn(&D) -> bool;

    /// Read all records in a range of primary keys.
    ///
    /// # Arguments
    ///
    /// - `range`: Range of keys to fetch (inclusive start, exclusive end)
    fn read_range(&self, range: std::ops::Range<D::DefKeys>) -> NetabaseResult<Vec<D>>;

    /// Update all records in a key range.
    ///
    /// # Arguments
    ///
    /// - `range`: Range of keys to update
    /// - `updater`: Function to apply to each record
    fn update_range<F>(&self, range: std::ops::Range<D::DefKeys>, updater: F) -> NetabaseResult<()>
    where
        F: Fn(&mut D);

    /// Update all records matching a predicate.
    ///
    /// # Arguments
    ///
    /// - `predicate`: Selects records to update
    /// - `updater`: Transformation to apply
    fn update_if<P, U>(&self, predicate: P, updater: U) -> NetabaseResult<()>
    where
        P: Fn(&D) -> bool,
        U: Fn(&mut D);

    /// Delete multiple records by their keys.
    fn delete_many(&self, keys: &[D::DefKeys]) -> NetabaseResult<()>;

    /// Delete all records matching a predicate.
    fn delete_if<F>(&self, predicate: F) -> NetabaseResult<()>
    where
        F: Fn(&D) -> bool;

    /// Delete all records in a key range.
    fn delete_range(&self, range: std::ops::Range<D::DefKeys>) -> NetabaseResult<()>;

    /// Execute a write operation within this transaction.
    ///
    /// Provides access to the underlying write transaction for low-level operations.
    fn write<F, R>(&self, f: F) -> NetabaseResult<R>
    where
        F: FnOnce(&Self::WriteTransaction) -> NetabaseResult<R>;

    /// Execute a read operation within this transaction.
    ///
    /// Provides access to the underlying read transaction for low-level operations.
    fn read_fn<F, R>(&self, f: F) -> NetabaseResult<R>
    where
        F: FnOnce(&Self::ReadTransaction) -> NetabaseResult<R>;

    /// Read a record from a different definition.
    ///
    /// Allows cross-definition queries within the same database.
    fn read_related<OD>(&self, key: &OD::DefKeys) -> NetabaseResult<Option<OD>>
    where
        OD: NetabaseDefinition,
        <OD as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug;

    /// Check if a definition is accessible from this transaction.
    ///
    /// Used for runtime permission checks in repository-scoped contexts.
    fn can_access_definition<OD>(&self) -> bool
    where
        OD: NetabaseDefinition,
        <OD as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug;
}

/// Repository-scoped transaction trait for type-safe cross-definition access.
///
/// This trait extends the basic transaction with repository awareness,
/// ensuring all accessed definitions belong to the same repository context.
pub trait NBRepositoryTransaction<'db, R: NetabaseRepository> {
    /// The underlying transaction type.
    type Transaction;

    /// Get the transaction configuration.
    fn config(&self) -> &TransactionConfig;

    /// Access a definition within this repository context.
    ///
    /// This method ensures compile-time safety that the accessed
    /// definition belongs to the same repository.
    fn with_definition<D, F, T>(&self, f: F) -> NetabaseResult<T>
    where
        D: NetabaseDefinition + crate::traits::registry::repository::InRepository<R>,
        D::Discriminant: 'static + std::fmt::Debug,
        F: FnOnce(&Self::Transaction) -> NetabaseResult<T>;

    /// Check if a definition is accessible within this repository.
    fn can_access<D>(&self) -> bool
    where
        D: NetabaseDefinition + crate::traits::registry::repository::InRepository<R>,
        D::Discriminant: 'static + std::fmt::Debug;
}
