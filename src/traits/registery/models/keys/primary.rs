use crate::traits::registery::definition::NetabaseDefinition;
use crate::traits::registery::models::model::NetabaseModelMarker;
use crate::traits::registery::models::{StoreKeyMarker, StoreValueMarker};

/// Marker trait for primary key types.
///
/// This is a simple marker trait without the K parameter to avoid
/// early/late-bound lifetime issues with GATs.
pub trait NetabaseModelPrimaryKey<D: NetabaseDefinition, M: NetabaseModelMarker<D>>:
    StoreValueMarker<D> + StoreKeyMarker<D> + Clone
where
    D::Discriminant: 'static + std::fmt::Debug,
{
}
