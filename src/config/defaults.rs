//! Default configuration management for stores and tables.
//!
//! This module provides the infrastructure for setting default configurations
//! at the store level (applies to all tables) and table level (applies to a
//! specific model).
//!
//! # Configuration Hierarchy
//!
//! When a query is executed, configurations are merged in this order:
//! 1. Store-level defaults (bottom priority)
//! 2. Table-level defaults (middle priority)
//! 3. Query-specific config (top priority)
//!
//! # Example
//!
//! ```rust
//! use netabase_store::config::{QueryConfig, ConfigDefaults};
//! use std::collections::HashMap;
//!
//! // Create store-level defaults
//! let mut store_defaults = ConfigDefaults::new();
//! store_defaults.set_store_default(
//!     QueryConfig::new()
//!         .with_limit(100)
//!         .no_blobs()
//! );
//!
//! // Override for specific table
//! store_defaults.set_table_default(
//!     "User",
//!     QueryConfig::new()
//!         .with_limit(50)
//!         .with_hydration(1)
//! );
//! ```

use super::{QueryConfig};
use std::collections::HashMap;
use std::ops::RangeFull;

/// Container for store-level and table-level default configurations.
///
/// This allows you to set consistent defaults across your entire database
/// or customize behavior per-table.
///
/// # Examples
///
/// ```rust
/// use netabase_store::config::{QueryConfig, ConfigDefaults};
///
/// let mut defaults = ConfigDefaults::new();
///
/// // All tables default to 100 records max
/// defaults.set_store_default(
///     QueryConfig::new().with_limit(100)
/// );
///
/// // User table has custom defaults
/// defaults.set_table_default(
///     "User",
///     QueryConfig::new()
///         .with_limit(50)
///         .with_hydration(2)
/// );
/// ```
#[derive(Debug, Clone, Default)]
pub struct ConfigDefaults {
    /// Default configuration for the entire store (all tables).
    store_default: QueryConfig<RangeFull>,
    
    /// Per-table default configurations.
    table_defaults: HashMap<String, QueryConfig<RangeFull>>,
}

impl ConfigDefaults {
    /// Create a new empty defaults container.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::config::ConfigDefaults;
    ///
    /// let defaults = ConfigDefaults::new();
    /// ```
    pub fn new() -> Self {
        Self {
            store_default: QueryConfig::default(),
            table_defaults: HashMap::new(),
        }
    }

    /// Set the store-level default configuration.
    ///
    /// This applies to all tables unless overridden by a table-specific default.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::config::{QueryConfig, ConfigDefaults};
    ///
    /// let mut defaults = ConfigDefaults::new();
    /// defaults.set_store_default(
    ///     QueryConfig::new()
    ///         .with_limit(100)
    ///         .no_blobs()
    /// );
    /// ```
    pub fn set_store_default(&mut self, config: QueryConfig<RangeFull>) {
        self.store_default = config;
    }

    /// Set a table-specific default configuration.
    ///
    /// This overrides the store default for the specified table.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::config::{QueryConfig, ConfigDefaults};
    ///
    /// let mut defaults = ConfigDefaults::new();
    /// defaults.set_table_default(
    ///     "User",
    ///     QueryConfig::new()
    ///         .with_limit(50)
    ///         .with_hydration(1)
    /// );
    /// ```
    pub fn set_table_default(&mut self, table_name: impl Into<String>, config: QueryConfig<RangeFull>) {
        self.table_defaults.insert(table_name.into(), config);
    }

    /// Remove a table-specific default, falling back to store default.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::config::{QueryConfig, ConfigDefaults};
    ///
    /// let mut defaults = ConfigDefaults::new();
    /// defaults.set_table_default("User", QueryConfig::new().with_limit(50));
    /// defaults.remove_table_default("User");
    /// ```
    pub fn remove_table_default(&mut self, table_name: &str) -> Option<QueryConfig<RangeFull>> {
        self.table_defaults.remove(table_name)
    }

    /// Get the effective default configuration for a table.
    ///
    /// Returns table-specific default merged with store default.
    /// Table defaults override store defaults where set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::config::{QueryConfig, ConfigDefaults};
    ///
    /// let mut defaults = ConfigDefaults::new();
    /// defaults.set_store_default(QueryConfig::new().with_limit(100));
    /// defaults.set_table_default("User", QueryConfig::new().with_limit(50));
    ///
    /// let user_config = defaults.get_for_table("User");
    /// assert_eq!(user_config.pagination.limit, Some(50));
    ///
    /// let post_config = defaults.get_for_table("Post");
    /// assert_eq!(post_config.pagination.limit, Some(100));
    /// ```
    pub fn get_for_table(&self, table_name: &str) -> QueryConfig<RangeFull> {
        match self.table_defaults.get(table_name) {
            Some(table_default) => {
                // Table defaults take priority, store defaults fill in gaps
                table_default.clone().merge_with_defaults(&self.store_default)
            }
            None => self.store_default.clone(),
        }
    }

    /// Get a reference to the store-level default.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::config::{QueryConfig, ConfigDefaults};
    ///
    /// let mut defaults = ConfigDefaults::new();
    /// defaults.set_store_default(QueryConfig::new().with_limit(100));
    ///
    /// let store_default = defaults.store_default();
    /// assert_eq!(store_default.pagination.limit, Some(100));
    /// ```
    pub fn store_default(&self) -> &QueryConfig<RangeFull> {
        &self.store_default
    }

