//! Database migration functionality for redb.
//!
//! This module provides functions for migrating data between schema versions.
//! It supports both automatic migration detection and manual migration execution.

use crate::errors::{NetabaseError, NetabaseResult};
use crate::traits::migration::{
    MigrationChainExecutor, MigrationError, MigrationPath, MigrationResult, VersionHeader,
};
use crate::traits::registery::definition::redb_definition::RedbDefinition;
use crate::traits::registery::definition::schema::{DefinitionSchema, SchemaComparisonResult};
use std::marker::PhantomData;
use strum::IntoDiscriminant;

/// Migration options for customizing migration behavior.
#[derive(Debug, Clone)]
pub struct MigrationOptions {
    /// Whether to automatically backup before migration.
    pub backup: bool,
    /// Whether to validate all records after migration.
    pub validate: bool,
    /// Whether to continue on individual record errors.
    pub continue_on_error: bool,
    /// Maximum number of errors before aborting.
    pub max_errors: usize,
    /// Whether to run in dry-run mode (no actual changes).
    pub dry_run: bool,
}

impl Default for MigrationOptions {
    fn default() -> Self {
        Self {
            backup: true,
            validate: true,
            continue_on_error: false,
            max_errors: 100,
            dry_run: false,
        }
    }
}

/// Result of a database migration.
#[derive(Debug, Clone)]
pub struct DatabaseMigrationResult {
    /// Number of tables migrated.
    pub tables_migrated: usize,
    /// Total records migrated across all tables.
    pub total_records: usize,
    /// Migration results per model family.
    pub family_results: Vec<(String, MigrationResult)>,
    /// Whether any errors occurred.
    pub has_errors: bool,
    /// Whether this was a dry run.
    pub dry_run: bool,
}

/// Migrator for a specific model family.
///
/// This trait is implemented by the macro-generated `MigrationChain_*` types
/// and provides the concrete migration logic for a model family.
pub trait ModelMigrator: MigrationChainExecutor {
    /// The table name for the main table of this model.
    const MAIN_TABLE: &'static str;

    /// Secondary table names for this model.
    const SECONDARY_TABLES: &'static [&'static str];

    /// Relational table names for this model.
    const RELATIONAL_TABLES: &'static [&'static str];

    /// Subscription table names for this model.
    const SUBSCRIPTION_TABLES: &'static [&'static str];

    /// Blob table names for this model.
    const BLOB_TABLES: &'static [&'static str];

    /// Migrate all tables for this model from source version to current.
    fn migrate_tables(
        db: &redb::Database,
        source_version: u32,
        options: &MigrationOptions,
    ) -> NetabaseResult<MigrationResult>;
}

/// Database-level migration coordinator.
pub struct DatabaseMigrator<'a, D: RedbDefinition>
where
    <D as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    db: &'a redb::Database,
    compiled_schema: DefinitionSchema,
    stored_schema: Option<DefinitionSchema>,
    options: MigrationOptions,
    _marker: PhantomData<D>,
}

impl<'a, D: RedbDefinition> DatabaseMigrator<'a, D>
where
    <D as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    /// Create a new migrator.
    pub fn new(db: &'a redb::Database, stored_schema: Option<DefinitionSchema>) -> Self {
        Self {
            db,
            compiled_schema: D::schema(),
            stored_schema,
            options: MigrationOptions::default(),
            _marker: PhantomData,
        }
    }

    /// Create a migrator with custom options.
    pub fn with_options(
        db: &'a redb::Database,
        stored_schema: Option<DefinitionSchema>,
        options: MigrationOptions,
    ) -> Self {
        Self {
            db,
            compiled_schema: D::schema(),
            stored_schema,
            options,
            _marker: PhantomData,
        }
    }

    /// Check what migrations would be needed.
    pub fn check(&self) -> Option<SchemaComparisonResult> {
        self.stored_schema
            .as_ref()
            .map(|stored| self.compiled_schema.compare(stored))
    }

    /// Get migration paths for all model families that need migration.
    pub fn get_migration_paths(&self) -> Vec<MigrationPath> {
        let stored = match &self.stored_schema {
            Some(s) => s,
            None => return Vec::new(),
        };

        let mut paths = Vec::new();

        for history in &self.compiled_schema.model_history {
            if let Some(stored_history) = stored
                .model_history
                .iter()
                .find(|h| h.family == history.family)
            {
                if stored_history.current_version != history.current_version {
                    paths.push(MigrationPath {
                        from_version: stored_history.current_version,
                        to_version: history.current_version,
                        family: Box::leak(history.family.clone().into_boxed_str()),
                        steps: (history.current_version - stored_history.current_version) as usize,
                        may_lose_data: false, // TODO: Track this properly
                    });
                }
            }
        }

        paths
    }

    /// Run migrations (placeholder - actual implementation needs per-model migrators).
    ///
    /// This method coordinates the migration process:
    /// 1. Creates a backup if enabled
    /// 2. Opens a write transaction
    /// 3. For each model family needing migration:
    ///    a. Reads all records from the main table
    ///    b. Deserializes using the old version's format
    ///    c. Applies the migration chain
    ///    d. Re-serializes and writes back
    ///    e. Rebuilds secondary/relational/subscription/blob tables
    /// 4. Commits the transaction
    pub fn run(&self) -> NetabaseResult<DatabaseMigrationResult> {
        let paths = self.get_migration_paths();

        if paths.is_empty() {
            return Ok(DatabaseMigrationResult {
                tables_migrated: 0,
                total_records: 0,
                family_results: Vec::new(),
                has_errors: false,
                dry_run: self.options.dry_run,
            });
        }

        if self.options.dry_run {
            // Just report what would be migrated
            return Ok(DatabaseMigrationResult {
                tables_migrated: paths.len(),
                total_records: 0, // Would need to count
                family_results: paths
                    .iter()
                    .map(|p| {
                        (
                            p.family.to_string(),
                            MigrationResult {
                                records_migrated: 0,
                                records_failed: 0,
                                errors: Vec::new(),
                                path: p.clone(),
                            },
                        )
                    })
                    .collect(),
                has_errors: false,
                dry_run: true,
            });
        }

        // TODO: Implement actual migration logic
        // This requires:
        // 1. Generic iteration over model families
        // 2. Per-family migrator instances
        // 3. Table-level iteration and rewriting

        Err(NetabaseError::MigrationError(
            "Migration execution not yet implemented. Use manual migration or regenerate the database.".into()
        ))
    }
}

/// Helper to migrate a single record's bytes from one version to another.
pub fn migrate_record_bytes<Chain: MigrationChainExecutor>(
    data: &[u8],
    source_version: u32,
) -> Result<Vec<u8>, MigrationError> {
    // Deserialize, migrate, re-serialize
    let migrated = Chain::migrate_bytes(source_version, data)?;

    // Re-serialize with current version header
    let current_version = Chain::VERSIONS.last().copied().unwrap_or(1);
    let mut output = VersionHeader::new(current_version).to_bytes().to_vec();
    output.extend(
        bincode::encode_to_vec(&migrated, bincode::config::standard()).map_err(|e| {
            MigrationError {
                record_key: String::new(),
                error: e.to_string(),
                at_version: current_version,
            }
        })?,
    );

    Ok(output)
}
