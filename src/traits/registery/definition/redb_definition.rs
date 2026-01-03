use crate::errors::NetabaseResult;
use crate::traits::registery::definition::NetabaseDefinition;
use strum::IntoDiscriminant;

/// Result of a database migration operation.
#[derive(Debug, Clone)]
pub struct MigrationResult {
    /// Total number of records migrated.
    pub records_migrated: usize,
    /// Number of records that failed to migrate.
    pub records_failed: usize,
    /// Error messages for failed records.
    pub errors: Vec<String>,
    /// Which model families were migrated and from which versions.
    pub migrations_performed: Vec<(String, u32, u32)>, // (family, from_version, to_version)
}

impl Default for MigrationResult {
    fn default() -> Self {
        Self {
            records_migrated: 0,
            records_failed: 0,
            errors: Vec::new(),
            migrations_performed: Vec::new(),
        }
    }
}

impl MigrationResult {
    /// Check if the migration was successful (no errors).
    pub fn is_success(&self) -> bool {
        self.records_failed == 0
    }

    /// Merge another result into this one.
    pub fn merge(&mut self, other: MigrationResult) {
        self.records_migrated += other.records_migrated;
        self.records_failed += other.records_failed;
        self.errors.extend(other.errors);
        self.migrations_performed.extend(other.migrations_performed);
    }
}

/// Options for controlling migration behavior.
#[derive(Debug, Clone)]
pub struct MigrationOptions {
    /// Whether to continue on individual record errors.
    pub continue_on_error: bool,
    /// Maximum number of errors before aborting.
    pub max_errors: usize,
    /// Whether to run in dry-run mode (no actual changes).
    pub dry_run: bool,
    /// Whether to delete old version tables after migration.
    pub delete_old_tables: bool,
}

impl Default for MigrationOptions {
    fn default() -> Self {
        Self {
            continue_on_error: false,
            max_errors: 100,
            dry_run: false,
            delete_old_tables: true,
        }
    }
}

/// Information about a detected table version.
#[derive(Debug, Clone)]
pub struct DetectedVersion {
    /// The model family name.
    pub family: String,
    /// The detected version number.
    pub version: u32,
    /// The table name that was found.
    pub table_name: String,
    /// Number of records in the table.
    pub record_count: u64,
}

pub trait RedbDefinition: NetabaseDefinition
where
    <Self as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    type ModelTableDefinition<'db>: Clone + Send + Sync;

    /// Probe the database to detect which version tables exist.
    ///
    /// This method tries to open tables for each known version of each model
    /// family, starting from the oldest. When a table opens successfully,
    /// that indicates the database contains data in that version's format.
    ///
    /// # Arguments
    ///
    /// * `db` - The redb database handle
    ///
    /// # Returns
    ///
    /// A vector of `DetectedVersion` for each model family where data was found.
    fn detect_versions(db: &redb::Database) -> NetabaseResult<Vec<DetectedVersion>>;

    /// Perform migration on all model families that need it.
    ///
    /// This method is implemented by the macro and has full knowledge of all
    /// model types, their migration chains, and table structures. It:
    ///
    /// 1. Probes the database to detect which version tables exist
    /// 2. For each model family where an old version is detected:
    ///    a. Opens the old version's table (which succeeded during probing)
    ///    b. Opens/creates the current version's table
    ///    c. Reads all records from the old table
    ///    d. Applies the migration chain to convert each record
    ///    e. Writes the converted records to the new table
    ///    f. Optionally deletes the old table
    /// 3. Returns a summary of what was migrated
    ///
    /// # Arguments
    ///
    /// * `db` - The redb database handle
    /// * `options` - Migration options
    ///
    /// # Returns
    ///
    /// A `MigrationResult` with counts and any errors.
    fn migrate_all(
        db: &redb::Database,
        options: &MigrationOptions,
    ) -> NetabaseResult<MigrationResult>;
}
