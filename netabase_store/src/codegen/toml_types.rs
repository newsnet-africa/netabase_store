//! TOML schema type definitions
//!
//! This module defines the Serde types used for parsing TOML schema definitions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root definition schema
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DefinitionSchema {
    pub definition: DefinitionConfig,
    pub model: ModelConfig,
    pub keys: KeysConfig,
    pub permissions: Option<PermissionsConfig>,
    pub subscriptions: Option<Vec<SubscriptionConfig>>,
    pub metadata: Option<MetadataConfig>,
}

/// Definition configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DefinitionConfig {
    pub name: String,
    pub version: String,
}

/// Model configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelConfig {
    pub fields: Vec<FieldConfig>,
}

/// Field configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FieldConfig {
    pub name: String,
    pub r#type: String,
    pub optional: Option<bool>,
    pub default: Option<String>,
}

/// Keys configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeysConfig {
    pub primary: PrimaryKeyConfig,
    pub secondary: Option<Vec<SecondaryKeyConfig>>,
    pub relational: Option<Vec<RelationalKeyConfig>>,
}

/// Primary key configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PrimaryKeyConfig {
    pub field: String,
    pub key_type: Option<String>,
    pub derive: Option<Vec<String>>,
}

/// Secondary key configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecondaryKeyConfig {
    pub name: String,
    pub field: String,
    pub unique: bool,
    pub key_type: Option<String>,
    pub derive: Option<Vec<String>>,
}

/// Relational key configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RelationalKeyConfig {
    pub name: String,
    pub target_definition: String,
    pub target_model: String,
    pub target_key_type: String,
    pub on_delete: Option<String>,
}

/// Permissions configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PermissionsConfig {
    pub can_reference_from: Option<Vec<String>>,
    pub can_reference_to: Option<Vec<String>>,
    pub roles: Option<Vec<RoleConfig>>,
}

/// Role configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RoleConfig {
    pub name: String,
    pub level: Option<String>,
    pub read: Option<Vec<String>>,
    pub write: Option<Vec<String>>,
    pub definitions: Option<Vec<String>>,
}

/// Subscription configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubscriptionConfig {
    pub name: String,
    pub description: Option<String>,
}

/// Metadata configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetadataConfig {
    pub generated_at: Option<String>,
    pub schema_hash: Option<String>,
}

/// Root manager schema
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ManagerSchema {
    pub manager: ManagerConfig,
    pub definitions: Vec<DefinitionReference>,
    pub permissions: Option<ManagerPermissionsConfig>,
}

/// Manager configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ManagerConfig {
    pub name: String,
    pub version: String,
    pub root_path: String,
}

/// Definition reference
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DefinitionReference {
    pub name: String,
    pub schema_file: String,
}

/// Manager permissions configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ManagerPermissionsConfig {
    pub roles: Option<Vec<RoleConfig>>,
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
}

/// Validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub message: String,
    pub field: Option<String>,
    pub error_type: ValidationErrorType,
}

/// Validation error types
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationErrorType {
    InvalidFieldType,
    DuplicateField,
    MissingPrimaryKey,
    InvalidRelationalTarget,
    CircularReference,
    PermissionConflict,
    InvalidTreeNaming,
}

/// Generated code structure
#[derive(Debug, Clone)]
pub struct GeneratedCode {
    pub definition_name: String,
    pub models: Vec<GeneratedModel>,
    pub keys: GeneratedKeys,
    pub traits: String,
    pub tree_manager: String,
}

/// Generated model
#[derive(Debug, Clone)]
pub struct GeneratedModel {
    pub name: String,
    pub fields: Vec<GeneratedField>,
    pub derives: Vec<String>,
}

/// Generated field
#[derive(Debug, Clone)]
pub struct GeneratedField {
    pub name: String,
    pub field_type: String,
    pub attributes: Vec<String>,
}

/// Generated keys structure
#[derive(Debug, Clone)]
pub struct GeneratedKeys {
    pub primary: GeneratedPrimaryKey,
    pub secondary: Vec<GeneratedSecondaryKey>,
    pub relational: Vec<GeneratedRelationalKey>,
}

/// Generated primary key
#[derive(Debug, Clone)]
pub struct GeneratedPrimaryKey {
    pub key_type: String,
    pub field: String,
    pub derives: Vec<String>,
}

/// Generated secondary key
#[derive(Debug, Clone)]
pub struct GeneratedSecondaryKey {
    pub name: String,
    pub key_type: String,
    pub field: String,
    pub unique: bool,
    pub derives: Vec<String>,
}

/// Generated relational key
#[derive(Debug, Clone)]
pub struct GeneratedRelationalKey {
    pub name: String,
    pub target_definition: String,
    pub target_model: String,
    pub target_key_type: String,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, message: String, field: Option<String>, error_type: ValidationErrorType) {
        self.is_valid = false;
        self.errors.push(ValidationError {
            message,
            field,
            error_type,
        });
    }

    pub fn add_warning(&mut self, message: String) {
        self.warnings.push(message);
    }
}