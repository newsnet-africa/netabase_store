//! Utility functions for redb zero-copy backend
//!
//! This module provides helper functions for transaction management
//! and table name generation.

use crate::error::NetabaseError;
use crate::traits::definition::NetabaseDefinitionTrait;

use super::store::RedbStoreZeroCopy;
use super::transaction::{RedbReadTransactionZC, RedbWriteTransactionZC};

/// Execute a function within a write transaction scope
///
/// This is a convenience function that automatically handles transaction
/// creation, execution, and commit. If the function returns an error,
/// the transaction is automatically aborted.
///
/// # Arguments
///
/// * `store` - The store to create the transaction from
/// * `f` - The function to execute within the transaction
///
/// # Returns
///
/// The result of the function execution
///
/// # Examples
///
/// ```no_run
/// # use netabase_store::databases::redb_zerocopy::*;
/// # use netabase_store::error::NetabaseError;
/// # use netabase_store::traits::{definition::NetabaseDefinitionTrait, model::NetabaseModelTrait};
/// # struct MyDefinition;
/// # impl NetabaseDefinitionTrait for MyDefinition { type Discriminant = u8; }
/// # struct User { id: u64, name: String }
/// # impl NetabaseModelTrait<MyDefinition> for User {
/// #     type Keys = u64;
/// #     const DISCRIMINANT: u8 = 1;
/// #     fn primary_key(&self) -> Self::Keys { self.id }
/// # }
/// # let store = RedbStoreZeroCopy::<MyDefinition>::new("./test.db")?;
/// let result = with_write_transaction(&store, |txn| {
///     let mut tree = txn.open_tree::<User>()?;
///     tree.put(User { id: 1, name: "Alice".to_string() })?;
///     Ok("Success".to_string())
/// })?;
/// assert_eq!(result, "Success");
/// # Ok::<(), NetabaseError>(())
/// ```
pub fn with_write_transaction<D, F, R>(
    store: &RedbStoreZeroCopy<D>,
    f: F,
) -> Result<R, NetabaseError>
where
    D: NetabaseDefinitionTrait,
    F: FnOnce(&mut RedbWriteTransactionZC<D>) -> Result<R, NetabaseError>,
{
    let mut txn = store.begin_write()?;
    let result = f(&mut txn)?;
    txn.commit()?;
    Ok(result)
}

/// Execute a function within a read transaction scope
///
/// This is a convenience function that automatically handles read transaction
/// creation and cleanup. Read transactions are automatically cleaned up
/// when they go out of scope.
///
/// # Arguments
///
/// * `store` - The store to create the transaction from
/// * `f` - The function to execute within the transaction
///
/// # Returns
///
/// The result of the function execution
///
/// # Examples
///
/// ```no_run
/// # use netabase_store::databases::redb_zerocopy::*;
/// # use netabase_store::error::NetabaseError;
/// # use netabase_store::traits::{definition::NetabaseDefinitionTrait, model::NetabaseModelTrait};
/// # struct MyDefinition;
/// # impl NetabaseDefinitionTrait for MyDefinition { type Discriminant = u8; }
/// # struct User { id: u64, name: String }
/// # impl NetabaseModelTrait<MyDefinition> for User {
/// #     type Keys = u64;
/// #     const DISCRIMINANT: u8 = 1;
/// #     fn primary_key(&self) -> Self::Keys { self.id }
/// # }
/// # let store = RedbStoreZeroCopy::<MyDefinition>::new("./test.db")?;
/// let user = with_read_transaction(&store, |txn| {
///     let tree = txn.open_tree::<User>()?;
///     tree.get(&1)
/// })?;
/// # Ok::<(), NetabaseError>(())
/// ```
pub fn with_read_transaction<D, F, R>(
    store: &RedbStoreZeroCopy<D>,
    f: F,
) -> Result<R, NetabaseError>
where
    D: NetabaseDefinitionTrait,
    F: FnOnce(&RedbReadTransactionZC<D>) -> Result<R, NetabaseError>,
{
    let txn = store.begin_read()?;
    f(&txn)
}

/// Get table name for a discriminant (leaks string to get 'static lifetime)
///
/// This creates a unique table name based on the discriminant value.
/// The string is intentionally leaked to provide a 'static lifetime
/// as required by redb's table definitions.
///
/// # Arguments
///
/// * `discriminant` - The discriminant value to create a table name for
///
/// # Returns
///
/// A 'static string containing the table name
///
/// # Note
///
/// This function intentionally leaks memory to provide the required
/// 'static lifetime. In typical usage, there are only a few different
/// discriminants per application, so the memory leak is minimal.
pub fn get_table_name<D>(discriminant: D::Discriminant) -> &'static str
where
    D: NetabaseDefinitionTrait,
{
    let name = format!("table_{}", discriminant);
    Box::leak(name.into_boxed_str())
}

