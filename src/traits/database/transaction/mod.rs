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
//! ```rust,no_run
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
#[derive(Debug, Clone, Default)]
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
// TODO: Implement Builder pattern here
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

// NOTE: The NBTransaction trait was removed as it required overly specialized
// trait bounds for each backend implementation. Users should interact with
// backend-specific transaction types directly (e.g., RedbTransaction).

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
