# Cross-Definition Access System - Implementation Plan

## Overview

Implement a multi-definition store manager with:
- **Isolated definition stores**: Each definition in its own database at `<parent>/<definition_name>/store.db`
- **TOML-based schema definition**: User writes TOML → Macros generate boilerplate code
- **Central manager**: Coordinates access across definitions with lazy loading
- **Compile-time permissions**: Type-safe permission enforcement
- **Standardized tree naming**: Consistent naming across all definitions for easy location

## Architecture Summary

### Current State
- User writes extensive boilerplate code (see `examples/boilerplate.rs` - 3846 lines)
- Single path → single database
- `RedbStore<D>` / `SledStore<D>` wraps backend
- `RelationalLink<M, D>` provides Unloaded/Loaded pattern

### Target State
- User writes TOML schema → Macros generate boilerplate
- Parent path → multiple definition databases
- `DefinitionManager<R, D, P>` coordinates multi-definition access
- `DefinitionStoreLink<D, B>` for lazy store loading
- Permissions checked at compile-time

## TOML Schema Design

### Definition TOML Structure
**Location**: User provides in their project, e.g., `schemas/User.netabase.toml`

```toml
[definition]
name = "User"
version = "1"

# The actual model fields - user-defined
[model]
fields = [
    { name = "id", type = "u64" },
    { name = "email", type = "String" },
    { name = "name", type = "String" },
    { name = "age", type = "u32" },
]

# Which field is the primary key
[keys.primary]
field = "id"
key_type = "UserId"  # Optional: custom type name, defaults to {Model}Id

# Secondary keys (indexes)
[[keys.secondary]]
name = "Email"
field = "email"
unique = true

[[keys.secondary]]
name = "Name"
field = "name"
unique = false

[[keys.secondary]]
name = "Age"
field = "age"
unique = false

# Relational keys (foreign keys to other models)
[[keys.relational]]
name = "CreatedProducts"
target_definition = "Product"  # Can be in different definition
target_model = "Product"
target_key_type = "ProductId"

# Subscriptions (event streams)
[[subscriptions]]
name = "Updates"
description = "All user updates"

[[subscriptions]]
name = "NewUsers"
description = "Newly created users"

# Permissions - which other definitions can access this
[permissions]
can_reference_from = ["Product", "Review"]  # Other definitions that can FK to this
can_reference_to = ["Product"]             # Definitions this can FK to

# Metadata (auto-generated, not user-written)
[metadata]
generated_at = "2025-12-10T00:00:00Z"
schema_hash = "blake3:..."
```

### Root Manager TOML
**Location**: User provides at project root, e.g., `restaurant.root.netabase.toml`

```toml
[manager]
name = "RestaurantManager"
version = "1"
root_path = "./data"  # Where definition databases will be stored

# All definitions in this manager
[[definitions]]
name = "User"
schema_file = "schemas/User.netabase.toml"

[[definitions]]
name = "Product"
schema_file = "schemas/Product.netabase.toml"

[[definitions]]
name = "Review"
schema_file = "schemas/Review.netabase.toml"

# Permission roles
[[permissions.roles]]
name = "Manager"
level = "ReadWrite"
definitions = ["User", "Product", "Review"]

[[permissions.roles]]
name = "Waiter"
read = ["Product"]
write = ["Review"]

[[permissions.roles]]
name = "Customer"
read = ["Product"]
```

### Macro-Generated Code Structure

From the TOML files above, macros generate:

```rust
// From User.netabase.toml
#[derive(Debug, Clone, Encode, Decode)]
pub struct User {
    pub id: u64,
    pub email: String,
    pub name: String,
    pub age: u32,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct UserId(pub u64);

#[derive(Debug, Clone, EnumDiscriminants, Encode, Decode)]
#[strum_discriminants(name(UserSecondaryKeysDiscriminants))]
pub enum UserSecondaryKeys {
    Email(UserEmail),
    Name(UserName),
    Age(UserAge),
}

#[derive(Debug, Clone, EnumDiscriminants, Encode, Decode)]
#[strum_discriminants(name(UserRelationalKeysDiscriminants))]
pub enum UserRelationalKeys {
    CreatedProducts(ProductId),
}

// ... all the trait implementations
```

## Standardized Tree Naming Convention

To enable easy cross-definition tree location, establish a consistent naming pattern:

