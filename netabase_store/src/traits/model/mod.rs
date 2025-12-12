use std::fmt::Debug;
use strum::{IntoDiscriminant, IntoEnumIterator};

use crate::traits::{
    definition::{DiscriminantName, NetabaseDefinition},
    model::key::NetabaseModelKeyTrait,
};

pub mod key;
pub mod relational;

pub use relational::RelationalLink;

/// Trait for model-specific nested type containers
/// This provides complete compile-time type safety and eliminates flat enum issues
pub trait ModelTypeContainer {
    type PrimaryKey: Clone + std::fmt::Debug + Send + 'static;
    type SecondaryKeys: IntoDiscriminant
        + IntoEnumIterator
        + Clone
        + std::fmt::Debug
        + Send
        + 'static;
    type RelationalKeys: IntoDiscriminant
        + IntoEnumIterator
        + Clone
        + std::fmt::Debug
        + Send
        + 'static;
    type Subscriptions: IntoEnumIterator + Clone + std::fmt::Debug + Send + AsRef<str> + 'static;

    /// Type-safe string conversion for tree names - replaces weak &str arguments
    type TreeName: AsRef<str> + Clone + std::fmt::Debug + 'static;

    /// Get the primary tree name for this model type
    fn primary_tree_name() -> Self::TreeName;
}

// User defined struct
pub trait NetabaseModelTrait<D: NetabaseDefinition>:
    std::marker::Sized + Clone + Send + ModelTypeContainer
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    type Keys: NetabaseModelKeyTrait<D, Self>
    where
        <D as strum::IntoDiscriminant>::Discriminant: AsRef<str>;
    type Hash: Clone + Send + Debug; // Blake3 hash type

    const MODEL_TREE_NAME: <D as strum::IntoDiscriminant>::Discriminant;

    fn primary_key(&self) -> Self::PrimaryKey;

    fn tree_name(&self) -> <D as strum::IntoDiscriminant>::Discriminant {
        Self::MODEL_TREE_NAME
    }
    // Concrete value functions
    fn get_secondary_keys(&self) -> Self::SecondaryKeys;
    fn get_relational_keys(&self) -> Self::RelationalKeys;
    fn get_subscriptions(&self) -> Vec<Self::Subscriptions>;
    fn compute_hash(&self) -> Self::Hash;
}
