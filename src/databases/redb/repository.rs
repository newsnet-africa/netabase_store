//! Repository-level store management for multi-definition databases.
//!
//! This module provides `RedbRepositoryStore` which manages multiple definition stores
//! within a single repository, enabling inter-definitional communication through
//! `RelationalLink`s.
//!
//! # Folder Structure
//!
//! A repository creates a parent folder containing a subfolder for each definition:
//!
//! ```text
//! my_repository/
//! ├── repository.toml           # Repository-level metadata
//! ├── Definition1/
//! │   ├── data.redb            # Definition1's database
//! │   └── schema.toml          # Definition1's schema
//! ├── Definition2/
//! │   ├── data.redb            # Definition2's database
//! │   └── schema.toml          # Definition2's schema
//! └── ...
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use netabase_store::databases::redb::RedbRepositoryStore;
//!
//! // Create a repository store - this creates all definition stores
//! let repo_store = RedbRepositoryStore::<MyRepo>::new("./my_repository")?;
//!
//! // Access individual definition stores
//! let user_store = repo_store.definition_store::<UserDef>()?;
//!
//! // Begin a repository-wide transaction
//! let txn = repo_store.begin_write()?;
//! txn.create::<UserDef, User>(&user)?;
//! txn.create::<InventoryDef, Item>(&item)?;
//! txn.commit()?;
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use redb::ReadableDatabase;

use crate::errors::{NetabaseError, NetabaseResult};
use crate::traits::registery::definition::redb_definition::RedbDefinition;
use crate::traits::registery::repository::NetabaseRepository;

/// The name of the repository metadata file.
const REPOSITORY_META_FILE: &str = "repository.toml";

/// Trait for types that can provide definition store paths and initialization.
///
/// This trait is implemented by the `#[netabase_repository]` macro to generate
/// the logic for creating and accessing individual definition stores.
pub trait RedbRepositoryDefinitions: NetabaseRepository {
    /// Get the list of definition names in this repository.
    fn definition_names() -> &'static [&'static str];

    /// Initialize all definition stores in the given repository folder.
    ///
    /// This creates the folder structure and initializes each definition's
    /// database and schema files.
    fn init_definition_stores(repo_path: &Path) -> NetabaseResult<()>;
}

/// Repository-level store that manages multiple definition stores.
///
/// This struct holds the repository folder path and provides access to
/// individual definition stores. It ensures that all definitions are
/// initialized when the repository is created.
pub struct RedbRepositoryStore<R: RedbRepositoryDefinitions> {
    /// Path to the repository folder.
    path: PathBuf,
    /// Repository marker (zero-sized).
    _marker: std::marker::PhantomData<R>,
    /// Cached database handles for each definition, keyed by definition name.
    databases: HashMap<&'static str, Arc<redb::Database>>,
}

impl<R: RedbRepositoryDefinitions> RedbRepositoryStore<R> {
    /// Create or open a repository store at the given path.
    ///
    /// This will:
    /// 1. Create the repository folder if it doesn't exist
    /// 2. Create subfolders for each definition
    /// 3. Initialize each definition's database and schema
    /// 4. Write/update the repository metadata file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the repository folder
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let store = RedbRepositoryStore::<MyRepo>::new("./my_repo")?;
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> NetabaseResult<Self> {
        let repo_path = path.as_ref().to_path_buf();

        // Create the repository folder
        if !repo_path.exists() {
            std::fs::create_dir_all(&repo_path).map_err(|e| {
                NetabaseError::IoError(format!(
                    "Failed to create repository folder {:?}: {}",
                    repo_path, e
                ))
            })?;
        }

        // Initialize all definition stores
        R::init_definition_stores(&repo_path)?;

        // Write repository metadata
        let meta_path = repo_path.join(REPOSITORY_META_FILE);
        let meta_content = Self::generate_repository_metadata();
        std::fs::write(&meta_path, &meta_content).map_err(|e| {
            NetabaseError::IoError(format!(
                "Failed to write repository metadata {:?}: {}",
                meta_path, e
            ))
        })?;

        // Open all definition databases
        let mut databases = HashMap::new();
        for def_name in R::definition_names() {
            let def_path = repo_path.join(def_name).join("data.redb");
            let db = redb::Database::create(&def_path)
                .map_err(|e| NetabaseError::RedbError(e.into()))?;
            databases.insert(*def_name, Arc::new(db));
        }

        Ok(Self {
            path: repo_path,
            _marker: std::marker::PhantomData,
            databases,
        })
    }

    /// Get the path to the repository folder.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the path to a specific definition's folder.
    pub fn definition_path(&self, definition_name: &str) -> PathBuf {
        self.path.join(definition_name)
    }

    /// Get a reference to a definition's database.
    ///
    /// # Arguments
    ///
    /// * `definition_name` - The name of the definition
    ///
    /// # Returns
    ///
    /// The database handle, or an error if the definition doesn't exist.
    pub fn database(&self, definition_name: &str) -> NetabaseResult<&Arc<redb::Database>> {
        self.databases.get(definition_name).ok_or_else(|| {
            NetabaseError::DefinitionNotFound(format!(
                "Definition '{}' not found in repository '{}'",
                definition_name,
                R::name()
            ))
        })
    }

