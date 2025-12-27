use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DefinitionSchema {
    pub name: String,
    pub models: Vec<ModelSchema>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub structs: Vec<StructSchema>,
    pub subscriptions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelSchema {
    pub name: String,
    pub fields: Vec<FieldSchema>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subscriptions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructSchema {
    pub name: String,
    pub fields: Vec<StructFieldSchema>,
    #[serde(default)]
    pub is_tuple: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructFieldSchema {
    pub name: String,
    pub type_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldSchema {
    pub name: String,
    pub type_name: String,
    #[serde(flatten)]
    pub key_type: KeyTypeSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "details")]
pub enum KeyTypeSchema {
    Primary,
    Secondary,
    Relational { definition: String, model: String },
    Blob,
    Regular,
}
