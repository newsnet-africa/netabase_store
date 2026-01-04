//! Tests for repository.toml generation and replication

mod common;

use netabase_store::traits::registery::repository::NetabaseRepository;
use netabase_store_examples::MainRepository;
use std::fs;
use std::path::PathBuf;

/// Test that repository.toml can be generated
#[test]
fn test_repository_toml_generation() {
    let toml = MainRepository::schema_toml();
    
    // Should contain repository name
    assert!(toml.contains("MainRepository"), "TOML should contain repository name");
    
    // Should contain both definitions
    assert!(toml.contains("Definition"), "TOML should contain Definition");
    assert!(toml.contains("DefinitionTwo"), "TOML should contain DefinitionTwo");
    
    // Should contain model names
    assert!(toml.contains("User"), "TOML should contain User model");
    assert!(toml.contains("Post"), "TOML should contain Post model");
    assert!(toml.contains("Category"), "TOML should contain Category model");
    
    // Should be valid TOML
    let parsed: toml::Value = toml::from_str(&toml).expect("Generated TOML should be valid");
    
    // Verify structure
    assert!(parsed.get("schema_format_version").is_some(), "Should have schema_format_version");
    assert!(parsed.get("name").is_some(), "Should have name");
    assert!(parsed.get("definitions").is_some(), "Should have definitions");
    
    // Verify definitions array
    let definitions = parsed["definitions"].as_array().expect("definitions should be an array");
    assert_eq!(definitions.len(), 2, "Should have 2 definitions");
}

/// Test that repository.toml can be written to a file
#[test]
fn test_write_repository_toml() -> std::io::Result<()> {
    let temp_dir = std::env::temp_dir().join(format!(
        "netabase_repo_toml_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    
    fs::create_dir_all(&temp_dir)?;
    
    let toml_path = temp_dir.join("repository.toml");
    
    // Write the schema
    MainRepository::write_schema_toml(&toml_path)?;
    
    // Verify file exists
    assert!(toml_path.exists(), "repository.toml should exist");
    
    // Read and verify content
    let content = fs::read_to_string(&toml_path)?;
    assert!(content.contains("MainRepository"), "File should contain repository name");
    
    // Verify it's valid TOML
    let _parsed: toml::Value = toml::from_str(&content).expect("File should contain valid TOML");
    
    // Clean up
    fs::remove_dir_all(&temp_dir).ok();
    
    Ok(())
}

/// Test that repository.toml contains version history
#[test]
fn test_repository_toml_version_history() {
    let toml = MainRepository::schema_toml();
    
    // Should contain model history
    assert!(toml.contains("model_history"), "TOML should contain model_history");
    
    // Should contain version information for User
    assert!(toml.contains("UserV1"), "TOML should contain UserV1 in version history");
    
    // Should contain version information for Post
    assert!(toml.contains("PostV1"), "TOML should contain PostV1 in version history");
    
    // Parse and verify structure
    let parsed: toml::Value = toml::from_str(&toml).expect("Generated TOML should be valid");
    let definitions = parsed["definitions"].as_array().expect("definitions should be an array");
    
    // Find Definition (which has versioned models)
    let definition = definitions.iter()
        .find(|d| d.get("name").and_then(|n| n.as_str()) == Some("Definition"))
        .expect("Should have Definition");
    
    let model_history = definition.get("model_history")
        .and_then(|h| h.as_array())
        .expect("Definition should have model_history array");
    
    assert!(!model_history.is_empty(), "Should have version history entries");
}

/// Test that schema hash is deterministic
#[test]
fn test_schema_hash_deterministic() {
    use netabase_store::traits::database::hash::FastHash;
    
    let hash1 = MainRepository::schema_hash::<FastHash>();
    let hash2 = MainRepository::schema_hash::<FastHash>();
    
    assert_eq!(hash1, hash2, "Schema hash should be deterministic");
}

/// Test schema comparison
#[test]
fn test_schema_comparison() {
    use netabase_store::traits::database::hash::FastHash;
    
    let hash = MainRepository::schema_hash::<FastHash>();
    
    // Should match itself
    assert!(
        MainRepository::schemas_match::<FastHash>(hash),
        "Schema should match its own hash"
    );
    
    // Should not match a different hash
    assert!(
        !MainRepository::schemas_match::<FastHash>(12345),
        "Schema should not match a different hash"
    );
}

/// Test that repository.toml can be parsed back
#[test]
fn test_repository_toml_roundtrip() {
    use netabase_store::traits::registery::definition::schema::RepositorySchema;
    
    let toml = MainRepository::schema_toml();
    
    // Parse it back
    let schema: RepositorySchema = toml::from_str(&toml)
        .expect("Should be able to parse generated TOML back into RepositorySchema");
    
    // Verify repository name
    assert_eq!(schema.name, "MainRepository");
    
    // Verify definitions count
    assert_eq!(schema.definitions.len(), 2, "Should have 2 definitions");
    
    // Verify definition names
    let def_names: Vec<_> = schema.definitions.iter().map(|d| d.name.as_str()).collect();
    assert!(def_names.contains(&"Definition"), "Should have Definition");
    assert!(def_names.contains(&"DefinitionTwo"), "Should have DefinitionTwo");
    
    // Verify models exist
    for def in &schema.definitions {
        assert!(!def.models.is_empty(), "Each definition should have models");
    }
}
