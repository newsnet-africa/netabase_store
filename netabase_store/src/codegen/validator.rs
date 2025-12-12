//! TOML schema validator
//!
//! This module provides validation functionality for TOML schema definitions.

use crate::codegen::toml_types::*;
use crate::error::NetabaseResult;
use std::collections::{HashMap, HashSet};

/// Validate a definition schema
pub fn validate_definition_schema(schema: &DefinitionSchema) -> ValidationResult {
    let mut result = ValidationResult::new();
    
    // Validate basic structure
    validate_definition_config(&schema.definition, &mut result);
    validate_model_config(&schema.model, &mut result);
    validate_keys_config(&schema.keys, &schema.model, &mut result);
    
    if let Some(perms) = &schema.permissions {
        validate_permissions_config(perms, &mut result);
    }
    
    if let Some(subs) = &schema.subscriptions {
        validate_subscriptions_config(subs, &mut result);
    }
    
    result
}

/// Validate definition configuration
fn validate_definition_config(config: &DefinitionConfig, result: &mut ValidationResult) {
    if config.name.is_empty() {
        result.add_error(
            "Definition name cannot be empty".to_string(),
            Some("definition.name".to_string()),
            ValidationErrorType::InvalidFieldType,
        );
    }
    
    if !config.name.chars().next().unwrap_or('a').is_uppercase() {
        result.add_warning(
            "Definition name should start with uppercase letter".to_string()
        );
    }
    
    if config.version.is_empty() {
        result.add_error(
            "Definition version cannot be empty".to_string(),
            Some("definition.version".to_string()),
            ValidationErrorType::InvalidFieldType,
        );
    }
}

