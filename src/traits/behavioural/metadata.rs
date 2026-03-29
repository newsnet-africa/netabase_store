use crate::traits::structural::metadata::{
    Inconsistency, RepositoryMetadata, DefinitionMetadata, ModelMetadata,
};

pub trait MetadataProvider {
    type Metadata;
    fn get_metadata(&self) -> Self::Metadata;
}

pub trait Scaffolder {
    fn check_inconsistencies() -> crate::results::NetabaseResult<Vec<Inconsistency>>;
    fn scaffold() -> crate::results::NetabaseResult<()>;
}

pub trait RepositoryMetadataTable {
    fn store_repository_metadata(metadata: &RepositoryMetadata) -> crate::results::NetabaseResult<()>;
    fn load_repository_metadata() -> crate::results::NetabaseResult<Option<RepositoryMetadata>>;
}

pub trait DefinitionMetadataTable {
    fn store_definition_metadata(metadata: &DefinitionMetadata) -> crate::results::NetabaseResult<()>;
    fn load_definition_metadata(name: &str) -> crate::results::NetabaseResult<Option<DefinitionMetadata>>;
    fn load_all_definitions() -> crate::results::NetabaseResult<Vec<DefinitionMetadata>>;
}

pub trait ModelMetadataTable {
    fn store_model_metadata(metadata: &ModelMetadata) -> crate::results::NetabaseResult<()>;
    fn load_model_metadata(family_name: &str, version: u32) -> crate::results::NetabaseResult<Option<ModelMetadata>>;
    fn load_latest_model_metadata(family_name: &str) -> crate::results::NetabaseResult<Option<ModelMetadata>>;
    fn load_all_models() -> crate::results::NetabaseResult<Vec<ModelMetadata>>;
}
