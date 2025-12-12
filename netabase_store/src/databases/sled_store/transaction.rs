//! Transaction implementations for sled backend
//!
//! This module provides read and write transaction wrappers that adapt sled's
//! API to Netabase's trait-based transaction interface.

use crate::{
    databases::sled_store::{
        SledNetabaseModelTrait, SledStore, deserialize_key, deserialize_value, serialize_key,
        serialize_value,
    },
    error::{NetabaseError, NetabaseResult},
    traits::{
        definition::{DiscriminantName, NetabaseDefinition},
        model::{NetabaseModelTrait, ModelTypeContainer, key::NetabaseModelKeyTrait},
    },
};
use log::{debug, trace};
use std::fmt::Debug;
use std::time::Instant;
use strum::{IntoDiscriminant, IntoEnumIterator};
use bincode::{Encode, Decode};

// =============================================================================
// Read Transaction
// =============================================================================

/// Sled read transaction wrapper
///
/// Provides read-only access to the database with snapshot isolation semantics.
/// All reads within a transaction see a consistent view of the database.
pub struct SledReadTransaction<'db, D: NetabaseDefinition>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    /// Reference to the sled database
    pub(crate) db: &'db sled::Db,

    /// Reference to the parent store
    pub(crate) _sled_store: &'db SledStore<D>,
}

impl<'db, D: NetabaseDefinition> SledReadTransaction<'db, D>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    pub fn get<M: NetabaseModelTrait<D> + SledNetabaseModelTrait<D>>(
        &self,
        key: M::PrimaryKey,
    ) -> NetabaseResult<Option<M>>
    {
        // Open the main tree for this model
        let tree_name_binding = M::MODEL_TREE_NAME;
        let tree_name = tree_name_binding.name();
        let tree = self.db.open_tree(tree_name)?;

        // Serialize the key
        let key_bytes = serialize_key(key)?;

        // Get the value
        match tree.get(key_bytes)? {
            Some(value_bytes) => {
                let model = deserialize_value(&value_bytes)?;
                Ok(Some(model))
            }
            None => Ok(None),
        }
    }

    pub fn get_pk_by_secondary_key<M: NetabaseModelTrait<D> + SledNetabaseModelTrait<D>>(
        &self,
        secondary_key: <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum,
    ) -> NetabaseResult<Option<M::PrimaryKey>>
    {
        // Get the discriminant to find the right tree
        let discriminant = secondary_key.discriminant();

        // Generate tree name
        let tree_name = M::secondary_key_table_name(discriminant);
        let tree = self.db.open_tree(&tree_name)?;

        // Serialize the secondary key
        let sk_bytes = serialize_key(secondary_key)?;

        // Get the primary key
        match tree.get(sk_bytes)? {
            Some(pk_bytes) => {
                let pk = deserialize_value(&pk_bytes)?;
                Ok(Some(pk))
            }
            None => Ok(None),
        }
    }

    pub fn get_by_secondary_key<M: NetabaseModelTrait<D> + SledNetabaseModelTrait<D>>(
        &self,
        secondary_key: <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum,
    ) -> NetabaseResult<Option<M>>
    {
        if let Some(pk) = self.get_pk_by_secondary_key::<M>(secondary_key)? {
            self.get::<M>(pk)
        } else {
            Ok(None)
        }
    }

    pub fn get_pk_by_relational_key<M: NetabaseModelTrait<D> + SledNetabaseModelTrait<D>>(
        &self,
        relational_key: <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum,
    ) -> NetabaseResult<Option<M::PrimaryKey>>
    {
        let discriminant = relational_key.discriminant();
        let tree_name = M::relational_key_table_name(discriminant);
        let tree = self.db.open_tree(&tree_name)?;
        let rk_bytes = serialize_key(relational_key)?;

        match tree.get(rk_bytes)? {
            Some(pk_bytes) => Ok(Some(deserialize_value(&pk_bytes)?)),
            None => Ok(None),
        }
    }

    pub fn get_by_relational_key<M: NetabaseModelTrait<D> + SledNetabaseModelTrait<D>>(
        &self,
        relational_key: <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum,
    ) -> NetabaseResult<Option<M>>
    {
        if let Some(pk) = self.get_pk_by_relational_key::<M>(relational_key)? {
            self.get::<M>(pk)
        } else {
            Ok(None)
        }
    }

    pub fn get_pk_by_hash<M: NetabaseModelTrait<D> + SledNetabaseModelTrait<D>>(
        &self,
        hash: M::Hash,
    ) -> NetabaseResult<Option<M::PrimaryKey>>
    {
        let tree_name = M::hash_tree_table_name();
        let tree = self.db.open_tree(&tree_name)?;
        let hash_bytes = serialize_key(hash)?;

        match tree.get(hash_bytes)? {
            Some(pk_bytes) => Ok(Some(deserialize_value(&pk_bytes)?)),
            None => Ok(None),
        }
    }

    pub fn get_by_hash<M: NetabaseModelTrait<D> + SledNetabaseModelTrait<D>>(
        &self,
        hash: M::Hash,
    ) -> NetabaseResult<Option<M>>
    {
        if let Some(pk) = self.get_pk_by_hash::<M>(hash)? {
            self.get::<M>(pk)
        } else {
            Ok(None)
        }
    }

    pub fn get_subscription_accumulator<M: NetabaseModelTrait<D> + SledNetabaseModelTrait<D>>(
        &self,
        subscription_discriminant: <M as ModelTypeContainer>::Subscriptions,
    ) -> NetabaseResult<([u8; 32], u64)>
    {
        // Generate tree name
        let tree_name = M::subscription_key_table_name(subscription_discriminant);
        let tree = self.db.open_tree(&tree_name)?;

        // Initialize accumulator and count
        let mut accumulator = [0u8; 32];
        let mut count = 0u64;

        // Iterate through all entries and XOR their hashes
        for entry in tree.iter() {
            let (_key, hash_bytes) = entry?;

            // Deserialize the hash
            let hash: [u8; 32] = deserialize_value(&hash_bytes)?;

            // XOR into accumulator (order-independent)
            for i in 0..32 {
                accumulator[i] ^= hash[i];
            }

            count += 1;
        }

        Ok((accumulator, count))
    }

    pub fn get_subscription_keys<M: NetabaseModelTrait<D> + SledNetabaseModelTrait<D>>(
        &self,
        subscription_discriminant: <M as ModelTypeContainer>::Subscriptions,
    ) -> NetabaseResult<Vec<M::PrimaryKey>>
    {
        // Generate tree name
        let tree_name = M::subscription_key_table_name(subscription_discriminant);
        let tree = self.db.open_tree(&tree_name)?;

        let mut keys = Vec::new();

        // Iterate through all entries and collect keys
        for entry in tree.iter() {
            let (key_bytes, _hash) = entry?;
            let key = deserialize_key(&key_bytes)?;
            keys.push(key);
        }

        Ok(keys)
    }
}

