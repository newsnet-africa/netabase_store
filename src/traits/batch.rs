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
/// ```no_run
/// # use netabase_store::netabase_definition_module;
/// # use netabase_store::traits::batch::{Batchable, BatchBuilder};
/// # #[netabase_definition_module(MyDef, MyKeys)]
/// # mod models {
/// #     use netabase_store::{NetabaseModel, netabase};
/// #     #[derive(NetabaseModel, Clone, Debug, PartialEq,
/// #              bincode::Encode, bincode::Decode,
/// #              serde::Serialize, serde::Deserialize)]
/// #     #[netabase(MyDef)]
/// #     pub struct User {
/// #         #[primary_key]
/// #         pub id: u64,
/// #         pub name: String,
/// #     }
/// # }
/// # use models::*;
/// # fn example<T: Batchable<MyDef, User>>(tree: &T) -> Result<(), Box<dyn std::error::Error>> {
/// # let user1 = User { id: 1, name: "Alice".to_string() };
/// # let user2 = User { id: 2, name: "Bob".to_string() };
/// # let old_user_id = UserPrimaryKey(3);
/// let mut batch = tree.create_batch()?;
/// batch.put(user1)?;
/// batch.put(user2)?;
/// batch.remove(old_user_id)?;
/// batch.commit()?;  // All operations applied atomically
/// # Ok(())
/// # }
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
/// ```no_run
/// # use netabase_store::netabase_definition_module;
/// # use netabase_store::traits::batch::{Batchable, BatchBuilder};
/// # #[netabase_definition_module(MyDef, MyKeys)]
/// # mod models {
/// #     use netabase_store::{NetabaseModel, netabase};
/// #     #[derive(NetabaseModel, Clone, Debug, PartialEq,
/// #              bincode::Encode, bincode::Decode,
/// #              serde::Serialize, serde::Deserialize)]
/// #     #[netabase(MyDef)]
/// #     pub struct User {
/// #         #[primary_key]
/// #         pub id: u64,
/// #         pub name: String,
/// #     }
/// # }
/// # use models::*;
/// # fn example<T: Batchable<MyDef, User>>(user_tree: &T) -> Result<(), Box<dyn std::error::Error>> {
/// # let user1 = User { id: 1, name: "Alice".to_string() };
/// # let user2 = User { id: 2, name: "Bob".to_string() };
/// // Using batch operations
/// let mut batch = user_tree.create_batch()?;
/// batch.put(user1)?;
/// batch.put(user2)?;
/// batch.commit()?;
/// # Ok(())
/// # }
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
/// ```no_run
/// # use netabase_store::traits::batch::CrossTreeBatchable;
/// # fn example<S: CrossTreeBatchable>(store: &S) -> Result<(), Box<dyn std::error::Error>> {
/// # let user_bytes = vec![1, 2, 3];
/// # let user_key = vec![1];
/// # let post_bytes = vec![4, 5, 6];
/// # let post_key = vec![2];
/// use netabase_store::traits::batch::CrossTreeBatchBuilder;
/// let mut batch = store.create_cross_tree_batch()?;
/// batch.put_raw("User", user_key, user_bytes)?;
/// batch.put_raw("Post", post_key, post_bytes)?;
/// batch.commit()?;  // Both user and post saved atomically
/// # Ok(())
/// # }
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
