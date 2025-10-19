use strum::IntoDiscriminant;

use crate::definition::{NetabaseDefinitionTrait, NetabaseDefinitionTraitKey};

/// Trait for user-defined models that can be stored in the database.
///
/// This trait is automatically derived via the `#[derive(NetabaseModel)]` macro.
/// Models must have:
/// - A primary key field marked with `#[primary_key]`
/// - Optional secondary key fields marked with `#[secondary_key]`
pub trait NetabaseModelTrait<D: NetabaseDefinitionTrait>:
    bincode::Encode + Sized + Clone + Send + Sync + 'static
where
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::convert::AsRef<str>,
{
    const DISCRIMINANT: <D as IntoDiscriminant>::Discriminant;

    /// The primary key type for this model
    type PrimaryKey: NetabaseModelTraitKey<D>;

    /// The secondary keys enum for this model
    type SecondaryKeys: NetabaseModelTraitKey<D>;

    /// The keys enum that wraps both primary and secondary keys
    type Keys: NetabaseModelTraitKey<D>;

    /// Extract the primary key from the model instance
    fn primary_key(&self) -> Self::PrimaryKey;

    /// Extract all secondary keys from the model instance
    fn secondary_keys(&self) -> Vec<Self::SecondaryKeys>;

    /// Get the discriminant name for this model (used for tree names)
    fn discriminant_name() -> &'static str;
}

/// Marker trait for key types (both primary and secondary).
///
/// This trait is automatically implemented by the macro-generated key types.
pub trait NetabaseModelTraitKey<D: NetabaseDefinitionTrait>:
    bincode::Encode + std::fmt::Debug + Clone + Send + Sync + 'static
where
    <<D as NetabaseDefinitionTrait>::Keys as IntoDiscriminant>::Discriminant:
        Clone + Copy + std::fmt::Debug + PartialEq + Eq + std::hash::Hash + Send + Sync + 'static,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::str::FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::convert::AsRef<str>,
{
    const DISCRIMINANT: <<D as NetabaseDefinitionTrait>::Keys as IntoDiscriminant>::Discriminant;
}
