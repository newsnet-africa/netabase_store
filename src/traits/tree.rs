///! Unified trait interface for tree operations across different backends
///!
///! This module provides a uniform API for CRUD operations on netabase store trees,
///! abstracting away the differences between SledStore, RedbStore, and IndexedDBStore.

use crate::error::NetabaseError;

/// Synchronous tree operations trait for native backends (SledStore, RedbStore)
///
/// This trait provides a uniform interface for CRUD operations on a specific model type.
/// All methods are synchronous and suitable for native (non-WASM) environments.
///
/// The generic parameters D and M are not bounded here to avoid duplicating complex trait bounds.
/// Implementors should ensure proper bounds are specified in their impl blocks.
pub trait NetabaseTreeSync<D, M> {
    /// Insert or update a model in the tree
    ///
    /// # Arguments
    /// * `model` - The model to insert or update
    ///
    /// # Returns
    /// * `Ok(())` if the operation succeeded
    /// * `Err(NetabaseError)` if the operation failed
    ///
    /// # Type Parameters
    /// * `PrimaryKey` - The primary key type from model M
    /// * `SecondaryKeys` - The secondary keys type from model M
    type PrimaryKey;
    type SecondaryKeys;

    fn put(&self, model: M) -> Result<(), NetabaseError>;

    /// Get a model by its primary key
    ///
    /// # Arguments
    /// * `key` - The primary key of the model to retrieve
    ///
    /// # Returns
    /// * `Ok(Some(model))` if the model was found
    /// * `Ok(None)` if the model was not found
    /// * `Err(NetabaseError)` if the operation failed
    fn get(&self, key: Self::PrimaryKey) -> Result<Option<M>, NetabaseError>;

    /// Delete a model by its primary key
    ///
    /// # Arguments
    /// * `key` - The primary key of the model to delete
    ///
    /// # Returns
    /// * `Ok(Some(model))` if the model was deleted (returns the deleted model)
    /// * `Ok(None)` if the model was not found
    /// * `Err(NetabaseError)` if the operation failed
    fn remove(&self, key: Self::PrimaryKey) -> Result<Option<M>, NetabaseError>;

    /// Find models by secondary key
    ///
    /// # Arguments
    /// * `secondary_key` - The secondary key to search for
    ///
    /// # Returns
    /// * `Ok(Vec<M>)` - Vector of all models matching the secondary key
    /// * `Err(NetabaseError)` if the operation failed
    fn get_by_secondary_key(&self, secondary_key: Self::SecondaryKeys)
        -> Result<Vec<M>, NetabaseError>;

    /// Check if the tree is empty
    ///
    /// # Returns
    /// * `Ok(true)` if the tree contains no models
    /// * `Ok(false)` if the tree contains at least one model
    /// * `Err(NetabaseError)` if the operation failed
    fn is_empty(&self) -> Result<bool, NetabaseError>;

    /// Get the number of models in the tree
    ///
    /// # Returns
    /// * `Ok(count)` - The number of models in the tree
    /// * `Err(NetabaseError)` if the operation failed
    fn len(&self) -> Result<usize, NetabaseError>;

    /// Clear all models from the tree
    ///
    /// # Returns
    /// * `Ok(())` if the operation succeeded
    /// * `Err(NetabaseError)` if the operation failed
    fn clear(&self) -> Result<(), NetabaseError>;
}

/// Asynchronous tree operations trait for WASM backends (IndexedDBStore)
///
/// This trait provides a uniform interface for CRUD operations on a specific model type.
/// All methods are asynchronous and suitable for WASM environments.
///
/// The generic parameters D and M are not bounded here to avoid duplicating complex trait bounds.
/// Implementors should ensure proper bounds are specified in their impl blocks.
#[cfg(feature = "wasm")]
pub trait NetabaseTreeAsync<D, M> {
    /// Insert or update a model in the tree
    ///
    /// # Arguments
    /// * `model` - The model to insert or update
    ///
    /// # Returns
    /// * `Ok(())` if the operation succeeded
    /// * `Err(NetabaseError)` if the operation failed
    ///
    /// # Type Parameters
    /// * `PrimaryKey` - The primary key type from model M
    /// * `SecondaryKeys` - The secondary keys type from model M
    type PrimaryKey;
    type SecondaryKeys;

    fn put(&self, model: M) -> impl std::future::Future<Output = Result<(), NetabaseError>>;

    /// Get a model by its primary key
    ///
    /// # Arguments
    /// * `key` - The primary key of the model to retrieve
    ///
    /// # Returns
    /// * `Ok(Some(model))` if the model was found
    /// * `Ok(None)` if the model was not found
    /// * `Err(NetabaseError)` if the operation failed
    fn get(
        &self,
        key: Self::PrimaryKey,
    ) -> impl std::future::Future<Output = Result<Option<M>, NetabaseError>>;

    /// Delete a model by its primary key
    ///
    /// # Arguments
    /// * `key` - The primary key of the model to delete
    ///
    /// # Returns
    /// * `Ok(Some(model))` if the model was deleted (returns the deleted model)
    /// * `Ok(None)` if the model was not found
    /// * `Err(NetabaseError)` if the operation failed
    fn remove(
        &self,
        key: Self::PrimaryKey,
    ) -> impl std::future::Future<Output = Result<Option<M>, NetabaseError>>;

    /// Find models by secondary key
    ///
    /// # Arguments
    /// * `secondary_key` - The secondary key to search for
    ///
    /// # Returns
    /// * `Ok(Vec<M>)` - Vector of all models matching the secondary key
    /// * `Err(NetabaseError)` if the operation failed
    fn get_by_secondary_key(
        &self,
        secondary_key: Self::SecondaryKeys,
    ) -> impl std::future::Future<Output = Result<Vec<M>, NetabaseError>>;

    /// Check if the tree is empty
    ///
    /// # Returns
    /// * `Ok(true)` if the tree contains no models
    /// * `Ok(false)` if the tree contains at least one model
    /// * `Err(NetabaseError)` if the operation failed
    fn is_empty(&self) -> impl std::future::Future<Output = Result<bool, NetabaseError>>;

    /// Get the number of models in the tree
    ///
    /// # Returns
    /// * `Ok(count)` - The number of models in the tree
    /// * `Err(NetabaseError)` if the operation failed
    fn len(&self) -> impl std::future::Future<Output = Result<usize, NetabaseError>>;

    /// Clear all models from the tree
    ///
    /// # Returns
    /// * `Ok(())` if the operation succeeded
    /// * `Err(NetabaseError)` if the operation failed
    fn clear(&self) -> impl std::future::Future<Output = Result<(), NetabaseError>>;
}
