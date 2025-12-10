// Integration tests for TOML generation and parsing
//
// These tests verify that TOML metadata files are correctly generated
// and can be parsed back, maintaining data integrity.

use netabase_store::databases::manager::toml_types::*;

#[test]
fn test_toml_types_serialization() {
    use chrono::Utc;

    // Test RootToml serialization
    let root = RootToml {
        manager: ManagerSection {
            name: "TestManager".to_string(),
            version: "1".to_string(),
            root_path: "/test/path".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        definitions: DefinitionsSection {
            all: vec!["User".to_string(), "Product".to_string(), "Order".to_string()],
            loaded: vec!["User".to_string()],
            warm_on_access: vec!["Product".to_string()],
        },
        permissions: vec![
            PermissionRoleSection {
                name: "Admin".to_string(),
                level: "ReadWrite".to_string(),
                definitions: vec!["User".to_string(), "Product".to_string(), "Order".to_string()],
            },
            PermissionRoleSection {
                name: "User".to_string(),
                level: "Read".to_string(),
                definitions: vec!["Product".to_string()],
            },
        ],
    };

    // Serialize to TOML
    let toml_str = toml::to_string_pretty(&root).expect("Failed to serialize");

    // Verify structure
    assert!(toml_str.contains("[manager]"));
    assert!(toml_str.contains("name = \"TestManager\""));
    assert!(toml_str.contains("[definitions]"));
    assert!(toml_str.contains("\"User\"") && toml_str.contains("\"Product\"") && toml_str.contains("\"Order\""));
    assert!(toml_str.contains("[[permissions]]"));
    assert!(toml_str.contains("name = \"Admin\""));

    // Parse back
    let parsed: RootToml = toml::from_str(&toml_str).expect("Failed to parse");
    assert_eq!(parsed.manager.name, "TestManager");
    assert_eq!(parsed.definitions.all.len(), 3);
    assert_eq!(parsed.definitions.loaded.len(), 1);
    assert_eq!(parsed.permissions.len(), 2);
    assert_eq!(parsed.permissions[0].name, "Admin");
}

#[test]
fn test_definition_toml_serialization() {
    use chrono::Utc;

    let def_toml = DefinitionToml {
        definition: DefinitionSection {
            name: "User".to_string(),
            discriminant: "User".to_string(),
            version: "1".to_string(),
        },
        trees: TreesSection {
            main: "User".to_string(),
            secondary: vec!["User_Email".to_string(), "User_Username".to_string()],
            relational: vec!["User_rel_Orders".to_string()],
            subscription: vec!["User_sub_Updates".to_string()],
        },
        permissions: DefinitionPermissionsSection {
            can_reference: vec!["Product".to_string(), "Order".to_string()],
            references: vec!["Organization".to_string()],
        },
        metadata: DefinitionMetadataSection {
            created_at: Utc::now(),
            updated_at: Utc::now(),
            schema_hash: "blake3:abcdef1234567890".to_string(),
        },
    };

    // Serialize to TOML
    let toml_str = toml::to_string_pretty(&def_toml).expect("Failed to serialize");

    // Verify structure
    assert!(toml_str.contains("[definition]"));
    assert!(toml_str.contains("name = \"User\""));
    assert!(toml_str.contains("[trees]"));
    assert!(toml_str.contains("main = \"User\""));
    assert!(toml_str.contains("\"User_Email\"") && toml_str.contains("\"User_Username\""));
    assert!(toml_str.contains("[permissions]"));
    assert!(toml_str.contains("\"Product\"") && toml_str.contains("\"Order\""));
    assert!(toml_str.contains("[metadata]"));
    assert!(toml_str.contains("schema_hash = \"blake3:abcdef1234567890\""));

    // Parse back
    let parsed: DefinitionToml = toml::from_str(&toml_str).expect("Failed to parse");
    assert_eq!(parsed.definition.name, "User");
    assert_eq!(parsed.trees.secondary.len(), 2);
    assert_eq!(parsed.permissions.can_reference.len(), 2);
    assert_eq!(parsed.metadata.schema_hash, "blake3:abcdef1234567890");
}

#[test]
fn test_toml_round_trip_preserves_data() {
    use chrono::Utc;

    let original = RootToml {
        manager: ManagerSection {
            name: "ProductionManager".to_string(),
            version: "1".to_string(),
            root_path: "/var/lib/netabase".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        definitions: DefinitionsSection {
            all: vec!["Model1".to_string(), "Model2".to_string()],
            loaded: vec![],
            warm_on_access: vec![],
        },
        permissions: vec![],
    };

    // Serialize
    let toml_str = toml::to_string_pretty(&original).unwrap();

    // Deserialize
    let roundtrip: RootToml = toml::from_str(&toml_str).unwrap();

    // Verify data integrity
    assert_eq!(roundtrip.manager.name, original.manager.name);
    assert_eq!(roundtrip.manager.root_path, original.manager.root_path);
    assert_eq!(roundtrip.definitions.all, original.definitions.all);
}

#[test]
fn test_empty_optional_fields() {
    use chrono::Utc;

    // Test that empty optional fields serialize/deserialize correctly
    let root = RootToml {
        manager: ManagerSection {
            name: "MinimalManager".to_string(),
            version: "1".to_string(),
            root_path: "/tmp".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        definitions: DefinitionsSection {
            all: vec!["OnlyOne".to_string()],
            loaded: vec![],
            warm_on_access: vec![],
        },
        permissions: vec![],
    };

    let toml_str = toml::to_string_pretty(&root).unwrap();

    // Empty permissions array should not be in output (skip_serializing_if)
    assert!(toml_str.contains("[manager]"));
    assert!(toml_str.contains("[definitions]"));

    // Should still parse correctly
    let parsed: RootToml = toml::from_str(&toml_str).unwrap();
    assert_eq!(parsed.permissions.len(), 0);
    assert_eq!(parsed.definitions.loaded.len(), 0);
}

#[test]
fn test_definition_toml_defaults() {
    use chrono::Utc;

    // Test that default values work correctly
    let minimal = DefinitionToml {
        definition: DefinitionSection {
            name: "Minimal".to_string(),
            discriminant: "Minimal".to_string(),
            version: "1".to_string(),
        },
        trees: TreesSection {
            main: "Minimal".to_string(),
            secondary: vec![],
            relational: vec![],
            subscription: vec![],
        },
        permissions: DefinitionPermissionsSection::default(),
        metadata: DefinitionMetadataSection {
            created_at: Utc::now(),
            updated_at: Utc::now(),
            schema_hash: "blake3:test".to_string(),
        },
    };

    let toml_str = toml::to_string_pretty(&minimal).unwrap();
    let parsed: DefinitionToml = toml::from_str(&toml_str).unwrap();

    assert_eq!(parsed.trees.secondary.len(), 0);
    assert_eq!(parsed.permissions.can_reference.len(), 0);
}

#[test]
fn test_schema_hash_format() {
    // Verify that schema hashes have the correct format
    let hash = blake3::hash(b"TestData");
    let hash_str = format!("blake3:{}", hash.to_hex());

    assert!(hash_str.starts_with("blake3:"));
    assert_eq!(hash_str.len(), 7 + 64); // "blake3:" + 64 hex chars

    // Verify it can be stored and retrieved in TOML
    use chrono::Utc;
    let def_toml = DefinitionToml {
        definition: DefinitionSection {
            name: "Test".to_string(),
            discriminant: "Test".to_string(),
            version: "1".to_string(),
        },
        trees: TreesSection {
            main: "Test".to_string(),
            secondary: vec![],
            relational: vec![],
            subscription: vec![],
        },
        permissions: DefinitionPermissionsSection::default(),
        metadata: DefinitionMetadataSection {
            created_at: Utc::now(),
            updated_at: Utc::now(),
            schema_hash: hash_str.clone(),
        },
    };

    let toml_str = toml::to_string_pretty(&def_toml).unwrap();
    let parsed: DefinitionToml = toml::from_str(&toml_str).unwrap();

    assert_eq!(parsed.metadata.schema_hash, hash_str);
}

#[test]
fn test_permission_role_section() {
    let role = PermissionRoleSection {
        name: "Moderator".to_string(),
        level: "ReadWrite".to_string(),
        definitions: vec!["Posts".to_string(), "Comments".to_string()],
    };

    // Test as part of a larger structure
    use chrono::Utc;
    let root = RootToml {
        manager: ManagerSection {
            name: "ForumManager".to_string(),
            version: "1".to_string(),
            root_path: "/data".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        definitions: DefinitionsSection {
            all: vec!["Posts".to_string(), "Comments".to_string()],
            loaded: vec![],
            warm_on_access: vec![],
        },
        permissions: vec![role],
    };

    let toml_str = toml::to_string_pretty(&root).unwrap();
    assert!(toml_str.contains("name = \"Moderator\""));
    assert!(toml_str.contains("level = \"ReadWrite\""));
    assert!(toml_str.contains("\"Posts\"") && toml_str.contains("\"Comments\""));

    let parsed: RootToml = toml::from_str(&toml_str).unwrap();
    assert_eq!(parsed.permissions[0].name, "Moderator");
    assert_eq!(parsed.permissions[0].definitions.len(), 2);
}
