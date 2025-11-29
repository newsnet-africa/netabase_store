//! Trait implementations for Redb backend
//!
//! This module contains all the trait implementations for RedbStore and RedbStoreTree.

use crate::error::NetabaseError;
use crate::traits::batch::Batchable;
use crate::traits::convert::ToIVec;
use crate::traits::definition::NetabaseDefinitionTrait;
use crate::traits::model::{NetabaseModelTrait, NetabaseModelTraitKey};
use crate::traits::store_ops::{OpenTree, StoreOps, StoreOpsIter, StoreOpsSecondary};
use crate::traits::tree::NetabaseTreeSync;

use redb::{ReadableDatabase, ReadableTable};
use std::fmt::Debug;
use strum::IntoDiscriminant;

use super::batch::RedbBatchBuilder;
use super::iterator::RedbIter;
use super::store::RedbStore;
use super::tree::RedbStoreTree;

// Implement the unified NetabaseTreeSync trait for RedbStoreTree
impl<'db, D, M> NetabaseTreeSync<'db, D, M> for RedbStoreTree<'db, D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D> + Debug + bincode::Decode<()>,
    M::Keys: Debug + bincode::Decode<()> + Ord + Clone + PartialEq,
    M::Keys: for<'a> From<<M::PrimaryKey as redb::Value>::SelfType<'a>>,
    <D as IntoDiscriminant>::Discriminant: crate::DiscriminantBounds,
{
    type PrimaryKey = <M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey;
    type SecondaryKeys = <M::Keys as NetabaseModelTraitKey<D>>::SecondaryKey;

    fn put(&self, model: M) -> Result<(), NetabaseError> {
        self.put(model)
    }

    fn get(&self, key: Self::PrimaryKey) -> Result<Option<M>, NetabaseError> {
        self.get(key.into())
    }

    fn remove(&self, key: Self::PrimaryKey) -> Result<Option<M>, NetabaseError> {
        self.remove(key.into())
    }

    fn get_by_secondary_key(
        &self,
        secondary_key: Self::SecondaryKeys,
    ) -> Result<Vec<M>, NetabaseError> {
        self.get_by_secondary_key(secondary_key)
    }

    fn is_empty(&self) -> Result<bool, NetabaseError> {
        self.is_empty()
    }

    fn len(&self) -> Result<usize, NetabaseError> {
        self.len()
    }

    fn clear(&self) -> Result<(), NetabaseError> {
        self.clear()
    }
}

// Implement StoreOps trait for RedbStoreTree
impl<'db, D, M> StoreOps<D, M> for RedbStoreTree<'db, D, M>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + ToIVec + From<M>,
    M: NetabaseModelTrait<D>
        + TryFrom<D>
        + Into<D>
        + Clone
        + Debug
        + bincode::Encode
        + bincode::Decode<()>,
    M::Keys: Debug + bincode::Decode<()> + Ord + Clone + bincode::Encode + PartialEq,
    <D as IntoDiscriminant>::Discriminant: crate::DiscriminantBounds,
{
    fn put_raw(&self, model: M) -> Result<(), NetabaseError> {
        // Store raw model directly (not wrapped in Definition)
        self.put(model)
    }

    fn get_raw(
        &self,
        key: <M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey,
    ) -> Result<Option<M>, NetabaseError> {
        // Retrieve raw model directly
        self.get(M::Keys::from(key))
    }

    fn remove_raw(
        &self,
        key: <M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey,
    ) -> Result<Option<M>, NetabaseError> {
        // Remove and return raw model directly
        self.remove(M::Keys::from(key))
    }

    fn discriminant(&self) -> &str {
        self.discriminant.as_ref()
    }
}

