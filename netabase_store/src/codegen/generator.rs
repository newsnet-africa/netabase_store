//! Code generator for TOML schemas
//!
//! This module converts parsed and validated TOML schemas into Rust code.

use crate::codegen::toml_types::*;
use crate::error::NetabaseResult;
use std::collections::HashMap;

/// Generate complete Rust code from a definition schema
pub fn generate_definition_code(schema: &DefinitionSchema) -> NetabaseResult<GeneratedCode> {
    let definition_name = &schema.definition.name;
    
    // Generate model
    let model = generate_model(schema)?;
    
    // Generate keys
    let keys = generate_keys(schema)?;
    
    // Generate trait implementations
    let traits = generate_trait_implementations(schema)?;
    
    // Generate tree manager implementation
    let tree_manager = generate_tree_manager(schema)?;
    
    Ok(GeneratedCode {
        definition_name: definition_name.clone(),
        models: vec![model],
        keys,
        traits,
        tree_manager,
    })
}

/// Generate model struct
fn generate_model(schema: &DefinitionSchema) -> NetabaseResult<GeneratedModel> {
    let name = &schema.definition.name;
    
    let fields: Vec<GeneratedField> = schema.model.fields.iter().map(|field| {
        let mut attributes = Vec::new();
        
        // Add primary key attribute if this is the primary key field
        if field.name == schema.keys.primary.field {
            attributes.push("#[primary_key]".to_string());
        }
        
        // Check if this field is a secondary key
        if let Some(secondary_keys) = &schema.keys.secondary {
            if secondary_keys.iter().any(|k| k.field == field.name) {
                attributes.push("#[secondary_key]".to_string());
            }
        }
        
        // Check if this field is a relational key
        if let Some(relational_keys) = &schema.keys.relational {
            if relational_keys.iter().any(|k| k.name.to_lowercase().contains(&field.name.to_lowercase())) {
                attributes.push("#[relation]".to_string());
            }
        }
        
        GeneratedField {
            name: field.name.clone(),
            field_type: field.r#type.clone(),
            attributes,
        }
    }).collect();
    
    let derives = vec![
        "Debug".to_string(),
        "Clone".to_string(),
        "PartialEq".to_string(),
        "serde::Serialize".to_string(),
        "serde::Deserialize".to_string(),
    ];
    
    Ok(GeneratedModel {
        name: name.clone(),
        fields,
        derives,
    })
}

/// Generate keys structures
fn generate_keys(schema: &DefinitionSchema) -> NetabaseResult<GeneratedKeys> {
    let definition_name = &schema.definition.name;
    
    // Generate primary key
    let primary = GeneratedPrimaryKey {
        key_type: schema.keys.primary.key_type.clone()
            .unwrap_or_else(|| format!("{}Id", definition_name)),
        field: schema.keys.primary.field.clone(),
        derives: schema.keys.primary.derive.clone().unwrap_or_else(|| {
            vec![
                "Debug".to_string(),
                "Clone".to_string(),
                "Copy".to_string(),
                "PartialEq".to_string(),
                "Eq".to_string(),
                "PartialOrd".to_string(),
                "Ord".to_string(),
                "Hash".to_string(),
            ]
        }),
    };
    
    // Generate secondary keys
    let mut secondary = Vec::new();
    if let Some(secondary_keys) = &schema.keys.secondary {
        for key in secondary_keys {
            secondary.push(GeneratedSecondaryKey {
                name: key.name.clone(),
                key_type: key.key_type.clone()
                    .unwrap_or_else(|| format!("{}{}", definition_name, key.name)),
                field: key.field.clone(),
                unique: key.unique,
                derives: key.derive.clone().unwrap_or_else(|| {
                    vec![
                        "Debug".to_string(),
                        "Clone".to_string(),
                        "PartialEq".to_string(),
                        "Eq".to_string(),
                        "Hash".to_string(),
                    ]
                }),
            });
        }
    }
    
    // Generate relational keys
    let mut relational = Vec::new();
    if let Some(relational_keys) = &schema.keys.relational {
        for key in relational_keys {
            relational.push(GeneratedRelationalKey {
                name: key.name.clone(),
                target_definition: key.target_definition.clone(),
                target_model: key.target_model.clone(),
                target_key_type: key.target_key_type.clone(),
            });
        }
    }
    
    Ok(GeneratedKeys {
        primary,
        secondary,
        relational,
    })
}