### Format
```
{DefinitionName}::{ModelName}::{TreeType}::{TreeName}
```

### Examples
```
User::User::Main                       // Primary key tree
User::User::Secondary::Email           // Email secondary index
User::User::Secondary::Name            // Name secondary index
User::User::Relational::CreatedProducts // Relational link to Products
User::User::Subscription::Updates       // Updates subscription
User::User::Hash                        // Hash tree

Product::Product::Main
Product::Product::Secondary::Name
Product::Product::Relational::CreatedBy
Product::Product::Subscription::New
```

### Benefits
1. **Namespace isolation**: `User::` prefix prevents collision with `UserProfile::User::`
2. **Predictable**: Given definition + model + type, tree name is deterministic
3. **Cross-definition lookup**: `{OtherDef}::{Model}::Main` is always the primary tree
4. **Easy parsing**: Split by `::` to extract components
5. **Supports multi-model definitions**: Each model namespaced within definition

### Implementation in TreeManager
```rust
impl TreeManager<RestaurantDefinitions> for RestaurantDefinitions {
    fn get_tree_name(model_discriminant: &RestaurantDefinitionsDiscriminants) -> Option<String> {
        Some(format!(
            "{definition}::{model}::Main",
            definition = Self::definition_name(model_discriminant),
            model = model_discriminant.name()
        ))
    }

    fn get_secondary_tree_names(model_discriminant: &RestaurantDefinitionsDiscriminants) -> Vec<String> {
        // From TOML metadata
        let secondary_keys = Self::secondary_key_names(model_discriminant);
        secondary_keys.iter().map(|key| {
            format!(
                "{definition}::{model}::Secondary::{key}",
                definition = Self::definition_name(model_discriminant),
                model = model_discriminant.name(),
                key = key
            )
        }).collect()
    }

    fn get_relational_tree_names(model_discriminant: &RestaurantDefinitionsDiscriminants) -> Vec<String> {
        let relational_keys = Self::relational_key_names(model_discriminant);
        relational_keys.iter().map(|key| {
            format!(
                "{definition}::{model}::Relational::{key}",
                definition = Self::definition_name(model_discriminant),
                model = model_discriminant.name(),
                key = key
            )
        }).collect()
    }
}
```

## Macro Design

### Phase 1: Simple Macro (Immediate)
```rust
// User writes:
netabase_definition! {
    from_toml = "schemas/User.netabase.toml"
}

// Macro generates all the boilerplate from examples/boilerplate.rs
```

### Phase 2: Manager Macro (After core implementation)
```rust
// User writes:
netabase_manager! {
    from_toml = "restaurant.root.netabase.toml"
}

// Macro generates:
// - All definitions (by reading their schema files)
// - Manager enum
// - Permission enum
// - All trait implementations
```

### Macro Responsibilities

1. **Parse TOML**: Read schema files
2. **Generate structs**: Models, keys, enums
3. **Generate traits**: All NetabaseModelTrait implementations
4. **Generate TreeManager**: With standardized naming
5. **Generate ModelAssociatedTypes**: Unified enum
6. **Generate extension traits**: Redb/Sled specific implementations
7. **Validate**: Check for:
   - Circular relational references
   - Invalid foreign key targets
   - Permission conflicts

## Implementation Phases

### Phase 1: Core Infrastructure
**Goal**: Establish manager traits and types (no TOML/macros yet)

**New Files**:
- `src/traits/permission/mod.rs` - Permission trait system
- `src/traits/manager/mod.rs` - Manager trait
- `src/traits/manager/store_link.rs` - DefinitionStoreLink enum
- `src/databases/manager/mod.rs` - Generic DefinitionManager
- `src/databases/redb_store/manager.rs` - RedbDefinitionManager

**Modified Files**:
- `src/traits/definition/mod.rs` - Add `type Permissions` (with default)

**Testing**: Manual tests with existing boilerplate.rs definitions

### Phase 2: Standardized Tree Naming
**Goal**: Implement and test the naming convention

**Modified Files**:
- `src/traits/store/tree_manager.rs` - Update trait methods for new format
- `examples/boilerplate.rs` - Update to use new naming pattern

**New Files**:
- `src/traits/store/tree_naming.rs` - Helper functions for tree name generation

