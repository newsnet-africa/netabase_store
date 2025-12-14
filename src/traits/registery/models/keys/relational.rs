use crate::traits::registery::definition::NetabaseDefinition;
use crate::traits::registery::models::StoreKey;
use crate::traits::registery::models::keys::NetabaseModelKeys;
pub use crate::traits::registery::models::keys::primary::NetabaseModelPrimaryKey;
use crate::traits::registery::models::model::NetabaseModelMarker;
pub trait NetabaseModelSecondaryKey<
    'a,
    D: NetabaseDefinition,
    M: NetabaseModelMarker,
    K: NetabaseModelKeys<D, M>,
>: StoreKey<D, K::Primary<'a>>
where
    D::Discriminant: 'static,
{
}
