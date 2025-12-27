use netabase_store::traits::registery::definition::NetabaseDefinition;

#[netabase_macros::netabase_definition(ImportedDefinition, subscriptions(TopicA, TopicB), from_file = "schema_import.toml")]
pub mod imported_def {
    use super::*;
}

#[test]
fn test_schema_import() {
    use imported_def::{ImportedDefinition, ImportedUser};

    // Verify Definition
    let schema = ImportedDefinition::schema();
    assert_eq!(schema.name, "ImportedDefinition");
    assert!(schema.subscriptions.contains(&"TopicA".to_string()));
    assert!(schema.subscriptions.contains(&"TopicB".to_string()));

    // Verify Model
    assert!(schema.models.iter().any(|m| m.name == "ImportedUser"));
    
    // Verify Struct Generation (if this compiles, struct exists)
    let user = ImportedUser {
        id: imported_def::ImportedUserID("user1".to_string()),
        username: "test".to_string(),
        age: 25,
        subscriptions: vec![],
    };
    
    assert_eq!(user.username, "test");
}
