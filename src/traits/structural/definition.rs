use crate::traits::behavioural::conversion::IntoDiscriminant;
use crate::traits::behavioural::metadata::MetadataProvider;
use crate::traits::structural::metadata::DefinitionMetadata;
use std::fmt::Debug;
use std::hash::Hash;

pub trait NetabaseDefinition:
    Sized + IntoDiscriminant + MetadataProvider<Metadata = DefinitionMetadata>
{
    type DefinitionName: Debug + Copy + Eq + Hash + 'static;
    fn name() -> Self::DefinitionName;
    fn version() -> u32;
    type TreeNames: NetabaseDefinitionTreeNames<Self>;
    type Keys: NetabaseDefinitionKeys<Self>;
    fn schema() -> DefinitionSchema<Self>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionSchema<D: NetabaseDefinition> {
    pub name: D::DefinitionName,
    pub version: u32,
}

pub trait NetabaseDefinitionTreeNames<D: NetabaseDefinition>: Sized + Clone {}

pub trait NetabaseDefinitionKeys<D: NetabaseDefinition>: Sized + Clone + Debug {}
