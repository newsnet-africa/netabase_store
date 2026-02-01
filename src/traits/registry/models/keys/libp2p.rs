use strum::IntoDiscriminant;

use crate::{
    prelude::NetabaseDefinition,
    traits::registry::models::{
        StoreKeyMarker, model::NetabaseModelMarker,
    },
};

// This is supposed to represent the extent to which the provider stores the item the reason it is generated is because the number of blobs is arbitrary:
// pub enum <Model>Libp2pProviderKey {
//     Full(PrimaryKey),
//     Bare(PrimaryKey),
//     WithBlobs(PrimarKey, Vec<BlobKeys>),
//     WithRelations(PrimaryKey, Vec<RelationKey>)
// }
pub trait NetabaseModelLibp2pProviderKey<D: NetabaseDefinition, M: NetabaseModelMarker<D>>:
    StoreKeyMarker<D> + Clone
where
    D::Discriminant: 'static + std::fmt::Debug,
    Self: IntoDiscriminant,
{
}
