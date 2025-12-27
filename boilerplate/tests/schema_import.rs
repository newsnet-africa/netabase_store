use netabase_store::traits::registery::definition::NetabaseDefinition;

#[netabase_macros::netabase_definition(
    ImportedDefinition,
    subscriptions(General),
    from_file = "definition_2_roundtrip_schema.toml"
)]
pub mod imported_def {
    use super::*;
}

#[test]
fn test_schema_import() {
    use imported_def::{Category as ImportedCategory, CategoryID, ImportedDefinition};

    // Verify Definition
    let schema = ImportedDefinition::schema();
    assert_eq!(schema.name, "ImportedDefinition");
    assert!(schema.subscriptions.contains(&"General".to_string()));

    // Verify Model
    assert!(schema.models.iter().any(|m| m.name == "Category"));

    // Verify Struct Generation (if this compiles, struct exists)
    let cat = ImportedCategory {
        id: CategoryID("cat1".to_string()),
        name: "test".to_string(),
        description: "desc".to_string(),
        subscriptions: vec![],
    };

    assert_eq!(cat.name, "test");
}

#[netabase_macros::netabase_definition(
    RoundtripDefinition,
    subscriptions(General),
    from_file = "definition_2_roundtrip_schema.toml"
)]
pub mod roundtrip_import {
    use super::*;
}

#[test]
fn test_roundtrip_translation() {
    // This test verifies that we can import the schema exported by DefinitionTwo
    use roundtrip_import::{Category, CategoryID, RoundtripDefinition};

    let schema = RoundtripDefinition::schema();
    assert_eq!(schema.name, "RoundtripDefinition");

    // Check models exist
    assert!(schema.models.iter().any(|m| m.name == "Category"));

    // Verify fields were correctly reconstructed
    let cat_model = schema.models.iter().find(|m| m.name == "Category").unwrap();
    assert!(cat_model.fields.iter().any(|f| f.name == "id"
        && matches!(
            f.key_type,
            netabase_store::traits::registery::definition::schema::KeyTypeSchema::Primary
        )));

    // Verify struct works
    let _cat = Category {
        id: CategoryID("cat1".to_string()),
        name: "test".to_string(),
        description: "desc".to_string(),
        subscriptions: vec![],
    };
}