/// Validate model configuration
fn validate_model_config(config: &ModelConfig, result: &mut ValidationResult) {
    if config.fields.is_empty() {
        result.add_error(
            "Model must have at least one field".to_string(),
            Some("model.fields".to_string()),
            ValidationErrorType::InvalidFieldType,
        );
        return;
    }
    
    // Check for duplicate field names
    let mut field_names = HashSet::new();
    for field in &config.fields {
        if !field_names.insert(&field.name) {
            result.add_error(
                format!("Duplicate field name: {}", field.name),
                Some(format!("model.fields.{}", field.name)),
                ValidationErrorType::DuplicateField,
            );
        }
        
        // Validate field types
        validate_field_type(&field.r#type, &field.name, result);
    }
}

/// Validate field type
fn validate_field_type(field_type: &str, field_name: &str, result: &mut ValidationResult) {
    let valid_primitives = [
        "u8", "u16", "u32", "u64", "u128",
        "i8", "i16", "i32", "i64", "i128",
        "f32", "f64", "bool", "String", "&str"
    ];
    
    // Simple validation - check for basic types
    if !valid_primitives.contains(&field_type) && 
       !field_type.starts_with("Vec<") && 
       !field_type.starts_with("Option<") && 
       !field_type.starts_with("HashMap<") &&
       !field_type.contains("::") {
        result.add_warning(
            format!("Field '{}' uses custom type '{}' - ensure it's in scope", field_name, field_type)
        );
    }
}

/// Validate keys configuration
fn validate_keys_config(config: &KeysConfig, model: &ModelConfig, result: &mut ValidationResult) {
    // Validate primary key
    let field_names: HashSet<_> = model.fields.iter().map(|f| &f.name).collect();
    
    if !field_names.contains(&config.primary.field) {
        result.add_error(
            format!("Primary key field '{}' not found in model fields", config.primary.field),
            Some("keys.primary.field".to_string()),
            ValidationErrorType::InvalidFieldType,
        );
    }
    
    // Validate secondary keys
    if let Some(secondary_keys) = &config.secondary {
        let mut secondary_names = HashSet::new();
        for key in secondary_keys {
            if !secondary_names.insert(&key.name) {
                result.add_error(
                    format!("Duplicate secondary key name: {}", key.name),
                    Some(format!("keys.secondary.{}", key.name)),
                    ValidationErrorType::DuplicateField,
                );
            }
            
            if !field_names.contains(&key.field) {
                result.add_error(
                    format!("Secondary key field '{}' not found in model fields", key.field),
                    Some(format!("keys.secondary.{}.field", key.name)),
                    ValidationErrorType::InvalidFieldType,
                );
            }
        }
    }
    
    // Validate relational keys
    if let Some(relational_keys) = &config.relational {
        let mut relational_names = HashSet::new();
        for key in relational_keys {
            if !relational_names.insert(&key.name) {
                result.add_error(
                    format!("Duplicate relational key name: {}", key.name),
                    Some(format!("keys.relational.{}", key.name)),
                    ValidationErrorType::DuplicateField,
                );
            }
            
            // Basic validation for target references
            if key.target_definition.is_empty() {
                result.add_error(
                    format!("Relational key '{}' missing target_definition", key.name),
                    Some(format!("keys.relational.{}.target_definition", key.name)),
                    ValidationErrorType::InvalidRelationalTarget,
                );
            }
        }
    }
}

/// Validate permissions configuration
fn validate_permissions_config(config: &PermissionsConfig, result: &mut ValidationResult) {
    if let Some(roles) = &config.roles {
        let mut role_names = HashSet::new();
        for role in roles {
            if !role_names.insert(&role.name) {
                result.add_error(
                    format!("Duplicate role name: {}", role.name),
                    Some(format!("permissions.roles.{}", role.name)),
                    ValidationErrorType::DuplicateField,
                );
            }
            
            // Validate role levels
            if let Some(level) = &role.level {
                let valid_levels = ["None", "Read", "Write", "ReadWrite", "Admin"];
                if !valid_levels.contains(&level.as_str()) {
                    result.add_error(
                        format!("Invalid permission level: {}", level),
                        Some(format!("permissions.roles.{}.level", role.name)),
                        ValidationErrorType::PermissionConflict,
                    );
                }
            }
        }
    }
}

/// Validate subscriptions configuration
fn validate_subscriptions_config(config: &[SubscriptionConfig], result: &mut ValidationResult) {
    let mut subscription_names = HashSet::new();
    for sub in config {
        if !subscription_names.insert(&sub.name) {
            result.add_error(
                format!("Duplicate subscription name: {}", sub.name),
                Some(format!("subscriptions.{}", sub.name)),
                ValidationErrorType::DuplicateField,
            );
        }
    }
}

/// Validate manager schema
pub fn validate_manager_schema(schema: &ManagerSchema) -> ValidationResult {
    let mut result = ValidationResult::new();
    
    if schema.manager.name.is_empty() {
        result.add_error(
            "Manager name cannot be empty".to_string(),
            Some("manager.name".to_string()),
            ValidationErrorType::InvalidFieldType,
        );
    }
    
    if schema.definitions.is_empty() {
        result.add_error(
            "Manager must have at least one definition".to_string(),
            Some("definitions".to_string()),
            ValidationErrorType::InvalidFieldType,
        );
    }
    
    // Check for duplicate definition names
    let mut def_names = HashSet::new();
    for def in &schema.definitions {
        if !def_names.insert(&def.name) {
            result.add_error(
                format!("Duplicate definition name: {}", def.name),
                Some(format!("definitions.{}", def.name)),
                ValidationErrorType::DuplicateField,
            );
        }
    }
    
    result
}

/// Validate cross-definition references
pub fn validate_cross_definition_references(
    schemas: &[(String, DefinitionSchema)]
) -> ValidationResult {
    let mut result = ValidationResult::new();
    let def_names: HashSet<_> = schemas.iter().map(|(name, _)| name).collect();
    
    for (def_name, schema) in schemas {
        if let Some(relational_keys) = &schema.keys.relational {
            for key in relational_keys {
                if !def_names.contains(&key.target_definition) {
                    result.add_error(
                        format!(
                            "Definition '{}' references unknown definition '{}' in relational key '{}'",
                            def_name, key.target_definition, key.name
                        ),
                        Some(format!("keys.relational.{}.target_definition", key.name)),
                        ValidationErrorType::InvalidRelationalTarget,
                    );
                }
            }
        }
        
        if let Some(perms) = &schema.permissions {
            if let Some(can_ref_from) = &perms.can_reference_from {
                for ref_def in can_ref_from {
                    if !def_names.contains(ref_def) {
                        result.add_error(
                            format!(
                                "Definition '{}' allows references from unknown definition '{}'",
                                def_name, ref_def
                            ),
                            Some("permissions.can_reference_from".to_string()),
                            ValidationErrorType::InvalidRelationalTarget,
                        );
                    }
                }
            }
            
            if let Some(can_ref_to) = &perms.can_reference_to {
                for ref_def in can_ref_to {
                    if !def_names.contains(ref_def) {
                        result.add_error(
                            format!(
                                "Definition '{}' can reference unknown definition '{}'",
                                def_name, ref_def
                            ),
                            Some("permissions.can_reference_to".to_string()),
                            ValidationErrorType::InvalidRelationalTarget,
                        );
                    }
                }
            }
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_definition() {
        let schema = DefinitionSchema {
            definition: DefinitionConfig {
                name: "User".to_string(),
                version: "1".to_string(),
            },
            model: ModelConfig {
                fields: vec![
                    FieldConfig {
                        name: "id".to_string(),
                        r#type: "u64".to_string(),
                        optional: None,
                        default: None,
                    },
                    FieldConfig {
                        name: "email".to_string(),
                        r#type: "String".to_string(),
                        optional: None,
                        default: None,
                    },
                ],
            },
            keys: KeysConfig {
                primary: PrimaryKeyConfig {
                    field: "id".to_string(),
                    key_type: None,
                    derive: None,
                },
                secondary: Some(vec![
                    SecondaryKeyConfig {
                        name: "Email".to_string(),
                        field: "email".to_string(),
                        unique: true,
                        key_type: None,
                        derive: None,
                    }
                ]),
                relational: None,
            },
            permissions: None,
            subscriptions: None,
            metadata: None,
        };

        let result = validate_definition_schema(&schema);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_invalid_primary_key() {
        let schema = DefinitionSchema {
            definition: DefinitionConfig {
                name: "User".to_string(),
                version: "1".to_string(),
            },
            model: ModelConfig {
                fields: vec![
                    FieldConfig {
                        name: "email".to_string(),
                        r#type: "String".to_string(),
                        optional: None,
                        default: None,
                    },
                ],
            },
            keys: KeysConfig {
                primary: PrimaryKeyConfig {
                    field: "id".to_string(), // Field doesn't exist
                    key_type: None,
                    derive: None,
                },
                secondary: None,
                relational: None,
            },
            permissions: None,
            subscriptions: None,
            metadata: None,
        };

        let result = validate_definition_schema(&schema);
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].error_type, ValidationErrorType::InvalidFieldType);
    }

    #[test]
    fn test_validate_duplicate_fields() {
        let schema = DefinitionSchema {
            definition: DefinitionConfig {
                name: "User".to_string(),
                version: "1".to_string(),
            },
            model: ModelConfig {
                fields: vec![
                    FieldConfig {
                        name: "id".to_string(),
                        r#type: "u64".to_string(),
                        optional: None,
                        default: None,
                    },
                    FieldConfig {
                        name: "id".to_string(), // Duplicate
                        r#type: "String".to_string(),
                        optional: None,
                        default: None,
                    },
                ],
            },
            keys: KeysConfig {
                primary: PrimaryKeyConfig {
                    field: "id".to_string(),
                    key_type: None,
                    derive: None,
                },
                secondary: None,
                relational: None,
            },
            permissions: None,
            subscriptions: None,
            metadata: None,
        };

        let result = validate_definition_schema(&schema);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.error_type == ValidationErrorType::DuplicateField));
    }
}