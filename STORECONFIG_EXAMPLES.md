# StoreConfig Examples

This file demonstrates various ways to use `StoreConfig` for database creation.

## Basic Usage

```rust
use netabase_store::prelude::StoreConfig;

// Simplest form - creates database with default settings
let store = StoreConfig::new("./my_database")
    .create::<MyDefinition>()?;
// Creates: data.redb, schema.toml
```

## With Client Binary Export

```rust
// Export a specific binary
let store = StoreConfig::new("./my_database")
    .with_client_binary(Some("./target/release/client"))
    .create::<MyDefinition>()?;
// Creates: data.redb, schema.toml, client (executable)

// Export the current executable
let store = StoreConfig::new("./my_database")
    .with_client_binary(None) // Uses std::env::current_exe()
    .create::<MyDefinition>()?;
```

## With README

```rust
// Auto-generate README from schema
let store = StoreConfig::new("./my_database")
    .with_readme_auto()
    .create::<MyDefinition>()?;
// Creates: data.redb, schema.toml, README.md

// Custom README content
let custom_readme = r#"
# My Custom Database

This is my custom documentation.
"#;

let store = StoreConfig::new("./my_database")
    .with_readme(Some(custom_readme))
    .create::<MyDefinition>()?;
```

## Complete Package

```rust
// Create a fully self-contained database package
let store = StoreConfig::new("./my_database")
    .with_client_binary(Some("./target/release/client"))
    .with_readme_auto()
    .export_schema(true)
    .create::<MyDefinition>()?;
// Creates: data.redb, schema.toml, client, README.md
```

## Custom File Names

```rust
// Customize all file names
let store = StoreConfig::new("./my_database")
    .db_file_name("mydata.redb")
    .schema_file_name("myschema.toml")
    .client_binary_name("mycli")
    .create::<MyDefinition>()?;
// Creates: mydata.redb, myschema.toml
```

## Without Schema Export

```rust
// Don't export schema.toml
let store = StoreConfig::new("./my_database")
    .export_schema(false)
    .create::<MyDefinition>()?;
// Creates: data.redb only
```

## Production Setup

```rust
// Typical production configuration
let store = StoreConfig::new("./production_db")
    .with_client_binary(Some("./target/release/client"))
    .with_readme_auto()
    .export_schema(true)
    .create::<MyDefinition>()?;

// Then ship the entire ./production_db folder
```

## Development Setup

```rust
// Quick development database
let store = StoreConfig::new("./dev_db")
    .with_client_binary(Some("./target/debug/client"))
    .create::<MyDefinition>()?;
```

## Testing Setup

```rust
// For testing, use the built-in methods instead:
use netabase_store::prelude::RedbStore;

// Temporary database (cleaned up automatically)
let (store, _temp) = RedbStore::<MyDefinition>::new_temporary()?;

// In-memory database (no disk I/O)
let store = RedbStore::<MyDefinition>::new_in_memory()?;
```

## Builder Pattern Chaining

```rust
// All options can be chained in any order
let store = StoreConfig::new("./my_database")
    .export_schema(true)
    .with_readme_auto()
    .db_file_name("custom.redb")
    .with_client_binary(Some("./client"))
    .schema_file_name("schema.toml")
    .client_binary_name("cli")
    .create::<MyDefinition>()?;
```

## Error Handling

```rust
use netabase_store::errors::NetabaseResult;

fn create_database() -> NetabaseResult<()> {
    let _store = StoreConfig::new("./my_database")
        .with_client_binary(Some("./target/release/client"))
        .create::<MyDefinition>()?;
    
    Ok(())
}

// Handle errors
match create_database() {
    Ok(_) => println!("Database created successfully"),
    Err(e) => eprintln!("Failed to create database: {}", e),
}
```

## Reusing Configuration

```rust
// Create a configuration template
let config = StoreConfig::new("./template")
    .with_client_binary(Some("./target/release/client"))
    .with_readme_auto()
    .export_schema(true);

// Use for different databases
let store1 = StoreConfig::new("./db1")
    .with_client_binary(config.client_binary.clone())
    .create::<MyDefinition>()?;

let store2 = StoreConfig::new("./db2")
    .with_client_binary(config.client_binary.clone())
    .create::<MyDefinition>()?;
```

## Legacy NBStore Compatibility

```rust
use netabase_store::traits::database::store::NBStore;
use netabase_store::prelude::RedbStore;

// Old way (still works!)
let store = RedbStore::<MyDefinition>::new("./my_database")?;
// Internally uses: StoreConfig::new(path).create::<MyDefinition>()

// New way (more features)
let store = StoreConfig::new("./my_database")
    .with_client_binary(Some("./client"))
    .create::<MyDefinition>()?;
```
