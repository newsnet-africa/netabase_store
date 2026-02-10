//! Generate repository.toml files for all repositories in the examples

use netabase_store::prelude::NetabaseDefinition;
use example::MainRepository;
use example::repository_example::{EmployeeRepo, ManagerRepo};
use example::simple_def_example::simple_definition::SimpleDefinition;

use std::fs;
use std::path::Path;

fn simple_dummy() -> std::io::Result<()> {
    let output_dir = Path::new("generated_schemas");
    fs::create_dir_all(output_dir)?;

    // Generate MainRepository schema
    println!("Generating MainRepository schema...");
    let main_repo_path = output_dir.join("simple_repo.toml");
    let toml = SimpleDefinition::export_toml();

    if let Err(e) = std::fs::write(&main_repo_path, &toml) {
        eprintln!("Warning: Failed to write schema file: {}", e);
    };

    Ok(())
}

fn dummy() -> std::io::Result<()> {
    println!("Generating repository.toml files...\n");

    // Create output directory if it doesn't exist
    let output_dir = Path::new("generated_schemas");
    fs::create_dir_all(output_dir)?;

    // Generate MainRepository schema
    println!("Generating MainRepository schema...");
    let main_repo_path = output_dir.join("main_repository.toml");
    MainRepository::write_schema_toml(&main_repo_path)?;
    println!("✓ Written to: {}", main_repo_path.display());
    println!("  Size: {} bytes\n", fs::metadata(&main_repo_path)?.len());

    // Generate EmployeeRepo schema
    println!("Generating EmployeeRepo schema...");
    let employee_repo_path = output_dir.join("employee_repository.toml");
    EmployeeRepo::write_schema_toml(&employee_repo_path)?;
    println!("✓ Written to: {}", employee_repo_path.display());
    println!(
        "  Size: {} bytes\n",
        fs::metadata(&employee_repo_path)?.len()
    );

    // Generate ManagerRepo schema
    println!("Generating ManagerRepo schema...");
    let manager_repo_path = output_dir.join("manager_repository.toml");
    ManagerRepo::write_schema_toml(&manager_repo_path)?;
    println!("✓ Written to: {}", manager_repo_path.display());
    println!(
        "  Size: {} bytes\n",
        fs::metadata(&manager_repo_path)?.len()
    );

    println!("All repository schemas generated successfully!");
    println!("\nThese files can be used to:");
    println!("  1. Replicate the database structure on another system");
    println!("  2. Version control your schema");
    println!("  3. Compare schemas between P2P nodes");
    println!("  4. Generate code from schema files");

    Ok(())
}

fn main() -> std::io::Result<()> {
    simple_dummy()?;
    Ok(())
}
