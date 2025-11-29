//! Store implementation for redb zero-copy backend
//!
//! This module contains the main store type and its implementation,
//! providing the entry point for database operations with explicit
//! transaction management.

use crate::config::FileConfig;
use crate::error::NetabaseError;
use crate::traits::backend_store::{BackendStore, PathBasedBackend};
use crate::traits::definition::NetabaseDefinitionTrait;
use crate::traits::model::{NetabaseModelTrait, NetabaseModelTraitKey};
use redb::{Database, ReadableDatabase};
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;

/// Main store handle for zero-copy redb backend
///
/// This is the entry point for all database operations. It holds the database
/// handle and provides methods to begin transactions.
///
/// # Examples
///
/// ```no_run
/// use netabase_store::databases::redb_zerocopy::RedbStoreZeroCopy;
/// use netabase_store::{netabase_definition_module, NetabaseModel, netabase, error::NetabaseError};
///
/// // Define your schema using the macro
/// #[netabase_definition_module(MyDefinition, MyKeys)]
/// mod my_models {
///     use netabase_store::{NetabaseModel, netabase};
///
///     #[derive(NetabaseModel, Clone, Debug, PartialEq,
///              bincode::Encode, bincode::Decode,
///              serde::Serialize, serde::Deserialize)]
///     #[netabase(MyDefinition)]
///     pub struct User {
///         #[primary_key]
///         pub id: u64,
///         pub name: String,
///     }
/// }
/// use my_models::*;
///
/// # fn main() -> Result<(), NetabaseError> {
/// let store = RedbStoreZeroCopy::<MyDefinition>::new("./database.redb")?;
///
/// // Begin a write transaction
/// let mut write_txn = store.begin_write()?;
/// let mut tree = write_txn.open_tree::<User>()?;
/// tree.put(User { id: 1, name: "Alice".to_string() })?;
/// drop(tree);
/// write_txn.commit()?;
///
/// // Begin a read transaction
/// let read_txn = store.begin_read()?;
/// let tree = read_txn.open_tree::<User>()?;
/// let user = tree.get(&UserPrimaryKey(1))?;
/// # Ok(())
/// # }
/// ```
pub struct RedbStoreZeroCopy<D>
where
    D: NetabaseDefinitionTrait,
{
    db: Arc<Database>,
    _phantom: PhantomData<D>,
}

