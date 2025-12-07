use redb::{Key, TableDefinition, Value};
use std::fmt::Debug;
use strum::{AsRefStr, IntoDiscriminant, IntoEnumIterator};

use crate::{
    databases::redb_store::RedbStore,
    traits::{
        definition::{DiscriminantName, NetabaseDefinition},
        model::key::NetabaseModelKeyTrait,
    },
};

pub mod key;

// User defined struct
pub trait NetabaseModelTrait<D: NetabaseDefinition>: std::marker::Sized + Clone + Send
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName,
{
    type Keys: NetabaseModelKeyTrait<D, Self>;
    const MODEL_TREE_NAME: <D as strum::IntoDiscriminant>::Discriminant;

    type PrimaryKey = <Self::Keys as NetabaseModelKeyTrait<D, Self>>::PrimaryKey;
    type SecondaryKeys: Iterator<
        Item = <Self::Keys as NetabaseModelKeyTrait<D, Self>>::SecondaryEnum,
    >;
    type RelationalKeys: Iterator<
        Item = <Self::Keys as NetabaseModelKeyTrait<D, Self>>::RelationalEnum,
    >;
    type Hash: Clone + Send + Debug; // Blake3 hash type

    fn primary_key(&self) -> Self::PrimaryKey;

    // Concrete value functions
    fn get_secondary_keys(&self) -> Self::SecondaryKeys;
    fn get_relational_keys(&self) -> Self::RelationalKeys;
    fn compute_hash(&self) -> Self::Hash;
}

pub trait RedbNetabaseModelTrait<D: NetabaseDefinition>: NetabaseModelTrait<D>
where
    Self: Value + 'static,
    Self::PrimaryKey: Key + 'static,
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName,
{
    fn definition<'a>(db: &RedbStore<D>) -> TableDefinition<'a, Self::PrimaryKey, Self>;
}
