//! Basic store operations trait for tree-level data access
//!
//! This module provides a low-level trait that abstracts the core operations
//! needed for put and get requests at the tree level. This trait is used by
//! the `NetabaseStore` macro to generate efficient RecordStore implementations.

use crate::error::NetabaseError;
use crate::traits::model::NetabaseModelTrait;
use crate::traits::definition::NetabaseDefinitionTrait;

/// Core store operations for a single tree/table.
///
/// This trait provides the fundamental operations needed to interact with a single
/// tree (table) in the database. It is designed to be implemented by tree types
/// (e.g., `SledStoreTree`, `RedbStoreTree`) and provides the foundation for
/// higher-level operations.
///
/// # Type Parameters
///
/// * `D` - The definition type (wraps all models in the schema)
/// * `M` - The model type stored in this tree
///
/// # Design
///
/// This trait focuses on raw data access without the Definition enum wrapper.
/// When data is stored, it should be stored as the raw model type, not wrapped
/// in the Definition enum. This provides:
///
/// - Better performance (no enum wrapping/unwrapping)
/// - Consistent data format (same format with or without RecordStore layer)
/// - Simpler debugging (raw model data in database)
///
/// # Usage
///
/// This trait is primarily used by the `NetabaseStore` macro to generate
/// RecordStore implementations. Users typically interact with higher-level
/// APIs like `NetabaseTreeSync` or `RecordStore`.
pub trait StoreOps<D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
    <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + std::str::FromStr,
{
    /// Insert or update a raw model in the tree
    ///
    /// # Arguments
    ///
    /// * `model` - The model to insert or update
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation succeeded
    /// * `Err(NetabaseError)` if the operation failed
    ///
    /// # Implementation Notes
    ///
    /// - The model should be stored directly, not wrapped in a Definition enum
    /// - Primary key is extracted from the model
    /// - Secondary key indexes should be updated
    /// - The operation should be atomic
    fn put_raw(&self, model: M) -> Result<(), NetabaseError>;

    /// Get a raw model by its primary key
    ///
    /// # Arguments
    ///
    /// * `key` - The primary key of the model to retrieve
    ///
    /// # Returns
    ///
    /// * `Ok(Some(model))` if the model was found
    /// * `Ok(None)` if the model was not found
    /// * `Err(NetabaseError)` if the operation failed
    ///
    /// # Implementation Notes
    ///
    /// - The model should be retrieved directly, not wrapped in a Definition enum
    fn get_raw(&self, key: M::PrimaryKey) -> Result<Option<M>, NetabaseError>;

    /// Delete a model by its primary key
    ///
    /// # Arguments
    ///
    /// * `key` - The primary key of the model to delete
    ///
    /// # Returns
    ///
    /// * `Ok(Some(model))` if the model was deleted (returns the deleted model)
    /// * `Ok(None)` if the model was not found
    /// * `Err(NetabaseError)` if the operation failed
    ///
    /// # Implementation Notes
    ///
    /// - The operation should clean up secondary key indexes
    /// - The operation should be atomic
    fn remove_raw(&self, key: M::PrimaryKey) -> Result<Option<M>, NetabaseError>;

    /// Get the discriminant name for this tree
    ///
    /// # Returns
    ///
    /// The discriminant string used as the tree/table name
    ///
    /// # Implementation Notes
    ///
    /// This is used to identify which tree/table this instance operates on.
    /// For example, "User", "Post", "Comment", etc.
    fn discriminant(&self) -> &str;
}

/// Extended store operations for secondary key access
///
/// This trait extends `StoreOps` with support for secondary key queries.
/// It is optional and should be implemented by stores that support secondary indexes.
pub trait StoreOpsSecondary<D, M>: StoreOps<D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
    <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + std::str::FromStr,
{
    /// Find models by secondary key
    ///
    /// # Arguments
    ///
    /// * `secondary_key` - The secondary key to search for
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<M>)` - Vector of all models matching the secondary key
    /// * `Err(NetabaseError)` if the operation failed
    fn get_by_secondary_key_raw(
        &self,
        secondary_key: M::SecondaryKeys,
    ) -> Result<Vec<M>, NetabaseError>;
}

/// Store operations for iteration
///
/// This trait provides methods for iterating over all records in a tree.
/// It is optional and should be implemented by stores that support iteration.
pub trait StoreOpsIter<D, M>: StoreOps<D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
    <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + std::str::FromStr,
{
    /// Iterate over all models in the tree
    ///
    /// # Returns
    ///
    /// An iterator over all models in the tree
    ///
    /// # Implementation Notes
    ///
    /// - The iterator should yield models in an implementation-defined order
    /// - The iterator may fail, so it should return `Result<M, NetabaseError>`
    type Iter: Iterator<Item = Result<M, NetabaseError>>;

    /// Create an iterator over all models in the tree
    fn iter(&self) -> Result<Self::Iter, NetabaseError>;

    /// Get the number of models in the tree
    ///
    /// # Returns
    ///
    /// * `Ok(count)` - The number of models in the tree
    /// * `Err(NetabaseError)` if the operation failed
    fn len(&self) -> Result<usize, NetabaseError>;

    /// Check if the tree is empty
    ///
    /// # Returns
    ///
    /// * `Ok(true)` if the tree contains no models
    /// * `Ok(false)` if the tree contains at least one model
    /// * `Err(NetabaseError)` if the operation failed
    fn is_empty(&self) -> Result<bool, NetabaseError> {
        Ok(self.len()? == 0)
    }
}
