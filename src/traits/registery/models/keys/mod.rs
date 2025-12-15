pub use crate::traits::registery::models::keys::primary::NetabaseModelPrimaryKey;
pub use crate::traits::registery::models::keys::relational::{
    NetabaseModelRelationalKey, NetabaseModelRelationalKeyForeign,
};
pub use crate::traits::registery::models::keys::secondary::NetabaseModelSecondaryKey;
use crate::traits::registery::{
    definition::NetabaseDefinition, models::model::NetabaseModelMarker,
};

pub mod primary;
pub mod relational;
pub mod secondary;

pub trait NetabaseModelKeys<D: NetabaseDefinition, M: NetabaseModelMarker<D>>:
    std::marker::Sized
where
    D::Discriminant: 'static,
    for<'a> Self::Primary<'a>: redb::Key,
    for<'a> Self::Secondary<'a>: redb::Key,
    for<'a> Self::Relational<'a>: redb::Key,
{
    type Primary<'a>: NetabaseModelPrimaryKey<'a, D, M, Self>;
    type Secondary<'a>: NetabaseModelSecondaryKey<'a, D, M, Self>;
    type Relational<'a>: NetabaseModelRelationalKey<'a, D, M, Self>; // More flexible, specific foreign types defined elsewhere
}
