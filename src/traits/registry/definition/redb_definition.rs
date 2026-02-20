use crate::errors::NetabaseResult;
use crate::traits::registry::definition::NetabaseDefinition;
use strum::IntoDiscriminant;

// TODO: AUDIT: QU? [L]
// TODO: REFAC: MNNa(mod migration) [i]
/// Result of a database migration operation.
#[derive(Debug, Clone, Default)]
pub struct MigrationResult {
    /// Total number of records migrated.
    pub records_migrated: usize,
    /// Number of records that failed to migrate.
    pub records_failed: usize,
    /// Error messages for failed records.
    pub errors: Vec<String>,
    // TODO: REFAC: TRNc((String, u32, u32)->ModelVersionInformation)
    /// Which model families were migrated and from which versions.
    pub migrations_performed: Vec<(String, u32, u32)>, // (family, from_version, to_version)
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

// TODO: REFAC: MNNc(RedbTransaction) [i](Many of these are basically transaction methods.)
pub trait RedbDefinition: NetabaseDefinition + Clone
where
    <Self as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    // TODO: AUDIT: QU? [L](is there a type for this?)
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

    /// Initialize all tables for this definition.
    ///
    /// This creates all main, secondary, relational, subscription, and blob tables
    /// for every model in the definition. This should be called when creating a new
    /// database to ensure all tables exist before any read operations.
    ///
    /// # Arguments
    ///
    /// * `db` - The redb database handle
    ///
    /// # Returns
    ///
    /// `Ok(())` if all tables were created successfully.
    fn init_tables(db: &redb::Database) -> NetabaseResult<()>;

    // TODO: REFAC: TRNc [S](Need to make this more type safe)
    /// Helper struct to hold open read-only tables.
    type ReadOnlyTables;

    /// The iterator type returned by `iter_records`.
    type RecordIter<'a>: Iterator<Item = NetabaseResult<libp2p::kad::Record>> + 'a
    where
        Self: 'a;

    /// Open all tables in read-only mode.
    fn open_read_only_tables(txn: &redb::ReadTransaction) -> NetabaseResult<Self::ReadOnlyTables>;

    /// Create an iterator over all records in the definition using the open tables.
    fn iter_records<'a>(tables: &'a Self::ReadOnlyTables) -> NetabaseResult<Self::RecordIter<'a>>;

    /// Find a record by key across all models.
    fn find_record(
        txn: &redb::ReadTransaction,
        key: &libp2p::kad::RecordKey,
    ) -> NetabaseResult<Option<libp2p::kad::Record>>;

    /// Put (upsert) a record.
    fn put_record(txn: &redb::WriteTransaction, record: libp2p::kad::Record) -> NetabaseResult<()>;

    /// Add a provider record to the appropriate model table.
    fn add_provider(
        txn: &redb::WriteTransaction,
        record: libp2p::kad::ProviderRecord,
    ) -> NetabaseResult<()>;

    /// Get providers for a key.
    fn get_providers(
        txn: &redb::ReadTransaction,
        key: &libp2p::kad::RecordKey,
    ) -> NetabaseResult<Vec<libp2p::kad::ProviderRecord>>;

    /// Remove a record by key.
    fn remove_record(
        txn: &redb::WriteTransaction,
        key: &libp2p::kad::RecordKey,
    ) -> NetabaseResult<()>;

    /// Remove a provider.
    fn remove_provider(
        txn: &redb::WriteTransaction,
        key: &libp2p::kad::RecordKey,
        provider: &libp2p::PeerId,
    ) -> NetabaseResult<()>;

    // TODO: AUDIT: QU? [l](Why are these here? what is the difference?)
    // ========================================================================
    // Dispatch methods for generic CRUD
    // ========================================================================

    /// Dispatch create operation to specific model implementation.
    fn dispatch_create(
        txn: &crate::databases::redb::transaction::NetabaseRedbWriteTransaction<'_, Self>,
        definition: &Self,
    ) -> NetabaseResult<()>;

    /// Dispatch read operation to specific model implementation.
    fn dispatch_read(
        txn: &crate::databases::redb::transaction::NetabaseRedbReadTransaction<'_, Self>,
        key: &Self::DefKeys,
    ) -> NetabaseResult<Option<Self>>
    where
        Self: serde::Serialize + for<'de> serde::Deserialize<'de>;

    /// Dispatch update operation to specific model implementation.
    fn dispatch_update(
        txn: &crate::databases::redb::transaction::NetabaseRedbWriteTransaction<'_, Self>,
        definition: &Self,
    ) -> NetabaseResult<()>;

    /// Dispatch delete operation to specific model implementation.
    fn dispatch_delete(
        txn: &crate::databases::redb::transaction::NetabaseRedbWriteTransaction<'_, Self>,
        key: &Self::DefKeys,
    ) -> NetabaseResult<()>;

    /// Dispatch list operation (read all) for a specific model variant.
    fn dispatch_list(
        txn: &crate::databases::redb::transaction::NetabaseRedbReadTransaction<'_, Self>,
        discriminant: <Self as strum::IntoDiscriminant>::Discriminant,
    ) -> NetabaseResult<Vec<Self>>
    where
        Self: serde::Serialize + for<'de> serde::Deserialize<'de>;

    /// Dispatch delete_if operation.
    fn dispatch_delete_if<F>(
        txn: &crate::databases::redb::transaction::NetabaseRedbWriteTransaction<'_, Self>,
        predicate: F,
    ) -> NetabaseResult<()>
    where
        F: Fn(&Self) -> bool;
}
