use crate::databases::manager::DefinitionManager;
use crate::databases::redb_store::RedbStore;
use crate::error::NetabaseResult;
use crate::traits::definition::{DiscriminantName, NetabaseDefinition};
use crate::traits::manager::DefinitionManagerTrait;
use crate::traits::permission::PermissionEnumTrait;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::path::Path;
use strum::{IntoDiscriminant, IntoEnumIterator};

pub mod transaction;

/// Redb-specific definition manager
///
/// This wrapper provides Redb-specific functionality for the generic
/// DefinitionManager, including store loading and initialization.
///
/// # Type Parameters
/// * `R` - The manager trait implementation
/// * `D` - The definition enum type
/// * `P` - The permission enum type
///
/// # Example
/// ```ignore
/// let manager = RedbDefinitionManager::<
///     RestaurantManager,
///     RestaurantDefinitions,
///     RestaurantPermissions
/// >::new("./data")?;
/// ```
pub struct RedbDefinitionManager<R, D, P>
where
    R: DefinitionManagerTrait<Definition = D, Permissions = P, Backend = RedbStore<D>>,
    D: NetabaseDefinition,
    P: PermissionEnumTrait,
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <R as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <P as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + Clone,
{
    pub(crate) inner: DefinitionManager<R, D, P, RedbStore<D>>,
    _marker: PhantomData<R>,
}

