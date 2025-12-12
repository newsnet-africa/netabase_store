use crate::traits::definition::{DiscriminantName, TreeName};
use strum::{IntoDiscriminant, IntoEnumIterator};

use crate::{
    prelude::{NetabaseDefinition, NetabaseModelTrait},
    traits::model::{ModelTypeContainer, key::NetabaseModelKeyTrait},
};

pub trait ModelTreeManager<D: NetabaseDefinition>: NetabaseModelTrait<D>
where
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator + std::hash::Hash + std::cmp::Eq + std::fmt::Debug + DiscriminantName + std::clone::Clone + PartialEq,
{
    type SecondaryTreeName: Clone + std::fmt::Debug + std::hash::Hash + std::cmp::Eq + PartialEq + DiscriminantName + IntoEnumIterator = <<<Self as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<
        D,
        Self,
    >>::SecondaryEnum as IntoDiscriminant>::Discriminant;

    type RelationalTreeName: Clone + std::fmt::Debug + std::hash::Hash + std::cmp::Eq + PartialEq + DiscriminantName + IntoEnumIterator = <<<Self as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<
        D,
        Self,
    >>::RelationalEnum as IntoDiscriminant>::Discriminant;

    type SubscriptionTreeNames: Clone + std::fmt::Debug + std::hash::Hash + std::cmp::Eq + PartialEq + DiscriminantName + IntoEnumIterator = <Self as ModelTypeContainer>::Subscriptions;

    fn secondary_tree_names() -> <Self::SecondaryTreeName as IntoEnumIterator>::Iterator 
    {
        <Self::SecondaryTreeName as IntoEnumIterator>::iter()
    }

    fn relational_tree_names() -> <Self::RelationalTreeName as IntoEnumIterator>::Iterator 
    {
        <Self::RelationalTreeName as IntoEnumIterator>::iter()
    }

    fn subscription_tree_names() -> <Self::SubscriptionTreeNames as IntoEnumIterator>::Iterator 
    {
        <Self::SubscriptionTreeNames as IntoEnumIterator>::iter()
    }

    // Centralized Tree Name Resolution Logic
    // Using explicit types instead of associated types to ensure type compatibility with RedbNetabaseModelTrait

    fn resolve_main_tree_name() -> String {
        Self::MODEL_TREE_NAME.name().to_string()
    }

    fn resolve_secondary_tree_name(
        key: impl DiscriminantName
    ) -> String {
        format!("{}_{}", Self::resolve_main_tree_name(), key.name())
    }

    fn resolve_relational_tree_name(
        key: impl DiscriminantName
    ) -> String {
        format!("{}_rel_{}", Self::resolve_main_tree_name(), key.name())
    }

    fn resolve_subscription_tree_name(
        key: impl DiscriminantName
    ) -> String {
        format!("{}_sub_{}", Self::resolve_main_tree_name(), key.name())
    }

    fn resolve_hash_tree_name() -> String {
        format!("{}_hash", Self::resolve_main_tree_name())
    }
}

pub trait TreeManager<D>: IntoEnumIterator {
    fn model_managers() -> <Self as IntoEnumIterator>::Iterator {
        <Self as IntoEnumIterator>::iter()
    }
}

/// Standardized generic tree name wrapper for all models
/// This allows using concrete types for tree names while maintaining a consistent structure
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StandardModelTreeName<D, M>
where
    D: NetabaseDefinition,
    M: ModelTreeManager<D>,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator + std::hash::Hash + std::cmp::Eq + std::fmt::Debug + DiscriminantName + std::clone::Clone,
{
    Main,
    Secondary(M::SecondaryTreeName),
    Relational(M::RelationalTreeName),
    Subscription(M::SubscriptionTreeNames),
}

impl<D, M> StandardModelTreeName<D, M>
where
    D: NetabaseDefinition,
    M: ModelTreeManager<D>,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator + std::hash::Hash + std::cmp::Eq + std::fmt::Debug + DiscriminantName + std::clone::Clone,
{
    /// Resolves the tree name to its string representation using the centralized logic
    pub fn resolve(&self) -> String {
        match self {
            Self::Main => M::resolve_main_tree_name(),
            Self::Secondary(k) => M::resolve_secondary_tree_name(TreeName::new(k.clone())),
            Self::Relational(k) => M::resolve_relational_tree_name(TreeName::new(k.clone())),
            Self::Subscription(k) => M::resolve_subscription_tree_name(TreeName::new(k.clone())),
        }
    }
}

impl<D, M> AsRef<str> for StandardModelTreeName<D, M>
where
    D: NetabaseDefinition,
    M: ModelTreeManager<D>,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator + std::hash::Hash + std::cmp::Eq + std::fmt::Debug + DiscriminantName + std::clone::Clone,
{
    fn as_ref(&self) -> &str {
        match self {
            Self::Main => "Main",
            Self::Secondary(_) => "Secondary",
            Self::Relational(_) => "Relational",
            Self::Subscription(_) => "Subscription",
        }
    }
}