/// Get secondary table name for a discriminant (leaks string to get 'static lifetime)
///
/// This creates a unique secondary index table name based on the discriminant value.
/// The string is intentionally leaked to provide a 'static lifetime
/// as required by redb's table definitions.
///
/// # Arguments
///
/// * `discriminant` - The discriminant value to create a secondary table name for
///
/// # Returns
///
/// A 'static string containing the secondary table name
///
/// # Note
///
/// This function intentionally leaks memory to provide the required
/// 'static lifetime. In typical usage, there are only a few different
/// discriminants per application, so the memory leak is minimal.
pub fn get_secondary_table_name<D>(discriminant: D::Discriminant) -> &'static str
where
    D: NetabaseDefinitionTrait,
{
    let name = format!("secondary_{}", discriminant);
    Box::leak(name.into_boxed_str())
}

/// Batch operation helper for bulk model insertion
///
/// This utility function provides a convenient way to perform bulk
/// insertions with automatic transaction management.
///
/// # Arguments
///
/// * `store` - The store to operate on
/// * `models` - Vector of models to insert
///
/// # Returns
///
/// Result indicating success or failure
///
/// # Examples
///
/// ```no_run
/// # use netabase_store::databases::redb_zerocopy::*;
/// # use netabase_store::error::NetabaseError;
/// # use netabase_store::traits::{definition::NetabaseDefinitionTrait, model::NetabaseModelTrait};
/// # struct MyDefinition;
/// # impl NetabaseDefinitionTrait for MyDefinition { type Discriminant = u8; }
/// # struct User { id: u64, name: String }
/// # impl NetabaseModelTrait<MyDefinition> for User {
/// #     type Keys = u64;
/// #     const DISCRIMINANT: u8 = 1;
/// #     fn primary_key(&self) -> Self::Keys { self.id }
/// # }
/// # let store = RedbStoreZeroCopy::<MyDefinition>::new("./test.db")?;
/// let users = vec![
///     User { id: 1, name: "Alice".to_string() },
///     User { id: 2, name: "Bob".to_string() },
/// ];
/// bulk_insert(&store, users)?;
/// # Ok::<(), NetabaseError>(())
/// ```
#[allow(dead_code)]
pub fn bulk_insert<D, M>(store: &RedbStoreZeroCopy<D>, models: Vec<M>) -> Result<(), NetabaseError>
where
    D: NetabaseDefinitionTrait,
    M: crate::traits::model::NetabaseModelTrait<D>,
    M::Keys: crate::traits::model::NetabaseModelTraitKey<D>,
{
    with_write_transaction(store, |txn| {
        let mut tree = txn.open_tree::<M>()?;
        tree.put_many(models)?;
        Ok(())
    })
}

/// Batch operation helper for bulk model removal
///
/// This utility function provides a convenient way to perform bulk
/// removals with automatic transaction management.
///
/// # Arguments
///
/// * `store` - The store to operate on
/// * `keys` - Vector of primary keys to remove
///
/// # Returns
///
/// Vector of removed models (Some if existed, None if not found)
///
/// # Examples
///
/// ```no_run
/// # use netabase_store::databases::redb_zerocopy::*;
/// # use netabase_store::error::NetabaseError;
/// # use netabase_store::traits::{definition::NetabaseDefinitionTrait, model::NetabaseModelTrait};
/// # struct MyDefinition;
/// # impl NetabaseDefinitionTrait for MyDefinition { type Discriminant = u8; }
/// # struct User { id: u64, name: String }
/// # impl NetabaseModelTrait<MyDefinition> for User {
/// #     type Keys = u64;
/// #     const DISCRIMINANT: u8 = 1;
/// #     fn primary_key(&self) -> Self::Keys { self.id }
/// # }
/// # let store = RedbStoreZeroCopy::<MyDefinition>::new("./test.db")?;
/// let removed = bulk_remove::<MyDefinition, User>(&store, vec![1, 2, 3])?;
/// println!("Removed {} models", removed.iter().filter(|m| m.is_some()).count());
/// # Ok::<(), NetabaseError>(())
/// ```
#[allow(dead_code)]
pub fn bulk_remove<D, M>(
    store: &RedbStoreZeroCopy<D>,
    keys: Vec<<<M as crate::traits::model::NetabaseModelTrait<D>>::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey>,
) -> Result<Vec<Option<M>>, NetabaseError>
where
    D: NetabaseDefinitionTrait,
    M: crate::traits::model::NetabaseModelTrait<D>,
    M::Keys: crate::traits::model::NetabaseModelTraitKey<D>,
{
    with_write_transaction(store, |txn| {
        let mut tree = txn.open_tree::<M>()?;
        tree.remove_many(keys)
    })
}

// Tests temporarily disabled due to macro resolution issues within the crate itself
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use tempfile::tempdir;
//
//     // Tests would go here but require proper macro setup
// }
