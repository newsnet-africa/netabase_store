//! Sled-specific trait extensions for Netabase models
//!
//! This module provides trait extensions that add sled-specific functionality
//! to Netabase models. These traits mirror the redb-specific traits but require
//! bincode serialization instead of redb's Key and Value traits.

use crate::{
    error::NetabaseResult,
    traits::{
        definition::{DiscriminantName, NetabaseDefinition, TreeName},
        model::{ModelTypeContainer, NetabaseModelTrait, key::NetabaseModelKeyTrait},
        store::tree_manager::ModelTreeManager,
    },
};
use std::fmt::Debug;
use strum::{IntoDiscriminant, IntoEnumIterator};

/// Sled-specific extension trait for models
///
/// This trait extends `NetabaseModelTrait` with sled-specific requirements and operations.
/// Unlike `RedbNetabaseModelTrait` which requires `redb::Key` and `redb::Value` traits,
/// this trait requires `bincode::Encode` and `bincode::Decode` for serialization.
///
/// ## Type Requirements
///
/// - `Self`: Must implement `bincode::Encode` and `bincode::Decode`
/// - `Self::PrimaryKey`: Must implement `bincode::Encode` and `bincode::Decode`
///
/// ## Tree Naming
///
/// This trait uses `ModelTreeManager` for generating sled tree names to ensure
/// consistency with other backends (like redb).
///
/// ## Example
///
/// ```ignore
/// use netabase_store::databases::sled_store::SledNetabaseModelTrait;
///
/// impl<D: NetabaseDefinition> SledNetabaseModelTrait<D> for User
/// where
///     <D as IntoDiscriminant>::Discriminant:
///         IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
/// {
///     // Methods use default implementations from ModelTreeManager
/// }
/// ```
pub trait SledNetabaseModelTrait<D: NetabaseDefinition>: NetabaseModelTrait<D> + ModelTreeManager<D>
where
    Self: bincode::Encode + bincode::Decode<()> + 'static,
    Self::PrimaryKey: bincode::Encode + bincode::Decode<()> + Send + Clone + 'static,
    <Self as NetabaseModelTrait<D>>::Hash: Clone + Into<[u8; 32]> + 'static,
    <Self as ModelTypeContainer>::SecondaryKeys: bincode::Encode + bincode::Decode<()> + 'static,
    <Self as ModelTypeContainer>::RelationalKeys: bincode::Encode + bincode::Decode<()> + 'static,
    <Self as ModelTypeContainer>::Subscriptions: bincode::Encode + bincode::Decode<()> + 'static,
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    // Explicitly repeat bounds required by ModelTreeManager for default implementations
    <<<Self as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, Self>>::SecondaryEnum as IntoDiscriminant>::Discriminant: DiscriminantName + Clone + Debug + std::hash::Hash + Eq + PartialEq,
    <<<Self as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, Self>>::RelationalEnum as IntoDiscriminant>::Discriminant: DiscriminantName + Clone + Debug + std::hash::Hash + Eq + PartialEq,
    <Self as ModelTypeContainer>::Subscriptions: DiscriminantName + Clone + Debug + std::hash::Hash + Eq + PartialEq,
{
    /// Generate the tree name for a secondary key index
    fn secondary_key_table_name(
        key_discriminant: <<<Self as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, Self>>::SecondaryEnum as IntoDiscriminant>::Discriminant,
    ) -> String {
        Self::resolve_secondary_tree_name(TreeName::new(key_discriminant))
    }

    /// Generate the tree name for a relational key index
    fn relational_key_table_name(
        key_discriminant: <<<Self as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, Self>>::RelationalEnum as IntoDiscriminant>::Discriminant,
    ) -> String {
        Self::resolve_relational_tree_name(TreeName::new(key_discriminant))
    }

    /// Generate the tree name for a subscription index
    fn subscription_key_table_name(
        key_discriminant: <Self as ModelTypeContainer>::Subscriptions,
    ) -> String {
        Self::resolve_subscription_tree_name(TreeName::new(key_discriminant))
    }

    /// Generate the tree name for the hash tree
    fn hash_tree_table_name() -> String {
        Self::resolve_hash_tree_name()
    }
}