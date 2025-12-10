use super::DefinitionManagerTrait;
use crate::error::NetabaseResult;
use crate::traits::definition::{DiscriminantName, NetabaseDefinition};
use std::fmt::Debug;
use strum::{IntoDiscriminant, IntoEnumIterator};

/// Transaction that can access multiple definitions with permission checks
///
/// This trait extends the standard read transaction to support accessing
/// multiple definition stores within a single transaction context.
///
/// # Type Parameters
/// * `R` - The manager type coordinating the definitions
///
/// # Example
/// ```ignore
/// manager.read(permission, |txn| {
///     // Access User definition
///     let user_txn = txn.definition_txn::<User, true>(
///         &RestaurantDefinitionsDiscriminants::User
///     )?;
///     let user = user_txn.get(user_id)?;
///
///     // Access Product definition
///     let product_txn = txn.definition_txn::<Product, true>(
///         &RestaurantDefinitionsDiscriminants::Product
///     )?;
///     let products = product_txn.get_subscription_keys(...)?;
///
///     Ok((user, products))
/// })?;
/// ```
pub trait MultiDefReadTransaction<R>
where
    R: DefinitionManagerTrait,
    <R as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    /// Get the current permission scope for this transaction
    ///
    /// # Returns
    /// Reference to the permission enum that was used to create this transaction
    fn permission(&self) -> &R::Permissions;

    /// Access a read transaction for a specific definition
    ///
    /// This method loads the definition store if it's not already loaded,
    /// then provides access to its read transaction.
    ///
    /// # Type Parameters
    /// * `D` - The definition type (must match R::Definition)
    /// * `PERM_CHECK` - Const bool: if true, compile-time permission check is performed
    /// * `F` - Closure that operates on the read transaction
    /// * `Ret` - Return type of the closure
    ///
    /// # Arguments
    /// * `discriminant` - Which definition to access
    /// * `f` - Closure that receives the read transaction
    ///
    /// # Returns
    /// * `Ok(result)` - If permission is granted and operation succeeds
    /// * `Err(NetabaseError::PermissionDenied)` - If permission check fails
    /// * `Err(NetabaseError::StoreNotLoaded)` - If store fails to load
    ///
    /// # Permission Checking
    /// When `PERM_CHECK` is `true`, the compiler will verify that the permission
    /// type implements `GrantsReadAccess<D>`. When `false`, runtime checking is used.
    ///
    /// # Example
    /// ```ignore
    /// // Compile-time check (recommended)
    /// let user = txn.definition_txn::<User, true, _, _>(&UserDiscriminant, |txn| {
    ///     txn.get(user_id)
    /// })?;
    /// ```
    fn definition_txn<D, const PERM_CHECK: bool, F, Ret>(
        &self,
        discriminant: &<R::Definition as strum::IntoDiscriminant>::Discriminant,
        f: F,
    ) -> NetabaseResult<Ret>
    where
        D: NetabaseDefinition,
        <D as strum::IntoDiscriminant>::Discriminant:
            strum::IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
        R::Backend: crate::traits::store::store::StoreTrait<R::Definition>,
        F: for<'txn> FnOnce(&<R::Backend as crate::traits::store::store::StoreTrait<R::Definition>>::ReadTxn<'txn>) -> NetabaseResult<Ret>;

    /// Get all loaded definition discriminants
    ///
    /// This can be used to see which definitions have been accessed
    /// during this transaction.
    ///
    /// # Returns
    /// Vector of definition discriminants that are currently loaded
    fn loaded_definitions(&self) -> Vec<&<R::Definition as strum::IntoDiscriminant>::Discriminant>;
}

