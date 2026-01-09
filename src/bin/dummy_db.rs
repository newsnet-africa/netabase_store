use netabase_macros::infer_netabase_definition;
use netabase_store::prelude::StoreConfig;

infer_netabase_definition!("src/bin/simple_repo.toml");
use SimpleDefinitionModule::*;

fn main() {
    let db_path = "./databases/dummy_db";

    // Create database with full configuration
    let _store = StoreConfig::new(db_path)
        .with_client_binary(Some("./target/debug/client"))
        .with_readme_auto()
        .create::<SimpleDefinition>()
        .expect("Failed to create database");

    println!("Database created at: {}", db_path);
    println!("  - data.redb");
    println!("  - schema.toml");
    println!("  - client (executable)");
    println!("  - README.md");
}
