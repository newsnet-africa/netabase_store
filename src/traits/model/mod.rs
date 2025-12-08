use redb::{Key, TableDefinition, Value};
use std::fmt::Debug;
use strum::{IntoDiscriminant, IntoEnumIterator};

use crate::{
    databases::redb_store::RedbStore,
    traits::{
        definition::{DiscriminantName, NetabaseDefinition},
        model::key::NetabaseModelKeyTrait,
    },
};

pub mod key;
pub mod relational;

pub use relational::RelationalLink;


// User defined struct
pub trait NetabaseModelTrait<D: NetabaseDefinition>: std::marker::Sized + Clone + Send
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
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

    // Type wrapping methods for ModelAssociatedTypes
    fn wrap_primary_key(key: Self::PrimaryKey) -> D::ModelAssociatedTypes;
    fn wrap_model(model: Self) -> D::ModelAssociatedTypes;
    fn wrap_secondary_key(key: <Self::Keys as NetabaseModelKeyTrait<D, Self>>::SecondaryEnum) -> D::ModelAssociatedTypes;
    fn wrap_relational_key(key: <Self::Keys as NetabaseModelKeyTrait<D, Self>>::RelationalEnum) -> D::ModelAssociatedTypes;
    fn wrap_secondary_key_discriminant(key: <<Self::Keys as NetabaseModelKeyTrait<D, Self>>::SecondaryEnum as IntoDiscriminant>::Discriminant) -> D::ModelAssociatedTypes;
    fn wrap_relational_key_discriminant(key: <<Self::Keys as NetabaseModelKeyTrait<D, Self>>::RelationalEnum as IntoDiscriminant>::Discriminant) -> D::ModelAssociatedTypes;
}

pub trait RedbNetabaseModelTrait<D: NetabaseDefinition>: NetabaseModelTrait<D>
where
    Self: Value + 'static,
    Self::PrimaryKey: Key + 'static,
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    fn definition<'a>(db: &RedbStore<D>) -> TableDefinition<'a, Self::PrimaryKey, Self>;

    // Secondary key table name generator
    fn secondary_key_table_name(
        key_discriminant: <<Self::Keys as NetabaseModelKeyTrait<D, Self>>::SecondaryEnum as IntoDiscriminant>::Discriminant,
    ) -> String;

    // Relational key table name generator
    fn relational_key_table_name(
        key_discriminant: <<Self::Keys as NetabaseModelKeyTrait<D, Self>>::RelationalEnum as IntoDiscriminant>::Discriminant,
    ) -> String;

    // Hash tree table name generator
    fn hash_tree_table_name() -> String;
}
