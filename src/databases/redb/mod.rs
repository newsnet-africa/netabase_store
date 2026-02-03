//! Redb database backend implementation.
//!
//! This module provides the [redb](https://github.com/cberner/redb) backend for netabase_store.
//! Redb is a simple, portable, high-performance, ACID-compliant embedded key-value store.
//!
//! # Module Structure
//!
//! - `bounds`: Redb-specific trait bounds (see [`bounds`])
//! - `migration`: Schema versioning and data migration
//! - `repository`: Repository-based database access
//! - `transaction`: Read/write transactions and CRUD operations
//! - `libp2p`: Peer-to-peer networking integration
//!
//! # Core Types
//!
//! - [`RedbStore<D>`]: Main database handle for a definition `D`
//! - [`RedbTransaction`]: Read or write transaction handle
//!
//! # Quick Start
//!
//! ```rust
//! use netabase_store::prelude::*;
//! use netabase_store::traits::database::store::NBStore;
//! use serde::{Serialize, Deserialize};
//!
//! #[netabase_macros::netabase_definition(MyApp)]
//! mod models {
//!     use super::*;
//!     
//!     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
//!     pub struct User {
//!         #[primary_key]
//!         pub id: String,
//!         pub name: String,
//!     }
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use models::*;
//!
//! // Create an in-memory database
//! let (store, _temp) = RedbStore::<MyApp>::new_temporary()?;
//!
//! // Write data
//! let txn = store.begin_write()?;
//! txn.create(&User {
//!     id: UserID("alice".into()),
//!     name: "Alice".into()
//! })?;
//! txn.commit()?;
//!
//! // Read data
//! let txn = store.begin_read()?;
//! let user: Option<User> = txn.read(&UserID("alice".into()))?;
//! assert_eq!(user.unwrap().name, "Alice");
//! # Ok(())
//! # }
//! ```
//!
//! # Schema Management
//!
//! RedbStore tracks schema versions and can detect when migration is needed:
//!
//! ```rust,ignore
//! if store.needs_migration() {
//!     let result = store.migrate_all()?;
//!     println!("Migrated {} records", result.records_migrated);
//! }
//! ```
//!
//! # Implementation Details
//!
//! - Uses postcard for efficient binary serialization
//! - Supports secondary indexes for fast non-primary-key lookups
//! - Blob data is automatically chunked for large values
//! - Type-safe relational links between models

pub mod bounds;
#[cfg(feature = "libp2p")]
pub mod libp2p;
#[cfg(feature = "migration")]
pub mod migration;
#[cfg(feature = "repository")]
pub mod repository;
pub mod transaction;

pub use bounds::RedbModelBounds;

use crate::errors::{NetabaseError, NetabaseResult};
use crate::traits::registry::definition::redb_definition::RedbDefinition;
use crate::traits::registry::definition::schema::{DefinitionSchema, SchemaComparisonResult};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use strum::IntoDiscriminant;

/// Metadata table name for storing schema version information.
#[allow(dead_code)]
const SCHEMA_META_TABLE: &str = "__netabase_schema_meta__";

pub struct RedbStore<D: RedbDefinition>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    _tree_names: D::TreeNames,
    db: Arc<redb::Database>,
    /// The schema that was stored in the database at open time.
    stored_schema: Option<DefinitionSchema>,
}

