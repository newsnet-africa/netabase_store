//! Unified configuration system for database operations.
//!
//! This module provides a consolidated configuration API that controls all aspects
//! of database queries and operations, including pagination, hydration, blob handling,
//! and subscriptions.
//!
//! # Configuration Hierarchy
//!
//! Configurations can be set at three levels:
//! 1. **Store-level (Definition)**: Defaults for all tables in a definition
//! 2. **Table-level (Model)**: Defaults for a specific model/table  
//! 3. **Query-level**: Per-operation override
//!
//! Query-level configs inherit from table-level, which inherit from store-level.
//!
//! # Example
//!
//! ```rust
//! use netabase_store::config::QueryConfig;
//!
//! // Simple pagination
//! let config = QueryConfig::new()
//!     .with_limit(50)
//!     .with_offset(100);
//!
//! // Complex configuration
//! let config = QueryConfig::new()
//!     .with_limit(25)
//!     .with_hydration(2)
//!     .no_blobs()
//!     .reversed();
//! ```

use serde::{Deserialize, Serialize};
use std::ops::RangeFull;

pub mod defaults;

// Re-export for convenience
pub use defaults::{ConfigDefaults, DefaultsBuilder};

/// Unified configuration for all database operations.
///
/// `QueryConfig` consolidates pagination, hydration, blob handling, subscriptions,
/// and query modes into a single, consistent API. It replaces the previous
/// `CrudOptions` and `QueryConfig` types.
///
/// # Configuration Hierarchy
///
/// Configurations cascade through three levels:
/// - **Store defaults**: Set via `RedbStore::with_defaults()`
/// - **Table defaults**: Set via `RedbStore::with_table_defaults()`  
/// - **Query overrides**: Provided per-operation
///
/// # Examples
///
/// ```rust
/// use netabase_store::config::QueryConfig;
///
/// // Default configuration
/// let config = QueryConfig::new();
///
/// // Paginated query
/// let config = QueryConfig::new()
///     .with_limit(25)
///     .with_offset(50);
///
/// // Complex query with hydration
/// let config = QueryConfig::new()
///     .with_limit(100)
///     .with_hydration(2)
///     .no_blobs()
///     .with_subscriptions(true);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryConfig<R = RangeFull> {
    /// Query execution mode (Fetch or Count).
    pub mode: QueryMode,
    
    /// Range of keys to query.
    #[serde(skip)]
    pub range: R,
    
    /// Pagination settings.
    pub pagination: PaginationConfig,
    
    /// Hydration settings for relational data.
    pub hydration: HydrationConfig,
    
    /// Blob data handling settings.
    pub blob: BlobConfig,
    
    /// Subscription/notification settings.
    pub subscription: SubscriptionConfig,
    
    /// Whether to reverse iteration order.
    pub reversed: bool,
}

impl Default for QueryConfig<RangeFull> {
    fn default() -> Self {
        Self {
            mode: QueryMode::default(),
            range: RangeFull,
            pagination: PaginationConfig::default(),
            hydration: HydrationConfig::default(),
            blob: BlobConfig::default(),
            subscription: SubscriptionConfig::default(),
            reversed: false,
        }
    }
}

impl<R> QueryConfig<R> {
    /// Creates a new `QueryConfig` with the specified range.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::config::QueryConfig;
    ///
    /// let config = QueryConfig::new_with_range(0..100);
    /// assert_eq!(config.range, 0..100);
    /// ```
    pub fn new_with_range(range: R) -> Self {
        Self {
            mode: QueryMode::default(),
            range,
            pagination: PaginationConfig::default(),
            hydration: HydrationConfig::default(),
            blob: BlobConfig::default(),
            subscription: SubscriptionConfig::default(),
            reversed: false,
        }
    }

    /// Change the range of this query config.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::config::QueryConfig;
    ///
    /// let options = QueryConfig::new()
    ///     .with_limit(5)
    ///     .with_range(0..100);
    /// ```
    pub fn with_range<NewR>(self, range: NewR) -> QueryConfig<NewR> {
        QueryConfig {
            mode: self.mode,
            range,
            pagination: self.pagination,
            hydration: self.hydration,
            blob: self.blob,
            subscription: self.subscription,
            reversed: self.reversed,
        }
    }

