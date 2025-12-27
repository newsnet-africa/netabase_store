use netabase_store::traits::registery::definition::NetabaseDefinition;
use netabase_store::blob::NetabaseBlobItem;
use netabase_store::relational::RelationalLink;
use bincode::{Encode, Decode};
use serde::{Serialize, Deserialize};

// Test 1: Import Definition (Definition) from definition_roundtrip_schema.toml
#[netabase_macros::netabase_definition(
    Definition,
    subscriptions(Topic1, Topic2, Topic3, Topic4),
    from_file = "definition_roundtrip_schema.toml"
)]
pub mod imported_def {
    use super::*;
    // Import DefinitionTwo and Category from lib for cross-links
    pub use netabase_store_examples::boilerplate_lib::{DefinitionTwo, Category, CategoryID};
}

#[test]
fn test_definition_roundtrip() {
    use imported_def::{Definition, User, UserID}; // Local definitions
    use netabase_store_examples::boilerplate_lib::CategoryID; // External ID

    // Verify Definition
    let schema = Definition::schema();
    assert_eq!(schema.name, "Definition");
    assert!(schema.subscriptions.contains(&"Topic1".to_string()));

    // Verify Model
    assert!(schema.models.iter().any(|m| m.name == "User"));

    // Verify Struct Generation
    let user = User {
        id: UserID("user1".to_string()),
        name: "test".to_string(),
        age: 25,
        partner: RelationalLink::new_dehydrated(UserID("partner1".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("cat1".to_string())),
        bio: imported_def::LargeUserFile::default(),
        another: imported_def::AnotherLargeUserFile::default(),
        subscriptions: vec![],
    };

    assert_eq!(user.name, "test");
    assert_eq!(user.age, 25);
}

// Test 2: Import DefinitionTwo (RoundtripDefinition) from definition_2_roundtrip_schema.toml
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
    use netabase_store::traits::registery::definition::NetabaseDefinition;

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