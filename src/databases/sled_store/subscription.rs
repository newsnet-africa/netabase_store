//! Subscription tree implementation for Sled database backend.
//!
//! This module provides subscription management functionality for the Sled backend,
//! allowing storage and retrieval of model subscription hashes.

use crate::error::NetabaseError;
use crate::traits::definition::{NetabaseDefinitionTrait, NetabaseDefinitionWithSubscription};
use std::marker::PhantomData;
use strum::IntoDiscriminant;

/// Subscription tree for managing subscription data in Sled backend
///
/// This provides methods to manage subscriptions within the Sled database.
/// Each subscription variant has its own tree, similar to regular model trees.
pub struct SledSubscriptionTree<'db, D, S>
where
    D: NetabaseDefinitionTrait + NetabaseDefinitionWithSubscription<Subscriptions = S>,
    S: strum::IntoDiscriminant,
{
    db: &'db sled::Db,
    tree_name: String,
    _phantom_d: PhantomData<D>,
    _phantom_s: PhantomData<S>,
}

impl<'db, D, S> SledSubscriptionTree<'db, D, S>
where
    D: NetabaseDefinitionTrait + NetabaseDefinitionWithSubscription<Subscriptions = S>,
    S: strum::IntoDiscriminant,
    <S as strum::IntoDiscriminant>::Discriminant: AsRef<str>,
{
    /// Create a new SledSubscriptionTree
    pub(super) fn new(db: &'db sled::Db, subscription: S) -> Self {
        let discriminant = subscription.discriminant();
        let tree_name = format!("subscription_{}", discriminant.as_ref());

        Self {
            db,
            tree_name,
            _phantom_d: PhantomData,
            _phantom_s: PhantomData,
        }
    }

    /// Subscribe to a specific subscription key with model hash
    pub fn subscribe(
        &mut self,
        subscription_key: D::Keys,
        model_hash: [u8; 32],
    ) -> Result<(), NetabaseError>
    {
        let key_bytes = bincode::encode_to_vec(&subscription_key, bincode::config::standard())
            .map_err(crate::error::EncodingDecodingError::from)?;
        let hash_bytes = bincode::encode_to_vec(&model_hash, bincode::config::standard())
            .map_err(crate::error::EncodingDecodingError::from)?;

        let tree = self.db.open_tree(&self.tree_name)?;
        tree.insert(key_bytes, hash_bytes)?;

        Ok(())
    }

    /// Unsubscribe from a specific subscription key
    pub fn unsubscribe(
        &mut self,
        subscription_key: &D::Keys,
    ) -> Result<Option<[u8; 32]>, NetabaseError> {
        let key_bytes = bincode::encode_to_vec(subscription_key, bincode::config::standard())
            .map_err(crate::error::EncodingDecodingError::from)?;

        let tree = self.db.open_tree(&self.tree_name)?;
        if let Some(hash_bytes) = tree.remove(key_bytes)? {
            let (model_hash, _) = bincode::decode_from_slice::<[u8; 32], _>(
                &hash_bytes, bincode::config::standard()
            ).map_err(crate::error::EncodingDecodingError::from)?;
            Ok(Some(model_hash))
        } else {
            Ok(None)
        }
    }

    /// Get subscription data for a specific key
    pub fn get_subscription(
        &self,
        subscription_key: &D::Keys,
    ) -> Result<Option<[u8; 32]>, NetabaseError> {
        let key_bytes = bincode::encode_to_vec(subscription_key, bincode::config::standard())
            .map_err(crate::error::EncodingDecodingError::from)?;

        let tree = self.db.open_tree(&self.tree_name)?;
        if let Some(hash_bytes) = tree.get(key_bytes)? {
            let (model_hash, _) = bincode::decode_from_slice::<[u8; 32], _>(
                &hash_bytes, bincode::config::standard()
            ).map_err(crate::error::EncodingDecodingError::from)?;
            Ok(Some(model_hash))
        } else {
            Ok(None)
        }
    }

    /// Clear all subscriptions for this subscription type
    pub fn clear_subscriptions(&mut self) -> Result<(), NetabaseError> {
        let tree = self.db.open_tree(&self.tree_name)?;
        tree.clear()?;
        Ok(())
    }

    /// Get the number of active subscriptions for this subscription type
    pub fn subscription_count(&self) -> Result<usize, NetabaseError> {
        let tree = self.db.open_tree(&self.tree_name)?;
        Ok(tree.len())
    }
}