**Testing**:
- Verify all tree names follow `{Def}::{Model}::{Type}::{Name}` format
- Test cross-definition tree lookup

### Phase 3: TOML Schema Parser
**Goal**: Parse TOML schemas and validate

**New Files**:
- `src/codegen/toml_parser.rs` - Parse TOML into intermediate representation
- `src/codegen/toml_types.rs` - Serde types for TOML structure
- `src/codegen/validator.rs` - Validate schema consistency

**Dependencies** (add to Cargo.toml):
```toml
[dependencies]
toml = "0.8"
serde = { version = "1.0", features = ["derive"] }
```

**Testing**: Parse example TOML files, validate against schema

### Phase 4: Code Generation
**Goal**: Generate Rust code from TOML schemas

**New Files**:
- `src/codegen/generator.rs` - Core code generation logic
- `src/codegen/model_gen.rs` - Generate model structs
- `src/codegen/key_gen.rs` - Generate key types and enums
- `src/codegen/trait_gen.rs` - Generate trait implementations
- `src/codegen/tree_gen.rs` - Generate TreeManager impl

**Testing**: Generate code from TOML, compare with manual boilerplate

### Phase 5: Macro Implementation
**Goal**: Provide procedural macros for users

**New Crate**: `netabase_macros/` (proc-macro crate)
- `netabase_macros/src/lib.rs` - Macro entry points
- `netabase_macros/src/definition.rs` - `netabase_definition!` macro
- `netabase_macros/src/manager.rs` - `netabase_manager!` macro

**Integration**:
- Main crate re-exports macros
- Macros use codegen modules from Phase 4

**Testing**: Integration tests with user-facing API

### Phase 6: TOML Auto-Generation (Future)
**Goal**: Bidirectional - generate TOML from existing code

**New Files**:
- `src/codegen/reverse_gen.rs` - Analyze code → generate TOML
- `bin/netabase_schema_gen.rs` - CLI tool for schema extraction

**Use Case**: Migrate existing boilerplate.rs to TOML format

### Phase 7: Documentation
**Goal**: Comprehensive guides and examples

**New Files**:
- `docs/TOML_SCHEMA.md` - Complete TOML schema reference
- `docs/TREE_NAMING.md` - Tree naming convention guide
- `docs/MACRO_USAGE.md` - How to use the macros
- `examples/macro_based/` - Full example using macros

## TOML Schema Reference

### Field Types (Supported in model.fields)

```toml
# Primitive types
{ name = "id", type = "u8" }
{ name = "id", type = "u16" }
{ name = "id", type = "u32" }
{ name = "id", type = "u64" }
{ name = "id", type = "u128" }
{ name = "id", type = "i8" }
{ name = "id", type = "i16" }
{ name = "id", type = "i32" }
{ name = "id", type = "i64" }
{ name = "id", type = "i128" }
{ name = "price", type = "f32" }
{ name = "price", type = "f64" }
{ name = "active", type = "bool" }

# String types
{ name = "name", type = "String" }
{ name = "data", type = "&str" }  # Requires lifetime annotations

# Collections
{ name = "tags", type = "Vec<String>" }
{ name = "scores", type = "Vec<u32>" }
{ name = "map", type = "HashMap<String, u32>" }

# Option types
{ name = "middle_name", type = "Option<String>" }

# Custom types (must be in scope)
{ name = "created_at", type = "chrono::DateTime<chrono::Utc>" }
{ name = "id", type = "uuid::Uuid" }
```

### Key Configuration Options

```toml
# Primary key
[keys.primary]
field = "id"                    # Required: which field
key_type = "UserId"            # Optional: custom wrapper type
derive = ["Ord", "PartialOrd"] # Optional: additional derives

# Secondary key
[[keys.secondary]]
name = "Email"          # Required: discriminant name
field = "email"         # Required: which field
unique = true          # Required: uniqueness constraint
key_type = "UserEmail" # Optional: custom type
derive = []            # Optional: additional derives

# Relational key
[[keys.relational]]
name = "CreatedBy"              # Required: discriminant name
target_definition = "UserDef"   # Required: definition containing target
target_model = "User"           # Required: model name
target_key_type = "UserId"      # Required: foreign key type
on_delete = "Cascade"           # Optional: delete behavior (future)
```

### Permission Levels

