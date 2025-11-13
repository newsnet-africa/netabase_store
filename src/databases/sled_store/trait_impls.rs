use crate::error::NetabaseError;
use crate::traits::convert::ToIVec;
use crate::traits::definition::NetabaseDefinitionTrait;
use crate::traits::model::NetabaseModelTrait;
use crate::traits::tree::NetabaseTreeSync;
use crate::{MaybeSend, MaybeSync, NetabaseModelTraitKey};
use std::marker::PhantomData;
use std::str::FromStr;
use strum::IntoDiscriminant;

use super::batch::SledBatchBuilder;
use super::store::SledStore;
use super::tree::SledStoreTree;

// Implement the unified NetabaseTreeSync trait for SledStoreTree
impl<'db, D, M> NetabaseTreeSync<'db, D, M> for SledStoreTree<'db, D, M>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + ToIVec + From<M>,
    M: NetabaseModelTrait<D> + TryFrom<D> + Into<D> + Clone,
    <M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey: bincode::Decode<()> + Clone,
    <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::SecondaryKey: bincode::Decode<()>,
    <D as IntoDiscriminant>::Discriminant: Clone
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
        + FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: MaybeSync,
    <D as strum::IntoDiscriminant>::Discriminant: MaybeSend,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <D as strum::IntoDiscriminant>::Discriminant: std::convert::AsRef<str>,
{
    type PrimaryKey = <M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey;
    type SecondaryKeys = <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::SecondaryKey;

    fn put(&self, model: M) -> Result<(), NetabaseError> {
        self.put(model)
    }

    fn get(&self, key: Self::PrimaryKey) -> Result<Option<M>, NetabaseError> {
        self.get(key)
    }

    fn remove(&self, key: Self::PrimaryKey) -> Result<Option<M>, NetabaseError> {
        self.remove(key)
    }

    fn get_by_secondary_key(
        &self,
        secondary_key: Self::SecondaryKeys,
    ) -> Result<Vec<M>, NetabaseError> {
        self.get_by_secondary_key(secondary_key)
    }

    fn is_empty(&self) -> Result<bool, NetabaseError> {
        Ok(self.is_empty())
    }

    fn len(&self) -> Result<usize, NetabaseError> {
        Ok(self.len())
    }

    fn clear(&self) -> Result<(), NetabaseError> {
        self.clear()
    }
}

// Implement StoreOps trait for SledStoreTree
impl<'db, D, M> crate::traits::store_ops::StoreOps<D, M> for SledStoreTree<'db, D, M>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + ToIVec + From<M>,
    M: NetabaseModelTrait<D> + TryFrom<D> + Into<D> + Clone + bincode::Decode<()>,
    <M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey: bincode::Decode<()> + Clone,
    <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::SecondaryKey: bincode::Decode<()>,
    <D as IntoDiscriminant>::Discriminant: Clone
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
        + FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: MaybeSync,
    <D as strum::IntoDiscriminant>::Discriminant: MaybeSend,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <D as strum::IntoDiscriminant>::Discriminant: std::convert::AsRef<str>,
{
    fn put_raw(&self, model: M) -> Result<(), NetabaseError> {
        // Store the model directly, not wrapped in Definition
        let primary_key = model.primary_key();
        let secondary_keys = model.secondary_keys();

        let key_bytes = bincode::encode_to_vec(&primary_key, bincode::config::standard())
            .map_err(crate::error::EncodingDecodingError::from)?;

        // Store raw model directly (not wrapped in Definition)
        let value_bytes = bincode::encode_to_vec(&model, bincode::config::standard())
            .map_err(crate::error::EncodingDecodingError::from)?;

        // Use batch for atomic operations
        let mut batch = sled::Batch::default();
        batch.insert(key_bytes, value_bytes);
        self.tree.apply_batch(batch)?;

        // Batch secondary key inserts
        if !secondary_keys.is_empty() {
            let mut sec_batch = sled::Batch::default();
            for sec_key in secondary_keys.values() {
                let composite_key = self.build_composite_key(sec_key, &primary_key)?;
                sec_batch.insert(composite_key, &[] as &[u8]);
            }
            self.secondary_tree.apply_batch(sec_batch)?;
        }

        Ok(())
    }

    fn get_raw(
        &self,
        key: <M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey,
    ) -> Result<Option<M>, NetabaseError> {
        // Get the model directly (not wrapped in Definition)
        let key_bytes = bincode::encode_to_vec(&key, bincode::config::standard())
            .map_err(crate::error::EncodingDecodingError::from)?;

        match self.tree.get(key_bytes)? {
            Some(ivec) => {
                // Decode directly as model (not Definition)
                let (model, _) =
                    bincode::decode_from_slice::<M, _>(&ivec, bincode::config::standard())
                        .map_err(crate::error::EncodingDecodingError::from)?;
                Ok(Some(model))
            }
            None => Ok(None),
        }
    }

    fn remove_raw(
        &self,
        key: <M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey,
    ) -> Result<Option<M>, NetabaseError> {
        let key_bytes = bincode::encode_to_vec(&key, bincode::config::standard())
            .map_err(crate::error::EncodingDecodingError::from)?;

        match self.tree.remove(key_bytes)? {
            Some(ivec) => {
                // Decode directly as model (not Definition)
                let (model, _) =
                    bincode::decode_from_slice::<M, _>(&ivec, bincode::config::standard())
                        .map_err(crate::error::EncodingDecodingError::from)?;

                // Clean up secondary keys using batch
                let secondary_keys = model.secondary_keys();
                if !secondary_keys.is_empty() {
                    let mut sec_batch = sled::Batch::default();
                    for sec_key in secondary_keys.values() {
                        let composite_key = self.build_composite_key(sec_key, &key)?;
                        sec_batch.remove(composite_key);
                    }
                    self.secondary_tree.apply_batch(sec_batch)?;
                }
                Ok(Some(model))
            }
            None => Ok(None),
        }
    }

    fn discriminant(&self) -> &str {
        M::discriminant_name()
    }
}

