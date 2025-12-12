use crate::error::NetabaseResult;
use crate::traits::definition::{DiscriminantName, NetabaseDefinition, TreeName};
use crate::traits::model::{ModelTypeContainer, NetabaseModelTrait, key::NetabaseModelKeyTrait};
use crate::traits::store::tree_manager::ModelTreeManager;
use redb::{Key, TableDefinition, Value};
use std::fmt::Debug;
use strum::{IntoDiscriminant, IntoEnumIterator};

use super::RedbStore;

/// Redb-specific extension trait for models
///
/// This trait extends NetabaseModelTrait with redb-specific requirements and operations.
/// It should only be implemented by types that need to work with redb as the backend.
pub trait RedbNetabaseModelTrait<D: NetabaseDefinition>: NetabaseModelTrait<D> + ModelTreeManager<D>
where
    Self: Value + 'static,
    Self::PrimaryKey: Key + 'static,
    <Self as ModelTypeContainer>::SecondaryKeys: Key + 'static,
    <Self as ModelTypeContainer>::RelationalKeys: Key + 'static,
    <Self as ModelTypeContainer>::Subscriptions: Key + 'static,
    <Self as NetabaseModelTrait<D>>::Hash: Key + Clone + Into<[u8; 32]> + 'static,
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    // Explicitly repeat bounds required by ModelTreeManager for default implementations
    <<<Self as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, Self>>::SecondaryEnum as IntoDiscriminant>::Discriminant: DiscriminantName + Clone + Debug + std::hash::Hash + Eq + PartialEq,
    <<<Self as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, Self>>::RelationalEnum as IntoDiscriminant>::Discriminant: DiscriminantName + Clone + Debug + std::hash::Hash + Eq + PartialEq,
    <Self as ModelTypeContainer>::Subscriptions: DiscriminantName + Clone + Debug + std::hash::Hash + Eq + PartialEq,
{
    fn definition<'a>(db: &'a RedbStore<D>) -> TableDefinition<'a, Self::PrimaryKey, Self>;

    // Secondary key table name generator
    fn secondary_key_table_name(
        key_discriminant: <<<Self as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, Self>>::SecondaryEnum as IntoDiscriminant>::Discriminant,
    ) -> String {
        Self::resolve_secondary_tree_name(TreeName::new(key_discriminant))
    }

    // Relational key table name generator
    fn relational_key_table_name(
        key_discriminant: <<<Self as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, Self>>::RelationalEnum as IntoDiscriminant>::Discriminant,
    ) -> String {
        Self::resolve_relational_tree_name(TreeName::new(key_discriminant))
    }

    // Subscription key table name generator
    fn subscription_key_table_name(
        key_discriminant: <Self as ModelTypeContainer>::Subscriptions,
    ) -> String {
        Self::resolve_subscription_tree_name(TreeName::new(key_discriminant))
    }

    // Hash tree table name generator
    fn hash_tree_table_name() -> String {
        Self::resolve_hash_tree_name()
    }
}