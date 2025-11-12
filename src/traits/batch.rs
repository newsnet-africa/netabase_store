//! Batching abstraction for efficient bulk operations
//!
//! This module provides traits for performing batch operations across different
//! backend implementations. Batching is crucial for performance when inserting
//! or updating multiple records.

use crate::error::NetabaseError;
use crate::traits::definition::NetabaseDefinitionTrait;
use crate::traits::model::NetabaseModelTrait;
use crate::{MaybeSend, MaybeSync, NetabaseModelTraitKey};

/// A batch operation builder for accumulating database operations.
///
/// This trait provides a unified interface for building batch operations
/// across different database backends (sled, redb, etc.).
///
/// # Type Parameters
///
/// * `D` - The definition type (wraps all models in the schema)
/// * `M` - The model type for this batch
///
/// # Design
///
/// Different backends have different batching mechanisms:
///
/// - **Sled**: Uses `sled::Batch` for atomic operations
/// - **Redb**: Uses `WriteTransaction` for ACID transactions
///
/// This trait abstracts these differences to provide a consistent API.
///
/// # Example Flow
///
/// ```ignore
/// let mut batch = store.batch_builder();
/// batch.put(user1)?;
/// batch.put(user2)?;
/// batch.remove(old_user_id)?;
/// batch.commit()?;  // All operations applied atomically
/// ```
pub trait BatchBuilder<D, M>
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
        + MaybeSend
        + MaybeSync
        + 'static
        + std::str::FromStr,
{
    /// Add a put operation to the batch
    ///
    /// # Arguments
    ///
    /// * `model` - The model to insert or update
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation was added to the batch
    /// * `Err(NetabaseError)` if the operation could not be added
    ///
    /// # Notes
    ///
    /// - The operation is not immediately executed
    /// - The operation will be executed when `commit()` is called
    fn put(&mut self, model: M) -> Result<(), NetabaseError>;

    /// Add a remove operation to the batch
    ///
    /// # Arguments
    ///
    /// * `key` - The primary key of the model to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation was added to the batch
    /// * `Err(NetabaseError)` if the operation could not be added
    ///
    /// # Notes
    ///
    /// - The operation is not immediately executed
    /// - The operation will be executed when `commit()` is called
    fn remove(
        &mut self,
        key: <M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey,
    ) -> Result<(), NetabaseError>;

    /// Commit all batched operations atomically
    ///
    /// # Returns
    ///
    /// * `Ok(())` if all operations succeeded
    /// * `Err(NetabaseError)` if any operation failed
    ///
    /// # Notes
    ///
    /// - All operations are applied atomically
    /// - If any operation fails, all operations are rolled back
    /// - After commit, the batch is consumed and cannot be reused
    fn commit(self) -> Result<(), NetabaseError>;
}

/// Store operations with batching support
///
/// This trait should be implemented by store types (e.g., `SledStoreTree`,
/// `RedbStoreTree`) to provide batch operation support.
///
/// # Type Parameters
///
/// * `D` - The definition type (wraps all models in the schema)
/// * `M` - The model type stored in this tree
///
/// # Example
///
/// ```ignore
/// // Using batch operations
/// let batch = user_tree.create_batch()?;
/// batch.put(user1)?;
/// batch.put(user2)?;
/// batch.commit()?;
/// ```
pub trait Batchable<D, M>
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
        + MaybeSend
        + MaybeSync
        + 'static
        + std::str::FromStr,
{
    /// The batch builder type for this store
    type Batch: BatchBuilder<D, M>;

    /// Create a new batch builder for this tree
    ///
    /// # Returns
    ///
    /// * `Ok(Batch)` - A new batch builder
    /// * `Err(NetabaseError)` if the batch could not be created
    fn create_batch(&self) -> Result<Self::Batch, NetabaseError>;
}