/// Generate trait implementations
fn generate_trait_implementations(schema: &DefinitionSchema) -> NetabaseResult<String> {
    let definition_name = &schema.definition.name;
    let model_name = definition_name; // Assuming single model per definition for now
    
    let primary_key_type = schema.keys.primary.key_type.clone()
        .unwrap_or_else(|| format!("{}Id", definition_name));
    
    // Build secondary keys enum if any
    let secondary_enum = if let Some(secondary_keys) = &schema.keys.secondary {
        if !secondary_keys.is_empty() {
            let variants: Vec<String> = secondary_keys.iter().map(|key| {
                let key_type = key.key_type.clone()
                    .unwrap_or_else(|| format!("{}{}", definition_name, key.name));
                format!("    {}({}),", key.name, key_type)
            }).collect();
            
            format!(
                r#"#[derive(Debug, Clone, PartialEq, Eq, Hash, strum::EnumDiscriminants)]
#[strum_discriminants(name({}SecondaryKeysDiscriminants))]
pub enum {}SecondaryKeys {{
{}
}}"#,
                definition_name,
                definition_name,
                variants.join("\n")
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };
    
    // Build relational keys enum if any
    let relational_enum = if let Some(relational_keys) = &schema.keys.relational {
        if !relational_keys.is_empty() {
            let variants: Vec<String> = relational_keys.iter().map(|key| {
                format!("    {}({}),", key.name, key.target_key_type)
            }).collect();
            
            format!(
                r#"#[derive(Debug, Clone, PartialEq, Eq, Hash, strum::EnumDiscriminants)]
#[strum_discriminants(name({}RelationalKeysDiscriminants))]
pub enum {}RelationalKeys {{
{}
}}"#,
                definition_name,
                definition_name,
                variants.join("\n")
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };
    
    // Generate basic NetabaseModelTrait implementation
    let trait_impl = format!(
        r#"impl crate::traits::model::NetabaseModelTrait for {} {{
    type PrimaryKey = {};
    type SecondaryKeys = {}SecondaryKeys;
    type RelationalKeys = {}RelationalKeys;
    type SubscriptionKeys = {}SubscriptionKeys;

    fn primary_key(&self) -> Self::PrimaryKey {{
        {}(self.{})
    }}

    fn get_model_name() -> &'static str {{
        "{}"
    }}
}}"#,
        model_name,
        primary_key_type,
        definition_name,
        definition_name,
        definition_name,
        primary_key_type,
        schema.keys.primary.field,
        model_name
    );
    
    Ok(format!(
        "{}\n\n{}\n\n{}",
        secondary_enum,
        relational_enum,
        trait_impl
    ))
}

/// Generate tree manager implementation
fn generate_tree_manager(schema: &DefinitionSchema) -> NetabaseResult<String> {
    let definition_name = &schema.definition.name;
    let model_name = definition_name;
    
    // Generate tree names
    let main_tree = format!("\"{}::{}::Main\"", definition_name, model_name);
    let hash_tree = format!("\"{}::{}::Hash\"", definition_name, model_name);
    
    let secondary_trees = if let Some(secondary_keys) = &schema.keys.secondary {
        secondary_keys.iter().map(|key| {
            format!("\"{}::{}::Secondary::{}\"", definition_name, model_name, key.name)
        }).collect::<Vec<_>>().join(", ")
    } else {
        String::new()
    };
    
    let relational_trees = if let Some(relational_keys) = &schema.keys.relational {
        relational_keys.iter().map(|key| {
            format!("\"{}::{}::Relational::{}\"", definition_name, model_name, key.name)
        }).collect::<Vec<_>>().join(", ")
    } else {
        String::new()
    };
    
    let subscription_trees = if let Some(subscriptions) = &schema.subscriptions {
        subscriptions.iter().map(|sub| {
            format!("\"{}::{}::Subscription::{}\"", definition_name, model_name, sub.name)
        }).collect::<Vec<_>>().join(", ")
    } else {
        String::new()
    };
    
    Ok(format!(
        r#"impl {} {{
    pub const MAIN_TREE_NAME: &'static str = {};
    pub const HASH_TREE_NAME: &'static str = {};
    pub const SECONDARY_TREE_NAMES: &'static [&'static str] = &[{}];
    pub const RELATIONAL_TREE_NAMES: &'static [&'static str] = &[{}];
    pub const SUBSCRIPTION_TREE_NAMES: &'static [&'static str] = &[{}];
}}"#,
        model_name,
        main_tree,
        hash_tree,
        secondary_trees,
        relational_trees,
        subscription_trees
    ))
}