```toml
[[permissions.roles]]
name = "Admin"
level = "ReadWrite"  # Options: "None", "Read", "Write", "ReadWrite"
definitions = ["*"]  # "*" means all definitions

[[permissions.roles]]
name = "Limited"
read = ["ModelA", "ModelB"]
write = ["ModelC"]
definitions = []  # Empty if using read/write lists
```

## Example: Restaurant Schema

### File: `schemas/User.netabase.toml`
```toml
[definition]
name = "User"

[model]
fields = [
    { name = "id", type = "u64" },
    { name = "email", type = "String" },
    { name = "name", type = "String" },
]

[keys.primary]
field = "id"

[[keys.secondary]]
name = "Email"
field = "email"
unique = true

[[subscriptions]]
name = "Updates"
```

### File: `schemas/Product.netabase.toml`
```toml
[definition]
name = "Product"

[model]
fields = [
    { name = "id", type = "u64" },
    { name = "name", type = "String" },
    { name = "price", type = "f64" },
]

[keys.primary]
field = "id"

[[keys.relational]]
name = "CreatedBy"
target_definition = "User"
target_model = "User"
target_key_type = "UserId"
```

### File: `restaurant.root.netabase.toml`
```toml
[manager]
name = "Restaurant"
root_path = "./data"

[[definitions]]
name = "User"
schema_file = "schemas/User.netabase.toml"

[[definitions]]
name = "Product"
schema_file = "schemas/Product.netabase.toml"

[[permissions.roles]]
name = "Manager"
level = "ReadWrite"
definitions = ["*"]
```

### Usage in Code
```rust
// Import the macro
use netabase_store::netabase_manager;

// Generate all code from TOML
netabase_manager! {
    from_toml = "restaurant.root.netabase.toml"
}

// Now use the generated types
fn main() -> NetabaseResult<()> {
    let mut manager = RestaurantManager::new("./data")?;

    let perm = RestaurantPermissions::Manager;

    manager.write(perm, |txn| {
        let user = User {
            id: 1,
            email: "test@example.com".to_string(),
            name: "Alice".to_string(),
        };

        let user_txn = txn.definition_txn_mut::<User, true>(
            &RestaurantDefinitionsDiscriminants::User
        )?;
        user_txn.put(user)?;

        Ok(())
    })?;

    Ok(())
}
```

## Tree Naming Quick Reference

| Tree Type | Format | Example |
|-----------|--------|---------|
| Main (Primary Key) | `{Def}::{Model}::Main` | `User::User::Main` |
| Secondary Index | `{Def}::{Model}::Secondary::{KeyName}` | `User::User::Secondary::Email` |
| Relational Link | `{Def}::{Model}::Relational::{LinkName}` | `Product::Product::Relational::CreatedBy` |
| Subscription | `{Def}::{Model}::Subscription::{SubName}` | `User::User::Subscription::Updates` |
| Hash Tree | `{Def}::{Model}::Hash` | `User::User::Hash` |

## Migration Path

### For Existing Code (examples/boilerplate.rs)

1. **Extract schema**: Run CLI tool to generate TOML from existing code
   ```bash
   netabase extract-schema examples/boilerplate.rs > definitions.root.netabase.toml
   ```

2. **Review and edit**: Manually review generated TOML, make adjustments

3. **Replace boilerplate**: Delete manual code, use macro
   ```rust
   netabase_manager! {
       from_toml = "definitions.root.netabase.toml"
   }
   ```

4. **Verify**: Run tests to ensure behavior unchanged

## Success Criteria

- ✅ TOML schemas fully define models, keys, and permissions
- ✅ Macros generate all boilerplate code
- ✅ Tree naming is consistent and predictable
- ✅ Cross-definition tree lookup works
- ✅ Generated code matches manual boilerplate behavior
- ✅ Compile-time permission enforcement
- ✅ Lazy loading of definition stores
- ✅ Backward compatible with existing API

## Next Steps

1. **Phase 1**: Implement core manager infrastructure (no macros)
2. **Phase 2**: Standardize tree naming across codebase
3. **Phase 3**: Build TOML parser and validator
4. **Phase 4**: Implement code generator
5. **Phase 5**: Create procedural macros
6. **Phase 6**: Add reverse generation (code → TOML)
7. **Phase 7**: Document everything

---

**Status**: Planning complete, ready for implementation
**Last Updated**: 2025-12-10
