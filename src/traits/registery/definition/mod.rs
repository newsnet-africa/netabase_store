pub mod redb_definition;
use strum::IntoDiscriminant;

pub trait NetabaseDefinition: IntoDiscriminant
where
    Self::Discriminant: 'static,
{
    type TreeNames: NetabaseDefinitionTreeNames;
    type ModelTableDefinition: Clone + Send + Sync + 'static;
}
pub trait NetabaseDefinitionTreeNames {}