    /// Get the names of all definitions in this repository.
    pub fn definition_names(&self) -> &'static [&'static str] {
        R::definition_names()
    }

    /// Generate repository metadata TOML content.
    fn generate_repository_metadata() -> String {
        let def_names: Vec<_> = R::definition_names().iter().collect();

        format!(
            r#"# Repository: {}
# Generated by netabase_store

[repository]
name = "{}"
definition_count = {}
model_count = {}

[definitions]
names = {:?}
"#,
            R::name(),
            R::name(),
            R::definition_count(),
            R::model_count(),
            def_names
        )
    }

    /// Check if all definition databases are healthy.
    pub fn check_health(&self) -> NetabaseResult<()> {
        for def_name in R::definition_names() {
            let db: &Arc<redb::Database> = self.database(def_name)?;
            // Try to begin a read transaction to verify database is accessible
            // Arc<Database> derefs to Database which has begin_read
            let _txn = (**db)
                .begin_read()
                .map_err(|e| NetabaseError::RedbTransactionError(e))?;
        }
        Ok(())
    }
}

/// Repository-level transaction that spans multiple definitions.
///
/// This allows reading and writing across definitions within a single
/// transaction context. Changes are only committed when all operations
/// succeed across all definitions.
///
/// # Note
///
/// Currently, redb doesn't support true distributed transactions across
/// multiple database files. This implementation provides logical grouping
/// and sequential commits. For true ACID across definitions, consider
/// using a single database file with table prefixes.
pub struct RedbRepositoryTransaction<'repo, R: RedbRepositoryDefinitions> {
    store: &'repo RedbRepositoryStore<R>,
    /// Write transactions for each definition that has been accessed.
    write_transactions: HashMap<&'static str, redb::WriteTransaction>,
    /// Whether this is a write transaction.
    is_write: bool,
}

impl<'repo, R: RedbRepositoryDefinitions> RedbRepositoryTransaction<'repo, R> {
    /// Create a new read-only repository transaction.
    pub fn new_read(store: &'repo RedbRepositoryStore<R>) -> NetabaseResult<Self> {
        Ok(Self {
            store,
            write_transactions: HashMap::new(),
            is_write: false,
        })
    }

    /// Create a new read-write repository transaction.
    pub fn new_write(store: &'repo RedbRepositoryStore<R>) -> NetabaseResult<Self> {
        // Pre-acquire write transactions for all definitions
        let mut write_transactions = HashMap::new();
        for def_name in R::definition_names() {
            let db = store.database(def_name)?;
            let txn = db
                .begin_write()
                .map_err(|e| NetabaseError::RedbError(e.into()))?;
            write_transactions.insert(*def_name, txn);
        }

        Ok(Self {
            store,
            write_transactions,
            is_write: true,
        })
    }

    /// Get a read transaction for a specific definition.
    ///
    /// # Type Parameters
    ///
    /// * `D` - The definition type to get a transaction for
    pub fn read_definition<D: RedbDefinition>(&self) -> NetabaseResult<redb::ReadTransaction>
    where
        D::Discriminant: 'static + std::fmt::Debug,
        D: Clone,
    {
        let def_name = D::debug_name();
        let def_name_str = format!("{:?}", def_name);
        let db = self.store.database(&def_name_str)?;
        (**db)
            .begin_read()
            .map_err(|e| NetabaseError::RedbTransactionError(e))
    }

    /// Get the write transaction for a specific definition.
    ///
    /// # Type Parameters
    ///
    /// * `D` - The definition type to get a transaction for
    ///
    /// # Panics
    ///
    /// Panics if this is not a write transaction.
    pub fn write_definition(
        &mut self,
        definition_name: &'static str,
    ) -> NetabaseResult<&mut redb::WriteTransaction> {
        if !self.is_write {
            return Err(NetabaseError::TransactionError(
                "Cannot get write transaction from read-only repository transaction".to_string(),
            ));
        }

        self.write_transactions
            .get_mut(definition_name)
            .ok_or_else(|| {
                NetabaseError::DefinitionNotFound(format!(
                    "Definition '{}' not found in repository transaction",
                    definition_name
                ))
            })
    }

    /// Commit all pending writes across all definitions.
    ///
    /// This commits each definition's transaction sequentially.
    /// If any commit fails, the remaining definitions are not committed.
    pub fn commit(self) -> NetabaseResult<()> {
        if !self.is_write {
            return Ok(());
        }

        for (def_name, txn) in self.write_transactions {
            txn.commit().map_err(|e| {
                NetabaseError::RedbError(redb::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to commit transaction for '{}': {}", def_name, e),
                )))
            })?;
        }

        Ok(())
    }

    /// Check if this is a write transaction.
    pub fn is_write(&self) -> bool {
        self.is_write
    }
}

impl<R: RedbRepositoryDefinitions> RedbRepositoryStore<R> {
    /// Begin a read-only repository transaction.
    pub fn begin_read(&self) -> NetabaseResult<RedbRepositoryTransaction<'_, R>> {
        RedbRepositoryTransaction::new_read(self)
    }

    /// Begin a read-write repository transaction.
    pub fn begin_write(&self) -> NetabaseResult<RedbRepositoryTransaction<'_, R>> {
        RedbRepositoryTransaction::new_write(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests will be added when the macro generates the required trait implementations
}
