//! Transaction support for sled_store.
//!
//! This module provides ACID transaction semantics for operations on sled trees.
//! It includes the `SledTransactionalTree` wrapper that allows atomic operations
//! on models within a transaction context.

use std::marker::PhantomData;
use std::str::FromStr;

use strum::IntoDiscriminant;

use crate::traits::convert::ToIVec;
use crate::traits::definition::NetabaseDefinitionTrait;
use crate::traits::model::NetabaseModelTrait;
use crate::{MaybeSend, MaybeSync, NetabaseModelTraitKey};

use super::types::SecondaryKeyOp;

/// Type-safe wrapper around a sled transactional tree.
///
/// `SledTransactionalTree` provides ACID transaction operations on a specific model type.
/// This wrapper is used within transaction closures to perform atomic operations.
///
/// # Type Parameters
///
/// * `D` - The definition type
/// * `M` - The model type
///
/// # Note
///
/// This type is only used within transaction closures and cannot be constructed directly.
/// Use `SledStore::transaction()` to create a transaction context.
pub struct SledTransactionalTree<D, M>
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
    pub(super) tree: sled::transaction::TransactionalTree,
    pub(super) secondary_tree: sled::Tree,
    // When Some, collect secondary key operations instead of inserting directly
    // This prevents deadlocks by deferring secondary writes until after commit
    pub(super) pending_secondary_keys: Option<std::sync::Arc<std::sync::Mutex<Vec<SecondaryKeyOp>>>>,
    pub(super) _phantom_d: PhantomData<D>,
    pub(super) _phantom_m: PhantomData<M>,
}

impl<D, M> SledTransactionalTree<D, M>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + ToIVec + From<M>,
    M: NetabaseModelTrait<D> + TryFrom<D> + Into<D>,
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
    /// Insert or update a model in the transaction.
    pub fn put(&self, model: M) -> Result<(), Box<dyn std::error::Error>> {
        let primary_key = model.primary_key();
        let secondary_keys = model.secondary_keys();

        let key_bytes = bincode::encode_to_vec(&primary_key, bincode::config::standard())?;
        let definition: D = model.into();
        let value_bytes = definition.to_ivec()?;

        self.tree.insert(key_bytes, value_bytes.as_ref())?;

        // Handle secondary keys
        if !secondary_keys.is_empty() {
            if let Some(pending) = &self.pending_secondary_keys {
                // Defer secondary key insertion until after transaction commits
                let mut ops = pending.lock().unwrap();
                for sec_key in secondary_keys.values() {
                    let composite_key = self.build_composite_key(sec_key, &primary_key)?;
                    ops.push(SecondaryKeyOp::Insert(composite_key));
                }
            } else {
                // Direct insertion (not in transaction context)
                for sec_key in secondary_keys.values() {
                    let composite_key = self.build_composite_key(sec_key, &primary_key)?;
                    self.secondary_tree.insert(composite_key, &[] as &[u8])?;
                }
            }
        }

        Ok(())
    }

    /// Get a model by its primary key within the transaction.
    pub fn get(
        &self,
        key: <M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey,
    ) -> Result<Option<M>, Box<dyn std::error::Error>> {
        let key_bytes = bincode::encode_to_vec(&key, bincode::config::standard())?;

        match self.tree.get(&key_bytes)? {
            Some(ivec) => {
                let definition = D::from_ivec(&ivec)?;
                match M::try_from(definition) {
                    Ok(model) => Ok(Some(model)),
                    Err(_) => Ok(None),
                }
            }
            None => Ok(None),
        }
    }

    /// Remove a model by its primary key within the transaction.
    pub fn remove(
        &self,
        key: <M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey,
    ) -> Result<Option<M>, Box<dyn std::error::Error>> {
        let key_bytes = bincode::encode_to_vec(&key, bincode::config::standard())?;

        match self.tree.remove(key_bytes.as_slice())? {
            Some(ivec) => {
                let definition = D::from_ivec(&ivec)?;
                match M::try_from(definition) {
                    Ok(model) => {
                        // Clean up secondary keys
                        let secondary_keys = model.secondary_keys();
                        if !secondary_keys.is_empty() {
                            if let Some(pending) = &self.pending_secondary_keys {
                                // Defer secondary key removal until after transaction commits
                                let mut ops = pending.lock().unwrap();
                                for sec_key in secondary_keys.values() {
                                    let composite_key = self.build_composite_key(sec_key, &key)?;
                                    ops.push(SecondaryKeyOp::Remove(composite_key));
                                }
                            } else {
                                // Direct removal (not in transaction context)
                                for sec_key in secondary_keys.values() {
                                    let composite_key = self.build_composite_key(sec_key, &key)?;
                                    self.secondary_tree.remove(composite_key)?;
                                }
                            }
                        }
                        Ok(Some(model))
                    }
                    Err(_) => Ok(None),
                }
            }
            None => Ok(None),
        }
    }

    /// Build a composite key from secondary key + primary key
    fn build_composite_key(
        &self,
        secondary_key: &<M::Keys as NetabaseModelTraitKey<D>>::SecondaryKey,
        primary_key: &<M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut composite_key = bincode::encode_to_vec(secondary_key, bincode::config::standard())?;
        let prim_key_bytes = bincode::encode_to_vec(primary_key, bincode::config::standard())?;
        composite_key.extend_from_slice(&prim_key_bytes);
        Ok(composite_key)
    }
}
