use crate::error::NetabaseResult;
use crate::traits::definition::{DiscriminantName, NetabaseDefinition};
use crate::traits::manager::{DefinitionManagerTrait, store_link::DefinitionStoreLink};
use crate::traits::permission::PermissionEnumTrait;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::path::PathBuf;
use strum::{IntoDiscriminant, IntoEnumIterator};

pub mod toml;
pub mod toml_types;

/// Generic manager for coordinating multiple definition stores
///
/// This struct manages a collection of definition databases, loading them
/// on-demand and tracking which definitions are currently loaded.
///
/// # Type Parameters
/// * `R` - The manager trait implementation
/// * `D` - The definition enum type
/// * `P` - The permission enum type
/// * `B` - The backend store type (RedbStore<D> or SledStore<D>)
///
/// # Example
/// ```ignore
/// let manager = DefinitionManager::<
///     RestaurantManager,
///     RestaurantDefinitions,
///     RestaurantPermissions,
///     RedbStore<RestaurantDefinitions>
/// >::new("./data")?;
/// ```
pub struct DefinitionManager<R, D, P, B>
where
    R: DefinitionManagerTrait<Definition = D, Permissions = P, Backend = B>,
    D: NetabaseDefinition,
    P: PermissionEnumTrait,
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <R as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <P as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + Clone,
{
    /// Root directory path where all definition databases are stored
    root_path: PathBuf,

    /// Map of definition discriminants to their stores (loaded or unloaded)
    pub(crate) stores: HashMap<
        <D as IntoDiscriminant>::Discriminant,
        DefinitionStoreLink<D, B>,
    >,

    /// Definitions that should be eagerly loaded when accessed
    /// Used for performance optimization when cross-definition access is known
    warm_on_access: HashSet<<D as IntoDiscriminant>::Discriminant>,

    /// Track which definitions were accessed in the current transaction
    /// Used for auto-close functionality
    accessed_in_transaction: HashSet<<D as IntoDiscriminant>::Discriminant>,

    _marker: PhantomData<(R, P)>,
}

