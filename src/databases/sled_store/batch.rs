use crate::error::NetabaseError;
use crate::traits::convert::ToIVec;
use crate::traits::definition::NetabaseDefinitionTrait;
use crate::traits::model::NetabaseModelTrait;
use crate::{MaybeSend, MaybeSync, NetabaseModelTraitKey};
use std::marker::PhantomData;
use std::str::FromStr;
use strum::IntoDiscriminant;

/// Batch builder for Sled
pub struct SledBatchBuilder<D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
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
    pub(crate) tree: sled::Tree,
    pub(crate) secondary_tree: sled::Tree,
    pub(crate) primary_batch: sled::Batch,
    pub(crate) secondary_batch: sled::Batch,
    pub(crate) _phantom_d: PhantomData<D>,
    pub(crate) _phantom_m: PhantomData<M>,
}

impl<D, M> crate::traits::batch::BatchBuilder<D, M> for SledBatchBuilder<D, M>
where
    D: NetabaseDefinitionTrait + From<M> + ToIVec,
    M: NetabaseModelTrait<D> + Clone,
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
    fn put(&mut self, model: M) -> Result<(), NetabaseError>
    where
        D: From<M>,
    {
        let primary_key = model.primary_key();
        let secondary_keys = model.secondary_keys();

        let key_bytes = bincode::encode_to_vec(&primary_key, bincode::config::standard())
            .map_err(crate::error::EncodingDecodingError::from)?;

        // Store as Definition (same format as regular put)
        let definition: D = model.into();
        let value_bytes = definition.to_ivec()?;

        self.primary_batch
            .insert(key_bytes.clone(), value_bytes.as_ref());

        // Add secondary key entries
        if !secondary_keys.is_empty() {
            for sec_key in secondary_keys.values() {
                let sec_key_bytes = bincode::encode_to_vec(sec_key, bincode::config::standard())
                    .map_err(crate::error::EncodingDecodingError::from)?;
                let prim_key_bytes =
                    bincode::encode_to_vec(&primary_key, bincode::config::standard())
                        .map_err(crate::error::EncodingDecodingError::from)?;

                let mut composite_key = sec_key_bytes;
                composite_key.extend_from_slice(&prim_key_bytes);

                self.secondary_batch.insert(composite_key, &[] as &[u8]);
            }
        }

        Ok(())
    }

    fn remove(
        &mut self,
        key: <M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey,
    ) -> Result<(), NetabaseError> {
        let key_bytes = bincode::encode_to_vec(&key, bincode::config::standard())
            .map_err(crate::error::EncodingDecodingError::from)?;

        self.primary_batch.remove(key_bytes);

        // Note: We can't clean up secondary keys in the batch without knowing them
        // This is a limitation of the batch API - ideally we'd fetch the model first

        Ok(())
    }

    fn commit(self) -> Result<(), NetabaseError> {
        self.tree.apply_batch(self.primary_batch)?;
        self.secondary_tree.apply_batch(self.secondary_batch)?;
        Ok(())
    }
}
