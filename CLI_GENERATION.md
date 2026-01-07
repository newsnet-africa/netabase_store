# CLI Generation Implementation Summary

## Overview

This implementation adds automatic CLI generation for Netabase Store schemas, allowing databases to be shipped with fully-functional client binaries.

## What Was Created

### 1. CLI Generation Macros

**Location**: `netabase_macros/src/generators/cli.rs`

- `generate_definition_cli()` - Generates CLI commands for a single Definition
- `generate_repository_cli()` - Generates CLI for multiple Definitions in a Repository
- `generate_store_cli()` - Main CLI structure generator

**Location**: `netabase_macros/src/macros/netabase_cli.rs`

- Helper functions for CLI macro integration

**Location**: `netabase_macros/src/lib.rs`

- `generate_cli!()` macro - Procedural macro that reads schema.toml and generates complete Clap CLI

### 2. Database Construction Configuration

**Location**: `src/databases/redb/mod.rs`

Added `StoreConfig` - A builder-style configuration object for database creation with:

- **Client Binary Export**: Automatically copy and configure CLI binary
- **README Generation**: Auto-generate or provide custom README
- **Schema Export Control**: Enable/disable schema.toml export
- **Custom File Names**: Configure database, schema, and binary names
- **Flexible Creation**: Builder pattern for easy configuration

**Key Features**:
```rust
StoreConfig::new(path)
    .with_client_binary(Some("./target/release/client"))
    .with_readme_auto()
    .export_schema(true)
    .db_file_name("custom.redb")
    .create::<MyDefinition>()?
```

### 3. CLI Features

The generated CLI provides:

- **CRUD Operations** for all models:
  - `create` - Create new records via JSON
  - `read` - Read records by ID
  - `update` - Update records via JSON
  - `delete` - Delete records by ID
  - `list` - List all records

- **Hierarchical Commands**:
  ```bash
  client <model> <operation> [args]
  ```

- **Database Path Configuration**:
  ```bash
  client --db-path ./path/to/db <command>
  ```

### 4. Improved NBStore::new()

The `NBStore::new()` trait method now delegates to `StoreConfig` internally, providing:
- Backward compatibility with existing code
- Simple default behavior for basic use cases
- Easy migration path to advanced configuration

**Location**: `src/bin/client/`

- `main.rs` - Main entry point using `generate_cli!()` macro
- `generated.rs` - Generated code from schema

The client binary:
- Reads schema from `src/bin/tmp/dummy_db/schema.toml`
- Auto-generates all CLI commands
- Can be compiled and shipped with the database

### 5. Database Export Example

**Location**: `src/bin/dummy_db.rs`

Updated to demonstrate:
- Creating a database with `RedbStore::new()`
- Exporting the client binary with `export_binary()`

### 6. Database Folder Structure

**Location**: `databases/dummy_db/`

A complete, shippable database package contains:

```
databases/dummy_db/
├── data.redb        # Database file
├── schema.toml      # Schema definition
├── client           # Executable CLI binary
└── README.md        # Usage documentation
```

## Usage Examples

### Generating a CLI for a Schema

```rust
// In your binary (e.g., src/bin/client/main.rs)
netabase_macros::generate_cli!("path/to/schema.toml");

fn main() {
    use clap::Parser;
    let cli = SimpleDefinitionCli::parse();
    // Handle commands...
}
```

### Creating and Exporting a Database

```rust
use netabase_store::prelude::StoreConfig;

// Simple creation (default configuration)
let db_path = "./databases/my_db";
let _store = StoreConfig::new(db_path)
    .create::<MyDefinition>()?;

// With client binary export
let _store = StoreConfig::new(db_path)
    .with_client_binary(Some("./target/release/client"))
    .create::<MyDefinition>()?;

// With auto-generated README
let _store = StoreConfig::new(db_path)
    .with_readme_auto()
    .create::<MyDefinition>()?;

// Full configuration
let _store = StoreConfig::new(db_path)
    .with_client_binary(Some("./target/release/client"))
    .with_readme_auto()
    .export_schema(true)
    .create::<MyDefinition>()?;

// Custom file names
let _store = StoreConfig::new(db_path)
    .db_file_name("mydata.redb")
    .schema_file_name("myschema.toml")
    .client_binary_name("myclient")
    .create::<MyDefinition>()?;
```