/// Write transaction for multi-definition access
///
/// This trait extends both `MultiDefReadTransaction` and provides write access
/// to multiple definition stores within a single transaction context.
///
/// All writes across all definitions are committed atomically when the
/// transaction is committed.
///
/// # Type Parameters
/// * `R` - The manager type coordinating the definitions
///
/// # Example
/// ```ignore
/// manager.write(permission, |txn| {
///     // Write to User definition
///     let user_txn = txn.definition_txn_mut::<User, true>(
///         &RestaurantDefinitionsDiscriminants::User
///     )?;
///     user_txn.put(user)?;
///
///     // Write to Product definition
///     let product_txn = txn.definition_txn_mut::<Product, true>(
///         &RestaurantDefinitionsDiscriminants::Product
///     )?;
///     product_txn.put(product)?;
///
///     Ok(())
/// })?;
/// // All writes committed here
/// ```
pub trait MultiDefWriteTransaction<R>: MultiDefReadTransaction<R>
where
    R: DefinitionManagerTrait,
    <R as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    /// Access a write transaction for a specific definition
    ///
    /// This method loads the definition store if it's not already loaded,
    /// then provides mutable access to its write transaction.
    ///
    /// # Type Parameters
    /// * `D` - The definition type (must match R::Definition)
    /// * `PERM_CHECK` - Const bool: if true, compile-time permission check is performed
    /// * `F` - Closure that operates on the write transaction
    /// * `Ret` - Return type of the closure
    ///
    /// # Arguments
    /// * `discriminant` - Which definition to access
    /// * `f` - Closure that receives the write transaction
    ///
    /// # Returns
    /// * `Ok(result)` - If permission is granted and operation succeeds
    /// * `Err(NetabaseError::PermissionDenied)` - If permission check fails
    /// * `Err(NetabaseError::StoreNotLoaded)` - If store fails to load
    ///
    /// # Permission Checking
    /// When `PERM_CHECK` is `true`, the compiler will verify that the permission
    /// type implements `GrantsWriteAccess<D>`. When `false`, runtime checking is used.
    ///
    /// # Example
    /// ```ignore
    /// // Compile-time check (recommended)
    /// txn.definition_txn_mut::<User, true, _, _>(&UserDiscriminant, |txn| {
    ///     txn.put(user)
    /// })?;
    /// ```
    fn definition_txn_mut<D, const PERM_CHECK: bool, F, Ret>(
        &mut self,
        discriminant: &<R::Definition as strum::IntoDiscriminant>::Discriminant,
        f: F,
    ) -> NetabaseResult<Ret>
    where
        D: NetabaseDefinition,
        <D as strum::IntoDiscriminant>::Discriminant:
            strum::IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
        R::Backend: crate::traits::store::store::StoreTrait<R::Definition>,
        F: FnOnce(&mut <R::Backend as crate::traits::store::store::StoreTrait<R::Definition>>::WriteTxn) -> NetabaseResult<Ret>;

    /// Commit all changes across all accessed definitions
    ///
    /// This commits all write transactions atomically. If any definition
    /// fails to commit, all definitions should roll back (best effort).
    ///
    /// # Returns
    /// * `Ok(())` - If all definitions committed successfully
    /// * `Err(...)` - If any definition failed to commit
    ///
    /// # Note
    /// The exact atomicity guarantees depend on the backend implementation.
    /// Redb provides ACID guarantees per-database, but cross-database
    /// atomicity is not guaranteed at the filesystem level.
    fn commit(self) -> NetabaseResult<()>;
}

/// Helper trait for definition access within transactions
///
/// This trait can be implemented by transaction types to provide
/// common functionality for accessing definitions.
pub trait DefinitionAccess<R>
where
    R: DefinitionManagerTrait,
    <R as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    /// Check if a definition is currently loaded in this transaction context
    fn is_definition_loaded(
        &self,
        discriminant: &<R::Definition as strum::IntoDiscriminant>::Discriminant,
    ) -> bool;

    /// Load a definition if not already loaded
    ///
    /// This is an internal method used by `definition_txn` to ensure
    /// the definition store is loaded before access.
    fn ensure_definition_loaded(
        &mut self,
        discriminant: &<R::Definition as strum::IntoDiscriminant>::Discriminant,
    ) -> NetabaseResult<()>;

    /// Check permission at runtime
    ///
    /// This is used when `PERM_CHECK = false` to perform runtime permission validation.
    fn check_permission_runtime(
        &self,
        discriminant: &<R::Definition as strum::IntoDiscriminant>::Discriminant,
        write_access: bool,
    ) -> NetabaseResult<()>;
}
