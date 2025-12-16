pub mod redb_model;
pub use redb_model::*;

use strum::IntoDiscriminant;

use crate::{
    traits::registery::{
        definition::NetabaseDefinition,
        models::{StoreKeyMarker, StoreValue, StoreValueMarker, keys::NetabaseModelKeys, treenames::ModelTreeNames},
    },
    relational::GlobalDefinitionEnum,
};

pub trait NetabaseModelMarker<D: NetabaseDefinition>: StoreValueMarker<D> 
where 
    D::Discriminant: 'static, <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug
{}

pub trait NetabaseModel<D: NetabaseDefinition>:
    NetabaseModelMarker<D>
    + Sized
    + for<'a> StoreValue<D, <Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'a>>
where
    D::Discriminant: 'static + std::fmt::Debug,
    D: GlobalDefinitionEnum,
    for<'a> <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Primary<'a>:
        StoreKeyMarker<D>,
    for<'a> <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Secondary<'a>:
        IntoDiscriminant,
    for<'a> <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Relational<'a>:
        IntoDiscriminant,
    for<'a> <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription<'a>:
        IntoDiscriminant,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Secondary<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Relational<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,

     <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription<'static>: 'static
{
    type Keys: NetabaseModelKeys<D, Self>;
    const TREE_NAMES: ModelTreeNames<'static, D, Self>;

    /// Model-level permissions (outbound, inbound, cross-definition)
    const PERMISSIONS: crate::traits::permissions::ModelPermissions<'static, D>;


    fn get_primary_key<'a>(&'a self) -> <Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'a>;
    fn get_secondary_keys<'a>(
        &'a self,
    ) -> Vec<<Self::Keys as NetabaseModelKeys<D, Self>>::Secondary<'a>>;
    fn get_relational_keys<'a>(
        &'a self,
    ) -> Vec<<Self::Keys as NetabaseModelKeys<D, Self>>::Relational<'a>>;
    fn get_subscription_keys<'a>(
        &'a self,
    ) -> Vec<<Self::Keys as NetabaseModelKeys<D, Self>>::Subscription<'a>>;

    /// Get all relational links from this model
    fn get_relational_links(&self) -> Vec<D::DefKeys> {
        Vec::new() // Default implementation returns empty
    }

    /// Check if this model has any relational links
    fn has_relational_links(&self) -> bool {
        !self.get_relational_links().is_empty()
    }

    /// Get the number of relational links in this model
    fn relational_link_count(&self) -> usize {
        self.get_relational_links().len()
    }
}