### Using the Legacy NBStore Trait

The `NBStore::new()` method now uses `StoreConfig` internally:

```rust
use netabase_store::prelude::RedbStore;
use netabase_store::traits::database::store::NBStore;

// This is equivalent to StoreConfig::new(db_path).create::<MyDefinition>()
let _store = RedbStore::<MyDefinition>::new(db_path)?;
```

### Using the Client

```bash
# Show help
./client --help

# Create a record
./client inventoryitem create --json '{"id": 1, "name": "Widget"}'

# Read a record
./client inventoryitem read --id 1

# List all records
./client inventoryitem list
```

## Dependencies Added

- `clap = { version = "4.5", features = ["derive"] }` in `Cargo.toml`

## Key Design Decisions

1. **Macro-based Generation**: Uses procedural macros to generate CLI from schema at compile-time
2. **JSON Interface**: Commands use JSON for create/update operations for flexibility
3. **Self-Contained Packages**: Database folders include everything needed to interact with the data
4. **Hierarchical Commands**: Model → Operation structure for intuitive usage
5. **Platform Support**: Handles Unix executable permissions automatically
6. **Builder Pattern Configuration**: `StoreConfig` provides flexible, type-safe database creation
7. **Backward Compatibility**: `NBStore::new()` still works, now using `StoreConfig` internally
8. **Auto-generation**: README and binary export happen during database creation, not as separate steps

## Future Enhancements

Possible improvements:
- Field-level arguments instead of JSON (more user-friendly but less flexible)
- Query support beyond primary key lookups
- Batch operations
- Import/export to JSON/CSV
- Interactive mode
- Tab completion generation

## Testing

The implementation has been tested with:
- Building the client binary: ✓
- Running CLI help: ✓
- Exporting binary to database folder: ✓
- Running exported binary: ✓

## Files Modified

1. `netabase_macros/src/generators/mod.rs` - Added cli module
2. `netabase_macros/src/generators/cli.rs` - New file for CLI generation
3. `netabase_macros/src/macros/mod.rs` - Added netabase_cli module
4. `netabase_macros/src/macros/netabase_cli.rs` - New file for CLI macro helpers
5. `netabase_macros/src/lib.rs` - Added generate_cli! macro
6. `Cargo.toml` - Added clap dependency
7. `src/databases/redb/mod.rs` - **Major update**: Added `StoreConfig` builder and refactored construction
8. `src/prelude.rs` - Exported `StoreConfig` for convenient access
9. `src/bin/client/main.rs` - Updated to use generate_cli!
10. `src/bin/dummy_db.rs` - Updated to use `StoreConfig` for database creation
11. `databases/dummy_db/README.md` - Auto-generated from `StoreConfig`
12. `CLI_GENERATION.md` - Updated documentation

## Summary

This implementation provides a complete solution for generating CLI tools from Netabase schemas and shipping databases with their client binaries. The new `StoreConfig` builder makes database creation highly configurable and maintainable, with all options centralized in one place.

### Key Improvements

1. **Unified Configuration**: All database creation options are now in `StoreConfig`
2. **Automatic Setup**: Binary export and README generation happen during database creation
3. **Type Safety**: Builder pattern ensures valid configurations
4. **Backward Compatible**: Existing `NBStore::new()` calls still work
5. **Extensible**: Easy to add new configuration options in the future

### Migration Guide

**Before:**
```rust
let store = RedbStore::<MyDef>::new("./db")?;
RedbStore::<MyDef>::export_binary("./db", Some("./client"))?;
```

**After:**
```rust
let store = StoreConfig::new("./db")
    .with_client_binary(Some("./client"))
    .with_readme_auto()
    .create::<MyDef>()?;
```

Or keep using the simple version:
```rust
let store = RedbStore::<MyDef>::new("./db")?; // Still works!
```
