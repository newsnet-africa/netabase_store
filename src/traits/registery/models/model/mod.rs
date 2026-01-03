pub mod redb_model;
pub use redb_model::*;

use strum::IntoDiscriminant;

use crate::traits::registery::{
    definition::NetabaseDefinition,
    models::{
        StoreKeyMarker, StoreValue, StoreValueMarker,
        keys::{NetabaseModelKeys, blob::NetabaseModelBlobKey},
        treenames::ModelTreeNames,
    },
};

pub trait NetabaseModelMarker<D: NetabaseDefinition>: StoreValueMarker<D>
where
    D::Discriminant: 'static,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
{
}

pub trait NetabaseModel<D: NetabaseDefinition>:
    NetabaseModelMarker<D>
    + Sized
    + Into<D>
    + TryFrom<D>
    + StoreValue<D, <Self::Keys as NetabaseModelKeys<D, Self>>::Primary>
where
    D::Discriminant: 'static + std::fmt::Debug,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Primary:
        StoreKeyMarker<D>,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Secondary:
        IntoDiscriminant,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Relational:
        IntoDiscriminant,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription:
        IntoDiscriminant,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob:
        IntoDiscriminant,
    <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Secondary as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Relational as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
     <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription: 'static
{
    type Keys: NetabaseModelKeys<D, Self>;
    const TREE_NAMES: ModelTreeNames<'static, D, Self>;


    fn get_primary_key(&self) -> <Self::Keys as NetabaseModelKeys<D, Self>>::Primary;
    fn get_secondary_keys(
        &self,
    ) -> Vec<<Self::Keys as NetabaseModelKeys<D, Self>>::Secondary>;
    fn get_relational_keys(
        &self,
    ) -> Vec<<Self::Keys as NetabaseModelKeys<D, Self>>::Relational>;
    fn get_subscription_keys(
        &self,
    ) -> Vec<<Self::Keys as NetabaseModelKeys<D, Self>>::Subscription>;
    fn get_blob_entries(
        &self,
    ) -> Vec<Vec<(
        <Self::Keys as NetabaseModelKeys<D, Self>>::Blob,
        <<Self::Keys as NetabaseModelKeys<D, Self>>::Blob as NetabaseModelBlobKey<D, Self>>::BlobItem,
    )>>;

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
