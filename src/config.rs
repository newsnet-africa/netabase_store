//! Unified configuration system for all database backends.
//!
//! This module provides a consistent configuration API across all database
//! backends using the builder pattern via `typed-builder`.

use std::path::PathBuf;
use typed_builder::TypedBuilder;

/// Configuration for file-based database backends (Sled, Redb).
///
/// # Examples
///
/// ```
/// use netabase_store::config::FileConfig;
///
/// // Create with defaults
/// let config = FileConfig::builder()
///     .path("my_database.db")
///     .build();
///
/// // Customize options
/// let config = FileConfig::builder()
///     .path("/data/store.db")
///     .cache_size_mb(512)
///     .create_if_missing(true)
///     .build();
/// ```
#[derive(Debug, Clone, TypedBuilder)]
#[builder(doc)]
pub struct FileConfig {
    /// Path to the database file or directory
    pub path: PathBuf,

    /// Cache size in megabytes (backend-specific interpretation)
    #[builder(default = 256)]
    pub cache_size_mb: usize,

    /// Whether to create the database if it doesn't exist
    #[builder(default = true)]
    pub create_if_missing: bool,

    /// Whether to truncate/recreate if database already exists
    #[builder(default = false)]
    pub truncate: bool,

    /// Read-only mode (if supported by backend)
    #[builder(default = false)]
    pub read_only: bool,

    /// Enable fsync for durability (may impact performance)
    #[builder(default = true)]
    pub use_fsync: bool,
}

impl FileConfig {
    /// Create a basic configuration with just a path
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            path: path.into(),
            cache_size_mb: 256,
            create_if_missing: true,
            truncate: false,
            read_only: false,
            use_fsync: true,
        }
    }

    /// Create configuration for a temporary database
    pub fn temp() -> Self {
        let temp_path = std::env::temp_dir().join(format!("netabase_{}", uuid::Uuid::new_v4()));
        Self::new(temp_path)
    }
}

/// Configuration for in-memory database backends.
///
/// # Examples
///
/// ```
/// use netabase_store::config::MemoryConfig;
///
/// let config = MemoryConfig::builder()
///     .initial_capacity(10000)
///     .build();
/// ```
#[derive(Debug, Clone, TypedBuilder)]
#[builder(doc)]
pub struct MemoryConfig {
    /// Initial capacity hint for the underlying storage
    #[builder(default = 1000)]
    pub initial_capacity: usize,

    /// Maximum number of entries before eviction starts
    #[builder(default = None)]
    pub max_entries: Option<usize>,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            initial_capacity: 1000,
            max_entries: None,
        }
    }
}

/// Configuration for IndexedDB backend (WASM only).
///
/// # Examples
///
/// ```ignore
/// use netabase_store::config::IndexedDBConfig;
///
/// let config = IndexedDBConfig::builder()
///     .database_name("my_app_store")
///     .version(2)
///     .build();
/// ```
#[derive(Debug, Clone, TypedBuilder)]
#[builder(doc)]
pub struct IndexedDBConfig {
    /// Name of the IndexedDB database
    pub database_name: String,

    /// Database version for schema migrations
    #[builder(default = 1)]
    pub version: u32,
}

impl IndexedDBConfig {
    /// Create a basic configuration with a database name
    pub fn new<S: Into<String>>(database_name: S) -> Self {
        Self {
            database_name: database_name.into(),
            version: 1,
        }
    }
}

/// Configuration specifically for Redb zero-copy backend.
///
/// This extends `FileConfig` with zero-copy specific options.
#[derive(Debug, Clone, TypedBuilder)]
#[builder(doc)]
pub struct RedbZeroCopyConfig {
    /// Base file configuration
    #[builder(default = FileConfig::new("redb_zc.db"))]
    pub file_config: FileConfig,

    /// Enable repair mode on open (tries to recover corrupted database)
    #[builder(default = false)]
    pub auto_repair: bool,

    /// Page size for database (must be power of 2, typically 4096)
    #[builder(default = 4096)]
    pub page_size: usize,
}

impl RedbZeroCopyConfig {
    /// Create from a file path
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            file_config: FileConfig::new(path),
            auto_repair: false,
            page_size: 4096,
        }
    }
}

// Helper function to create UUID (used in temp())
mod uuid {
    use std::time::{SystemTime, UNIX_EPOCH};

    pub struct Uuid(u128);

    impl Uuid {
        pub fn new_v4() -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            Self(nanos)
        }
    }

    impl std::fmt::Display for Uuid {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:032x}", self.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_config_builder() {
        let config = FileConfig::builder()
            .path(PathBuf::from("/tmp/test.db"))
            .cache_size_mb(512)
            .create_if_missing(false)
            .build();

        assert_eq!(config.path, PathBuf::from("/tmp/test.db"));
        assert_eq!(config.cache_size_mb, 512);
        assert_eq!(config.create_if_missing, false);
    }

    #[test]
    fn test_file_config_defaults() {
        let config = FileConfig::new("/tmp/default.db");
        assert_eq!(config.cache_size_mb, 256);
        assert_eq!(config.create_if_missing, true);
        assert_eq!(config.truncate, false);
    }

    #[test]
    fn test_memory_config_default() {
        let config = MemoryConfig::default();
        assert_eq!(config.initial_capacity, 1000);
        assert_eq!(config.max_entries, None);
    }
}
