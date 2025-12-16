use crate::traits::registery::{
    definition::NetabaseDefinition,
    models::{keys::NetabaseModelKeys, model::NetabaseModelMarker},
};

pub trait NetabaseModelSubscriptionKey<
    D: NetabaseDefinition,
    M: NetabaseModelMarker<D>,
    K: NetabaseModelKeys<D, M>,
> where
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: 'static,
{
}