    /// Get a reference to a table-specific default if it exists.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::config::{QueryConfig, ConfigDefaults};
    ///
    /// let mut defaults = ConfigDefaults::new();
    /// defaults.set_table_default("User", QueryConfig::new().with_limit(50));
    ///
    /// assert!(defaults.table_default("User").is_some());
    /// assert!(defaults.table_default("Post").is_none());
    /// ```
    pub fn table_default(&self, table_name: &str) -> Option<&QueryConfig<RangeFull>> {
        self.table_defaults.get(table_name)
    }

    /// Apply defaults to a query configuration.
    ///
    /// This merges the provided config with table/store defaults, where
    /// explicitly set values in the config take precedence.
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::config::{QueryConfig, ConfigDefaults};
    ///
    /// let mut defaults = ConfigDefaults::new();
    /// defaults.set_store_default(QueryConfig::new().with_limit(100));
    /// defaults.set_table_default("User", QueryConfig::new().no_blobs());
    ///
    /// // Query config only sets offset
    /// let config = QueryConfig::new().with_offset(20);
    ///
    /// // After applying defaults for User table
    /// let final_config = defaults.apply_to("User", config);
    /// assert_eq!(final_config.pagination.limit, Some(100));
    /// assert_eq!(final_config.pagination.offset, Some(20));
    /// assert!(final_config.blob.strip_blobs);
    /// ```
    pub fn apply_to(&self, table_name: &str, config: QueryConfig<RangeFull>) -> QueryConfig<RangeFull> {
        let table_default = self.get_for_table(table_name);
        config.merge_with_defaults(&table_default)
    }
}

/// Builder for common default configurations.
///
/// Provides convenient presets for typical use cases.
pub struct DefaultsBuilder;

impl DefaultsBuilder {
    /// Create defaults optimized for performance (no hydration, no blobs).
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::config::defaults::DefaultsBuilder;
    ///
    /// let defaults = DefaultsBuilder::performance_optimized();
    /// assert!(defaults.store_default().blob.strip_blobs);
    /// assert_eq!(defaults.store_default().hydration.depth, 0);
    /// ```
    pub fn performance_optimized() -> ConfigDefaults {
        let mut defaults = ConfigDefaults::new();
        defaults.set_store_default(
            QueryConfig::new()
                .no_blobs()
                .no_hydration()
                .with_limit(1000)
        );
        defaults
    }

    /// Create defaults optimized for rich data (with hydration and blobs).
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::config::defaults::DefaultsBuilder;
    ///
    /// let defaults = DefaultsBuilder::rich_data();
    /// assert!(!defaults.store_default().blob.strip_blobs);
    /// assert_eq!(defaults.store_default().hydration.depth, 2);
    /// ```
    pub fn rich_data() -> ConfigDefaults {
        let mut defaults = ConfigDefaults::new();
        defaults.set_store_default(
            QueryConfig::new()
                .with_blobs(true)
                .with_hydration(2)
                .with_limit(100)
        );
        defaults
    }

    /// Create defaults for API endpoints (paginated, moderate hydration).
    ///
    /// # Example
    ///
    /// ```rust
    /// use netabase_store::config::defaults::DefaultsBuilder;
    ///
    /// let defaults = DefaultsBuilder::api_optimized();
    /// assert_eq!(defaults.store_default().pagination.limit, Some(50));
    /// assert_eq!(defaults.store_default().hydration.depth, 1);
    /// ```
    pub fn api_optimized() -> ConfigDefaults {
        let mut defaults = ConfigDefaults::new();
        defaults.set_store_default(
            QueryConfig::new()
                .with_limit(50)
                .with_hydration(1)
                .no_blobs()
        );
        defaults
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults_hierarchy() {
        let mut defaults = ConfigDefaults::new();
        
        // Set store default
        defaults.set_store_default(QueryConfig::new().with_limit(100));
        
        // Set table override
        defaults.set_table_default("User", QueryConfig::new().with_limit(50));
        
        // User should get table-specific default
        let user_config = defaults.get_for_table("User");
        assert_eq!(user_config.pagination.limit, Some(50));
        
        // Post should get store default
        let post_config = defaults.get_for_table("Post");
        assert_eq!(post_config.pagination.limit, Some(100));
    }

    #[test]
    fn test_apply_merges_correctly() {
        let mut defaults = ConfigDefaults::new();
        defaults.set_store_default(
            QueryConfig::new()
                .with_limit(100)
                .no_blobs()
        );
        defaults.set_table_default(
            "User",
            QueryConfig::new().with_hydration(2)
        );
        
        // Query sets offset only
        let config = QueryConfig::new().with_offset(20);
        let final_config = defaults.apply_to("User", config);
        
        // Should have limit from store, hydration from table, offset from query
        assert_eq!(final_config.pagination.limit, Some(100));
        assert_eq!(final_config.pagination.offset, Some(20));
        assert_eq!(final_config.hydration.depth, 2);
        assert!(final_config.blob.strip_blobs);
    }

    #[test]
    fn test_presets() {
        let perf = DefaultsBuilder::performance_optimized();
        assert!(perf.store_default().blob.strip_blobs);
        assert_eq!(perf.store_default().hydration.depth, 0);

        let rich = DefaultsBuilder::rich_data();
        assert!(!rich.store_default().blob.strip_blobs);
        assert_eq!(rich.store_default().hydration.depth, 2);

        let api = DefaultsBuilder::api_optimized();
        assert_eq!(api.store_default().pagination.limit, Some(50));
        assert_eq!(api.store_default().hydration.depth, 1);
    }
}
