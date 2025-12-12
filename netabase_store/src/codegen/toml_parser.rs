//! TOML schema parser
//!
//! This module provides functionality for parsing TOML schema definitions.

use crate::codegen::toml_types::*;
use crate::error::{NetabaseError, NetabaseResult};
use std::fs;
use std::path::Path;

/// Parse a definition schema from a TOML file
pub fn parse_definition_schema<P: AsRef<Path>>(path: P) -> NetabaseResult<DefinitionSchema> {
    let content = fs::read_to_string(&path)
        .map_err(|e| NetabaseError::Configuration(
            format!("Failed to read schema file '{}': {}", path.as_ref().display(), e)
        ))?;

    parse_definition_schema_from_str(&content)
}

/// Parse a definition schema from a TOML string
pub fn parse_definition_schema_from_str(content: &str) -> NetabaseResult<DefinitionSchema> {
    toml::from_str(content)
        .map_err(|e| NetabaseError::Configuration(
            format!("Failed to parse TOML schema: {}", e)
        ))
}

/// Parse a manager schema from a TOML file
pub fn parse_manager_schema<P: AsRef<Path>>(path: P) -> NetabaseResult<ManagerSchema> {
    let content = fs::read_to_string(&path)
        .map_err(|e| NetabaseError::Configuration(
            format!("Failed to read manager schema file '{}': {}", path.as_ref().display(), e)
        ))?;

    parse_manager_schema_from_str(&content)
}

/// Parse a manager schema from a TOML string
pub fn parse_manager_schema_from_str(content: &str) -> NetabaseResult<ManagerSchema> {
    toml::from_str(content)
        .map_err(|e| NetabaseError::Configuration(
            format!("Failed to parse TOML manager schema: {}", e)
        ))
}

/// Load all definition schemas referenced by a manager schema
pub fn load_all_definition_schemas(
    manager_schema: &ManagerSchema,
    base_path: Option<&Path>
) -> NetabaseResult<Vec<(String, DefinitionSchema)>> {
    let mut results = Vec::new();
    
    for def_ref in &manager_schema.definitions {
        let schema_path = if let Some(base) = base_path {
            base.join(&def_ref.schema_file)
        } else {
            Path::new(&def_ref.schema_file).to_path_buf()
        };
        
        let schema = parse_definition_schema(&schema_path)?;
        results.push((def_ref.name.clone(), schema));
    }
    
    Ok(results)
}

/// Generate tree names from a definition schema
pub fn generate_tree_names(schema: &DefinitionSchema) -> NetabaseResult<TreeNames> {
    let def_name = &schema.definition.name;
    let model_name = def_name; // For now, assume single model per definition
    
    let main_tree = format!("{}::{}::Main", def_name, model_name);
    let hash_tree = format!("{}::{}::Hash", def_name, model_name);
    
    let mut secondary_trees = Vec::new();
    if let Some(secondary_keys) = &schema.keys.secondary {
        for key in secondary_keys {
            secondary_trees.push(format!(
                "{}::{}::Secondary::{}", 
                def_name, 
                model_name, 
                key.name
            ));
        }
    }
    
    let mut relational_trees = Vec::new();
    if let Some(relational_keys) = &schema.keys.relational {
        for key in relational_keys {
            relational_trees.push(format!(
                "{}::{}::Relational::{}", 
                def_name, 
                model_name, 
                key.name
            ));
        }
    }
    
    let mut subscription_trees = Vec::new();
    if let Some(subscriptions) = &schema.subscriptions {
        for sub in subscriptions {
            subscription_trees.push(format!(
                "{}::{}::Subscription::{}", 
                def_name, 
                model_name, 
                sub.name
            ));
        }
    }
    
    Ok(TreeNames {
        main_tree,
        hash_tree,
        secondary_trees,
        relational_trees,
        subscription_trees,
    })
}

/// Tree names structure
#[derive(Debug, Clone)]
pub struct TreeNames {
    pub main_tree: String,
    pub hash_tree: String,
    pub secondary_trees: Vec<String>,
    pub relational_trees: Vec<String>,
    pub subscription_trees: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_definition_schema() {
        let toml = r#"
        [definition]
        name = "User"
        version = "1"

        [model]
        fields = [
            { name = "id", type = "u64" },
            { name = "email", type = "String" },
        ]

        [keys]
        [keys.primary]
        field = "id"

        [[keys.secondary]]
        name = "Email"
        field = "email"
        unique = true
        "#;

        let schema = parse_definition_schema_from_str(toml).unwrap();
        assert_eq!(schema.definition.name, "User");
        assert_eq!(schema.model.fields.len(), 2);
        assert_eq!(schema.keys.primary.field, "id");
        assert!(schema.keys.secondary.is_some());
        assert_eq!(schema.keys.secondary.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_parse_manager_schema() {
        let toml = r#"
        [manager]
        name = "Restaurant"
        version = "1"
        root_path = "./data"

        [[definitions]]
        name = "User"
        schema_file = "schemas/User.netabase.toml"

        [[definitions]]
        name = "Product"
        schema_file = "schemas/Product.netabase.toml"
        "#;

        let schema = parse_manager_schema_from_str(toml).unwrap();
        assert_eq!(schema.manager.name, "Restaurant");
        assert_eq!(schema.definitions.len(), 2);
    }

    #[test]
    fn test_generate_tree_names() {
        let schema = DefinitionSchema {
            definition: DefinitionConfig {
                name: "User".to_string(),
                version: "1".to_string(),
            },
            model: ModelConfig {
                fields: vec![],
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

        let tree_names = generate_tree_names(&schema).unwrap();
        assert_eq!(tree_names.main_tree, "User::User::Main");
        assert_eq!(tree_names.hash_tree, "User::User::Hash");
        assert_eq!(tree_names.secondary_trees, vec!["User::User::Secondary::Email"]);
    }
}