    // === Mode Configuration ===

    /// Set the mode to count only, without fetching data.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::config::{QueryConfig, QueryMode};
    ///
    /// let options = QueryConfig::new().count_only();
    /// assert_eq!(options.mode, QueryMode::Count);
    /// ```
    pub fn count_only(mut self) -> Self {
        self.mode = QueryMode::Count;
        self
    }

    /// Reverse the iteration order.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::config::QueryConfig;
    ///
    /// let options = QueryConfig::new().reversed();
    /// assert!(options.reversed);
    /// ```
    pub fn reversed(mut self) -> Self {
        self.reversed = true;
        self
    }

    // === Pagination Configuration ===

    /// Sets the maximum number of records to return.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::config::QueryConfig;
    ///
    /// let options = QueryConfig::new().with_limit(100);
    /// assert_eq!(options.pagination.limit, Some(100));
    /// ```
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.pagination.limit = Some(limit);
        self
    }

    /// Sets the number of records to skip before returning results.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::config::QueryConfig;
    ///
    /// let options = QueryConfig::new()
    ///     .with_offset(50)
    ///     .with_limit(25);
    /// assert_eq!(options.pagination.offset, Some(50));
    /// ```
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.pagination.offset = Some(offset);
        self
    }

    // === Hydration Configuration ===

    /// Set the maximum depth to follow relational links.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::config::QueryConfig;
    ///
    /// let options = QueryConfig::new().with_hydration(2);
    /// assert_eq!(options.hydration.depth, 2);
    /// ```
    pub fn with_hydration(mut self, depth: usize) -> Self {
        self.hydration.depth = depth;
        self.hydration.enabled = depth > 0;
        self
    }

    /// Disable hydration of related models.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::config::QueryConfig;
    ///
    /// let options = QueryConfig::new().no_hydration();
    /// assert_eq!(options.hydration.depth, 0);
    /// assert!(!options.hydration.enabled);
    /// ```
    pub fn no_hydration(mut self) -> Self {
        self.hydration.depth = 0;
        self.hydration.enabled = false;
        self
    }

    /// Specify which relations to fetch.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::config::QueryConfig;
    ///
    /// let options = QueryConfig::new()
    ///     .with_relations(vec!["posts".to_string(), "comments".to_string()]);
    /// assert_eq!(options.hydration.relations.len(), 2);
    /// ```
    pub fn with_relations(mut self, relations: Vec<String>) -> Self {
        self.hydration.relations = relations;
        self
    }

    // === Blob Configuration ===

    /// Control whether blobs should be included in results.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::config::QueryConfig;
    ///
    /// let options = QueryConfig::new().with_blobs(false);
    /// assert!(options.blob.strip_blobs);
    /// ```
    pub fn with_blobs(mut self, include: bool) -> Self {
        self.blob.strip_blobs = !include;
        self
    }

    /// Exclude blobs from the query results.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::config::QueryConfig;
    ///
    /// let options = QueryConfig::new().no_blobs();
    /// assert!(options.blob.strip_blobs);
    /// ```
    pub fn no_blobs(mut self) -> Self {
        self.blob.strip_blobs = true;
        self
    }

    // === Subscription Configuration ===

    /// Control whether operations trigger subscription notifications.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::config::QueryConfig;
    ///
    /// let options = QueryConfig::new().with_subscriptions(true);
    /// assert!(options.subscription.notify);
    /// ```
    pub fn with_subscriptions(mut self, notify: bool) -> Self {
        self.subscription.notify = notify;
        self
    }

    // === Merge Configuration ===

    /// Merge this configuration with defaults, preferring explicitly set values.
    ///
    /// This is used internally to apply store-level or table-level defaults.
    pub fn merge_with_defaults(self, defaults: &QueryConfig<R>) -> Self
    where
        R: Clone,
    {
        Self {
            mode: self.mode,
            range: self.range,
            pagination: self.pagination.merge_with(&defaults.pagination),
            hydration: self.hydration.merge_with(&defaults.hydration),
            blob: self.blob.merge_with(&defaults.blob),
            subscription: self.subscription.merge_with(&defaults.subscription),
            reversed: self.reversed,
        }
    }
}