impl<D: RedbDefinition> RedbStore<D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    /// Begin a read-only transaction on the database.
    ///
    /// Read transactions provide a consistent snapshot view of the database
    /// and allow concurrent reads without blocking other readers.
    pub fn begin_read(&self) -> NetabaseResult<transaction::RedbTransaction<'_, D>> {
        transaction::RedbTransaction::new_read(&self.db)
    }

    /// Begin a read-write transaction on the database.
    ///
    /// Write transactions are exclusive - only one write transaction can be
    /// active at a time. Use read transactions when you don't need to modify data.
    pub fn begin_write(&self) -> NetabaseResult<transaction::RedbTransaction<'_, D>> {
        transaction::RedbTransaction::new_write(&self.db)
    }

    /// Get the current compiled schema.
    pub fn compiled_schema(&self) -> DefinitionSchema {
        D::schema()
    }

    /// Get the schema that was stored in the database when it was opened.
    pub fn stored_schema(&self) -> Option<&DefinitionSchema> {
        self.stored_schema.as_ref()
    }

    /// Compare the compiled schema with the stored schema.
    ///
    /// Returns `None` if there is no stored schema (new database).
    pub fn compare_schemas(&self) -> Option<SchemaComparisonResult> {
        self.stored_schema
            .as_ref()
            .map(|stored| self.compiled_schema().compare(stored))
    }

    /// Check if migration is needed.
    ///
    /// This method uses probing to detect which version tables exist in the
    /// database and compares with the current compiled schema.
    pub fn needs_migration(&self) -> bool {
        // First try schema comparison if we have stored schema
        if let Some(SchemaComparisonResult::Identical) = self.compare_schemas() {
            return false;
        }

        // If schemas differ or no stored schema, probe the database
        match D::detect_versions(&self.db) {
            Ok(detected) => {
                let schema = D::schema();
                detected.iter().any(|d| {
                    schema
                        .model_history
                        .iter()
                        .find(|h| h.family == d.family)
                        .map(|h| d.version < h.current_version)
                        .unwrap_or(false)
                })
            }
            Err(_) => false,
        }
    }

    /// Detect which version tables exist in the database.
    ///
    /// This probes the database by trying to open tables with different
    /// version definitions. Useful for understanding what data is in the
    /// database before migration.
    pub fn detect_versions(&self) -> NetabaseResult<Vec<migration::DetectedVersion>> {
        D::detect_versions(&self.db)
    }

    /// Migrate the database to the current schema version.
    ///
    /// This will:
    /// 1. Probe the database to detect which version tables exist
    /// 2. For each model family where an old version is detected:
    ///    - Read all records from the old version's table
    ///    - Apply the migration chain from old to new version
    ///    - Write the migrated records to the new version's table
    /// 3. Optionally delete old tables
    ///
    /// Returns a `MigrationResult` with counts and any errors.
    #[cfg(feature = "migration")]
    pub fn migrate(&self) -> NetabaseResult<migration::DatabaseMigrationResult> {
        let migrator = migration::DatabaseMigrator::<D>::new(&self.db, self.stored_schema.clone());
        migrator.run()
    }

    /// Migrate with custom options.
    ///
    /// See [`migrate`](Self::migrate) for details on what migration does.
    #[cfg(feature = "migration")]
    pub fn migrate_with_options(
        &self,
        options: migration::MigrationOptions,
    ) -> NetabaseResult<migration::DatabaseMigrationResult> {
        let migrator = migration::DatabaseMigrator::<D>::with_options(
            &self.db,
            self.stored_schema.clone(),
            options,
        );
        migrator.run()
    }

    /// Open a database and automatically migrate if needed.
    ///
    /// This is a convenience method that combines opening the database with
    /// automatic migration detection and execution. It will:
    ///
    /// 1. Open the database at the given path
    /// 2. Check if migration is needed by comparing compiled vs stored schema
    /// 3. If needed, run migrations automatically
    /// 4. Return the store and migration result (if any migration was performed)
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the database folder
    ///
    /// # Returns
    ///
    /// A tuple of:
    /// - `RedbStore<D>` - The opened database store
    /// - `Option<DatabaseMigrationResult>` - Migration result if migration was performed,
    ///   `None` if no migration was needed
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use netabase_store::databases::redb::RedbStore;
    /// use myapp::MyAppDef;
    ///
    /// // Open with auto-migration
    /// let (store, migration_result) = RedbStore::<MyAppDef>::open_with_auto_migrate("./my_db")?;
    ///
    /// if let Some(result) = migration_result {
    ///     println!("Migrated {} records", result.total_records);
    ///     if result.has_errors {
    ///         eprintln!("Migration had errors!");
    ///     }
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The database cannot be opened
    /// - Migration fails (partial data may be migrated)
    #[cfg(feature = "migration")]
    pub fn open_with_auto_migrate<P: AsRef<Path>>(
        path: P,
    ) -> NetabaseResult<(Self, Option<migration::DatabaseMigrationResult>)>
    where
        D::TreeNames: Default,
        <D as IntoDiscriminant>::Discriminant: PartialEq,
    {
        let store = StoreConfig::new(path.as_ref().to_path_buf()).create::<D>()?;
        
        let migration_result = if store.needs_migration() {
            Some(store.migrate()?)
        } else {
            None
        };
        
        Ok((store, migration_result))
    }

    /// Open a database and automatically migrate with custom options.
    ///
    /// Like [`open_with_auto_migrate`](Self::open_with_auto_migrate) but allows
    /// specifying custom migration options like dry-run mode.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use netabase_store::databases::redb::{RedbStore, migration::MigrationOptions};
    /// use myapp::MyAppDef;
    ///
    /// // Do a dry-run first
    /// let options = MigrationOptions { dry_run: true, ..Default::default() };
    /// let (store, result) = RedbStore::<MyAppDef>::open_with_auto_migrate_options(
    ///     "./my_db",
    ///     options
    /// )?;
    ///
    /// if let Some(result) = result {
    ///     println!("Would migrate {} records", result.total_records);
    /// }
    /// ```
    #[cfg(feature = "migration")]
    pub fn open_with_auto_migrate_options<P: AsRef<Path>>(
        path: P,
        options: migration::MigrationOptions,
    ) -> NetabaseResult<(Self, Option<migration::DatabaseMigrationResult>)>
    where
        D::TreeNames: Default,
        <D as IntoDiscriminant>::Discriminant: PartialEq,
    {
        let store = StoreConfig::new(path.as_ref().to_path_buf()).create::<D>()?;
        
        let migration_result = if store.needs_migration() {
            Some(store.migrate_with_options(options)?)
        } else {
            None
        };
        
        Ok((store, migration_result))
    }

    /// Get the raw database reference for advanced operations.
    pub fn raw_db(&self) -> &Arc<redb::Database> {
        &self.db
    }
}

