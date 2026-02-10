//! Test the generated CLI functionality with all features

use netabase_macros::{infer_netabase_definition, generate_cli, generate_cli_tests};
use netabase_store::prelude::*;

// Define a test schema from file
infer_netabase_definition!("tests/test_cli_schema.toml");
use TestDefinitionModule::*;

// Generate CLI for the definition
generate_cli!("tests/test_cli_schema.toml");

// Generate Nushell test script
generate_cli_tests!("tests/test_cli_schema.toml", "test_cli", "tests/test_cli.nu");

#[test]
fn test_cli_struct_exists() {
    // Test that the CLI structure was generated
    // This won't actually parse arguments, just checks that types exist
    let _ = std::marker::PhantomData::<TestDefinitionCli>;
}

#[test]
fn test_store_with_crud() -> NetabaseResult<()> {
    let store = RedbStore::<TestDefinition>::new_in_memory()?;
    
    // Test CRUD operations with model directly
    let user = User {
        id: UserID("alice".to_string()),
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        age: 30,
    };
    
    let txn = store.begin_write()?;
    txn.create(&user)?;
    txn.commit()?;
    
    // Test list
    let txn = store.begin_read()?;
    let results: Vec<User> = txn.list()?;
    assert_eq!(results.len(), 1);
    
    Ok(())
}

#[test]
fn test_json_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let user = User {
        id: UserID("bob".to_string()),
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
        age: 25,
    };
    
    // Test JSON serialization (as used by CLI)
    let json = serde_json::to_string(&user)?;
    let deserialized: User = serde_json::from_str(&json)?;
    assert_eq!(user.id, deserialized.id);
    assert_eq!(user.name, deserialized.name);
    
    // Test pretty JSON (as used by CLI)
    let pretty_json = serde_json::to_string_pretty(&user)?;
    assert!(pretty_json.contains("bob"));
    
    Ok(())
}

#[test]
fn test_ron_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let user = User {
        id: UserID("charlie".to_string()),
        name: "Charlie".to_string(),
        email: "charlie@example.com".to_string(),
        age: 35,
    };
    
    // Test RON serialization (as used by CLI)
    let ron = ron::to_string(&user)?;
    let deserialized: User = ron::from_str(&ron)?;
    assert_eq!(user.id, deserialized.id);
    assert_eq!(user.name, deserialized.name);
    
    Ok(())
}

#[test]
fn test_key_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let key = TestDefinitionKeys::User(UserKeys::Primary(UserID("alice".to_string())));
    
    // Test JSON serialization of keys (as used by CLI)
    let json = serde_json::to_string(&key)?;
    let deserialized: TestDefinitionKeys = serde_json::from_str(&json)?;
    if let TestDefinitionKeys::User(UserKeys::Primary(id)) = deserialized {
        assert_eq!(id.0, "alice");
    } else {
        panic!("Wrong key variant");
    }
    
    // Test RON serialization of keys
    let ron = ron::to_string(&key)?;
    let deserialized: TestDefinitionKeys = ron::from_str(&ron)?;
    if let TestDefinitionKeys::User(UserKeys::Primary(id)) = deserialized {
        assert_eq!(id.0, "alice");
    } else {
        panic!("Wrong key variant");
    }
    
    Ok(())
}
