//! Example binary to generate a CLI client for ExampleDef.
//!
//! This creates a database at `./my_database` with a working CLI client.

use netabase_store::doc_example::ExampleDef;
use netabase_store::databases::redb::StoreConfig;

fn main() -> netabase_store::errors::NetabaseResult<()> {
    let db_path = "./my_database";
    
    println!("Generating database and CLI client at: {}", db_path);
    
    let _store = StoreConfig::new(db_path)
        .with_client_binary(Some("./target/release/example_client"))
        .with_readme_auto()
        .create::<ExampleDef>()?;
    
    println!("✓ Database created successfully!");
    println!("  - {}/data.redb", db_path);
    println!("  - {}/schema.toml", db_path);
    println!("  - {}/client (executable)", db_path);
    println!("  - {}/README.md", db_path);
    println!();
    println!("Try it out:");
    println!("  cd {}", db_path);
    println!("  ./client --help");
    
    Ok(())
}
