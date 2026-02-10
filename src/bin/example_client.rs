//! CLI client binary for ExampleDef database.
//!
//! This binary provides a command-line interface to interact with
//! an ExampleDef database (User, Product, Author, Book models).

use clap::Parser;

// Import the CLI generation from the schema file
netabase_macros::generate_cli!("schema_example.toml");

fn main() {
    let cli = ExampleDefCli::parse();
    
    if let Err(e) = cli.run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
