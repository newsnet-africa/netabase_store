use crate::databases::sled_store::manager::SledDefinitionManager;
use crate::databases::sled_store::{SledStore, SledReadTransaction, SledWriteTransaction, SledStoreTrait};
use crate::error::NetabaseResult;
use crate::traits::definition::{DiscriminantName, NetabaseDefinition};
use crate::traits::manager::DefinitionManagerTrait;
use crate::traits::permission::PermissionEnumTrait;
use std::fmt::Debug;
use strum::{IntoDiscriminant, IntoEnumIterator};

/// Read transaction for multi-definition access with Sled backend
///
/// This transaction coordinates read access across multiple definition stores,
/// loading them on-demand as they're accessed.
pub struct SledMultiDefReadTxn<'a, R, D, P>
where
    R: DefinitionManagerTrait<Definition = D, Permissions = P, Backend = SledStore<D>>,
    D: NetabaseDefinition,
    P: PermissionEnumTrait,
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <R as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <P as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + Clone,
{
    manager: &'a mut SledDefinitionManager<R, D, P>,
    permission: P,
}

impl<'a, R, D, P> SledMultiDefReadTxn<'a, R, D, P>
where
    R: DefinitionManagerTrait<Definition = D, Permissions = P, Backend = SledStore<D>>,
    D: NetabaseDefinition,
    P: PermissionEnumTrait,
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <R as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <P as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + Clone,
{
    /// Create a new multi-definition read transaction
    ///
    /// # Arguments
    /// * `manager` - The definition manager
    /// * `permission` - The permission scope for this transaction
    pub fn new(manager: &'a mut SledDefinitionManager<R, D, P>, permission: P) -> Self {
        Self { manager, permission }
    }

    /// Ensure a definition is loaded before accessing it
    ///
    /// # Arguments
    /// * `discriminant` - Which definition to load
    ///
    /// # Returns
    /// * `Ok(())` - If the definition is loaded
    /// * `Err(...)` - If loading failed
    #[allow(dead_code)]
    fn ensure_loaded(
        &mut self,
        discriminant: &<D as IntoDiscriminant>::Discriminant,
    ) -> NetabaseResult<()> {
        if !self.manager.is_loaded(discriminant) {
            self.manager.load_definition(discriminant.clone())?;
        }
        self.manager.inner.mark_accessed(discriminant.clone());
        Ok(())
    }
}

impl<'a, R, D, P> SledMultiDefReadTxn<'a, R, D, P>
where
    R: DefinitionManagerTrait<Definition = D, Permissions = P, Backend = SledStore<D>>,
    D: NetabaseDefinition,
    P: PermissionEnumTrait,
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <R as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <P as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + Clone,
{
    /// Get the current permission scope for this transaction
    pub fn permission(&self) -> &P {
        &self.permission
    }

    /// Access a read transaction for a specific definition
    ///
    /// This method loads the definition store if it's not already loaded,
    /// then provides access to its read transaction.
    ///
    /// # Permission Checking
    /// Runtime permission checking is performed using the transaction's permission object.
    /// The permission must grant read access to the requested definition.
    ///
    /// # Arguments
    /// * `discriminant` - Which definition to access
    /// * `f` - Closure that receives the read transaction
    ///
    /// # Returns
    /// * `Ok(result)` - If permission is granted and operation succeeds
    /// * `Err(NetabaseError::PermissionDenied)` - If permission check fails
    pub fn definition_txn<F, Ret>(
        &self,
        discriminant: &<D as IntoDiscriminant>::Discriminant,
        f: F,
    ) -> NetabaseResult<Ret>
    where
        for<'b> F: FnOnce(&SledReadTransaction<'b, D>) -> NetabaseResult<Ret>,
        D: 'static,
    {
        // Check read permission at runtime
        if !self.permission.can_read_definition::<D>(discriminant) {
            return Err(crate::error::NetabaseError::PermissionDenied(format!(
                "Permission does not grant read access to definition: {:?}",
                discriminant
            )));
        }

        // Get the store (will fail if not loaded)
        let store = self.manager.inner.get_store(discriminant)?;

        // Execute the read transaction on the store
        store.read(f)
    }

    /// Get all loaded definition discriminants
    pub fn loaded_definitions(&self) -> Vec<&<D as IntoDiscriminant>::Discriminant> {
        self.manager.loaded_definitions()
    }
}

/// Write transaction for multi-definition access with Sled backend
///
/// This transaction coordinates write access across multiple definition stores,
/// loading them on-demand and committing all changes atomically.
pub struct SledMultiDefWriteTxn<'a, R, D, P>
where
    R: DefinitionManagerTrait<Definition = D, Permissions = P, Backend = SledStore<D>>,
    D: NetabaseDefinition,
    P: PermissionEnumTrait,
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <R as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <P as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + Clone,
{
    manager: &'a mut SledDefinitionManager<R, D, P>,
    permission: P,
}

