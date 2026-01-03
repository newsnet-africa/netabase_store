pub mod migration;
pub mod transaction;

use crate::errors::{NetabaseError, NetabaseResult};
use crate::traits::registery::definition::redb_definition::RedbDefinition;
use crate::traits::registery::definition::schema::{DefinitionSchema, SchemaComparisonResult};
use std::path::Path;
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
    pub fn migrate(&self) -> NetabaseResult<migration::DatabaseMigrationResult> {
        let migrator = migration::DatabaseMigrator::<D>::new(&self.db, self.stored_schema.clone());
        migrator.run()
    }

    /// Migrate with custom options.
    ///
    /// See [`migrate`](Self::migrate) for details on what migration does.
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

    /// Get the raw database reference for advanced operations.
    pub fn raw_db(&self) -> &Arc<redb::Database> {
        &self.db
    }
}

use crate::traits::database::store::NBStore;

/// Compute the schema file path from a database path.
///
/// If the database path is a file (e.g., `/path/to/db.redb`), the schema file
/// is stored as a sibling file with `.netabase_schema.toml` extension
/// (e.g., `/path/to/db.netabase_schema.toml`).
///
/// If the database path is a directory, the schema file is stored inside it.
fn schema_path_for_db(db_path: &Path) -> std::path::PathBuf {
    // Check if path has a file extension (likely a file, not a directory)
    if db_path.extension().is_some() {
        // Store as sibling file: /path/to/db.redb -> /path/to/db.netabase_schema.toml
        let mut schema_path = db_path.to_path_buf();
        let stem = db_path.file_stem().and_then(|s| s.to_str()).unwrap_or("db");
        schema_path.set_file_name(format!("{}.netabase_schema.toml", stem));
        schema_path
    } else {
        // Store inside directory: /path/to/db/ -> /path/to/db/.netabase_schema.toml
        db_path.join(".netabase_schema.toml")
    }
}

impl<D: RedbDefinition> NBStore<D> for RedbStore<D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug + PartialEq,
    <D as IntoDiscriminant>::Discriminant: PartialEq,
    D: Clone,
{
    /// Create a new RedbStore
    fn new<P: AsRef<Path>>(path: P) -> NetabaseResult<Self>
    where
        D::TreeNames: Default,
    {
        let path_ref = path.as_ref();

        // Ensure parent directory exists
        if let Some(parent) = path_ref.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    NetabaseError::IoError(format!(
                        "Failed to create parent directory {:?}: {}",
                        parent, e
                    ))
                })?;
            }
        }

        let db =
            redb::Database::create(path_ref).map_err(|e| NetabaseError::RedbError(e.into()))?;

        // Compute schema path (sibling file for .redb files, inside for directories)
        let schema_path = schema_path_for_db(path_ref);

        // Try to read existing schema
        let stored_schema = if schema_path.exists() {
            std::fs::read_to_string(&schema_path)
                .ok()
                .and_then(|content| toml::from_str(&content).ok())
        } else {
            None
        };

        // Write current schema
        let toml = D::export_toml();
        if let Err(e) = std::fs::write(&schema_path, &toml) {
            eprintln!("Warning: Failed to write schema file: {}", e);
        }

        Ok(Self {
            _tree_names: Default::default(),
            db: Arc::new(db),
            stored_schema,
        })
    }

    fn execute_transaction<F: Fn()>(f: F) {
        f()
    }
}
