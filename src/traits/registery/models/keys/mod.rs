pub use crate::traits::registery::models::keys::primary::NetabaseModelPrimaryKey;
pub use crate::traits::registery::models::keys::relational::NetabaseModelSecondaryKey;
pub use crate::traits::registery::models::keys::secondary::NetabaseModelRelationalKey;
use crate::traits::registery::{
    definition::NetabaseDefinition, models::model::NetabaseModelMarker,
};

pub mod primary;
pub mod relational;
pub mod secondary;

pub trait NetabaseModelKeys<D: NetabaseDefinition, M: NetabaseModelMarker>:
    std::marker::Sized
where
    D::Discriminant: 'static,
{
    type Primary<'a>: NetabaseModelPrimaryKey<'a, D, M, Self>;
    type Secondary<'a>: NetabaseModelSecondaryKey<'a, D, M, Self>;
    type Relational<'a>: NetabaseModelRelationalKey<'a, D, M, Self>;
}
