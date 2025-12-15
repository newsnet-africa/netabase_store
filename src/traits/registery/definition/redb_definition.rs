use crate::traits::registery::definition::NetabaseDefinition;
use strum::IntoDiscriminant;

pub trait RedbDefinition: NetabaseDefinition
where
    <Self as IntoDiscriminant>::Discriminant: 'static
{}
