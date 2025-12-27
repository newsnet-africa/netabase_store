use netabase_store_examples::boilerplate_lib::{Definition, DefinitionTwo};
use netabase_store::traits::registery::definition::NetabaseDefinition;

#[test]
fn test_definition_schema_export() {
    let toml = Definition::export_toml();
    println!("Definition TOML:\n{}", toml);

    // Verify main definition
    assert!(toml.contains("name = \"Definition\""));
    assert!(toml.contains("name = \"User\""));
    assert!(toml.contains("name = \"Post\""));
    assert!(toml.contains("name = \"HeavyModel\""));
    
    // Verify fields and types
    assert!(toml.contains("name = \"partner\""));
    assert!(toml.contains("definition = \"Definition\""));
    assert!(toml.contains("model = \"User\""));
    
    assert!(toml.contains("name = \"category\""));
    assert!(toml.contains("definition = \"DefinitionTwo\""));
    assert!(toml.contains("model = \"Category\""));
}

#[test]
fn test_definition_two_schema_export() {
    let toml = DefinitionTwo::export_toml();
    println!("DefinitionTwo TOML:\n{}", toml);

    assert!(toml.contains("name = \"DefinitionTwo\""));
    assert!(toml.contains("name = \"Category\""));
    assert!(toml.contains("\"General\""));
}
