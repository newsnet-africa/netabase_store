pub mod redb_definition;
pub mod subscription;

use strum::IntoDiscriminant;
use subscription::{DefinitionSubscriptionRegistry, NetabaseDefinitionSubscriptionKeys};

use crate::traits::registery::models::{
    keys::NetabaseModelKeys,
    model::NetabaseModel,
    treenames::{DiscriminantTableName, ModelTreeNames},
};

pub trait NetabaseDefinition: IntoDiscriminant + Sized
where
    Self::Discriminant: 'static + std::fmt::Debug,
{
    type TreeNames: NetabaseDefinitionTreeNames<Self> + 'static;
    type DefKeys: NetabaseDefinitionKeys<Self>;

    /// Definition-level subscription keys enum
    /// This enum holds all subscription topics for the definition
    /// and serves as the unified key type for subscription tables
    type SubscriptionKeys: NetabaseDefinitionSubscriptionKeys<Discriminant = Self::SubscriptionKeysDiscriminant>;

    /// Discriminant type for subscription keys
    type SubscriptionKeysDiscriminant: 'static + std::fmt::Debug;

    /// Subscription registry mapping topics to models
    const SUBSCRIPTION_REGISTRY: DefinitionSubscriptionRegistry<'static, Self>;
}

/// Trait for an enum that encapsulates the tree names for all models in a definition
/// This structure should be nested: Definition -> Model -> TreeNames
pub trait NetabaseDefinitionTreeNames<D: NetabaseDefinition>: Sized + Clone
where
    <D as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    Self: TryInto<DiscriminantTableName<D>>,
{
    // Methods to access specific tree names based on the definition's discriminant
    fn get_tree_names(discriminant: D::Discriminant) -> Vec<Self>;

    fn get_model_tree<M: NetabaseModel<D>>(&self) -> Option<M>
    where
        for<'a> Self: From<ModelTreeNames<'a, Self, M>>,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'a>:
            IntoDiscriminant,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'a>:
            IntoDiscriminant,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'a>:
            IntoDiscriminant,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
         <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'static>: 'static
    ;
}

/// Trait for an enum that encapsulates the keys for all models in a definition
/// This structure should be nested: Definition -> Model -> KeyType -> ConcreteKey
pub trait NetabaseDefinitionKeys<D: NetabaseDefinition>: Sized + Clone + std::fmt::Debug
where
    <D as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    // Methods to access specific keys, potentially converting from/to generic representations
}
