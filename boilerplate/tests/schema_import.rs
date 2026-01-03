use netabase_store::blob::NetabaseBlobItem;
use netabase_store::relational::RelationalLink;
use netabase_store::traits::registery::definition::NetabaseDefinition;

// Import DefinitionTwo and Category from lib for cross-links
pub use netabase_store_examples::boilerplate_lib::{Category, CategoryID, DefinitionTwo};

// Test 1: Import Definition (Definition) from definition_roundtrip_schema.toml
netabase_store_examples::import_netabase_schema!("definition_roundtrip_schema.toml", imported_def);

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
// Note: We name it differently to avoid conflict with LibDefinitionTwo above if name was the same.
// But name in TOML is "DefinitionTwo", so it will conflict if we don't use a different module name or alias.
netabase_store_examples::import_netabase_schema!(
    "definition_2_roundtrip_schema.toml",
    roundtrip_import
);

#[test]
fn test_roundtrip_translation() {
    // This test verifies that we can import the schema exported by DefinitionTwo
    use netabase_store::traits::registery::definition::NetabaseDefinition;
    use roundtrip_import::{Category, CategoryID, DefinitionTwo};

    let schema = DefinitionTwo::schema();
    assert_eq!(schema.name, "DefinitionTwo");

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
