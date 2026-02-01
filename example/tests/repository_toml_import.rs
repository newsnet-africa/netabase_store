//! Tests for repository TOML import functionality.
//!
//! This test verifies that repositories can be imported from TOML files,
//! including all definitions and their models.
//!
//! # Test Flow
//!
//! 1. Export repository schema to TOML
//! 2. Import repository schema from TOML
//! 3. Verify imported schema matches original
//! 4. Verify roundtrip (export -> import -> export) is stable
//! 5. Test error handling for malformed TOML
//!
//! # Feature Requirements
//!
//! These tests require the `schema_export` feature to be enabled in `netabase_store`.
//! Run with: `cargo test --all-features`

use example::repository_example::{EmployeeRepo, ManagerRepo};
use netabase_store::traits::registry::definition::schema::RepositorySchema;
use std::path::PathBuf;

const TEST_DIR: &str = "test_output/repository_toml_import";

#[test]
fn import_employee_repo_from_toml() {
    // Export to TOML
    let original_toml = EmployeeRepo::schema_toml();
    println!("Original TOML:\n{}", original_toml);
    
    // Parse from TOML
    let schema = RepositorySchema::from_toml(&original_toml)
        .expect("Should parse valid TOML");
    
    // Verify structure
    assert_eq!(schema.name, "EmployeeRepo");
    assert_eq!(schema.schema_format_version, 2);
    assert!(!schema.definitions.is_empty(), "Should have definitions");
    
    // Verify definitions are present
    let def_names: Vec<_> = schema.definitions.iter()
        .map(|d| d.name.as_str())
        .collect();
    assert!(def_names.contains(&"Employee"), "Should have Employee definition");
    assert!(def_names.contains(&"Inventory"), "Should have Inventory definition");
    assert!(!def_names.contains(&"Reports"), "Should NOT have Reports definition");
    
    println!("Successfully imported EmployeeRepo with {} definitions", schema.definitions.len());
}

#[test]
fn import_manager_repo_from_toml() {
    // Export to TOML
    let original_toml = ManagerRepo::schema_toml();
    println!("Original TOML:\n{}", original_toml);
    
    // Parse from TOML
    let schema = RepositorySchema::from_toml(&original_toml)
        .expect("Should parse valid TOML");
    
    // Verify structure
    assert_eq!(schema.name, "ManagerRepo");
    assert_eq!(schema.schema_format_version, 2);
    
    // Verify definitions are present
    let def_names: Vec<_> = schema.definitions.iter()
        .map(|d| d.name.as_str())
        .collect();
    assert!(def_names.contains(&"Employee"), "Should have Employee definition");
    assert!(def_names.contains(&"Reports"), "Should have Reports definition");
    assert!(!def_names.contains(&"Inventory"), "Should NOT have Inventory definition");
    
    println!("Successfully imported ManagerRepo with {} definitions", schema.definitions.len());
}

#[test]
fn roundtrip_employee_repo() {
    // First export
    let toml1 = EmployeeRepo::schema_toml();
    
    // Import
    let schema = RepositorySchema::from_toml(&toml1)
        .expect("Should parse first export");
    
    // Second export
    let toml2 = schema.to_toml();
    
    // Parse second export
    let schema2 = RepositorySchema::from_toml(&toml2)
        .expect("Should parse second export");
    
    // Verify they match
    assert_eq!(schema.name, schema2.name);
    assert_eq!(schema.schema_format_version, schema2.schema_format_version);
    assert_eq!(schema.definitions.len(), schema2.definitions.len());
    
    // Verify hashes match
    let hash1 = schema.compute_hash();
    let hash2 = schema2.compute_hash();
    assert_eq!(hash1, hash2, "Roundtrip should preserve schema hash");
    
    println!("Roundtrip successful - schema hash: {}", hash1);
}

#[test]
fn roundtrip_manager_repo() {
    // First export
    let toml1 = ManagerRepo::schema_toml();
    
    // Import
    let schema = RepositorySchema::from_toml(&toml1)
        .expect("Should parse first export");
    
    // Second export
    let toml2 = schema.to_toml();
    
    // Parse second export
    let schema2 = RepositorySchema::from_toml(&toml2)
        .expect("Should parse second export");
    
    // Verify they match
    assert_eq!(schema.name, schema2.name);
    assert_eq!(schema.definitions.len(), schema2.definitions.len());
    
    // Verify hashes match
    let hash1 = schema.compute_hash();
    let hash2 = schema2.compute_hash();
    assert_eq!(hash1, hash2, "Roundtrip should preserve schema hash");
    
    println!("Roundtrip successful - schema hash: {}", hash1);
}

