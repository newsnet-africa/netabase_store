use crate::traits::registery::definition::NetabaseDefinition;
use serde::{Serialize, Deserialize};
use bincode::{Encode, Decode, BorrowDecode};

pub mod keys;
pub mod model;
pub mod treenames;

// Marker traits to avoid cyclical dependencies
pub trait StoreKeyMarker<D: NetabaseDefinition>: Serialize + for<'de> Deserialize<'de> + Encode + Decode<()> + for<'de> BorrowDecode<'de, ()> + Eq + std::hash::Hash + PartialOrd + Ord
where
    D::Discriminant: 'static + std::fmt::Debug,
{}

pub trait StoreValueMarker<D: NetabaseDefinition>: Serialize + for<'de> Deserialize<'de> + Encode + Decode<()> + for<'de> BorrowDecode<'de, ()> + Eq + std::hash::Hash + PartialOrd + Ord
where
    D::Discriminant: 'static + std::fmt::Debug,
{}

pub trait StoreKey<D: NetabaseDefinition, V: StoreValueMarker<D> + ?Sized>: StoreKeyMarker<D>
where
    D::Discriminant: 'static + std::fmt::Debug,
{}

pub trait StoreValue<D: NetabaseDefinition, K: StoreKeyMarker<D>>: StoreValueMarker<D> 
where
    D::Discriminant: 'static + std::fmt::Debug,
{}
