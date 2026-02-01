//! Tests for repository TOML export functionality.
//!
//! This test verifies that repositories can export their complete schema
//! as TOML, including all definitions and their models.
//!
//! # Test Flow
//!
//! 1. Export EmployeeRepo schema to TOML
//! 2. Export ManagerRepo schema to TOML
//! 3. Verify TOML contains all expected definitions
//! 4. Verify TOML can be written to file
//! 5. Verify schema hashes are consistent

use example::repository_example::{EmployeeRepo, ManagerRepo};
use netabase_store::traits::database::hash::{CryptoHash, FastHash};
use std::path::PathBuf;

const TEST_DIR: &str = "test_output/repository_toml";

#[test]
fn export_employee_repo_schema() {
    let toml = EmployeeRepo::schema_toml();
    
    println!("EmployeeRepo TOML:\n{}", toml);
    
    // Verify TOML structure
    assert!(toml.contains("schema_format_version"));
    assert!(toml.contains("name = \"EmployeeRepo\""));
    
    // Verify Employee definition is present
    assert!(toml.contains("[[definitions]]"));
    assert!(toml.contains("name = \"Employee\""));
    
    // Verify Inventory definition is present
    assert!(toml.contains("name = \"Inventory\""));
    
    // Should NOT contain Reports (that's only in ManagerRepo)
    assert!(!toml.contains("name = \"Reports\""));
    
    // Verify models are present
    assert!(toml.contains("[[definitions.models]]"));
    assert!(toml.contains("User"));
    assert!(toml.contains("Shift"));
    assert!(toml.contains("Product"));
}

#[test]
fn export_manager_repo_schema() {
    let toml = ManagerRepo::schema_toml();
    
    println!("ManagerRepo TOML:\n{}", toml);
    
    // Verify TOML structure
    assert!(toml.contains("schema_format_version"));
    assert!(toml.contains("name = \"ManagerRepo\""));
    
    // Verify Employee definition is present
    assert!(toml.contains("name = \"Employee\""));
    
    // Verify Reports definition is present
    assert!(toml.contains("name = \"Reports\""));
    
    // Should NOT contain Inventory (that's only in EmployeeRepo)
    assert!(!toml.contains("name = \"Inventory\""));
    
    // Verify models are present
    assert!(toml.contains("User"));
    assert!(toml.contains("Shift"));
    assert!(toml.contains("Report"));
}

#[test]
fn write_employee_repo_schema_to_file() {
    let dir = PathBuf::from(TEST_DIR);
    std::fs::create_dir_all(&dir).unwrap();
    
    let schema_path = dir.join("employee_repo.toml");
    EmployeeRepo::write_schema_toml(&schema_path).unwrap();
    
    // Verify file was created and contains expected content
    let content = std::fs::read_to_string(&schema_path).unwrap();
    assert!(content.contains("name = \"EmployeeRepo\""));
    assert!(content.contains("name = \"Employee\""));
    assert!(content.contains("name = \"Inventory\""));
    
    println!("Employee repository schema written to: {}", schema_path.display());
}

#[test]
fn write_manager_repo_schema_to_file() {
    let dir = PathBuf::from(TEST_DIR);
    std::fs::create_dir_all(&dir).unwrap();
    
    let schema_path = dir.join("manager_repo.toml");
    ManagerRepo::write_schema_toml(&schema_path).unwrap();
    
    // Verify file was created and contains expected content
    let content = std::fs::read_to_string(&schema_path).unwrap();
    assert!(content.contains("name = \"ManagerRepo\""));
    assert!(content.contains("name = \"Employee\""));
    assert!(content.contains("name = \"Reports\""));
    
    println!("Manager repository schema written to: {}", schema_path.display());
}

#[test]
fn repository_schema_hashes() {
    // Test FastHash
    let fast_hash_1 = EmployeeRepo::schema_hash::<FastHash>();
    let fast_hash_2 = EmployeeRepo::schema_hash::<FastHash>();
    assert_eq!(fast_hash_1, fast_hash_2, "FastHash should be deterministic");
    
    // Test CryptoHash
    let crypto_hash_1 = EmployeeRepo::schema_hash::<CryptoHash>();
    let crypto_hash_2 = EmployeeRepo::schema_hash::<CryptoHash>();
    assert_eq!(crypto_hash_1, crypto_hash_2, "CryptoHash should be deterministic");
    
    // Different repositories should have different hashes
    let manager_fast_hash = ManagerRepo::schema_hash::<FastHash>();
    assert_ne!(
        fast_hash_1,
        manager_fast_hash,
        "Different repositories should have different hashes"
    );
    
    println!("EmployeeRepo FastHash: {}", fast_hash_1);
    println!("EmployeeRepo CryptoHash: {}", crypto_hash_1);
    println!("ManagerRepo FastHash: {}", manager_fast_hash);
}

#[test]
fn repository_schema_comparison() {
    let employee_hash = EmployeeRepo::schema_hash::<FastHash>();
    
    // Schemas should match themselves
    assert!(
        EmployeeRepo::schemas_match::<FastHash>(employee_hash),
        "Repository should match its own schema hash"
    );
    
    // Schemas should NOT match different repositories
    let manager_hash = ManagerRepo::schema_hash::<FastHash>();
    assert!(
        !EmployeeRepo::schemas_match::<FastHash>(manager_hash),
        "Different repositories should not match"
    );
}

#[test]
fn repository_migration_metadata() {
    // Migration metadata should exist (even if empty)
    let employee_metadata = EmployeeRepo::migration_metadata();
    assert_eq!(employee_metadata.field_renames.len(), 0);
    assert_eq!(employee_metadata.type_changes.len(), 0);
    assert_eq!(employee_metadata.added_fields.len(), 0);
    assert_eq!(employee_metadata.removed_fields.len(), 0);
    
    let manager_metadata = ManagerRepo::migration_metadata();
    assert_eq!(manager_metadata.field_renames.len(), 0);
    assert_eq!(manager_metadata.type_changes.len(), 0);
    assert_eq!(manager_metadata.added_fields.len(), 0);
    assert_eq!(manager_metadata.removed_fields.len(), 0);
}

#[test]
fn verify_repository_toml_structure() {
    use netabase_store::traits::registry::definition::schema::RepositorySchema;
    
    let toml = EmployeeRepo::schema_toml();
    
    // Parse using RepositorySchema to verify structure
    let schema = RepositorySchema::from_toml(&toml)
        .expect("Repository TOML should be valid");
    
    // Verify top-level fields
    assert!(schema.schema_format_version > 0);
    assert_eq!(schema.name, "EmployeeRepo");
    assert!(!schema.definitions.is_empty());
    
    // Verify each definition has required fields
    for def in &schema.definitions {
        assert!(!def.name.is_empty(), "Definition should have name");
        assert!(!def.models.is_empty(), "Definition should have at least one model");
        
        // Verify each model has required fields
        for model in &def.models {
            assert!(!model.name.is_empty(), "Model should have name");
            assert!(!model.fields.is_empty(), "Model should have fields");
        }
    }
    
    println!("Repository TOML structure verified successfully");
}
