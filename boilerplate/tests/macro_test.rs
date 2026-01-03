use netabase_store::blob::NetabaseBlobItem;
use netabase_store::traits::registery::definition::NetabaseDefinition;
use netabase_store_examples::{CategoryID, DefinitionTwo};

// Use the declarative macro
netabase_store_examples::import_netabase_schema!("testing/testing.netabase_schema.toml");

#[test]
fn test_automatic_import() {
    // The macro generates a module named `Definition` (inferred from file)
    // inside which is the struct `Definition`.
    use Definition::Definition;

    let schema = Definition::schema();
    println!("Schema: {:?}", schema);
    assert_eq!(schema.name, "Definition");

    // Check if models are generated
    assert!(schema.models.iter().any(|m| m.name == "User"));
}
