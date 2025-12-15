use crate::traits::registery::definition::NetabaseDefinition;
use crate::traits::registery::models::model::NetabaseModelMarker;
use crate::traits::registery::models::{
    StoreKeyMarker, StoreValueMarker, keys::NetabaseModelKeys,
};

pub trait NetabaseModelPrimaryKey<
    'a,
    D: NetabaseDefinition,
    M: NetabaseModelMarker<D>,
    K: NetabaseModelKeys<D, M>,
>:
    StoreValueMarker<D>
    + StoreKeyMarker<D>
    + Clone
where
    D::Discriminant: 'static,
    K::Secondary<'a>: StoreKeyMarker<D>,
    K::Relational<'a>: StoreKeyMarker<D>,
{
}
