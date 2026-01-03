use crate::traits::registery::definition::NetabaseDefinition;
use crate::traits::registery::models::StoreKeyMarker;
pub use crate::traits::registery::models::keys::primary::NetabaseModelPrimaryKey;
use crate::traits::registery::models::model::NetabaseModelMarker;

/// Marker trait for secondary key types.
///
/// This is a simple marker trait without the K parameter to avoid
/// early/late-bound lifetime issues with GATs.
pub trait NetabaseModelSecondaryKey<D: NetabaseDefinition, M: NetabaseModelMarker<D>>:
    StoreKeyMarker<D> + Clone
where
    D::Discriminant: 'static + std::fmt::Debug,
{
    type PrimaryKey: NetabaseModelPrimaryKey<D, M>;
}