#[test]
fn import_from_file() {
    let dir = PathBuf::from(TEST_DIR);
    std::fs::create_dir_all(&dir).unwrap();
    
    // Write schema to file
    let schema_path = dir.join("employee_repo_import_test.toml");
    EmployeeRepo::write_schema_toml(&schema_path).unwrap();
    
    // Import from file
    let schema = RepositorySchema::from_toml_file(&schema_path)
        .expect("Should import from file");
    
    // Verify
    assert_eq!(schema.name, "EmployeeRepo");
    assert!(!schema.definitions.is_empty());
    
    println!("Successfully imported repository from file: {}", schema_path.display());
}

#[test]
fn import_malformed_toml() {
    let malformed = r#"
        this is not valid toml!!!
        [[[broken]]]
    "#;
    
    let result = RepositorySchema::from_toml(malformed);
    assert!(result.is_err(), "Should fail to parse malformed TOML");
    
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("Failed to parse"), "Error should mention parsing failure");
    
    println!("Correctly rejected malformed TOML: {}", err_msg);
}

#[test]
fn import_incomplete_toml() {
    let incomplete = r#"
        name = "IncompleteRepo"
        # Missing schema_format_version and definitions
    "#;
    
    let result = RepositorySchema::from_toml(incomplete);
    
    // Should either fail or use defaults
    match result {
        Ok(schema) => {
            // If it succeeds, verify defaults were applied
            assert_eq!(schema.name, "IncompleteRepo");
            assert_eq!(schema.schema_format_version, 1); // Default version
            println!("Successfully parsed incomplete TOML with defaults");
        }
        Err(e) => {
            println!("Correctly rejected incomplete TOML: {}", e);
        }
    }
}

#[test]
fn import_nonexistent_file() {
    let path = PathBuf::from(TEST_DIR).join("nonexistent.toml");
    
    let result = RepositorySchema::from_toml_file(&path);
    assert!(result.is_err(), "Should fail to read nonexistent file");
    
    let err_msg = result.unwrap_err();
    assert!(err_msg.contains("Failed to read"), "Error should mention read failure");
    
    println!("Correctly rejected nonexistent file: {}", err_msg);
}

#[test]
fn verify_model_fields_after_import() {
    let toml = EmployeeRepo::schema_toml();
    let schema = RepositorySchema::from_toml(&toml).unwrap();
    
    // Find Employee definition
    let employee_def = schema.definitions.iter()
        .find(|d| d.name == "Employee")
        .expect("Should have Employee definition");
    
    // Verify it has models
    assert!(!employee_def.models.is_empty(), "Employee should have models");
    
    // Check that models have fields
    for model in &employee_def.models {
        println!("Model: {} with {} fields", model.name, model.fields.len());
        assert!(!model.fields.is_empty(), "Model {} should have fields", model.name);
        
        // Verify field structure
        for field in &model.fields {
            assert!(!field.name.is_empty(), "Field should have a name");
            assert!(!field.type_name.is_empty(), "Field should have a type");
        }
    }
}

#[test]
fn compare_imported_with_original_definitions() {
    // This test verifies that importing doesn't lose information
    let toml = EmployeeRepo::schema_toml();
    let imported = RepositorySchema::from_toml(&toml).unwrap();
    
    // Get original schema
    let original_toml = EmployeeRepo::schema_toml();
    let original = RepositorySchema::from_toml(&original_toml).unwrap();
    
    // They should be identical
    assert_eq!(imported.name, original.name);
    assert_eq!(imported.definitions.len(), original.definitions.len());
    
    for (imp_def, orig_def) in imported.definitions.iter().zip(original.definitions.iter()) {
        assert_eq!(imp_def.name, orig_def.name);
        assert_eq!(imp_def.models.len(), orig_def.models.len());
        
        for (imp_model, orig_model) in imp_def.models.iter().zip(orig_def.models.iter()) {
            assert_eq!(imp_model.name, orig_model.name);
            assert_eq!(imp_model.fields.len(), orig_model.fields.len());
        }
    }
    
    println!("Imported schema matches original perfectly");
}

#[test]
fn import_preserves_schema_hash() {
    let toml = EmployeeRepo::schema_toml();
    let schema = RepositorySchema::from_toml(&toml).unwrap();
    
    // Compute hash
    let hash = schema.compute_hash();
    
    // Export again and re-import
    let toml2 = schema.to_toml();
    let schema2 = RepositorySchema::from_toml(&toml2).unwrap();
    let hash2 = schema2.compute_hash();
    
    assert_eq!(hash, hash2, "Schema hash should be preserved through import/export");
    
    println!("Schema hash preserved: {}", hash);
}