impl<'a, R, D, P> SledMultiDefWriteTxn<'a, R, D, P>
where
    R: DefinitionManagerTrait<Definition = D, Permissions = P, Backend = SledStore<D>>,
    D: NetabaseDefinition,
    P: PermissionEnumTrait,
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <R as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <P as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + Clone,
{
    /// Create a new multi-definition write transaction
    ///
    /// # Arguments
    /// * `manager` - The definition manager
    /// * `permission` - The permission scope for this transaction
    pub fn new(manager: &'a mut SledDefinitionManager<R, D, P>, permission: P) -> Self {
        Self { manager, permission }
    }

    /// Ensure a definition is loaded before accessing it
    ///
    /// # Arguments
    /// * `discriminant` - Which definition to load
    ///
    /// # Returns
    /// * `Ok(())` - If the definition is loaded
    /// * `Err(...)` - If loading failed
    #[allow(dead_code)]
    fn ensure_loaded(
        &mut self,
        discriminant: &<D as IntoDiscriminant>::Discriminant,
    ) -> NetabaseResult<()> {
        if !self.manager.is_loaded(discriminant) {
            self.manager.load_definition(discriminant.clone())?;
        }
        self.manager.inner.mark_accessed(discriminant.clone());
        Ok(())
    }
}

impl<'a, R, D, P> SledMultiDefWriteTxn<'a, R, D, P>
where
    R: DefinitionManagerTrait<Definition = D, Permissions = P, Backend = SledStore<D>>,
    D: NetabaseDefinition,
    P: PermissionEnumTrait,
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <R as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <P as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + Clone,
{
    /// Get the current permission scope for this transaction
    pub fn permission(&self) -> &P {
        &self.permission
    }

    /// Access a read transaction for a specific definition
    ///
    /// This provides read-only access even within a write transaction.
    ///
    /// # Permission Checking
    /// Runtime permission checking is performed using the transaction's permission object.
    /// The permission must grant read access to the requested definition.
    pub fn definition_txn<F, Ret>(
        &self,
        discriminant: &<D as IntoDiscriminant>::Discriminant,
        f: F,
    ) -> NetabaseResult<Ret>
    where
        for<'b> F: FnOnce(&SledReadTransaction<'b, D>) -> NetabaseResult<Ret>,
        D: 'static,
    {
        // Check read permission at runtime
        if !self.permission.can_read_definition::<D>(discriminant) {
            return Err(crate::error::NetabaseError::PermissionDenied(format!(
                "Permission does not grant read access to definition: {:?}",
                discriminant
            )));
        }

        // Get the loaded store
        let store = self.manager.inner.get_store(discriminant)?;

        // Execute the read transaction
        store.read(f)
    }

    /// Access a write transaction for a specific definition
    ///
    /// This method loads the definition store if it's not already loaded,
    /// then provides mutable access to its write transaction.
    ///
    /// # Permission Checking
    /// Runtime permission checking is performed using the transaction's permission object.
    /// The permission must grant write access to the requested definition.
    pub fn definition_txn_mut<F, Ret>(
        &mut self,
        discriminant: &<D as IntoDiscriminant>::Discriminant,
        f: F,
    ) -> NetabaseResult<Ret>
    where
        F: FnOnce(&mut SledWriteTransaction<D>) -> NetabaseResult<Ret>,
    {
        // Check write permission at runtime
        if !self.permission.can_write_definition::<D>(discriminant) {
            return Err(crate::error::NetabaseError::PermissionDenied(format!(
                "Permission does not grant write access to definition: {:?}",
                discriminant
            )));
        }

        // Ensure the definition is loaded
        self.ensure_loaded(discriminant)?;

        // Get the loaded store (mutable)
        let store = self.manager.inner.get_store_mut(discriminant)?;

        // Execute the write transaction
        store.write(f)
    }

    /// Get all loaded definition discriminants
    pub fn loaded_definitions(&self) -> Vec<&<D as IntoDiscriminant>::Discriminant> {
        self.manager.loaded_definitions()
    }

    /// Commit all changes across all accessed definitions
    ///
    /// This commits all write transactions and performs cleanup.
    pub fn commit(self) -> NetabaseResult<()> {
        // All individual write transactions have already been committed
        // by the store.write() calls above (Sled flushes on write)

        // Unload unused definitions
        self.manager.unload_unused();

        // Clear the accessed tracking
        self.manager.inner.clear_accessed();

        Ok(())
    }
}
