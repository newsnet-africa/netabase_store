//! Test CLI client generation

use example::simple_def_example::simple_definition::SimpleDefinition;

#[test]
fn test_generate_cli_client() {
    let output_path = "test_generated_cli";
    
    // Clean up from previous run
    std::fs::remove_dir_all(output_path).ok();
    
    // Generate the CLI client
    SimpleDefinition::generate_client(output_path).expect("Failed to generate CLI client");
    
    // Verify files were created
    assert!(std::path::Path::new(output_path).exists());
    assert!(std::path::Path::new(&format!("{}/Cargo.toml", output_path)).exists());
    assert!(std::path::Path::new(&format!("{}/src/main.rs", output_path)).exists());
    assert!(std::path::Path::new(&format!("{}/schema.toml", output_path)).exists());
    assert!(std::path::Path::new(&format!("{}/README.md", output_path)).exists());
    
    println!("CLI client generated successfully at: {}", output_path);
    
    // Don't clean up so we can inspect the generated code
}
