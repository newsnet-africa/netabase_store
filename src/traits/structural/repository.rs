use crate::traits::structural::definition::NetabaseDefinition;
use crate::traits::behavioural::conversion::IntoDiscriminant;
use crate::traits::behavioural::metadata::MetadataProvider;
use crate::traits::structural::metadata::RepositoryMetadata;
use crate::traits::structural::metadata::RepositoryKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RepositoryLabel(pub &'static str);

pub trait NetabaseRepository: Sized + 'static + IntoDiscriminant + MetadataProvider<Metadata = RepositoryMetadata> {
    type RepositoryName: std::fmt::Debug + Copy + Eq + std::hash::Hash + 'static;
    fn name() -> Self::RepositoryName;
    fn version() -> u32;
    type Definitions;
    type TreeNames: NetabaseRepositoryTreeNames<Self>;
    fn definition_count() -> usize;
}

pub trait NetabaseRepositoryTreeNames<R: NetabaseRepository>: Sized + Clone {
    type DefinitionTreeNames: ?Sized;
    fn for_definition(&self, discriminant: R::Discriminant) -> &Self::DefinitionTreeNames;
}

pub trait InRepository<R: NetabaseRepository>: NetabaseDefinition {
    fn repository_discriminant() -> R::Discriminant;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, strum::EnumDiscriminants)]
#[strum_discriminants(derive(Hash))]
pub enum Standalone {
    #[default]
    Placeholder,
}

impl IntoDiscriminant for Standalone {
    type Discriminant = StandaloneDiscriminants;
    fn discriminant(&self) -> Self::Discriminant {
        match self {
            Standalone::Placeholder => StandaloneDiscriminants::Placeholder,
        }
    }
}

impl MetadataProvider for Standalone {
    type Metadata = RepositoryMetadata;
    fn get_metadata(&self) -> Self::Metadata {
        RepositoryMetadata {
            key: RepositoryKey([0; 16]),
            name: "Standalone".to_string(),
            version: 0,
            definitions: vec![],
        }
    }
}

impl NetabaseRepository for Standalone {
    type RepositoryName = RepositoryLabel;
    fn name() -> Self::RepositoryName {
        RepositoryLabel("Standalone")
    }

    fn version() -> u32 {
        0
    }

    type Definitions = ();
    type TreeNames = ();

    fn definition_count() -> usize {
        0
    }
}

impl NetabaseRepositoryTreeNames<Standalone> for () {
    type DefinitionTreeNames = ();
    fn for_definition(&self, _discriminant: StandaloneDiscriminants) -> &Self::DefinitionTreeNames {
        panic!("Standalone repository has no definitions")
    }
}