impl<R, D, P> RedbDefinitionManager<R, D, P>
where
    R: DefinitionManagerTrait<Definition = D, Permissions = P, Backend = RedbStore<D>>,
    D: NetabaseDefinition,
    P: PermissionEnumTrait,
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <R as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <P as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + Clone,
{
    /// Create a new Redb definition manager
    ///
    /// # Arguments
    /// * `root_path` - Path to the parent directory for all definition databases
    ///
    /// # Returns
    /// * `Ok(manager)` - If initialization succeeded
    /// * `Err(IoError)` - If directory creation failed
    pub fn new<Q: AsRef<Path>>(root_path: Q) -> NetabaseResult<Self> {
        Ok(Self {
            inner: DefinitionManager::new(root_path)?,
            _marker: PhantomData,
        })
    }

    /// Load a definition store from disk
    ///
    /// This opens the Redb database at the appropriate path and marks
    /// the definition as loaded.
    ///
    /// # Arguments
    /// * `discriminant` - Which definition to load
    ///
    /// # Returns
    /// * `Ok(())` - If the store was loaded successfully
    /// * `Err(RedbError)` - If database opening failed
    /// * `Err(DefinitionNotFound)` - If the discriminant is invalid
    pub fn load_definition(
        &mut self,
        discriminant: <D as IntoDiscriminant>::Discriminant,
    ) -> NetabaseResult<()> {
        // Check if already loaded
        if self.inner.is_loaded(&discriminant) {
            return Ok(());
        }

        // Get the path for this definition's database
        let path = self
            .inner
            .root_path()
            .join(discriminant.name())
            .join("store.db");

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Open the Redb database
        let store = RedbStore::new(&path)?;

        // Update the store link to Loaded state
        if let Some(link) = self.inner.stores.get_mut(&discriminant) {
            link.load(store);
        }

        Ok(())
    }

    /// Unload a definition store
    ///
    /// This closes the database connection and marks the definition as unloaded.
    ///
    /// # Arguments
    /// * `discriminant` - Which definition to unload
    ///
    /// # Returns
    /// * `Ok(())` - If the store was unloaded (or was already unloaded)
    pub fn unload_definition(
        &mut self,
        discriminant: <D as IntoDiscriminant>::Discriminant,
    ) -> NetabaseResult<()> {
        if let Some(link) = self.inner.stores.get_mut(&discriminant) {
            link.unload();
        }
        Ok(())
    }

    /// Check if a definition is currently loaded
    pub fn is_loaded(&self, discriminant: &<D as IntoDiscriminant>::Discriminant) -> bool {
        self.inner.is_loaded(discriminant)
    }

    /// Get all loaded definition discriminants
    pub fn loaded_definitions(&self) -> Vec<&<D as IntoDiscriminant>::Discriminant> {
        self.inner.loaded_definitions()
    }

    /// Get the root path
    pub fn root_path(&self) -> &Path {
        self.inner.root_path()
    }

    /// Mark a definition for eager loading
    pub fn add_warm_on_access(&mut self, discriminant: <D as IntoDiscriminant>::Discriminant) {
        self.inner.add_warm_on_access(discriminant);
    }

    /// Get manager statistics
    ///
    /// # Returns
    /// Tuple of (total_definitions, loaded_count, warm_count)
    pub fn stats(&self) -> (usize, usize, usize) {
        self.inner.stats()
    }

    /// Unload all unused definitions
    ///
    /// # Returns
    /// Number of definitions that were unloaded
    pub fn unload_unused(&mut self) -> usize {
        self.inner.unload_unused()
    }

    /// Unload all definitions
    ///
    /// # Returns
    /// Number of definitions that were unloaded
    pub fn unload_all(&mut self) -> usize {
        self.inner.unload_all()
    }

    /// Execute a read transaction with the given permission
    ///
    /// Creates a multi-definition read transaction that can access multiple
    /// definition stores based on the provided permission.
    ///
    /// # Arguments
    /// * `permission` - The permission scope for this transaction
    /// * `f` - Closure that receives the read transaction
    ///
    /// # Returns
    /// * `Ok(result)` - If the transaction succeeds
    /// * `Err(...)` - If the transaction fails
    ///
    /// # Example
    /// ```ignore
    /// manager.read(permission, |txn| {
    ///     let result = txn.definition_txn(&UserDiscriminant, |user_txn| {
    ///         user_txn.get(user_id)
    ///     })?;
    ///     Ok(result)
    /// })?;
    /// ```
    pub fn read<F, Ret>(&mut self, permission: P, f: F) -> NetabaseResult<Ret>
    where
        F: FnOnce(&transaction::RedbMultiDefReadTxn<'_, R, D, P>) -> NetabaseResult<Ret>,
    {
        let txn = transaction::RedbMultiDefReadTxn::new(self, permission);
        f(&txn)
    }

    /// Execute a write transaction with the given permission
    ///
    /// Creates a multi-definition write transaction that can access and modify
    /// multiple definition stores based on the provided permission.
    ///
    /// The transaction commits all changes when the closure completes successfully.
    /// If the closure returns an error, changes are not committed.
    ///
    /// # Arguments
    /// * `permission` - The permission scope for this transaction
    /// * `f` - Closure that receives the write transaction
    ///
    /// # Returns
    /// * `Ok(result)` - If the transaction succeeds and commits
    /// * `Err(...)` - If the transaction fails or commit fails
    ///
    /// # Example
    /// ```ignore
    /// manager.write(permission, |txn| {
    ///     txn.definition_txn_mut(&UserDiscriminant, |user_txn| {
    ///         user_txn.put(user)
    ///     })?;
    ///     Ok(())
    /// })?;
    /// ```
    pub fn write<F, Ret>(&mut self, permission: P, f: F) -> NetabaseResult<Ret>
    where
        F: FnOnce(&mut transaction::RedbMultiDefWriteTxn<'_, R, D, P>) -> NetabaseResult<Ret>,
    {
        let mut txn = transaction::RedbMultiDefWriteTxn::new(self, permission);
        let result = f(&mut txn)?;
        txn.commit()?;
        Ok(result)
    }

    /// Generate the root TOML file for this manager
    ///
    /// This creates or updates the file at:
    /// `<root_path>/<manager_name>.root.netabase.toml`
    ///
    /// The file contains metadata about the manager, all available definitions,
    /// and current state (loaded definitions, warm-on-access hints).
    ///
    /// # Returns
    /// * `Ok(())` - If the file was written successfully
    /// * `Err(...)` - If file generation failed
    ///
    /// # Example
    /// ```ignore
    /// let mut manager = RedbDefinitionManager::new("./data")?;
    /// manager.generate_root_toml()?;
    /// // File created at: ./data/MyManager.root.netabase.toml
    /// ```
    pub fn generate_root_toml(&self) -> NetabaseResult<()> {
        self.inner.generate_root_toml()
    }

    /// Generate a definition TOML file for a specific definition
    ///
    /// This creates or updates the file at:
    /// `<root_path>/<definition_name>/<definition_name>.netabase.toml`
    ///
    /// The file contains metadata about the definition including its
    /// structure, version, and schema hash.
    ///
    /// # Arguments
    /// * `discriminant` - Which definition to generate TOML for
    ///
    /// # Returns
    /// * `Ok(())` - If the file was written successfully
    /// * `Err(...)` - If file generation failed
    pub fn generate_definition_toml(
        &self,
        discriminant: &<D as IntoDiscriminant>::Discriminant,
    ) -> NetabaseResult<()> {
        self.inner.generate_definition_toml(discriminant)
    }

    /// Generate TOML files for all definitions
    ///
    /// This is a convenience method that generates definition TOML files
    /// for every definition in the system.
    ///
    /// # Returns
    /// * `Ok(count)` - Number of definition TOML files generated
    /// * `Err(...)` - If any file generation failed
    pub fn generate_all_definition_tomls(&self) -> NetabaseResult<usize> {
        self.inner.generate_all_definition_tomls()
    }

    /// Read and parse the root TOML file
    ///
    /// # Returns
    /// * `Ok(RootToml)` - If the file exists and was parsed successfully
    /// * `Err(...)` - If the file doesn't exist, can't be read, or parsing failed
    pub fn read_root_toml(&self) -> NetabaseResult<crate::databases::manager::toml_types::RootToml> {
        self.inner.read_root_toml()
    }

    /// Read and parse a definition TOML file
    ///
    /// # Arguments
    /// * `discriminant` - Which definition to read
    ///
    /// # Returns
    /// * `Ok(DefinitionToml)` - If the file exists and was parsed successfully
    /// * `Err(...)` - If the file doesn't exist, can't be read, or parsing failed
    pub fn read_definition_toml(
        &self,
        discriminant: &<D as IntoDiscriminant>::Discriminant,
    ) -> NetabaseResult<crate::databases::manager::toml_types::DefinitionToml> {
        self.inner.read_definition_toml(discriminant)
    }

    /// Check if a definition's schema has changed
    ///
    /// Compares the current schema hash with the one stored in the TOML file.
    ///
    /// # Arguments
    /// * `discriminant` - Which definition to check
    ///
    /// # Returns
    /// * `Ok(true)` - If the schema has changed
    /// * `Ok(false)` - If the schema is unchanged
    /// * `Err(...)` - If the TOML file can't be read
    pub fn has_schema_changed(
        &self,
        discriminant: &<D as IntoDiscriminant>::Discriminant,
    ) -> NetabaseResult<bool> {
        self.inner.has_schema_changed(discriminant)
    }
}

impl<R, D, P> Debug for RedbDefinitionManager<R, D, P>
where
    R: DefinitionManagerTrait<Definition = D, Permissions = P, Backend = RedbStore<D>>,
    D: NetabaseDefinition,
    P: PermissionEnumTrait,
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <R as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <P as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedbDefinitionManager")
            .field("inner", &self.inner)
            .finish()
    }
}