/// Convenience methods for common batch operations
///
/// This trait provides helper methods for common batching patterns.
/// It is automatically implemented for all types that implement `Batchable`.
pub trait BatchOperations<D, M>: Batchable<D, M>
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
        + MaybeSend
        + MaybeSync
        + 'static
        + std::str::FromStr,
    Self: Sized,
{
    /// Insert or update multiple models in a single batch
    ///
    /// # Arguments
    ///
    /// * `models` - The models to insert or update
    ///
    /// # Returns
    ///
    /// * `Ok(())` if all operations succeeded
    /// * `Err(NetabaseError)` if any operation failed
    ///
    /// # Notes
    ///
    /// - All operations are applied atomically
    /// - If any operation fails, all operations are rolled back
    fn put_batch<I>(&self, models: I) -> Result<(), NetabaseError>
    where
        I: IntoIterator<Item = M>,
    {
        let mut batch = self.create_batch()?;
        for model in models {
            batch.put(model)?;
        }
        batch.commit()
    }

    /// Remove multiple models in a single batch
    ///
    /// # Arguments
    ///
    /// * `keys` - The primary keys of the models to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` if all operations succeeded
    /// * `Err(NetabaseError)` if any operation failed
    ///
    /// # Notes
    ///
    /// - All operations are applied atomically
    /// - If any operation fails, all operations are rolled back
    fn remove_batch<I>(&self, keys: I) -> Result<(), NetabaseError>
    where
        I: IntoIterator<Item = <M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey>,
    {
        let mut batch = self.create_batch()?;
        for key in keys {
            batch.remove(key)?;
        }
        batch.commit()
    }
}

// Blanket implementation for all Batchable types
impl<T, D, M> BatchOperations<D, M> for T
where
    T: Batchable<D, M>,
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
        + MaybeSend
        + MaybeSync
        + 'static
        + std::str::FromStr,
{
}

/// Cross-tree batch operations for multi-model transactions
///
/// This trait provides support for batching operations across multiple
/// trees (models) in a single transaction. This is useful for maintaining
/// consistency across related models.
///
/// # Type Parameters
///
/// * `D` - The definition type (wraps all models in the schema)
///
/// # Example
///
/// ```ignore
/// let mut batch = store.create_cross_tree_batch()?;
/// batch.put_for_tree("User", user)?;
/// batch.put_for_tree("Post", post)?;
/// batch.commit()?;  // Both user and post saved atomically
/// ```
pub trait CrossTreeBatchable {
    /// The cross-tree batch builder type
    type CrossTreeBatch: CrossTreeBatchBuilder;

    /// Create a new cross-tree batch builder
    ///
    /// # Returns
    ///
    /// * `Ok(CrossTreeBatch)` - A new cross-tree batch builder
    /// * `Err(NetabaseError)` if the batch could not be created
    fn create_cross_tree_batch(&self) -> Result<Self::CrossTreeBatch, NetabaseError>;
}

/// A batch operation builder that can operate across multiple trees
///
/// This trait allows batching operations for different model types
/// in a single atomic transaction.
pub trait CrossTreeBatchBuilder {
    /// Add a put operation for a specific tree
    ///
    /// # Arguments
    ///
    /// * `discriminant` - The tree/table discriminant (e.g., "User", "Post")
    /// * `key_bytes` - The encoded primary key
    /// * `value_bytes` - The encoded model value
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation was added to the batch
    /// * `Err(NetabaseError)` if the operation could not be added
    fn put_raw(
        &mut self,
        discriminant: &str,
        key_bytes: Vec<u8>,
        value_bytes: Vec<u8>,
    ) -> Result<(), NetabaseError>;

    /// Add a remove operation for a specific tree
    ///
    /// # Arguments
    ///
    /// * `discriminant` - The tree/table discriminant (e.g., "User", "Post")
    /// * `key_bytes` - The encoded primary key
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the operation was added to the batch
    /// * `Err(NetabaseError)` if the operation could not be added
    fn remove_raw(&mut self, discriminant: &str, key_bytes: Vec<u8>) -> Result<(), NetabaseError>;

    /// Commit all batched operations atomically across all trees
    ///
    /// # Returns
    ///
    /// * `Ok(())` if all operations succeeded
    /// * `Err(NetabaseError)` if any operation failed
    ///
    /// # Notes
    ///
    /// - All operations across all trees are applied atomically
    /// - If any operation fails, all operations are rolled back
    fn commit(self) -> Result<(), NetabaseError>;
}