use crate::traits::database::store::NBStore;

/// The name of the main database file inside a netabase folder.
const DB_FILE_NAME: &str = "data.redb";
/// The name of the schema file inside a netabase folder.
const SCHEMA_FILE_NAME: &str = "schema.toml";
/// The name of the CLI binary inside a netabase folder.
const CLI_BINARY_NAME: &str = "client";
/// The name of the README file inside a netabase folder.
const README_FILE_NAME: &str = "README.md";

/// Configuration for creating a new RedbStore.
///
/// This builder-style struct provides a flexible way to configure database creation
/// with various options like client binary export, README generation, and custom paths.
///
/// # Examples
///
/// ```rust,no_run
/// use netabase_store::databases::redb::{RedbStore, StoreConfig};
/// use netabase_store::doc_example::ExampleDef;
///
/// // Simple creation
/// let store = StoreConfig::new("./my_database")
///     .create::<ExampleDef>()
///     .unwrap();
///
/// // With client binary export
/// let store = StoreConfig::new("./my_database")
///     .with_client_binary(Some("./target/release/client"))
///     .create::<ExampleDef>()
///     .unwrap();
///
/// // With README generation
/// let store = StoreConfig::new("./my_database")
///     .with_readme(Some("My custom README content"))
///     .create::<ExampleDef>()
///     .unwrap();
///
/// // Full configuration
/// let store = StoreConfig::new("./my_database")
///     .with_client_binary(Some("./target/release/client"))
///     .with_readme_auto() // Auto-generate README from schema
///     .export_schema(true)
///     .create::<ExampleDef>()
///     .unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct StoreConfig {
    /// Path to the database folder
    pub path: PathBuf,
    /// Optional path to client binary to export (None = don't export, Some(None) = use current exe)
    pub client_binary: Option<Option<PathBuf>>,
    /// Optional README content to write (None = don't write, Some(content))
    pub readme_content: Option<String>,
    /// Whether to export schema.toml (default: true)
    pub export_schema: bool,
    /// Custom database file name (default: "data.redb")
    pub db_file_name: Option<String>,
    /// Custom schema file name (default: "schema.toml")
    pub schema_file_name: Option<String>,
    /// Custom client binary name (default: "client")
    pub client_binary_name: Option<String>,
}

