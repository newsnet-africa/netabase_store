pub mod redb_definition;
pub mod subscription;

use strum::IntoDiscriminant;
use subscription::DefinitionSubscriptionRegistry;

pub trait NetabaseDefinition: IntoDiscriminant + Sized + crate::relational::GlobalDefinitionEnum
where
    Self::Discriminant: 'static + std::fmt::Debug,
{
    type TreeNames: NetabaseDefinitionTreeNames<Self>;
    type DefKeys: NetabaseDefinitionKeys<Self>;

    /// Subscription registry mapping topics to models
    const SUBSCRIPTION_REGISTRY: DefinitionSubscriptionRegistry<'static, Self>;

    /// Definition-level permissions specifying per-model access control
    const PERMISSIONS: crate::traits::permissions::DefinitionPermissions<'static, Self>;
}

/// Trait for an enum that encapsulates the tree names for all models in a definition
/// This structure should be nested: Definition -> Model -> TreeNames
pub trait NetabaseDefinitionTreeNames<D: NetabaseDefinition>: Sized + Clone
where
    <D as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    // Methods to access specific tree names based on the definition's discriminant
    // fn get_tree_names(discriminant: D::Discriminant) -> ...
}

/// Trait for an enum that encapsulates the keys for all models in a definition
/// This structure should be nested: Definition -> Model -> KeyType -> ConcreteKey
pub trait NetabaseDefinitionKeys<D: NetabaseDefinition>: Sized + Clone + std::fmt::Debug
where
    <D as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    // Methods to access specific keys, potentially converting from/to generic representations
}