// Implement StoreOpsSecondary trait for RedbStoreTree
impl<'db, D, M> StoreOpsSecondary<D, M> for RedbStoreTree<'db, D, M>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + ToIVec + From<M>,
    M: NetabaseModelTrait<D>
        + TryFrom<D>
        + Into<D>
        + Clone
        + Debug
        + bincode::Encode
        + bincode::Decode<()>,
    M::Keys: Debug + bincode::Decode<()> + Ord + Clone + bincode::Encode + PartialEq,
    M::Keys: for<'a> From<<M::PrimaryKey as redb::Value>::SelfType<'a>>,
    <D as IntoDiscriminant>::Discriminant: crate::DiscriminantBounds,
{
    fn get_by_secondary_key_raw(
        &self,
        secondary_key: <M::Keys as NetabaseModelTraitKey<D>>::SecondaryKey,
    ) -> Result<Vec<M>, NetabaseError> {
        self.get_by_secondary_key(secondary_key)
    }
}

// Implement StoreOpsIter trait for RedbStoreTree
impl<'db, D, M> StoreOpsIter<D, M> for RedbStoreTree<'db, D, M>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + ToIVec + From<M>,
    M: NetabaseModelTrait<D>
        + TryFrom<D>
        + Into<D>
        + Clone
        + Debug
        + bincode::Encode
        + bincode::Decode<()>,
    M::Keys: Debug + bincode::Decode<()> + Ord + Clone + bincode::Encode + PartialEq,
    <D as IntoDiscriminant>::Discriminant: crate::DiscriminantBounds,
{
    type Iter = RedbIter<M>;

    fn iter(&self) -> Result<Self::Iter, NetabaseError> {
        // Inline the iteration logic to avoid name conflicts
        let table_def = self.table_def();
        let read_txn = self.db.as_ref().begin_read()?;

        let table = match read_txn.open_table(table_def) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => {
                return Ok(RedbIter::empty());
            }
            Err(e) => return Err(NetabaseError::RedbTableError(e)),
        };

        let mut models = Vec::new();
        for item in table.iter()? {
            let (_, value_guard) = item?;
            let model: M = value_guard.value();
            models.push(model);
        }

        Ok(RedbIter::new(models))
    }

    fn len(&self) -> Result<usize, NetabaseError> {
        // Inline the len logic to avoid name conflicts
        let table_def = self.table_def();
        let read_txn = self.db.as_ref().begin_read()?;

        match read_txn.open_table(table_def) {
            Ok(table) => {
                use redb::ReadableTableMetadata;
                Ok(table.len()? as usize)
            }
            Err(redb::TableError::TableDoesNotExist(_)) => Ok(0),
            Err(e) => Err(NetabaseError::RedbTableError(e)),
        }
    }
}

// Implement Batchable trait for RedbStoreTree
impl<'db, D, M> Batchable<D, M> for RedbStoreTree<'db, D, M>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + ToIVec + From<M>,
    M: NetabaseModelTrait<D>
        + TryFrom<D>
        + Into<D>
        + Clone
        + Debug
        + bincode::Encode
        + bincode::Decode<()>,
    M::Keys: Debug + bincode::Decode<()> + Ord + Clone + bincode::Encode + PartialEq,
    <D as IntoDiscriminant>::Discriminant: crate::DiscriminantBounds,
{
    type Batch = RedbBatchBuilder<D, M>;

    fn create_batch(&self) -> Result<Self::Batch, NetabaseError> {
        Ok(RedbBatchBuilder::new(
            std::sync::Arc::clone(&self.db),
            self.table_name,
            self.secondary_table_name,
        ))
    }
}

// Implement OpenTree trait for RedbStore
impl<D, M> OpenTree<D, M> for RedbStore<D>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + ToIVec + From<M>,
    M: NetabaseModelTrait<D>
        + TryFrom<D>
        + Into<D>
        + Clone
        + Debug
        + bincode::Encode
        + bincode::Decode<()>,
    M::Keys: Debug + bincode::Decode<()> + Ord + Clone + bincode::Encode + PartialEq,
    <D as IntoDiscriminant>::Discriminant: crate::DiscriminantBounds,
{
    type Tree<'a>
        = RedbStoreTree<'a, D, M>
    where
        Self: 'a;

    fn open_tree(&self) -> Self::Tree<'_> {
        RedbStoreTree::new(std::sync::Arc::clone(&self.db), M::DISCRIMINANT)
    }
}