impl<D> RedbStoreZeroCopy<D>
where
    D: NetabaseDefinitionTrait,
{
    /// Create a new database, removing any existing database at the path
    ///
    /// This will delete any existing database file and create a fresh one.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the database file should be created
    ///
    /// # Errors
    ///
    /// Returns an error if the database cannot be created or if there are
    /// permission issues with the file system.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, NetabaseError> {
        // Remove existing database if it exists
        let _ = std::fs::remove_file(path.as_ref());

        let db = Database::create(path)?;
        Ok(Self {
            db: Arc::new(db),
            _phantom: PhantomData,
        })
    }

    /// Open an existing database or create if it doesn't exist
    ///
    /// If the database file doesn't exist, it will be created. If it does exist,
    /// it will be opened with its existing data intact.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the database file
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, NetabaseError> {
        let db = Database::open(path)?;
        Ok(Self {
            db: Arc::new(db),
            _phantom: PhantomData,
        })
    }

    /// Begin a write transaction
    ///
    /// Write transactions are exclusive - only one can be active at a time.
    /// The transaction must be explicitly committed or aborted.
    ///
    /// # Returns
    ///
    /// A write transaction that borrows from this store. The transaction
    /// must be committed to persist changes.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use netabase_store::databases::redb_zerocopy::RedbStoreZeroCopy;
    /// use netabase_store::{netabase_definition_module, NetabaseModel, netabase, error::NetabaseError};
    ///
    /// #[netabase_definition_module(MyDefinition, MyKeys)]
    /// mod my_models {
    ///     use netabase_store::{NetabaseModel, netabase};
    ///
    ///     #[derive(NetabaseModel, Clone, Debug, PartialEq,
    ///              bincode::Encode, bincode::Decode,
    ///              serde::Serialize, serde::Deserialize)]
    ///     #[netabase(MyDefinition)]
    ///     pub struct User {
    ///         #[primary_key]
    ///         pub id: u64,
    ///         pub name: String,
    ///     }
    /// }
    /// use my_models::*;
    ///
    /// # fn main() -> Result<(), NetabaseError> {
    /// let store = RedbStoreZeroCopy::<MyDefinition>::new("./test.db")?;
    /// let mut txn = store.begin_write()?;
    /// let mut tree = txn.open_tree::<User>()?;
    /// tree.put(User { id: 1, name: "Alice".to_string() })?;
    /// drop(tree);
    /// txn.commit()?; // All changes are now persistent
    /// # Ok(())
    /// # }
    /// ```
    pub fn begin_write(
        &self,
    ) -> Result<super::transaction::RedbWriteTransactionZC<'_, D>, NetabaseError> {
        let txn = self.db.as_ref().begin_write()?;
        Ok(super::transaction::RedbWriteTransactionZC::new(txn))
    }

    /// Begin a read transaction
    ///
    /// Read transactions provide a consistent snapshot of the database.
    /// Multiple read transactions can be active concurrently.
    ///
    /// # Returns
    ///
    /// A read transaction that borrows from this store. The transaction
    /// provides a consistent view of the database at the time it was created.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use netabase_store::databases::redb_zerocopy::RedbStoreZeroCopy;
    /// use netabase_store::{netabase_definition_module, NetabaseModel, netabase, error::NetabaseError};
    ///
    /// #[netabase_definition_module(MyDefinition, MyKeys)]
    /// mod my_models {
    ///     use netabase_store::{NetabaseModel, netabase};
    ///
    ///     #[derive(NetabaseModel, Clone, Debug, PartialEq,
    ///              bincode::Encode, bincode::Decode,
    ///              serde::Serialize, serde::Deserialize)]
    ///     #[netabase(MyDefinition)]
    ///     pub struct User {
    ///         #[primary_key]
    ///         pub id: u64,
    ///         pub name: String,
    ///     }
    /// }
    /// use my_models::*;
    ///
    /// # fn main() -> Result<(), NetabaseError> {
    /// let store = RedbStoreZeroCopy::<MyDefinition>::new("./test.db")?;
    /// let txn = store.begin_read()?;
    /// let tree = txn.open_tree::<User>()?;
    /// let user = tree.get(&UserPrimaryKey(1))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn begin_read(
        &self,
    ) -> Result<super::transaction::RedbReadTransactionZC<'_, D>, NetabaseError> {
        let txn = self.db.as_ref().begin_read()?;
        Ok(super::transaction::RedbReadTransactionZC::new(txn))
    }

    /// Insert a single model with auto-commit (convenience method)
    ///
    /// This is equivalent to begin_write() -> open_tree() -> put() -> commit()
    /// but handles all the transaction management automatically.
    ///
    /// # Arguments
    ///
    /// * `model` - The model instance to insert
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use netabase_store::databases::redb_zerocopy::RedbStoreZeroCopy;
    /// use netabase_store::{netabase_definition_module, NetabaseModel, netabase, error::NetabaseError};
    ///
    /// #[netabase_definition_module(MyDefinition, MyKeys)]
    /// mod my_models {
    ///     use netabase_store::{NetabaseModel, netabase};
    ///
    ///     #[derive(NetabaseModel, Clone, Debug, PartialEq,
    ///              bincode::Encode, bincode::Decode,
    ///              serde::Serialize, serde::Deserialize)]
    ///     #[netabase(MyDefinition)]
    ///     pub struct User {
    ///         #[primary_key]
    ///         pub id: u64,
    ///         pub name: String,
    ///     }
    /// }
    /// use my_models::*;
    ///
    /// # fn main() -> Result<(), NetabaseError> {
    /// let store = RedbStoreZeroCopy::<MyDefinition>::new("./test.db")?;
    /// let user = User { id: 1, name: "Alice".to_string() };
    /// store.quick_put(user)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn quick_put<M>(&self, model: M) -> Result<(), NetabaseError>
    where
        M: NetabaseModelTrait<D>,
        M::Keys: NetabaseModelTraitKey<D>,
    {
        let mut txn = self.begin_write()?;
        let mut tree = txn.open_tree::<M>()?;
        tree.put(model)?;
        // tree will be dropped automatically here
        txn.commit()
    }

    /// Get a single model (cloned) with auto-transaction (convenience method)
    ///
    /// This is equivalent to begin_read() -> open_tree() -> get()
    /// but handles all the transaction management automatically.
    ///
    /// # Arguments
    ///
    /// * `key` - The primary key of the model to retrieve
    ///
    /// # Returns
    ///
    /// Some(model) if found, None if not found
    pub fn quick_get<M>(
        &self,
        key: &<M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey,
    ) -> Result<Option<M>, NetabaseError>
    where
        M: NetabaseModelTrait<D>,
        M::Keys: NetabaseModelTraitKey<D>,
    {
        let txn = self.begin_read()?;
        let tree = txn.open_tree::<M>()?;
        tree.get(key)
    }

    /// Remove a single model with auto-commit (convenience method)
    ///
    /// This is equivalent to begin_write() -> open_tree() -> remove() -> commit()
    /// but handles all the transaction management automatically.
    ///
    /// # Arguments
    ///
    /// * `key` - The primary key of the model to remove
    ///
    /// # Returns
    ///
    /// Some(model) if the model existed and was removed, None if it didn't exist
    pub fn quick_remove<M>(
        &self,
        key: &<M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey,
    ) -> Result<Option<M>, NetabaseError>
    where
        M: NetabaseModelTrait<D>,
        M::Keys: NetabaseModelTraitKey<D>,
    {
        let mut txn = self.begin_write()?;
        let mut tree = txn.open_tree::<M>()?;
        let result = tree.remove(key.clone())?;
        // tree will be dropped automatically here
        txn.commit()?;
        Ok(result)
    }

    /// Get access to the underlying redb database
    ///
    /// This provides low-level access to the redb Database for advanced use cases.
    /// Most users should use the higher-level transaction API instead.
    pub fn database(&self) -> &Database {
        &self.db
    }
}

