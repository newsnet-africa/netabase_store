use crate::traits::registery::definition::NetabaseDefinition;
use serde::{Deserialize, Serialize};

pub mod keys;
pub mod model;
pub mod treenames;
pub mod content_addressed;

pub use keys::NetabaseModelKeys;
pub use model::NetabaseModel;
pub use treenames::DiscriminantTableName;
// NetabaseDefinitionTreeNames is in definition module, not models::treenames
pub use crate::traits::registery::definition::NetabaseDefinitionTreeNames;
pub use content_addressed::ContentAddressedModel;
// Marker traits to avoid cyclical dependencies
pub trait StoreKeyMarker<D: NetabaseDefinition>:
    Serialize + for<'de> Deserialize<'de> + Eq + std::hash::Hash + PartialOrd + Ord
where
    D::Discriminant: 'static + std::fmt::Debug,
{
}

pub trait StoreValueMarker<D: NetabaseDefinition>:
    Serialize + for<'de> Deserialize<'de> + Eq + std::hash::Hash + PartialOrd + Ord
where
    D::Discriminant: 'static + std::fmt::Debug,
{
}

pub trait StoreKey<D: NetabaseDefinition, V: StoreValueMarker<D> + ?Sized>:
    StoreKeyMarker<D>
where
    D::Discriminant: 'static + std::fmt::Debug,
{
}

pub trait StoreValue<D: NetabaseDefinition, K: StoreKeyMarker<D>>: StoreValueMarker<D>
where
    D::Discriminant: 'static + std::fmt::Debug,
{
}
