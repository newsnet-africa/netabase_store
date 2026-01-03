use crate::{
    errors::NetabaseResult,
    traits::registery::{definition::NetabaseDefinition, repository::NetabaseRepository},
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

impl Default for TransactionConfig {
    fn default() -> Self {
        Self {
            enable_table_cache: false,
            max_cache_size: None,
            cache_strategy: CacheStrategy::default(),
        }
    }
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

pub trait NBTransaction<'db, D: NetabaseDefinition>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    type ReadTransaction;
    type WriteTransaction;

    fn create(&self, definition: &D) -> NetabaseResult<()>;

    fn read(&self, key: &D::DefKeys) -> NetabaseResult<Option<D>>;

    fn update(&self, definition: &D) -> NetabaseResult<()>;

    fn delete(&self, key: &D::DefKeys) -> NetabaseResult<()>;

    fn create_many(&self, definitions: &[D]) -> NetabaseResult<()>;

    fn read_if<F>(&self, predicate: F) -> NetabaseResult<Vec<D>>
    where
        F: Fn(&D) -> bool;

    fn read_range(&self, range: std::ops::Range<D::DefKeys>) -> NetabaseResult<Vec<D>>;

    fn update_range<F>(&self, range: std::ops::Range<D::DefKeys>, updater: F) -> NetabaseResult<()>
    where
        F: Fn(&mut D);

    fn update_if<P, U>(&self, predicate: P, updater: U) -> NetabaseResult<()>
    where
        P: Fn(&D) -> bool,
        U: Fn(&mut D);

    fn delete_many(&self, keys: &[D::DefKeys]) -> NetabaseResult<()>;

    fn delete_if<F>(&self, predicate: F) -> NetabaseResult<()>
    where
        F: Fn(&D) -> bool;

    fn delete_range(&self, range: std::ops::Range<D::DefKeys>) -> NetabaseResult<()>;

    fn write<F, R>(&self, f: F) -> NetabaseResult<R>
    where
        F: FnOnce(&Self::WriteTransaction) -> NetabaseResult<R>;

    fn read_fn<F, R>(&self, f: F) -> NetabaseResult<R>
    where
        F: FnOnce(&Self::ReadTransaction) -> NetabaseResult<R>;

    // Cross-definition relational operations
    fn read_related<OD>(&self, key: &OD::DefKeys) -> NetabaseResult<Option<OD>>
    where
        OD: NetabaseDefinition,
        <OD as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug;

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
        D: NetabaseDefinition + crate::traits::registery::repository::InRepository<R>,
        D::Discriminant: 'static + std::fmt::Debug,
        F: FnOnce(&Self::Transaction) -> NetabaseResult<T>;

    /// Check if a definition is accessible within this repository.
    fn can_access<D>(&self) -> bool
    where
        D: NetabaseDefinition + crate::traits::registery::repository::InRepository<R>,
        D::Discriminant: 'static + std::fmt::Debug;
}
