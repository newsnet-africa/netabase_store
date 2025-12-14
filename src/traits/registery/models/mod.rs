use crate::traits::registery::definition::NetabaseDefinition;

pub mod keys;
pub mod model;
pub mod treenames;

pub trait StoreValueMarker {}

pub trait StoreKey<D: NetabaseDefinition, V: StoreValueMarker + ?Sized> 
where
    D::Discriminant: 'static,
{}

pub trait StoreValue<D: NetabaseDefinition, K: StoreKey<D, Self>>: StoreValueMarker 
where
    D::Discriminant: 'static,
{}
