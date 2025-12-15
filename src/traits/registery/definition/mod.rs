pub mod redb_definition;
use strum::IntoDiscriminant;

pub trait NetabaseDefinition: IntoDiscriminant + Sized
where
    Self::Discriminant: 'static + std::fmt::Debug,
{
    type TreeNames: NetabaseDefinitionTreeNames<Self>;
    type DefKeys: NetabaseDefinitionKeys<Self>;
    type ModelTableDefinition<'db>: Clone + Send + Sync;
}

/// Trait for an enum that encapsulates the tree names for all models in a definition
/// This structure should be nested: Definition -> Model -> TreeNames
pub trait NetabaseDefinitionTreeNames<D: NetabaseDefinition>: Sized + Clone 
where <D as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug 
{
    // Methods to access specific tree names based on the definition's discriminant
    // fn get_tree_names(discriminant: D::Discriminant) -> ...
}

/// Trait for an enum that encapsulates the keys for all models in a definition
/// This structure should be nested: Definition -> Model -> KeyType -> ConcreteKey
pub trait NetabaseDefinitionKeys<D: NetabaseDefinition>: Sized + Clone + std::fmt::Debug 
where <D as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug 
{
    // Methods to access specific keys, potentially converting from/to generic representations
}

/// Wrapper enum for any model that belongs to a NetabaseDefinition
/// This provides strong typing while allowing conversion from concrete model types
#[derive(Debug, Clone)]
pub enum NetabaseDefinitionModel<D: NetabaseDefinition> 
where 
    <D as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    /// Holds the specific model types for this definition
    Model(D::Discriminant),
}