impl<R, D, P, B> DefinitionManager<R, D, P, B>
where
    R: DefinitionManagerTrait<Definition = D, Permissions = P, Backend = B>,
    D: NetabaseDefinition,
    P: PermissionEnumTrait,
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <R as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <P as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + Clone,
{
    /// Create a new definition manager at the given root path
    ///
    /// All definition stores will be created as subdirectories under this path.
    /// Initially, all definitions are in the Unloaded state.
    ///
    /// # Arguments
    /// * `root_path` - Path to the parent directory for all definition databases
    ///
    /// # Returns
    /// * `Ok(manager)` - If the root directory was created successfully
    /// * `Err(IoError)` - If directory creation failed
    ///
    /// # Example
    /// ```ignore
    /// let manager = DefinitionManager::new("./restaurant_data")?;
    /// ```
    pub fn new<Q: AsRef<std::path::Path>>(root_path: Q) -> NetabaseResult<Self> {
        let root_path = root_path.as_ref().to_path_buf();

        // Create root directory if it doesn't exist
        std::fs::create_dir_all(&root_path)?;

        // Initialize all definitions as Unloaded
        let stores = <D as IntoDiscriminant>::Discriminant::iter()
            .map(|disc| (disc.clone(), DefinitionStoreLink::Unloaded(disc)))
            .collect();

        Ok(Self {
            root_path,
            stores,
            warm_on_access: HashSet::new(),
            accessed_in_transaction: HashSet::new(),
            _marker: PhantomData,
        })
    }

    /// Get the root path where all definition databases are stored
    pub fn root_path(&self) -> &std::path::Path {
        &self.root_path
    }

    /// Check if a definition is currently loaded
    ///
    /// # Arguments
    /// * `discriminant` - The definition to check
    ///
    /// # Returns
    /// `true` if the definition store is loaded in memory
    pub fn is_loaded(&self, discriminant: &<D as IntoDiscriminant>::Discriminant) -> bool {
        self.stores
            .get(discriminant)
            .map(|link| link.is_loaded())
            .unwrap_or(false)
    }

    /// Get all currently loaded definition discriminants
    ///
    /// # Returns
    /// Vector of discriminants for definitions that are currently loaded
    pub fn loaded_definitions(&self) -> Vec<&<D as IntoDiscriminant>::Discriminant> {
        self.stores
            .iter()
            .filter_map(|(disc, link)| {
                if link.is_loaded() {
                    Some(disc)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get a reference to a loaded store
    ///
    /// # Arguments
    /// * `discriminant` - The definition to get
    ///
    /// # Returns
    /// * `Ok(&B)` - If the store is loaded
    /// * `Err(StoreNotLoaded)` - If the store is not loaded
    pub fn get_store(
        &self,
        discriminant: &<D as IntoDiscriminant>::Discriminant,
    ) -> NetabaseResult<&B> {
        self.stores
            .get(discriminant)
            .ok_or_else(|| {
                crate::error::NetabaseError::DefinitionNotFound(discriminant.name().to_string())
            })?
            .get_store()
    }

    /// Get a mutable reference to a loaded store
    ///
    /// # Arguments
    /// * `discriminant` - The definition to get
    ///
    /// # Returns
    /// * `Ok(&mut B)` - If the store is loaded
    /// * `Err(StoreNotLoaded)` - If the store is not loaded
    pub fn get_store_mut(
        &mut self,
        discriminant: &<D as IntoDiscriminant>::Discriminant,
    ) -> NetabaseResult<&mut B> {
        self.stores
            .get_mut(discriminant)
            .ok_or_else(|| {
                crate::error::NetabaseError::DefinitionNotFound(discriminant.name().to_string())
            })?
            .get_store_mut()
    }

    /// Mark a definition for eager loading
    ///
    /// When a definition is in the warm_on_access set, it will be automatically
    /// loaded the first time it's accessed in any transaction.
    ///
    /// # Arguments
    /// * `discriminant` - The definition to mark for warming
    pub fn add_warm_on_access(&mut self, discriminant: <D as IntoDiscriminant>::Discriminant) {
        self.warm_on_access.insert(discriminant);
    }

    /// Remove a definition from the eager loading set
    ///
    /// # Arguments
    /// * `discriminant` - The definition to remove from warming
    pub fn remove_warm_on_access(
        &mut self,
        discriminant: &<D as IntoDiscriminant>::Discriminant,
    ) {
        self.warm_on_access.remove(discriminant);
    }

    /// Mark a definition as accessed in the current transaction
    ///
    /// This is used internally to track which definitions should remain loaded
    /// after the transaction completes.
    ///
    /// # Arguments
    /// * `discriminant` - The definition that was accessed
    pub(crate) fn mark_accessed(&mut self, discriminant: <D as IntoDiscriminant>::Discriminant) {
        self.accessed_in_transaction.insert(discriminant);
    }

    /// Clear the transaction access tracking
    ///
    /// This should be called at the end of each transaction.
    pub(crate) fn clear_accessed(&mut self) {
        self.accessed_in_transaction.clear();
    }

    /// Get the set of definitions accessed in the current transaction
    #[allow(dead_code)]
    pub(crate) fn accessed_definitions(
        &self,
    ) -> &HashSet<<D as IntoDiscriminant>::Discriminant> {
        &self.accessed_in_transaction
    }

    /// Unload all definitions that were not accessed in the last transaction
    ///
    /// This is called automatically after write transactions to free resources.
    /// Definitions in the `warm_on_access` set are never unloaded.
    ///
    /// # Returns
    /// Number of definitions that were unloaded
    pub fn unload_unused(&mut self) -> usize {
        let to_unload: Vec<_> = self
            .stores
            .iter()
            .filter_map(|(disc, link)| {
                if link.is_loaded()
                    && !self.accessed_in_transaction.contains(disc)
                    && !self.warm_on_access.contains(disc)
                {
                    Some(disc.clone())
                } else {
                    None
                }
            })
            .collect();

        let count = to_unload.len();
        for disc in to_unload {
            if let Some(link) = self.stores.get_mut(&disc) {
                link.unload();
            }
        }

        count
    }

    /// Unload all definitions
    ///
    /// This closes all database connections. Use this for cleanup or
    /// when you need to free all resources.
    ///
    /// # Returns
    /// Number of definitions that were unloaded
    pub fn unload_all(&mut self) -> usize {
        let mut count = 0;
        for link in self.stores.values_mut() {
            if link.is_loaded() {
                link.unload();
                count += 1;
            }
        }
        count
    }

    /// Get statistics about the manager state
    ///
    /// # Returns
    /// Tuple of (total_definitions, loaded_count, warm_count)
    pub fn stats(&self) -> (usize, usize, usize) {
        let total = self.stores.len();
        let loaded = self.loaded_definitions().len();
        let warm = self.warm_on_access.len();
        (total, loaded, warm)
    }
}

impl<R, D, P, B> Debug for DefinitionManager<R, D, P, B>
where
    R: DefinitionManagerTrait<Definition = D, Permissions = P, Backend = B>,
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
        let (total, loaded, warm) = self.stats();
        f.debug_struct("DefinitionManager")
            .field("root_path", &self.root_path)
            .field("total_definitions", &total)
            .field("loaded", &loaded)
            .field("warm_on_access", &warm)
            .field("accessed_in_transaction", &self.accessed_in_transaction.len())
            .finish()
    }
}
