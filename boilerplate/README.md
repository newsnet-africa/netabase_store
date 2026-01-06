# Netabase Store Boilerplate & Examples

This crate serves as the comprehensive testbed, boilerplate, and example suite for `netabase_store`. It demonstrates the full power of the macro-based definition system for creating type-safe, relational database schemas on top of `redb`.

## Overview

The boilerplate provides a reference implementation of:
- **Models**: Strongly typed structs with primary/secondary keys, blobs, and relationships.
- **Definitions**: Groups of models that form a cohesive schema unit.
- **Repositories**: Isolated contexts that enforce strict graph completeness for definitions.
- **Migrations**: Versioned models with upgrade/downgrade paths.
- **Macros**: Full usage of `netabase_macros` to generate all boilerplate code.

## Key Features Demonstrated

1.  **Strong Typing**: All IDs (`UserID`, `ShiftID`) are strongly typed wrappers, preventing accidental ID mixing.
2.  **Relational Links**: Type-safe links between models (`RelationalLink<Definition, User>`) that handle hydration/dehydration.
3.  **Cross-Definition Linking**: Linking models across different definitions (e.g., `User` in `Definition` linking to `Category` in `DefinitionTwo`).
4.  **Repository Isolation**: `EmployeeRepo` and `ManagerRepo` show how to expose different subsets of data to different contexts while sharing underlying definitions.
5.  **Blob Storage**: Handling large binary data (`LargeUserFile`) separately from the main record for efficiency.
6.  **Schema Evolution**: Full examples of versioned models (`UserV1` -> `User`) with migration logic.

## Project Structure

- **`src/boilerplate_lib/mod.rs`**: The main example showcasing a standard application schema (`User`, `Post`).
- **`src/boilerplate_lib/repository_example.rs`**: Advanced example showcasing the Repository pattern for access control and modularity.
- **`tests/`**: Integration tests verifying schema export, import, and migration logic.

## Usage

### Running Tests

Run the full suite of tests, including unit tests, integration tests, and doctests:

```bash
cargo test -p netabase_store_examples
```

### Running Benchmarks

Performance benchmarks for CRUD operations and stress testing:

```bash
# Basic CRUD operations
cargo bench --bench crud

# High-load stress testing
cargo bench --bench stress
```

## Code Examples

### Defining a Model

```rust
use netabase_store_examples::{User, UserID, CategoryID, LargeUserFile, AnotherLargeUserFile};
use netabase_store::relational::RelationalLink;

// Models are regular Rust structs with attributes
let user = User {
    id: UserID("user_123".to_string()),
    first_name: "Alice".to_string(),
    last_name: "Smith".to_string(),
    age: 30,
    // Relationships are type-checked
    partner: RelationalLink::new_dehydrated(UserID("user_456".to_string())),
    category: RelationalLink::new_dehydrated(CategoryID("cat_789".to_string())),
    bio: LargeUserFile {
        data: vec![1, 2, 3],
        metadata: "User bio".to_string(),
    },
    another: AnotherLargeUserFile(vec![4, 5, 6]),
    subscriptions: Default::default(),
};
```

### Repository Pattern

The repository pattern allows you to define strict boundaries for your data graph.

```rust
use netabase_store_examples::repository_example::{EmployeeRepo, ManagerRepo};

// EmployeeRepo can access: Employee (User, Shift), Inventory
// ManagerRepo can access: Employee (User, Shift), Reports
```

See `src/boilerplate_lib/repository_example.rs` for the full implementation.

## Schema Migration

The boilerplate includes a complete example of schema evolution in `src/boilerplate_lib/mod.rs`:

1.  **`UserV1`**: Original version.
2.  **`User`**: Current version (marked with `current`).
3.  **`MigrateFrom<UserV1> for User`**: Implements the upgrade logic.
4.  **`MigrateTo<UserV1> for User`**: Implements the downgrade logic (optional).

This setup allows `netabase_store` to automatically handle data migration when schemas change.
