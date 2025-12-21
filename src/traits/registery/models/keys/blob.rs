use strum::IntoDiscriminant;

use crate::traits::registery::{
    definition::NetabaseDefinition,
    models::{
        StoreKey, StoreKeyMarker, StoreValue, StoreValueMarker, keys::NetabaseModelKeys,
        model::NetabaseModelMarker,
    },
};

pub trait NetabaseModelBlobKey<
    'a,
    D: NetabaseDefinition,
    M: NetabaseModelMarker<D>,
    K: NetabaseModelKeys<D, M> + 'static,
>: StoreKeyMarker<D> + Clone + StoreKey<D, Self::BlobTypes> where
    D::Discriminant: 'static + std::fmt::Debug,
    <<K as NetabaseModelKeys<D, M>>::Blob<'a> as IntoDiscriminant>::Discriminant: 'static,
    <Self::BlobTypes as IntoDiscriminant>::Discriminant: 'static,
{
    type BlobTypes: StoreValue<D, Self> + StoreValueMarker<D> + 'static + IntoDiscriminant;
}
