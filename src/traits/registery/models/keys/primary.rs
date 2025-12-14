use crate::traits::registery::definition::NetabaseDefinition;
use crate::traits::registery::models::model::NetabaseModelMarker;
use crate::traits::registery::models::{
    StoreKey, StoreValue, StoreValueMarker, keys::NetabaseModelKeys,
};

pub trait NetabaseModelPrimaryKey<
    'a,
    D: NetabaseDefinition,
    M: NetabaseModelMarker,
    K: NetabaseModelKeys<D, M>,
>:
    StoreValueMarker
    + StoreKey<D, M>
    + StoreValue<D, K::Secondary<'a>>
    + StoreValue<D, K::Relational<'a>> 
where
    D::Discriminant: 'static,
    K::Secondary<'a>: StoreKey<D, Self>,
    K::Relational<'a>: StoreKey<D, Self>,
{
}
