use std::ops::RangeFull;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum QueryMode {
    #[default]
    Fetch,
    Count,
}

#[derive(Debug, Clone)]
pub struct QueryConfig<R = RangeFull> {
    pub mode: QueryMode,
    pub range: R,
    pub pagination: Pagination,
    pub fetch_options: FetchOptions,
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

/// Factory methods that return concrete types (no generic inference needed)
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

#[derive(Debug, Clone, Default)]
pub struct Pagination {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct FetchOptions {
    pub include_blobs: bool,
    pub hydration_depth: usize,
    pub relations: Vec<String>,
}

impl Default for FetchOptions {
    fn default() -> Self {
        Self {
            include_blobs: true,
            hydration_depth: 0,
            relations: Vec::new(),
        }
    }
}

/// Result of a query operation.
///
/// Can represent a single value, multiple values, or a count.
#[derive(Debug, Clone)]
pub enum QueryResult<T> {
    Single(Option<T>),
    Multiple(Vec<T>),
    Count(u64),
}

impl<T> QueryResult<T> {
    /// Convert the result into a vector.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryResult;
    ///
    /// let single = QueryResult::Single(Some(42));
    /// assert_eq!(single.into_vec(), vec![42]);
    ///
    /// let multiple = QueryResult::Multiple(vec![1, 2, 3]);
    /// assert_eq!(multiple.into_vec(), vec![1, 2, 3]);
    ///
    /// let count: QueryResult<i32> = QueryResult::Count(5);
    /// assert_eq!(count.into_vec(), Vec::<i32>::new());
    /// ```
    pub fn into_vec(self) -> Vec<T> {
        match self {
            QueryResult::Multiple(vec) => vec,
            QueryResult::Single(Some(item)) => vec![item],
            _ => Vec::new(),
        }
    }

    /// Get the count if this is a count result.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryResult;
    ///
    /// let count: QueryResult<i32> = QueryResult::Count(42);
    /// assert_eq!(count.count(), Some(42));
    ///
    /// let single = QueryResult::Single(Some(1));
    /// assert_eq!(single.count(), None);
    /// ```
    pub fn count(&self) -> Option<u64> {
        match self {
            QueryResult::Count(c) => Some(*c),
            _ => None,
        }
    }

    /// Check if the result is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryResult;
    ///
    /// let empty: QueryResult<i32> = QueryResult::Single(None);
    /// assert!(empty.is_empty());
    ///
    /// let not_empty = QueryResult::Single(Some(42));
    /// assert!(!not_empty.is_empty());
    ///
    /// let multiple_empty: QueryResult<i32> = QueryResult::Multiple(vec![]);
    /// assert!(multiple_empty.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        match self {
            QueryResult::Single(None) => true,
            QueryResult::Multiple(vec) => vec.is_empty(),
            QueryResult::Count(0) => true,
            _ => false,
        }
    }

    /// Get the number of items in the result.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryResult;
    ///
    /// let single = QueryResult::Single(Some(42));
    /// assert_eq!(single.len(), 1);
    ///
    /// let multiple = QueryResult::Multiple(vec![1, 2, 3]);
    /// assert_eq!(multiple.len(), 3);
    ///
    /// let count: QueryResult<i32> = QueryResult::Count(100);
    /// assert_eq!(count.len(), 100);
    /// ```
    pub fn len(&self) -> usize {
        match self {
            QueryResult::Single(Some(_)) => 1,
            QueryResult::Single(None) => 0,
            QueryResult::Multiple(vec) => vec.len(),
            QueryResult::Count(c) => *c as usize,
        }
    }

    /// Unwrap a single result, panicking if None.
    ///
    /// # Panics
    ///
    /// Panics if the result is not a Single variant or if it contains None.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryResult;
    ///
    /// let result = QueryResult::Single(Some(42));
    /// assert_eq!(result.unwrap_single(), 42);
    /// ```
    pub fn unwrap_single(self) -> T {
        match self {
            QueryResult::Single(Some(val)) => val,
            QueryResult::Single(None) => {
                panic!("called `QueryResult::unwrap_single()` on a `None` value")
            }
            _ => panic!("called `QueryResult::unwrap_single()` on a non-Single variant"),
        }
    }

    /// Unwrap a single result with a custom panic message.
    ///
    /// # Panics
    ///
    /// Panics with the given message if the result is not a Single variant or contains None.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryResult;
    ///
    /// let result = QueryResult::Single(Some(42));
    /// assert_eq!(result.expect_single("should have value"), 42);
    /// ```
    pub fn expect_single(self, msg: &str) -> T {
        match self {
            QueryResult::Single(Some(val)) => val,
            _ => panic!("{}", msg),
        }
    }

    /// Get a reference to the single value if present.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryResult;
    ///
    /// let result = QueryResult::Single(Some(42));
    /// assert_eq!(result.as_single(), Some(&42));
    ///
    /// let empty: QueryResult<i32> = QueryResult::Single(None);
    /// assert_eq!(empty.as_single(), None);
    /// ```
    pub fn as_single(&self) -> Option<&T> {
        match self {
            QueryResult::Single(Some(val)) => Some(val),
            _ => None,
        }
    }

    /// Get a reference to the multiple values if present.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryResult;
    ///
    /// let result = QueryResult::Multiple(vec![1, 2, 3]);
    /// assert_eq!(result.as_multiple(), Some(&vec![1, 2, 3]));
    ///
    /// let single = QueryResult::Single(Some(42));
    /// assert_eq!(single.as_multiple(), None);
    /// ```
    pub fn as_multiple(&self) -> Option<&Vec<T>> {
        match self {
            QueryResult::Multiple(vec) => Some(vec),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_config_defaults() {
        let config = QueryConfig::default();
        assert_eq!(config.mode, QueryMode::Fetch);
        assert_eq!(config.pagination.limit, None);
        assert_eq!(config.pagination.offset, None);
        assert!(config.fetch_options.include_blobs);
        assert_eq!(config.fetch_options.hydration_depth, 0);
        assert!(!config.reversed);
    }

    #[test]
    fn test_query_config_builder() {
        let config = QueryConfig::default()
            .count_only()
            .with_limit(10)
            .with_offset(5)
            .with_blobs(false)
            .with_hydration(2)
            .reversed();

        assert_eq!(config.mode, QueryMode::Count);
        assert_eq!(config.pagination.limit, Some(10));
        assert_eq!(config.pagination.offset, Some(5));
        assert!(!config.fetch_options.include_blobs);
        assert_eq!(config.fetch_options.hydration_depth, 2);
        assert!(config.reversed);
    }

    #[test]
    fn test_query_config_helpers() {
        let config = QueryConfig::default().no_blobs().no_hydration();
        assert!(!config.fetch_options.include_blobs);
        assert_eq!(config.fetch_options.hydration_depth, 0);
    }

    #[test]
    fn test_query_config_range_change() {
        let config = QueryConfig::default().with_limit(5);
        // Default range is RangeFull
        let config_range = config.with_range(0..10);

        assert_eq!(config_range.range, 0..10);
        assert_eq!(config_range.pagination.limit, Some(5));
    }
}
