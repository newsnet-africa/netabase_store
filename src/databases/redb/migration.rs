//! Database migration functionality for redb.
//!
//! This module provides functions for migrating data between schema versions.
//! It supports both automatic migration detection and manual migration execution.
//!
//! # Migration Strategy
//!
//! The migration system uses a "probing" approach to detect which version of data
//! is actually in the database:
//!
//! 1. For each model family with versioned models, try to open tables with different
//!    version definitions (starting from oldest)
//! 2. When a table opens successfully and contains data, that's the source version
//! 3. Read all records from the old version's table
//! 4. Apply the migration chain to convert each record to the current version
//! 5. Write the migrated records to the current version's table
//! 6. Optionally delete the old table

use crate::errors::NetabaseResult;
use crate::traits::migration::{
    MigrationChainExecutor, MigrationError, MigrationPath, MigrationResult, VersionHeader,
};
use crate::traits::registery::definition::redb_definition::RedbDefinition;
use crate::traits::registery::definition::schema::{DefinitionSchema, SchemaComparisonResult};
use std::marker::PhantomData;
use strum::IntoDiscriminant;

// Re-export migration types from redb_definition for convenience
pub use crate::traits::registery::definition::redb_definition::{
    DetectedVersion, MigrationOptions, MigrationResult as ModelMigrationResult,
};

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

/// Database-level migration coordinator.
#[allow(dead_code)]
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

    /// Detect which version tables exist in the database.
    ///
    /// This probes the database by trying to open tables with different
    /// version definitions. Returns information about each detected version.
    pub fn detect_versions(&self) -> NetabaseResult<Vec<DetectedVersion>> {
        D::detect_versions(self.db)
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

    /// Run migrations using the definition's generated migrate_all method.
    ///
    /// This method coordinates the migration process:
    /// 1. Probes the database to detect which version tables exist
    /// 2. For each model family where an old version is detected:
    ///    a. Opens the old version's table
    ///    b. Opens/creates the current version's table
    ///    c. Reads all records from the old table
    ///    d. Applies the migration chain to convert each record
    ///    e. Writes the migrated records to the new table
    /// 3. Returns a summary of what was migrated
    pub fn run(&self) -> NetabaseResult<DatabaseMigrationResult> {
        // First, detect what versions exist in the database
        let detected = D::detect_versions(self.db)?;

        // Check if there are any old versions that need migration
        let needs_migration: Vec<_> = detected
            .iter()
            .filter(|d| {
                // Check if this is an old version by comparing with the compiled schema
                self.compiled_schema
                    .model_history
                    .iter()
                    .find(|h| h.family == d.family)
                    .map(|h| d.version < h.current_version)
                    .unwrap_or(false)
            })
            .collect();

        if needs_migration.is_empty() {
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
                tables_migrated: needs_migration.len(),
                total_records: needs_migration
                    .iter()
                    .map(|d| d.record_count as usize)
                    .sum(),
                family_results: needs_migration
                    .iter()
                    .filter_map(|d| {
                        let current_version = self
                            .compiled_schema
                            .model_history
                            .iter()
                            .find(|h| h.family == d.family)
                            .map(|h| h.current_version)?;

                        Some((
                            d.family.clone(),
                            MigrationResult {
                                records_migrated: 0,
                                records_failed: 0,
                                errors: Vec::new(),
                                path: MigrationPath {
                                    from_version: d.version,
                                    to_version: current_version,
                                    family: Box::leak(d.family.clone().into_boxed_str()),
                                    steps: (current_version - d.version) as usize,
                                    may_lose_data: false,
                                },
                            },
                        ))
                    })
                    .collect(),
                has_errors: false,
                dry_run: true,
            });
        }

        // Use the definition's generated migration method (with probing)
        let result = D::migrate_all(self.db, &self.options)?;

        Ok(DatabaseMigrationResult {
            tables_migrated: result.migrations_performed.len(),
            total_records: result.records_migrated,
            family_results: result
                .migrations_performed
                .iter()
                .map(|(family, from_ver, to_ver)| {
                    (
                        family.clone(),
                        MigrationResult {
                            records_migrated: result.records_migrated,
                            records_failed: result.records_failed,
                            errors: result
                                .errors
                                .iter()
                                .map(|e| MigrationError {
                                    record_key: String::new(),
                                    error: e.clone(),
                                    at_version: *from_ver,
                                })
                                .collect(),
                            path: MigrationPath {
                                from_version: *from_ver,
                                to_version: *to_ver,
                                family: Box::leak(family.clone().into_boxed_str()),
                                steps: (*to_ver - *from_ver) as usize,
                                may_lose_data: false,
                            },
                        },
                    )
                })
                .collect(),
            has_errors: result.records_failed > 0,
            dry_run: false,
        })
    }
}

/// Helper to migrate a single record's bytes from one version to another.
pub fn migrate_record_bytes<Chain: MigrationChainExecutor>(
    data: &[u8],
    source_version: u32,
) -> Result<Vec<u8>, MigrationError>
where
    Chain::Current: bincode::Encode,
{
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
