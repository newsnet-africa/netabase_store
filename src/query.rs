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
    pub fn new(range: R) -> Self {
        Self {
            mode: QueryMode::default(),
            range,
            pagination: Pagination::default(),
            fetch_options: FetchOptions::default(),
            reversed: false,
        }
    }

    pub fn count_only(mut self) -> Self {
        self.mode = QueryMode::Count;
        self
    }
    
    pub fn reversed(mut self) -> Self {
        self.reversed = true;
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.pagination.limit = Some(limit);
        self
    }

    pub fn with_offset(mut self, offset: usize) -> Self {
        self.pagination.offset = Some(offset);
        self
    }
    
    pub fn with_blobs(mut self, include: bool) -> Self {
        self.fetch_options.include_blobs = include;
        self
    }

    pub fn no_blobs(mut self) -> Self {
        self.fetch_options.include_blobs = false;
        self
    }

    pub fn with_hydration(mut self, depth: usize) -> Self {
        self.fetch_options.hydration_depth = depth;
        self
    }
    
    pub fn no_hydration(mut self) -> Self {
        self.fetch_options.hydration_depth = 0;
        self
    }

    pub fn with_relations(mut self, relations: Vec<String>) -> Self {
        self.fetch_options.relations = relations;
        self
    }

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

#[derive(Debug, Clone)]
pub enum QueryResult<T> {
    Single(Option<T>),
    Multiple(Vec<T>),
    Count(u64),
}

impl<T> QueryResult<T> {
    pub fn into_vec(self) -> Vec<T> {
        match self {
            QueryResult::Multiple(vec) => vec,
            QueryResult::Single(Some(item)) => vec![item],
            _ => Vec::new(),
        }
    }
    
    pub fn count(&self) -> Option<u64> {
         match self {
            QueryResult::Count(c) => Some(*c),
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
