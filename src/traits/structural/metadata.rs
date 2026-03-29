use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RepositoryKey(pub [u8; 16]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DefinitionKey(pub [u8; 16]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FamilyKey(pub [u8; 16]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModelKey(pub [u8; 16]);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryMetadata {
    pub key: RepositoryKey,
    pub name: String,
    pub version: u32,
    pub definitions: Vec<DefinitionKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefinitionMetadata {
    pub key: DefinitionKey,
    pub repository_key: RepositoryKey,
    pub name: String,
    pub version: u32,
    pub models: Vec<ModelKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub key: ModelKey,
    pub family_key: FamilyKey,
    pub definition_key: DefinitionKey,
    pub name: String,
    pub version: u32,
    pub tables: Vec<TableMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableMetadata {
    pub name: String,
    pub table_type: TableType,
    pub key_type_id: String,
    pub value_type_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableType {
    Primary,
    Secondary,
    Blob,
    Relational,
    Subscription,
    System,
}

#[derive(Debug)]
pub enum Inconsistency {
    MissingModel(String),
    MissingTable(String),
    VersionMismatch {
        name: String,
        stored: u32,
        code: u32,
    },
    UnknownTable(String),
}
