use crate::traits::registery::{
    definition::NetabaseDefinition, models::model::NetabaseModelMarker,
};

/// Marker trait for subscription key types.
///
/// This is a simple marker trait without the K parameter to avoid
/// early/late-bound lifetime issues with GATs.
pub trait NetabaseModelSubscriptionKey<D: NetabaseDefinition, M: NetabaseModelMarker<D>>:
    From<D::SubscriptionKeys> + TryInto<D::SubscriptionKeys>
where
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: 'static,
{
}
