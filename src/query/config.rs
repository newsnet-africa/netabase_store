//! Query configuration builder.

use std::ops::RangeFull;

use super::options::{FetchOptions, Pagination, QueryMode};

/// Configuration for database queries.
///
/// Provides a builder API for configuring query behavior including
/// pagination, fetch modes, and iteration order.
///
/// # Type Parameters
///
/// - `R`: Range type (defaults to `RangeFull` for unbounded queries)
///
/// # Example
///
/// ```
/// use netabase_store::query::{QueryConfig, QueryMode};
///
/// let config = QueryConfig::default()
///     .with_limit(10)
///     .with_offset(20)
///     .reversed();
/// ```
#[derive(Debug, Clone)]
pub struct QueryConfig<R = RangeFull> {
    /// Query mode (Fetch or Count).
    pub mode: QueryMode,
    /// Range of keys to query.
    pub range: R,
    /// Pagination settings.
    pub pagination: Pagination,
    /// Fetch options.
    pub fetch_options: FetchOptions,
    /// Whether to reverse iteration order.
    pub reversed: bool,
}

impl Default for QueryConfig<RangeFull> {
    fn default() -> Self {
        Self {
            mode: QueryMode::default(),
            range: RangeFull,
            pagination: Pagination::default(),
            fetch_options: FetchOptions::default(),
            reversed: false,
        }
    }
}

impl<R> QueryConfig<R> {
    /// Create a new query config with the specified range.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryConfig;
    ///
    /// let config = QueryConfig::new(0..100);
    /// assert_eq!(config.range, 0..100);
    /// ```
    pub fn new(range: R) -> Self {
        Self {
            mode: QueryMode::default(),
            range,
            pagination: Pagination::default(),
            fetch_options: FetchOptions::default(),
            reversed: false,
        }
    }

    /// Set the mode to count only, without fetching data.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::{QueryConfig, QueryMode};
    ///
    /// let config = QueryConfig::default().count_only();
    /// assert_eq!(config.mode, QueryMode::Count);
    /// ```
    pub fn count_only(mut self) -> Self {
        self.mode = QueryMode::Count;
        self
    }

    /// Reverse the iteration order.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryConfig;
    ///
    /// let config = QueryConfig::default().reversed();
    /// assert!(config.reversed);
    /// ```
    pub fn reversed(mut self) -> Self {
        self.reversed = true;
        self
    }

    /// Set a limit on the number of results.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryConfig;
    ///
    /// let config = QueryConfig::default().with_limit(10);
    /// assert_eq!(config.pagination.limit, Some(10));
    /// ```
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.pagination.limit = Some(limit);
        self
    }

    /// Set an offset for pagination.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryConfig;
    ///
    /// let config = QueryConfig::default().with_offset(5);
    /// assert_eq!(config.pagination.offset, Some(5));
    /// ```
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.pagination.offset = Some(offset);
        self
    }

    /// Control whether blobs should be included.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryConfig;
    ///
    /// let config = QueryConfig::default().with_blobs(false);
    /// assert!(!config.fetch_options.include_blobs);
    /// ```
    pub fn with_blobs(mut self, include: bool) -> Self {
        self.fetch_options.include_blobs = include;
        self
    }

    /// Exclude blobs from the query results.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryConfig;
    ///
    /// let config = QueryConfig::default().no_blobs();
    /// assert!(!config.fetch_options.include_blobs);
    /// ```
    pub fn no_blobs(mut self) -> Self {
        self.fetch_options.include_blobs = false;
        self
    }

    /// Set the hydration depth for related models.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryConfig;
    ///
    /// let config = QueryConfig::default().with_hydration(2);
    /// assert_eq!(config.fetch_options.hydration_depth, 2);
    /// ```
    pub fn with_hydration(mut self, depth: usize) -> Self {
        self.fetch_options.hydration_depth = depth;
        self
    }

    /// Disable hydration of related models.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryConfig;
    ///
    /// let config = QueryConfig::default().no_hydration();
    /// assert_eq!(config.fetch_options.hydration_depth, 0);
    /// ```
    pub fn no_hydration(mut self) -> Self {
        self.fetch_options.hydration_depth = 0;
        self
    }

    /// Specify which relations to fetch.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryConfig;
    ///
    /// let config = QueryConfig::default()
    ///     .with_relations(vec!["posts".to_string(), "comments".to_string()]);
    /// assert_eq!(config.fetch_options.relations.len(), 2);
    /// ```
    pub fn with_relations(mut self, relations: Vec<String>) -> Self {
        self.fetch_options.relations = relations;
        self
    }

    /// Change the range of this query config.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryConfig;
    ///
    /// let config = QueryConfig::default()
    ///     .with_limit(5)
    ///     .with_range(0..100);
    /// assert_eq!(config.range, 0..100);
    /// assert_eq!(config.pagination.limit, Some(5));
    /// ```
    pub fn with_range<NewR>(self, range: NewR) -> QueryConfig<NewR> {
        QueryConfig {
            mode: self.mode,
            range,
            pagination: self.pagination,
            fetch_options: self.fetch_options,
            reversed: self.reversed,
        }
    }
}

/// Factory methods that return concrete types (no generic inference needed).
impl QueryConfig {
    /// Create a simple config for full table scan.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryConfig;
    ///
    /// let config = QueryConfig::all();
    /// // Returns a config that fetches all records
    /// ```
    pub fn all() -> QueryConfig<std::ops::RangeFull> {
        QueryConfig::<std::ops::RangeFull>::default()
    }

    /// Create a config to dump all records for inspection.
    /// Includes blobs and disables hydration for raw data access.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryConfig;
    ///
    /// let config = QueryConfig::dump_all();
    /// assert!(config.fetch_options.include_blobs);
    /// assert_eq!(config.fetch_options.hydration_depth, 0);
    /// ```
    pub fn dump_all() -> QueryConfig<std::ops::RangeFull> {
        QueryConfig::<std::ops::RangeFull>::default()
            .with_blobs(true)
            .with_hydration(0)
    }

    /// Create a config to fetch just the first record.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryConfig;
    ///
    /// let config = QueryConfig::first();
    /// assert_eq!(config.pagination.limit, Some(1));
    /// ```
    pub fn first() -> QueryConfig<std::ops::RangeFull> {
        QueryConfig::<std::ops::RangeFull>::default().with_limit(1)
    }

    /// Create a config for inspecting a specific range.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryConfig;
    ///
    /// let config = QueryConfig::inspect_range(0u64..10u64);
    /// // Fetches records in the range with all data
    /// ```
    pub fn inspect_range<NewR>(range: NewR) -> QueryConfig<NewR> {
        QueryConfig::<NewR>::new(range)
            .with_blobs(true)
            .with_hydration(0)
    }
}
