pub mod redb_definition;
use strum::IntoDiscriminant;

pub trait NetabaseDefinition: IntoDiscriminant
where
    Self::Discriminant: 'static,
{
    type TreeNames: NetabaseDefinitionTreeNames;
    type ModelTableDefinition<'db>: Clone + Send + Sync + 'static;
}

pub trait NetabaseDefinitionTreeNames {}

/// Wrapper enum for any keys that belong to a NetabaseDefinition
/// This provides strong typing while allowing conversion from concrete key types
#[derive(Debug, Clone)]
pub enum NetabaseDefinitionKeys<D: NetabaseDefinition> 
where 
    <D as IntoDiscriminant>::Discriminant: 'static,
{
    /// Holds the specific key types for this definition
    Keys(D::Discriminant),
}

/// Wrapper enum for any model that belongs to a NetabaseDefinition
/// This provides strong typing while allowing conversion from concrete model types
#[derive(Debug, Clone)]
pub enum NetabaseDefinitionModel<D: NetabaseDefinition> 
where 
    <D as IntoDiscriminant>::Discriminant: 'static,
{
    /// Holds the specific model types for this definition
    Model(D::Discriminant),
}