// =============================================================================
// Write Transaction
// =============================================================================

/// Sled write transaction wrapper
///
/// Provides write access to the database with operation queueing.
/// All operations are queued and executed atomically in priority order
/// when the transaction is committed.
pub struct SledWriteTransaction<'db, D: NetabaseDefinition>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    /// The sled database (Arc internally, cheap to clone)
    pub(crate) db: sled::Db,

    /// Reference to the parent store
    pub(crate) _sled_store: &'db SledStore<D>,
}

// =============================================================================
// Write Methods
// =============================================================================
impl<'db, D: NetabaseDefinition> SledWriteTransaction<'db, D>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    pub(crate) fn new(db: &sled::Db, store: &'db SledStore<D>) -> Self {
        Self {
            db: db.clone(),
            _sled_store: store,
        }
    }

    pub fn put<M: NetabaseModelTrait<D> + SledNetabaseModelTrait<D> + Send + Clone>(
        &mut self,
        model: M,
    ) -> NetabaseResult<()>
    where M: Encode
    {
        let store = unsafe { &*self._sled_store };
        let pk = model.primary_key();
        let model_tree_name = M::MODEL_TREE_NAME;
        let model_hash: [u8; 32] = model.compute_hash().into();

        // 1. Insert into main tree
        let main_tree_name =
            D::get_tree_name(&model_tree_name).ok_or(NetabaseError::TreeNotFound)?;
        let main_tree = store.db.open_tree(main_tree_name)?;
        main_tree.insert(
            bincode::encode_to_vec(pk.clone(), bincode::config::standard())?,
            bincode::encode_to_vec(model.clone(), bincode::config::standard())?,
        )?;

        // 2. Insert into secondary key trees
        for sk in <M::Keys as NetabaseModelKeyTrait<D, M>>::secondary_keys(&model) {
            let sk_discriminant = sk.discriminant();
            let secondary_tree_name = M::secondary_key_table_name(sk_discriminant);
            let secondary_tree = store.db.open_tree(secondary_tree_name)?;
            secondary_tree.insert(
                bincode::encode_to_vec(sk, bincode::config::standard())?,
                bincode::encode_to_vec(pk.clone(), bincode::config::standard())?,
            )?;
        }

        // 3. Insert into relational key trees
        for rk in <M::Keys as NetabaseModelKeyTrait<D, M>>::relational_keys(&model) {
            let rk_discriminant = rk.discriminant();
            let relational_tree_name = M::relational_key_table_name(rk_discriminant);
            let relational_tree = store.db.open_tree(relational_tree_name)?;
            relational_tree.insert(
                bincode::encode_to_vec(rk, bincode::config::standard())?,
                bincode::encode_to_vec(pk.clone(), bincode::config::standard())?,
            )?;
        }

        // 4. Insert into hash tree
        let hash_tree_name = M::hash_tree_table_name();
        let hash_tree = store.db.open_tree(hash_tree_name)?;
        hash_tree.insert(
            bincode::encode_to_vec(model_hash, bincode::config::standard())?,
            bincode::encode_to_vec(pk.clone(), bincode::config::standard())?,
        )?;

        // 5. Insert into subscription trees
        for sub in model.get_subscriptions() {
            let sub_discriminant = sub.clone();
            let subscription_tree_name = M::subscription_key_table_name(sub_discriminant);
            let subscription_tree = store.db.open_tree(subscription_tree_name)?;
            subscription_tree.insert(
                bincode::encode_to_vec(sub, bincode::config::standard())?,
                bincode::encode_to_vec(pk.clone(), bincode::config::standard())?,
            )?;
        }

        Ok(())
    }

    pub fn delete<M: NetabaseModelTrait<D> + SledNetabaseModelTrait<D> + Clone + Send>(
        &mut self,
        key: M::PrimaryKey,
    ) -> NetabaseResult<()>
    {
        let store = unsafe { &*self._sled_store };
        let model_tree_name = M::MODEL_TREE_NAME;
        let model_option = self.get::<M>(key.clone())?; // This will use the SledWriteTransaction's get method.

        // If the model exists, delete all its associated entries
        if let Some(model) = model_option {
            // 1. Delete from main tree
            let main_tree_name =
                D::get_tree_name(&model_tree_name).ok_or(NetabaseError::TreeNotFound)?;
            let main_tree = store.db.open_tree(main_tree_name)?;
            main_tree.remove(bincode::encode_to_vec(&key, bincode::config::standard())?)?;

            // 2. Delete from secondary key trees
            for sk in <M::Keys as NetabaseModelKeyTrait<D, M>>::secondary_keys(&model) {
                let sk_discriminant = sk.discriminant();
                let secondary_tree_name = M::secondary_key_table_name(sk_discriminant);
                let secondary_tree = store.db.open_tree(secondary_tree_name)?;
                secondary_tree
                    .remove(bincode::encode_to_vec(&sk, bincode::config::standard())?)?;
            }

            // 3. Delete from relational key trees
            for rk in <M::Keys as NetabaseModelKeyTrait<D, M>>::relational_keys(&model) {
                let rk_discriminant = rk.discriminant();
                let relational_tree_name = M::relational_key_table_name(rk_discriminant);
                let relational_tree = store.db.open_tree(relational_tree_name)?;
                relational_tree
                    .remove(bincode::encode_to_vec(&rk, bincode::config::standard())?)?;
            }

            // 4. Delete from hash tree
            let model_hash: [u8; 32] = model.compute_hash().into();
            let hash_tree_name = M::hash_tree_table_name();
            let hash_tree = store.db.open_tree(hash_tree_name)?;
            hash_tree.remove(bincode::encode_to_vec(&model_hash, bincode::config::standard())?)?;

            // 5. Delete from subscription trees
            for sub in model.get_subscriptions() {
                let sub_discriminant = sub.clone();
                let subscription_tree_name = M::subscription_key_table_name(sub_discriminant);
                let subscription_tree = store.db.open_tree(subscription_tree_name)?;
                subscription_tree
                    .remove(bincode::encode_to_vec(&sub, bincode::config::standard())?)?;
            }
        } else {
            debug!(
                "Attempted to delete non-existent model with primary key: {:?}",
                key
            );
        }

        Ok(())
    }

    pub fn commit(self) -> NetabaseResult<()> {
        self.db.flush()?;
        Ok(())
    }
}