impl StoreConfig {
    /// Create a new store configuration with the given database path.
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            path: path.into(),
            client_binary: None,
            readme_content: None,
            export_schema: true,
            db_file_name: None,
            schema_file_name: None,
            client_binary_name: None,
        }
    }

    /// Export the client binary to the database folder.
    ///
    /// - `Some(Some(path))` - Export the binary at the given path
    /// - `Some(None)` - Export the current executable
    /// - `None` - Don't export any binary (default)
    pub fn with_client_binary(mut self, binary_path: Option<&str>) -> Self {
        self.client_binary = Some(binary_path.map(PathBuf::from));
        self
    }

    /// Set custom README content to write to the database folder.
    pub fn with_readme(mut self, content: Option<&str>) -> Self {
        self.readme_content = content.map(|s| s.to_string());
        self
    }

    /// Auto-generate a README based on the schema.
    ///
    /// This will be populated during database creation with information
    /// about models and available CLI commands.
    pub fn with_readme_auto(mut self) -> Self {
        self.readme_content = Some(String::new()); // Will be filled during create()
        self
    }

    /// Set whether to export the schema.toml file (default: true).
    pub fn export_schema(mut self, export: bool) -> Self {
        self.export_schema = export;
        self
    }

    /// Set a custom database file name (default: "data.redb").
    pub fn db_file_name(mut self, name: &str) -> Self {
        self.db_file_name = Some(name.to_string());
        self
    }

    /// Set a custom schema file name (default: "schema.toml").
    pub fn schema_file_name(mut self, name: &str) -> Self {
        self.schema_file_name = Some(name.to_string());
        self
    }

    /// Set a custom client binary name (default: "client").
    pub fn client_binary_name(mut self, name: &str) -> Self {
        self.client_binary_name = Some(name.to_string());
        self
    }

    /// Create the database with this configuration.
    pub fn create<D: RedbDefinition>(self) -> NetabaseResult<RedbStore<D>>
    where
        <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug + PartialEq,
        <D as IntoDiscriminant>::Discriminant: PartialEq,
        D: Clone,
        D::TreeNames: Default,
    {
        RedbStore::with_config(self)
    }
}

impl<D: RedbDefinition> NBStore<D> for RedbStore<D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug + PartialEq,
    <D as IntoDiscriminant>::Discriminant: PartialEq,
    D: Clone,
{
    /// Create a new RedbStore with default configuration.
    ///
    /// The path provided is treated as a folder that will contain:
    /// - `data.redb` - The main database file
    /// - `schema.toml` - The schema definition file
    ///
    /// For more control over database creation, use [`StoreConfig`] instead:
    ///
    /// ```rust,no_run
    /// use netabase_store::databases::redb::StoreConfig;
    /// use netabase_store::doc_example::ExampleDef;
    ///
    /// let store = StoreConfig::new("./my_database")
    ///     .with_client_binary(Some("./target/release/client"))
    ///     .create::<ExampleDef>()
    ///     .unwrap();
    /// ```
    ///
    /// If the folder doesn't exist, it will be created along with all parent directories.
    fn new<P: AsRef<Path>>(path: P) -> NetabaseResult<Self>
    where
        D::TreeNames: Default,
    {
        StoreConfig::new(path.as_ref().to_path_buf()).create::<D>()
    }

    fn execute_transaction<F: Fn()>(f: F) {
        f()
    }
}

