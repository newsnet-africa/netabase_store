use crate::traits::registery::definition::NetabaseDefinition;
use crate::traits::registery::models::StoreKey;
use crate::traits::registery::models::keys::NetabaseModelKeys;
use crate::traits::registery::models::model::NetabaseModelMarker;
pub trait NetabaseModelRelationalKey<
    'a,
    D: NetabaseDefinition,
    M: NetabaseModelMarker,
    K: NetabaseModelKeys<D, M>,
>: StoreKey<D, K::Primary<'a>>
where
    D::Discriminant: 'static,
{
}