impl<D> Clone for RedbStoreZeroCopy<D>
where
    D: NetabaseDefinitionTrait,
{
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            _phantom: PhantomData,
        }
    }
}

// BackendStore trait implementation
impl<D> BackendStore<D> for RedbStoreZeroCopy<D>
where
    D: NetabaseDefinitionTrait,
{
    type Config = FileConfig;

    fn new(config: Self::Config) -> Result<Self, NetabaseError> {
        // Remove existing database if truncate is true
        if config.truncate && config.path.exists() {
            std::fs::remove_dir_all(&config.path)?;
        }

        let db = Database::create(&config.path)?;
        Ok(Self {
            db: Arc::new(db),
            _phantom: PhantomData,
        })
    }

    fn open(config: Self::Config) -> Result<Self, NetabaseError> {
        let db = Database::open(&config.path)?;
        Ok(Self {
            db: Arc::new(db),
            _phantom: PhantomData,
        })
    }

    fn temp() -> Result<Self, NetabaseError> {
        let config = FileConfig::temp();
        <Self as BackendStore<D>>::new(config)
    }
}

impl<D> PathBasedBackend<D> for RedbStoreZeroCopy<D>
where
    D: NetabaseDefinitionTrait,
{
    fn at_path<P: AsRef<Path>>(path: P) -> Result<Self, NetabaseError> {
        let config = FileConfig::new(path.as_ref());
        <Self as BackendStore<D>>::open(config)
    }
}

// Tests temporarily disabled due to macro resolution issues within the crate itself
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use tempfile::tempdir;
//
//     // Tests would go here but require proper macro setup
// }
