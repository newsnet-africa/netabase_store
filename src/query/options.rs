//! Query mode and options types.

/// Query execution mode.
///
/// Determines whether to fetch data or just count records.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum QueryMode {
    /// Fetch and return the actual data (default)
    #[default]
    Fetch,
    /// Only count matching records without fetching data
    Count,
}

/// Pagination settings for queries.
#[derive(Debug, Clone, Default)]
pub struct Pagination {
    /// Maximum number of records to return.
    pub limit: Option<usize>,
    /// Number of records to skip.
    pub offset: Option<usize>,
}

/// Options controlling how data is fetched.
#[derive(Debug, Clone)]
pub struct FetchOptions {
    /// Whether to include blob data in the results.
    pub include_blobs: bool,
    /// Depth of hydration for related models (0 = no hydration).
    pub hydration_depth: usize,
    /// Specific relations to fetch (empty = all).
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
