use crate::traits::definition::{NetabaseDefinition, DiscriminantName};
use crate::traits::model::{NetabaseModelTrait, key::NetabaseModelKeyTrait};
use crate::error::NetabaseResult;
use redb::{Key, TableDefinition, Value};
use std::fmt::Debug;
use strum::{IntoDiscriminant, IntoEnumIterator};

use super::RedbStore;

/// Redb-specific extension trait for models
///
/// This trait extends NetabaseModelTrait with redb-specific requirements and operations.
/// It should only be implemented by types that need to work with redb as the backend.
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

    // Subscription key table name generator
    fn subscription_key_table_name(
        key_discriminant: <<Self as NetabaseModelTrait<D>>::SubscriptionEnum as IntoDiscriminant>::Discriminant,
    ) -> String;

    // Hash tree table name generator
    fn hash_tree_table_name() -> String;
}

/// Extension trait for ModelAssociatedTypes to provide Redb execution methods
pub trait RedbModelAssociatedTypesExt<D>
where
    D: NetabaseDefinition,
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    /// Insert a model into the main tree
    fn insert_model_into_redb(
        &self,
        txn: &redb::WriteTransaction,
        table_name: &str,
        key: &D::ModelAssociatedTypes,
    ) -> NetabaseResult<()>;

    /// Insert a secondary key mapping
    fn insert_secondary_key_into_redb(
        &self,
        txn: &redb::WriteTransaction,
        table_name: &str,
        primary_key_ref: &D::ModelAssociatedTypes,
    ) -> NetabaseResult<()>;

    /// Insert a relational key mapping
    fn insert_relational_key_into_redb(
        &self,
        txn: &redb::WriteTransaction,
        table_name: &str,
        primary_key_ref: &D::ModelAssociatedTypes,
    ) -> NetabaseResult<()>;

    /// Insert a hash tree mapping
    fn insert_hash_into_redb(
        hash: &[u8; 32],
        txn: &redb::WriteTransaction,
        table_name: &str,
        primary_key_ref: &D::ModelAssociatedTypes,
    ) -> NetabaseResult<()>;

    /// Insert a subscription tree mapping
    fn insert_subscription_into_redb(
        hash: &[u8; 32],
        txn: &redb::WriteTransaction,
        table_name: &str,
        primary_key_ref: &D::ModelAssociatedTypes,
    ) -> NetabaseResult<()>;

    /// Delete a model from the main tree
    fn delete_model_from_redb(
        &self,
        txn: &redb::WriteTransaction,
        table_name: &str,
    ) -> NetabaseResult<()>;

    /// Delete a subscription tree mapping
    fn delete_subscription_from_redb(
        &self,
        txn: &redb::WriteTransaction,
        table_name: &str,
    ) -> NetabaseResult<()>;
}