// Implement StoreOpsSecondary trait for SledStoreTree
impl<'db, D, M> crate::traits::store_ops::StoreOpsSecondary<D, M> for SledStoreTree<'db, D, M>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + ToIVec + From<M>,
    M: NetabaseModelTrait<D> + TryFrom<D> + Into<D> + Clone + bincode::Decode<()>,
    <M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey: bincode::Decode<()> + Clone,
    <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::SecondaryKey: bincode::Decode<()>,
    <D as IntoDiscriminant>::Discriminant: Clone
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
        + FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: MaybeSync,
    <D as strum::IntoDiscriminant>::Discriminant: MaybeSend,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <D as strum::IntoDiscriminant>::Discriminant: std::convert::AsRef<str>,
{
    fn get_by_secondary_key_raw(
        &self,
        secondary_key: <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::SecondaryKey,
    ) -> Result<Vec<M>, NetabaseError> {
        // Use existing get_by_secondary_key which already returns raw models
        self.get_by_secondary_key(secondary_key)
    }
}

// Implement Batchable trait for SledStoreTree
impl<'db, D, M> crate::traits::batch::Batchable<D, M> for SledStoreTree<'db, D, M>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + ToIVec + From<M>,
    M: NetabaseModelTrait<D> + TryFrom<D> + Into<D> + Clone,
    <M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey: bincode::Decode<()> + Clone,
    <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::SecondaryKey: bincode::Decode<()>,
    <D as IntoDiscriminant>::Discriminant: Clone
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
        + FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: MaybeSync,
    <D as strum::IntoDiscriminant>::Discriminant: MaybeSend,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <D as strum::IntoDiscriminant>::Discriminant: std::convert::AsRef<str>,
{
    type Batch = SledBatchBuilder<D, M>;

    fn create_batch(&self) -> Result<Self::Batch, NetabaseError> {
        Ok(SledBatchBuilder {
            tree: self.tree.clone(),
            secondary_tree: self.secondary_tree.clone(),
            primary_batch: sled::Batch::default(),
            secondary_batch: sled::Batch::default(),
            _phantom_d: PhantomData,
            _phantom_m: PhantomData,
        })
    }
}

// Implement OpenTree trait for SledStore
impl<D, M> crate::traits::store_ops::OpenTree<D, M> for SledStore<D>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + ToIVec + From<M>,
    M: NetabaseModelTrait<D> + TryFrom<D> + Into<D> + Clone + bincode::Decode<()>,
    <M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey: bincode::Decode<()> + Clone,
    <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::SecondaryKey: bincode::Decode<()>,
    <D as strum::IntoDiscriminant>::Discriminant: crate::traits::definition::NetabaseDiscriminant,
{
    type Tree<'a>
        = SledStoreTree<'a, D, M>
    where
        Self: 'a;

    fn open_tree(&self) -> Self::Tree<'_> {
        SledStoreTree::new(&self.db, M::DISCRIMINANT)
    }
}
