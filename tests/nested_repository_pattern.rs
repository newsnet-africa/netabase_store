//! Test nested repository pattern

use netabase_store_examples::repository_example::{EmployeeRepo, ManagerRepo, Employee};

#[test]
fn test_employee_repo_name() {
    use netabase_store::traits::registery::repository::NetabaseRepository;
    
    assert_eq!(EmployeeRepo::name(), "EmployeeRepo");
}

#[test]
fn test_employee_repo_schema() {
    let schema_toml = EmployeeRepo::schema_toml();
    
    println!("EmployeeRepo schema:\n{}", schema_toml);
    
    // Should have the repository name
    assert!(schema_toml.contains("EmployeeRepo"), "Should contain repository name");
}

#[test]
fn test_manager_repo_schema() {
    let schema_toml = ManagerRepo::schema_toml();
    
    println!("ManagerRepo schema:\n{}", schema_toml);
    
    // Should have the repository name
    assert!(schema_toml.contains("ManagerRepo"), "Should contain repository name");
}
