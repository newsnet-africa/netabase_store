pub mod migration;
pub mod transaction;

use crate::errors::{NetabaseError, NetabaseResult};
use crate::traits::registery::definition::redb_definition::RedbDefinition;
use crate::traits::registery::definition::schema::{DefinitionSchema, SchemaComparisonResult};
use std::path::Path;
use std::sync::Arc;
use strum::IntoDiscriminant;

/// Metadata table name for storing schema version information.
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
    /// Begin a transaction on the database
    pub fn begin_transaction(&self) -> NetabaseResult<transaction::RedbTransaction<'_, D>> {
        transaction::RedbTransaction::new(&self.db)
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
    pub fn needs_migration(&self) -> bool {
        match self.compare_schemas() {
            Some(SchemaComparisonResult::Identical) => false,
            Some(_) => true,
            None => false, // New database, no migration needed
        }
    }

    /// Get the raw database reference for advanced operations.
    pub fn raw_db(&self) -> &Arc<redb::Database> {
        &self.db
    }
}

use crate::traits::database::store::NBStore;

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
        let db =
            redb::Database::create(path_ref).map_err(|e| NetabaseError::RedbError(e.into()))?;

        // Try to read existing schema
        let schema_path = path_ref.join(".netabase_schema.toml");
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

        println!(
            "Written toml to path: {:?}.\n\tToml File: {:?}",
            schema_path, toml
        );

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
