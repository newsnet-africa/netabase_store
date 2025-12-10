//! Sled-specific store trait
//!
//! This module defines a store trait specific to the sled backend. This mirrors
//! the redb `StoreTrait` but uses sled-specific type bounds.
//!
//! ## Future Work
//!
//! This parallel trait hierarchy is a temporary solution. Future refactoring should
//! create a unified backend-agnostic trait system that both redb and sled can implement.

use crate::{
    databases::sled_store::{SledNetabaseModelTrait, SledReadTransaction, SledWriteTransaction},
    error::NetabaseResult,
    traits::{
        definition::{DiscriminantName, NetabaseDefinition},
        model::{NetabaseModelTrait, key::NetabaseModelKeyTrait},
    },
};
use std::fmt::Debug;
use strum::{IntoDiscriminant, IntoEnumIterator};

/// Sled-specific store trait
///
/// This trait provides the same API as `StoreTrait` but with sled-specific type bounds.
/// It allows sled stores to provide transaction-based access to the database.
///
/// ## Note
///
/// This is a separate trait from `StoreTrait` because the current trait hierarchy
/// is tightly coupled to redb. Future refactoring should unify these under a common
/// backend-agnostic abstraction.
pub trait SledStoreTrait<D: NetabaseDefinition>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    /// Execute a read transaction
    fn read<'a, F, R>(&'a self, f: F) -> NetabaseResult<R>
    where
        F: FnOnce(&SledReadTransaction<'a, D>) -> NetabaseResult<R>;

    /// Execute a write transaction
    fn write<F, R>(&self, f: F) -> NetabaseResult<R>
    where
        F: FnOnce(&mut SledWriteTransaction<D>) -> NetabaseResult<R>;

    /// Get a single model by its primary key (convenience method)
    fn get_one<M>(&self, key: M::PrimaryKey) -> NetabaseResult<Option<M>>
    where
        M: NetabaseModelTrait<D> + SledNetabaseModelTrait<D>,
        M: bincode::Encode + bincode::Decode<()> + 'static,
        M::PrimaryKey: bincode::Encode + bincode::Decode<()> + 'static,
    {
        self.read(|txn| txn.get::<M>(key))
    }

    /// Put a single model (convenience method)
    fn put_one<M>(&self, model: M) -> NetabaseResult<()>
    where
        M: NetabaseModelTrait<D> + SledNetabaseModelTrait<D> + Clone + Send,
        M: bincode::Encode + bincode::Decode<()> + 'static,
        M::PrimaryKey: bincode::Encode + bincode::Decode<()> + Send + Clone + 'static,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum:
            IntoDiscriminant + Clone + bincode::Encode,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum:
            IntoDiscriminant + Clone + bincode::Encode,
        M::Hash: Into<[u8; 32]>,
    {
        self.write(|txn| txn.put(model))
    }

    /// Put multiple models in a single transaction (batch operation)
    fn put_many<M>(&self, models: Vec<M>) -> NetabaseResult<()>
    where
        M: NetabaseModelTrait<D> + SledNetabaseModelTrait<D> + Clone + Send,
        M: bincode::Encode + bincode::Decode<()> + 'static,
        M::PrimaryKey: bincode::Encode + bincode::Decode<()> + Send + Clone + 'static,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum:
            IntoDiscriminant + Clone + bincode::Encode,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum:
            IntoDiscriminant + Clone + bincode::Encode,
        M::Hash: Into<[u8; 32]>,
    {
        self.write(|txn| {
            for model in models {
                txn.put(model)?;
            }
            Ok(())
        })
    }

    /// Get multiple models by their primary keys in a single read transaction
    fn get_many<M>(&self, keys: Vec<M::PrimaryKey>) -> NetabaseResult<Vec<Option<M>>>
    where
        M: NetabaseModelTrait<D> + SledNetabaseModelTrait<D>,
        M: bincode::Encode + bincode::Decode<()> + 'static,
        M::PrimaryKey: bincode::Encode + bincode::Decode<()> + 'static,
    {
        self.read(|txn| {
            let mut results = Vec::with_capacity(keys.len());
            for key in keys {
                results.push(txn.get::<M>(key)?);
            }
            Ok(results)
        })
    }

    /// Delete a model by its primary key (convenience method)
    fn delete_one<M>(&self, key: M::PrimaryKey) -> NetabaseResult<()>
    where
        M: NetabaseModelTrait<D> + SledNetabaseModelTrait<D> + Clone + Send,
        M: bincode::Encode + bincode::Decode<()> + 'static,
        M::PrimaryKey: bincode::Encode + bincode::Decode<()> + Send + Clone + 'static,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum:
            IntoDiscriminant + Clone + bincode::Encode,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum:
            IntoDiscriminant + Clone + bincode::Encode,
    {
        self.write(|txn| txn.delete::<M>(key))
    }
}