/// Generate a complete manager from a manager schema
pub fn generate_manager_code(
    manager_schema: &ManagerSchema,
    definition_schemas: &[(String, DefinitionSchema)]
) -> NetabaseResult<String> {
    let manager_name = &manager_schema.manager.name;
    
    // Generate all definitions
    let mut all_code = Vec::new();
    
    for (def_name, def_schema) in definition_schemas {
        let def_code = generate_definition_code(def_schema)?;
        all_code.push(format!(
            "// Definition: {}\n{}",
            def_name,
            format_generated_code(&def_code)?
        ));
    }
    
    // Generate manager struct
    let definition_variants: Vec<String> = definition_schemas.iter()
        .map(|(name, _)| format!("    {},", name))
        .collect();
    
    let manager_struct = format!(
        r#"#[derive(Debug, Clone, PartialEq, Eq, Hash, strum::EnumDiscriminants)]
#[strum_discriminants(name({}DefinitionsDiscriminants))]
pub enum {}Definitions {{
{}
}}

pub struct {}Manager<B> {{
    root_path: std::path::PathBuf,
    _phantom: std::marker::PhantomData<B>,
}}

impl<B> {}Manager<B> {{
    pub fn new<P: AsRef<std::path::Path>>(root_path: P) -> Self {{
        Self {{
            root_path: root_path.as_ref().to_path_buf(),
            _phantom: std::marker::PhantomData,
        }}
    }}

    pub fn root_path(&self) -> &std::path::Path {{
        &self.root_path
    }}
}}"#,
        manager_name,
        manager_name,
        definition_variants.join("\n"),
        manager_name,
        manager_name
    );
    
    // Combine everything
    Ok(format!(
        "{}\n\n{}",
        all_code.join("\n\n"),
        manager_struct
    ))
}

/// Format generated code structure into Rust code
fn format_generated_code(code: &GeneratedCode) -> NetabaseResult<String> {
    let mut result = String::new();
    
    // Generate model structs
    for model in &code.models {
        result.push_str(&format!(
            "#[derive({})]\npub struct {} {{\n",
            model.derives.join(", "),
            model.name
        ));
        
        for field in &model.fields {
            for attr in &field.attributes {
                result.push_str(&format!("    {}\n", attr));
            }
            result.push_str(&format!("    pub {}: {},\n", field.name, field.field_type));
        }
        
        result.push_str("}\n\n");
    }
    
    // Generate primary key type
    result.push_str(&format!(
        "#[derive({})]\npub struct {}(pub {});\n\n",
        code.keys.primary.derives.join(", "),
        code.keys.primary.key_type,
        "u64" // TODO: infer from field type
    ));
    
    // Generate secondary key types
    for key in &code.keys.secondary {
        result.push_str(&format!(
            "#[derive({})]\npub struct {}(pub {});\n",
            key.derives.join(", "),
            key.key_type,
            "String" // TODO: infer from field type
        ));
    }
    
    if !code.keys.secondary.is_empty() {
        result.push('\n');
    }
    
    // Add trait implementations
    result.push_str(&code.traits);
    result.push_str("\n\n");
    
    // Add tree manager implementation
    result.push_str(&code.tree_manager);
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_simple_model() {
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

        let result = generate_definition_code(&schema).unwrap();
        assert_eq!(result.definition_name, "User");
        assert_eq!(result.models.len(), 1);
        assert_eq!(result.models[0].name, "User");
        assert_eq!(result.keys.primary.key_type, "UserId");
        assert_eq!(result.keys.secondary.len(), 1);
        assert_eq!(result.keys.secondary[0].name, "Email");
    }

    #[test]
    fn test_generate_formatted_code() {
        let generated = GeneratedCode {
            definition_name: "User".to_string(),
            models: vec![GeneratedModel {
                name: "User".to_string(),
                fields: vec![
                    GeneratedField {
                        name: "id".to_string(),
                        field_type: "u64".to_string(),
                        attributes: vec!["#[primary_key]".to_string()],
                    },
                    GeneratedField {
                        name: "email".to_string(),
                        field_type: "String".to_string(),
                        attributes: vec!["#[secondary_key]".to_string()],
                    },
                ],
                derives: vec!["Debug".to_string(), "Clone".to_string()],
            }],
            keys: GeneratedKeys {
                primary: GeneratedPrimaryKey {
                    key_type: "UserId".to_string(),
                    field: "id".to_string(),
                    derives: vec!["Debug".to_string(), "Clone".to_string()],
                },
                secondary: vec![GeneratedSecondaryKey {
                    name: "Email".to_string(),
                    key_type: "UserEmail".to_string(),
                    field: "email".to_string(),
                    unique: true,
                    derives: vec!["Debug".to_string()],
                }],
                relational: vec![],
            },
            traits: "impl NetabaseModelTrait for User {}".to_string(),
            tree_manager: "impl User { const MAIN_TREE: &str = \"User::User::Main\"; }".to_string(),
        };

        let formatted = format_generated_code(&generated).unwrap();
        assert!(formatted.contains("#[derive(Debug, Clone)]"));
        assert!(formatted.contains("pub struct User {"));
        assert!(formatted.contains("#[primary_key]"));
        assert!(formatted.contains("pub id: u64,"));
    }
}