impl<D: RedbDefinition> RedbStore<D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug + PartialEq,
    <D as IntoDiscriminant>::Discriminant: PartialEq,
    D: Clone,
{
    /// Create a new RedbStore with the given configuration.
    ///
    /// This is the main constructor that handles all configuration options.
    /// Use [`StoreConfig`] for a builder-style API.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use netabase_store::databases::redb::{RedbStore, StoreConfig};
    /// use netabase_store::doc_example::ExampleDef;
    ///
    /// let config = StoreConfig::new("./my_database")
    ///     .with_client_binary(Some("./target/release/client"))
    ///     .with_readme_auto();
    ///
    /// let store = RedbStore::<ExampleDef>::with_config(config).unwrap();
    /// ```
    pub fn with_config(config: StoreConfig) -> NetabaseResult<Self>
    where
        D::TreeNames: Default,
    {
        let folder_path = &config.path;

        // Create the database folder and all parent directories
        if !folder_path.exists() {
            std::fs::create_dir_all(folder_path).map_err(|e| {
                NetabaseError::IoError(format!(
                    "Failed to create database folder {:?}: {}",
                    folder_path, e
                ))
            })?;
        }

        // Database file inside the folder
        let db_file_name = config.db_file_name.as_deref().unwrap_or(DB_FILE_NAME);
        let db_path = folder_path.join(db_file_name);
        let db =
            redb::Database::create(&db_path).map_err(|e| NetabaseError::RedbError(e.into()))?;

        // Initialize all tables for the definition
        D::init_tables(&db)?;

        // Schema file inside the folder
        let schema_file_name = config
            .schema_file_name
            .as_deref()
            .unwrap_or(SCHEMA_FILE_NAME);
        let schema_path = folder_path.join(schema_file_name);

        // Try to read existing schema
        let stored_schema = if schema_path.exists() {
            std::fs::read_to_string(&schema_path)
                .ok()
                .and_then(|content| toml::from_str(&content).ok())
        } else {
            None
        };

        // Write current schema if configured
        if config.export_schema {
            let toml = D::export_toml();
            if let Err(e) = std::fs::write(&schema_path, &toml) {
                eprintln!("Warning: Failed to write schema file: {}", e);
            }
        }

        // Export client binary if configured
        if let Some(binary_path_opt) = config.client_binary {
            let binary_name = config
                .client_binary_name
                .as_deref()
                .unwrap_or(CLI_BINARY_NAME);

            let source_path = if let Some(path) = binary_path_opt {
                path
            } else {
                // Use the current executable
                std::env::current_exe().map_err(|e| {
                    NetabaseError::IoError(format!("Failed to get current executable path: {}", e))
                })?
            };

            if !source_path.exists() {
                eprintln!(
                    "Warning: Binary {:?} does not exist, skipping export",
                    source_path
                );
            } else {
                let dest_path = folder_path.join(binary_name);

                if let Err(e) = std::fs::copy(&source_path, &dest_path) {
                    eprintln!("Warning: Failed to copy binary: {}", e);
                } else {
                    // Make it executable on Unix systems
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if let Ok(metadata) = std::fs::metadata(&dest_path) {
                            let mut perms = metadata.permissions();
                            perms.set_mode(0o755);
                            let _ = std::fs::set_permissions(&dest_path, perms);
                        }
                    }
                }
            }
        }

        // Write README if configured
        if let Some(readme_content) = config.readme_content {
            let readme_path = folder_path.join(README_FILE_NAME);

            // If content is empty, auto-generate
            let content = if readme_content.is_empty() {
                Self::generate_readme()
            } else {
                readme_content
            };

            if let Err(e) = std::fs::write(&readme_path, content) {
                eprintln!("Warning: Failed to write README: {}", e);
            }
        }

        Ok(Self {
            _tree_names: Default::default(),
            db: Arc::new(db),
            stored_schema,
        })
    }

    /// Generate a README template based on the definition.
    fn generate_readme() -> String {
        let def_name = std::any::type_name::<D>()
            .split("::")
            .last()
            .unwrap_or("Database");

        format!(
            r#"# {} Database

This folder contains a complete Netabase database with its CLI client.

## Contents

- **data.redb** - The main database file containing all stored data
- **schema.toml** - The database schema definition
- **client** - CLI executable for interacting with the database (if exported)

## Usage

The client binary provides a command-line interface for all database operations.

### Basic Commands

```bash
# Show help
./client --help

# Specify database path
./client --db-path ./ <command>
```

### Model Operations

Each model in the schema has CRUD operations available:

```bash
# Create a record
./client <model> create --json '{{...}}'

# Read a record by ID
./client <model> read --id <id>

# Update a record
./client <model> update --id <id> --json '{{...}}'

# Delete a record
./client <model> delete --id <id>

# List all records
./client <model> list
```

## Schema

The database schema is defined in `schema.toml`. You can view it to see:
- Available models and their fields
- Field types and constraints
- Relationships between models

## Shipping the Database

This entire folder is self-contained and can be shipped as-is. Recipients need only:
1. The database folder with all files
2. Execute permissions on the `client` binary (already set on Unix systems)

## Development

This database was generated using Netabase Store.
"#,
            def_name
        )
    }

    /// Create a new temporary in-memory RedbStore for testing and doctests.
    ///
    /// This creates a database in a temporary directory that will be automatically
    /// cleaned up when the returned guard is dropped. Perfect for examples and tests
    /// that don't need persistence.
    ///
    /// # Returns
    ///
    /// Returns a tuple of `(RedbStore<D>, TempDir)`. The `TempDir` guard must be kept
    /// alive for the database to remain accessible. When dropped, the temporary
    /// directory and all its contents are deleted.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use netabase_store::databases::redb::RedbStore;
    /// use netabase_store::doc_example::ExampleDef;
    ///
    /// let (store, _temp) = RedbStore::<ExampleDef>::new_temporary().unwrap();
    /// // Use store for testing...
    /// // _temp is automatically cleaned up when it goes out of scope
    /// ```
    ///
    /// For doctests, prefer [`new_in_memory`](Self::new_in_memory) which has zero IO overhead.
    pub fn new_temporary() -> NetabaseResult<(Self, tempfile::TempDir)>
    where
        D::TreeNames: Default,
    {
        let temp_dir = tempfile::tempdir()
            .map_err(|e| NetabaseError::IoError(format!("Failed to create temp dir: {}", e)))?;
        let store = <Self as NBStore<D>>::new(temp_dir.path())?;
        Ok((store, temp_dir))
    }

    /// Create a new purely in-memory RedbStore.
    ///
    /// This uses redb's `InMemoryBackend` for a lightweight, zero-IO database
    /// that exists only in RAM. Perfect for doctests and unit tests that don't
    /// need any disk operations.
    ///
    /// # Note
    ///
    /// The database is ephemeral - all data is lost when the store is dropped.
    /// No schema file is written since there's no filesystem involved.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use netabase_store::doc_example::*;
    /// use netabase_store::databases::redb::RedbStore;
    /// use netabase_store::databases::redb::transaction::RedbModelCrud;
    /// use netabase_store::traits::database::store::NBStore;
    ///
    /// let store = RedbStore::<ExampleDef>::new_in_memory().unwrap();
    ///
    /// // Write data
    /// let txn = store.begin_write().unwrap();
    /// txn.create(&User {
    ///     id: UserID("alice".into()),
    ///     name: "Alice".into(),
    ///     email: "alice@example.com".into(),
    /// }).unwrap();
    /// txn.commit().unwrap();
    ///
    /// // Read data
    /// let txn = store.begin_read().unwrap();
    /// let user: Option<User> = txn.read(&UserID("alice".into())).unwrap();
    /// assert_eq!(user.unwrap().name, "Alice");
    /// ```
    pub fn new_in_memory() -> NetabaseResult<Self>
    where
        D::TreeNames: Default,
    {
        let db = redb::Database::builder()
            .create_with_backend(redb::backends::InMemoryBackend::new())
            .map_err(|e| NetabaseError::RedbError(e.into()))?;

        // Initialize all tables for the definition
        D::init_tables(&db)?;

        Ok(Self {
            _tree_names: Default::default(),
            db: Arc::new(db),
            stored_schema: None, // No stored schema for in-memory databases
        })
    }

    /// Export the CLI binary to the database folder.
    ///
    /// This method copies the current executable (or a specified binary) to the
    /// database folder, allowing users to ship the database with its client.
    ///
    /// # Arguments
    ///
    /// * `db_path` - Path to the database folder
    /// * `binary_path` - Optional path to the binary to export. If None, uses the current executable.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use netabase_store::databases::redb::RedbStore;
    /// use netabase_store::doc_example::ExampleDef;
    ///
    /// // Export the current binary to the database folder
    /// RedbStore::<ExampleDef>::export_binary("./my_database", None).unwrap();
    ///
    /// // Or specify a custom binary path
    /// RedbStore::<ExampleDef>::export_binary("./my_database", Some("./target/release/client")).unwrap();
    /// ```
    pub fn export_binary<P: AsRef<Path>>(
        db_path: P,
        binary_path: Option<&str>,
    ) -> NetabaseResult<()> {
        let folder_path = db_path.as_ref();

        // Ensure the database folder exists
        if !folder_path.exists() {
            return Err(NetabaseError::IoError(format!(
                "Database folder {:?} does not exist. Create the database first.",
                folder_path
            )));
        }

        // Determine the source binary path
        let source_path = if let Some(path) = binary_path {
            PathBuf::from(path)
        } else {
            // Use the current executable
            std::env::current_exe().map_err(|e| {
                NetabaseError::IoError(format!("Failed to get current executable path: {}", e))
            })?
        };

        if !source_path.exists() {
            return Err(NetabaseError::IoError(format!(
                "Binary {:?} does not exist",
                source_path
            )));
        }

        // Determine the destination path
        let dest_path = folder_path.join(CLI_BINARY_NAME);

        // Copy the binary
        std::fs::copy(&source_path, &dest_path).map_err(|e| {
            NetabaseError::IoError(format!(
                "Failed to copy binary from {:?} to {:?}: {}",
                source_path, dest_path, e
            ))
        })?;

        // Make it executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&dest_path)
                .map_err(|e| {
                    NetabaseError::IoError(format!("Failed to get binary metadata: {}", e))
                })?
                .permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&dest_path, perms).map_err(|e| {
                NetabaseError::IoError(format!("Failed to set binary permissions: {}", e))
            })?;
        }

        Ok(())
    }
}
