//! Configuration options for database operations.
//!
//! This module provides configuration structs that control the behavior of CRUD operations,
//! including pagination, hydration, blob handling, and subscriptions.
//!
//! # Usage Example
//!
//! ```rust,ignore
//! let options = CrudOptions::new()
//!     .with_limit(50)
//!     .with_offset(100);
//! ```
//!
//! # Configuration Domains
//!
//! - **List Config**: Controls pagination (limit, offset)
//! - **Hydration Config**: Controls relational data loading (depth, fetch_relations)
//! - **Blob Config**: Controls blob data handling (strip_blobs for performance)
//! - **Subscription Config**: Controls pub/sub notifications

use serde::{Deserialize, Serialize};

/// Configuration options for CRUD operations.
///
/// Provides fine-grained control over how data is fetched, paginated, and processed.
/// All fields have sensible defaults and can be overridden using builder methods.
///
/// # Examples
///
/// ```
/// use netabase_store::databases::redb::transaction::options::CrudOptions;
///
/// // Default configuration
/// let options = CrudOptions::new();
///
/// // With pagination
/// let paginated = CrudOptions::new()
///     .with_limit(25)
///     .with_offset(50);
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrudOptions {
    pub list: ListConfig,
    pub hydration: HydrationConfig,
    pub blob: BlobConfig,
    pub subscription: SubscriptionConfig,
}

impl CrudOptions {
    /// Creates a new `CrudOptions` with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum number of records to return.
    ///
    /// Useful for implementing pagination and controlling memory usage.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::databases::redb::transaction::options::CrudOptions;
    ///
    /// let options = CrudOptions::new().with_limit(100);
    /// ```
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.list.limit = Some(limit);
        self
    }

    /// Sets the number of records to skip before returning results.
    ///
    /// Combines with `with_limit` for pagination.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::databases::redb::transaction::options::CrudOptions;
    ///
    /// // Skip first 50 records, return next 25
    /// let options = CrudOptions::new()
    ///     .with_offset(50)
    ///     .with_limit(25);
    /// ```
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.list.offset = Some(offset);
        self
    }
}

/// Configuration for list/query operations.
///
/// Controls pagination behavior when fetching multiple records.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListConfig {
    /// Maximum number of records to return. `None` means unlimited.
    pub limit: Option<usize>,
    /// Number of records to skip before returning results.
    pub offset: Option<usize>,
}

/// Configuration for relational data hydration.
///
/// Controls how deeply to follow relational links when loading data.
///
/// # Performance Note
///
/// Higher depth values can significantly impact query performance.
/// Set `fetch_relations` to `false` to disable hydration entirely.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HydrationConfig {
    /// Maximum depth to follow relational links (0 = no hydration).
    pub depth: usize,
    /// Whether to fetch related entities. If `false`, links remain dehydrated.
    pub fetch_relations: bool,
}

/// Configuration for blob data handling.
///
/// Controls whether large blob fields are loaded or stripped for performance.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlobConfig {
    /// If `true`, blob data is not loaded, reducing memory usage and improving speed.
    /// Useful when you only need metadata or relational structure.
    pub strip_blobs: bool,
}

/// Configuration for subscription/pub-sub notifications.
///
/// Controls whether operations trigger subscription events.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SubscriptionConfig {
    /// If `true`, mutations trigger subscription notifications.
    pub notify: bool,
}