// Factory methods
impl QueryConfig {
    /// Creates a new `QueryConfig` with default settings.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::config::QueryConfig;
    ///
    /// let options = QueryConfig::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a simple config for full table scan.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::config::QueryConfig;
    ///
    /// let options = QueryConfig::all();
    /// ```
    pub fn all() -> QueryConfig<RangeFull> {
        QueryConfig::default()
    }

    /// Create a config to dump all records for inspection.
    ///
    /// Includes blobs and disables hydration for raw data access.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::config::QueryConfig;
    ///
    /// let options = QueryConfig::dump_all();
    /// assert!(!options.blob.strip_blobs);
    /// assert_eq!(options.hydration.depth, 0);
    /// ```
    pub fn dump_all() -> QueryConfig<RangeFull> {
        QueryConfig::default()
            .with_blobs(true)
            .with_hydration(0)
    }

    /// Create a config to fetch just the first record.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::config::QueryConfig;
    ///
    /// let options = QueryConfig::first();
    /// assert_eq!(options.pagination.limit, Some(1));
    /// ```
    pub fn first() -> QueryConfig<RangeFull> {
        QueryConfig::default().with_limit(1)
    }

    /// Create a config for inspecting a specific range.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::config::QueryConfig;
    ///
    /// let options = QueryConfig::inspect_range(0u64..10u64);
    /// ```
    pub fn inspect_range<NewR>(range: NewR) -> QueryConfig<NewR> {
        QueryConfig::new_with_range(range)
            .with_blobs(true)
            .with_hydration(0)
    }
}

// === Sub-configuration types ===

/// Query execution mode.
///
/// Determines whether to fetch data or just count records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum QueryMode {
    /// Fetch and return the actual data (default).
    #[default]
    Fetch,
    /// Only count matching records without fetching data.
    Count,
}

/// Pagination configuration.
///
/// Controls how many records to return and where to start.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PaginationConfig {
    /// Maximum number of records to return. `None` means unlimited.
    pub limit: Option<usize>,
    /// Number of records to skip before returning results.
    pub offset: Option<usize>,
}

impl PaginationConfig {
    fn merge_with(&self, defaults: &Self) -> Self {
        Self {
            limit: self.limit.or(defaults.limit),
            offset: self.offset.or(defaults.offset),
        }
    }
}

/// Hydration configuration for relational data.
///
/// Controls how deeply to follow relational links when loading data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HydrationConfig {
    /// Whether hydration is enabled at all.
    pub enabled: bool,
    /// Maximum depth to follow relational links (0 = no hydration).
    pub depth: usize,
    /// Specific relations to fetch (empty = all).
    pub relations: Vec<String>,
}

impl Default for HydrationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            depth: 0,
            relations: Vec::new(),
        }
    }
}

impl HydrationConfig {
    fn merge_with(&self, defaults: &Self) -> Self {
        Self {
            enabled: self.enabled || defaults.enabled,
            depth: if self.depth > 0 { self.depth } else { defaults.depth },
            relations: if !self.relations.is_empty() {
                self.relations.clone()
            } else {
                defaults.relations.clone()
            },
        }
    }
}

/// Blob data handling configuration.
///
/// Controls whether large blob fields are loaded or stripped for performance.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlobConfig {
    /// If `true`, blob data is not loaded, reducing memory usage.
    pub strip_blobs: bool,
}

impl BlobConfig {
    fn merge_with(&self, defaults: &Self) -> Self {
        Self {
            strip_blobs: self.strip_blobs || defaults.strip_blobs,
        }
    }
}

/// Subscription/notification configuration.
///
/// Controls whether operations trigger subscription events.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubscriptionConfig {
    /// If `true`, mutations trigger subscription notifications.
    pub notify: bool,
}

impl SubscriptionConfig {
    fn merge_with(&self, defaults: &Self) -> Self {
        Self {
            notify: self.notify || defaults.notify,
        }
    }
}

// === Type alias for convenience ===

/// Alias for `QueryConfig` used in CRUD operation contexts.
///
/// This is the same type as `QueryConfig`, provided for clarity when
/// the configuration is being used for CRUD operations specifically.
pub type CrudOptions = QueryConfig;
