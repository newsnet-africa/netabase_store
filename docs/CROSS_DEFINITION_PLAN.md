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

## Implementation Status

### Phase 1: Core Infrastructure ✅ **COMPLETED**
**Goal**: Establish manager traits and types (no TOML/macros yet)

**Status**: All core infrastructure is implemented and working.

**Completed Work**:
- ✅ `src/traits/permission/mod.rs` - Permission trait system
- ✅ `src/traits/manager/mod.rs` - Manager trait
- ✅ `src/traits/manager/store_link.rs` - DefinitionStoreLink enum
- ✅ `src/databases/manager/mod.rs` - Generic DefinitionManager with lazy loading
- ✅ `src/databases/redb_store/manager.rs` - RedbDefinitionManager implementation
- ✅ `src/databases/sled_store/manager.rs` - SledDefinitionManager implementation
- ✅ Manager transaction support (read/write transactions across definitions)
- ✅ Cross-definition access tracking and auto-unload
- ✅ Permissions system with hierarchical roles

**Testing**: ✅ Manual tests passing, manager integration tests exist

---

### Phase 2: Standardized Tree Naming ✅ **COMPLETED**
**Goal**: Implement and test the naming convention

**Status**: Tree naming convention implemented and standardized across the codebase.

**Completed Work**:
- ✅ Standardized format: `{Definition}::{Model}::{Type}::{Name}`
- ✅ Implementation in `src/codegen/toml_parser.rs` (see `generate_tree_names()`)
- ✅ Examples:
  - Main: `User::User::Main`
  - Secondary: `User::User::Secondary::Email`
  - Relational: `Product::Product::Relational::CreatedBy`
  - Subscription: `User::User::Subscription::UserEvents`
  - Hash: `User::User::Hash`

**Testing**: ✅ Unit tests for tree name generation passing

---

### Phase 3: TOML Schema Parser ✅ **COMPLETED**
**Goal**: Parse TOML schemas and validate

**Status**: Fully functional TOML parser with comprehensive types.

**Completed Work**:
- ✅ `src/codegen/toml_parser.rs` - Complete TOML parsing
- ✅ `src/codegen/toml_types.rs` - Serde types for all TOML structures
- ✅ `src/codegen/validator.rs` - Schema validation
- ✅ Support for:
  - Definition schemas (individual models)
  - Manager schemas (multi-definition managers)
  - Nested schema loading (`load_all_definition_schemas()`)
  - Tree name generation from schemas
- ✅ Example TOML files:
  - `ecommerce.root.netabase.toml` (manager)
  - `schemas/User.netabase.toml`
  - `schemas/Product.netabase.toml`
  - `schemas/Order.netabase.toml`

**Testing**: ✅ Unit tests for parsing and validation passing

---

### Phase 4: Code Generation 🟡 **PARTIALLY COMPLETE**
**Goal**: Generate Rust code from TOML schemas

**Status**: Basic generator exists but needs completion and integration.

**Completed Work**:
- ✅ `src/codegen/generator.rs` - Basic code generation framework
- ✅ `generate_definition_code()` - Generates complete definitions
- ✅ `generate_model()` - Model struct generation
- ✅ `generate_keys()` - Primary, secondary, and relational keys
- ✅ `generate_trait_implementations()` - Basic trait impls
- ✅ `generate_tree_manager()` - Tree name constants
- ✅ `generate_manager_code()` - Manager struct generation
- ✅ Unit tests for code generation

**Missing Work**:
- ❌ Complete trait implementations (currently generates minimal stubs)
- ❌ Backend-specific extensions (Redb/Sled trait implementations)
- ❌ Cross-definition link generation
- ❌ Subscription enum generation
- ❌ Type inference for key field types (currently hardcoded)
- ❌ derive macro attribute preservation
- ❌ Permission role generation from manager schema
- ❌ Format generated code properly (indentation, newlines)

**Next Steps**:
1. Enhance `generate_trait_implementations()` to match manual boilerplate
2. Add backend-specific trait generation
3. Add proper type inference from field types
4. Add code formatting/pretty-printing

---

### Phase 5: Macro Implementation 🟡 **PARTIALLY COMPLETE**
**Goal**: Provide procedural macros for users

**Status**: Manual definition macros work; TOML-based macros are stubbed.

**Completed Work**:
- ✅ `netabase_macros/` proc-macro crate exists
- ✅ `netabase_definition_module` macro - **FULLY WORKING**
  - Processes modules with `NetabaseModel` structs
  - Generates all boilerplate code
  - Reduces ~3846 lines to ~50 lines (94% reduction)
- ✅ `NetabaseModel` derive macro - **FULLY WORKING**
  - Generates keys, traits, implementations
- ✅ `netabase_definition_from_toml!()` - **STUBBED** (compile_error!)
- ✅ `netabase_manager_from_toml!()` - **STUBBED** (compile_error!)

**Missing Work**:
- ❌ Wire `netabase_definition_from_toml!()` to codegen module
- ❌ Wire `netabase_manager_from_toml!()` to codegen module
- ❌ Handle compile-time file reading and path resolution
- ❌ Generate proc_macro2::TokenStream from generated code strings
- ❌ Add proper error handling and diagnostics
- ❌ Add incremental compilation support (track TOML file changes)

**Technical Challenge**:
The macro crate (`netabase_macros`) is a proc-macro crate and cannot depend on the main crate (`netabase_store`) which contains the codegen module. Solutions:

1. **Option A**: Extract codegen into separate library crate
   - Create `netabase_codegen` as a normal library crate
   - Both `netabase_macros` and `netabase_store` depend on it
   - Clean separation of concerns

2. **Option B**: Duplicate codegen in macro crate
   - Copy generator logic into `netabase_macros`
   - Keep synchronized manually
   - Less clean but avoids extra crate

3. **Option C**: Generate code at build time
   - Use build.rs to pre-generate from TOML
   - Macros just include! the generated files
   - Different model but might be simpler

**Recommended**: Option A (separate codegen crate)

**Next Steps**:
1. Create `netabase_codegen` crate with all codegen modules
2. Update `netabase_macros` to use `netabase_codegen`
3. Implement TOML macro bodies using the generator
4. Add comprehensive integration tests

---

### Phase 6: TOML Auto-Generation (Future) ⏸️ **NOT STARTED**
**Goal**: Bidirectional - generate TOML from existing code

**Status**: Planned for future. Not critical for initial release.

**Planned Work**:
- `src/codegen/reverse_gen.rs` - Analyze code → generate TOML
- `bin/netabase_schema_gen.rs` - CLI tool for schema extraction
- Syn-based parsing of existing Rust code
- Extract model fields, keys, relationships
- Generate valid TOML schema files

**Use Case**: Migrate existing boilerplate.rs to TOML format

**Priority**: Low (can be added later)

---

### Phase 7: Documentation 🔴 **NOT STARTED**
**Goal**: Comprehensive guides and examples

**Status**: Minimal documentation exists. Needs comprehensive guides.

**Completed Work**:
- ✅ This plan document (CROSS_DEFINITION_PLAN.md)
- ✅ Inline code documentation (rustdoc)
- ✅ Basic examples (boilerplate.rs, ecommerce.rs)

**Missing Work**:
- ❌ `docs/TOML_SCHEMA.md` - Complete TOML schema reference
- ❌ `docs/TREE_NAMING.md` - Tree naming convention guide
- ❌ `docs/MACRO_USAGE.md` - How to use the macros
- ❌ `docs/CROSS_DEFINITION_ACCESS.md` - How to work across definitions
- ❌ `docs/PERMISSIONS.md` - Permission system guide
- ❌ `examples/macro_based/` - Full example using TOML macros
- ❌ Migration guide (manual → TOML)
- ❌ API reference documentation
- ❌ Tutorial series for new users

**Priority**: High for v1.0 release

---

## Current Capabilities Summary

### ✅ What Works Today

1. **Manual Macro-Based Definitions**
   ```rust
   #[netabase_definition_module(EcommerceDefinitions, EcommerceKeys)]
   pub mod ecommerce {
       #[derive(NetabaseModel)]
       pub struct User {
           #[primary_key]
           pub id: u64,
           #[secondary_key]
           pub email: String,
       }
   }
   // Generates ~3800 lines of boilerplate automatically
   ```

2. **Multi-Definition Managers**
   ```rust
   let manager = DefinitionManager::<
       EcommerceManager,
       EcommerceDefinitions,
       EcommercePermissions,
       RedbStore<EcommerceDefinitions>
   >::new("./data")?;
   ```

3. **Cross-Definition Transactions**
   - Read/write transactions across multiple definitions
   - Lazy loading of definition stores
   - Auto-unload of unused definitions
   - Transaction-level access tracking

4. **Permissions System**
   - Hierarchical permission roles
   - Compile-time permission checking
   - Per-definition access control

5. **Standardized Tree Naming**
   - Consistent naming across all definitions
   - Easy cross-definition tree lookup
   - Format: `{Def}::{Model}::{Type}::{Name}`

### 🟡 What's Partially Working

1. **TOML Schema Parsing**
   - Can parse TOML files ✅
   - Can validate schemas ✅
   - Can generate code strings ✅
   - Cannot use in proc macros yet ❌

2. **Code Generation**
   - Basic generation works ✅
   - Missing complete trait impls ❌
   - Missing backend extensions ❌
   - Missing type inference ❌

### ❌ What Doesn't Work Yet

1. **TOML-Based Macros**
   ```rust
   // This compiles but shows compile_error!
   netabase_definition_from_toml!("schemas/User.netabase.toml");
   ```

2. **Manager-Level TOML Generation**
   ```rust
   // This compiles but shows compile_error!
   netabase_manager_from_toml!("ecommerce.root.netabase.toml");
   ```

3. **Reverse Generation** (code → TOML)

---

## Immediate Next Steps (Priority Order)

### 1. Complete Code Generation (Phase 4) - **HIGH PRIORITY**
**Estimated Effort**: 2-3 days

Tasks:
- [ ] Enhance trait implementation generation
  - [ ] Complete NetabaseModelTrait impl
  - [ ] Generate proper SecondaryKeys enum with all variants
  - [ ] Generate RelationalKeys enum with all variants
  - [ ] Generate SubscriptionKeys enum
- [ ] Add type inference for key fields
  - [ ] Parse field types from TOML
  - [ ] Map Rust types correctly
  - [ ] Handle Option<T>, Vec<T>, etc.
- [ ] Generate backend-specific extensions
  - [ ] Redb trait implementations
  - [ ] Sled trait implementations
- [ ] Add code formatting
  - [ ] Use prettyplease or similar for formatting
  - [ ] Proper indentation and newlines
- [ ] Write comprehensive tests
  - [ ] Compare generated vs manual boilerplate
  - [ ] Test all TOML schema variants

### 2. Extract Codegen Into Separate Crate (Phase 5) - **HIGH PRIORITY**
**Estimated Effort**: 1-2 days

Tasks:
- [ ] Create `netabase_codegen` crate
  - [ ] Move all codegen modules
  - [ ] Update dependencies
  - [ ] Add proper exports
- [ ] Update `netabase_store` to use `netabase_codegen`
- [ ] Update `netabase_macros` to use `netabase_codegen`
- [ ] Verify all tests still pass

### 3. Implement TOML Macros (Phase 5) - **HIGH PRIORITY**
**Estimated Effort**: 2-3 days

Tasks:
- [ ] Implement `netabase_definition_from_toml!()`
  - [ ] Read TOML file at compile time
  - [ ] Parse using netabase_codegen
  - [ ] Generate TokenStream
  - [ ] Handle errors gracefully
- [ ] Implement `netabase_manager_from_toml!()`
  - [ ] Load manager schema
  - [ ] Load all definition schemas
  - [ ] Generate all definitions
  - [ ] Generate manager enum
  - [ ] Generate permission enum
- [ ] Add integration tests
  - [ ] Test with example TOML files
  - [ ] Verify generated code compiles
  - [ ] Test runtime behavior

### 4. Create Comprehensive Examples (Phase 7) - **MEDIUM PRIORITY**
**Estimated Effort**: 2-3 days

Tasks:
- [ ] Create `examples/toml_based_ecommerce/`
  - [ ] Full e-commerce example using TOML
  - [ ] User, Product, Order definitions
  - [ ] Cross-definition relationships
  - [ ] CRUD operations
  - [ ] Transaction examples
- [ ] Create `examples/permissions_demo/`
  - [ ] Demonstrate permission system
  - [ ] Multiple roles
  - [ ] Compile-time checks
- [ ] Create `examples/multi_backend/`
  - [ ] Same schema, different backends
  - [ ] Demonstrate backend abstraction

### 5. Write Documentation (Phase 7) - **MEDIUM PRIORITY**
**Estimated Effort**: 3-5 days

Priority order:
1. [ ] `docs/MACRO_USAGE.md` - **HIGHEST**
   - How to use netabase_definition_module
   - How to use TOML macros
   - Migration from manual to TOML
2. [ ] `docs/TOML_SCHEMA.md` - **HIGH**
   - Complete TOML reference
   - All field types
   - All configuration options
3. [ ] `docs/CROSS_DEFINITION_ACCESS.md` - **HIGH**
   - How managers work
   - Cross-definition transactions
   - Lazy loading behavior
4. [ ] `docs/PERMISSIONS.md` - **MEDIUM**
   - Permission system guide
   - Role hierarchies
   - Best practices
5. [ ] Tutorial series - **MEDIUM**
   - Getting started
   - Building your first app
   - Advanced features

### 6. Reverse Generation (Phase 6) - **LOW PRIORITY**
**Estimated Effort**: 3-5 days

This can wait until after v1.0 release.

---

## Technical Debt & Improvements

### Code Quality
- [ ] Add more comprehensive error messages
- [ ] Improve validation error reporting
- [ ] Add schema version compatibility checks
- [ ] Add migration tooling for schema changes

### Performance
- [ ] Benchmark code generation time
- [ ] Optimize TOML parsing
- [ ] Add caching for frequently accessed definitions
- [ ] Profile manager overhead

### Testing
- [ ] Add fuzzing tests for TOML parser
- [ ] Add property-based tests for code generation
- [ ] Add performance regression tests
- [ ] Add memory leak tests for managers

### Developer Experience
- [ ] Better compile error messages from macros
- [ ] IDE integration (rust-analyzer support)
- [ ] Schema validation in editors
- [ ] TOML schema autocomplete

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

## Release Roadmap

### v0.5.0 (Current State) ✅
**Status**: Released (internal)
- ✅ Core manager infrastructure
- ✅ Manual macro-based definitions (netabase_definition_module)
- ✅ Standardized tree naming
- ✅ TOML parsing and validation
- ✅ Permission system
- ✅ Cross-definition transactions
- ✅ Both Redb and Sled backends

### v0.6.0 (Next Release) 🎯
**Target**: Complete TOML-based code generation
**Estimated Timeline**: 1-2 weeks

**Goals**:
- ✅ Complete code generation from TOML schemas
- ✅ Extract codegen into separate crate
- ✅ Working `netabase_definition_from_toml!()` macro
- ✅ Working `netabase_manager_from_toml!()` macro
- ✅ Comprehensive integration tests

**Deliverables**:
- Users can generate complete definitions from TOML
- Manager generation from root schema works end-to-end
- Generated code matches manual boilerplate in functionality

### v0.7.0 (Polish Release)
**Target**: Examples and initial documentation
**Estimated Timeline**: 1-2 weeks

**Goals**:
- ✅ Multiple complete examples using TOML macros
- ✅ Basic documentation (MACRO_USAGE.md, TOML_SCHEMA.md)
- ✅ Migration guide from manual to TOML
- ✅ Performance benchmarks

**Deliverables**:
- 3-5 comprehensive examples
- Getting started guide
- API documentation complete

### v1.0.0 (Stable Release)
**Target**: Production-ready with full documentation
**Estimated Timeline**: 2-3 weeks after v0.7.0

**Goals**:
- ✅ Complete documentation suite
- ✅ Tutorial series
- ✅ Stability guarantees
- ✅ Migration tooling
- ✅ Performance optimizations

**Deliverables**:
- Production-ready system
- Comprehensive docs
- Full test coverage (>90%)
- Performance benchmarks
- SemVer guarantees

### v2.0.0 (Future)
**Target**: Advanced features
**Estimated Timeline**: TBD

**Potential Features**:
- Reverse generation (code → TOML)
- Schema migration system
- GraphQL integration
- WASM backend support
- Schema versioning and migrations
- Advanced query language

---

## Success Criteria

### Phase 4-5 Completion (v0.6.0)
- [ ] `netabase_definition_from_toml!()` generates working code
- [ ] `netabase_manager_from_toml!()` generates working managers
- [ ] Generated code compiles without warnings
- [ ] Generated code passes all existing tests
- [ ] Generated code matches manual boilerplate behavior
- [ ] Code generation time < 2 seconds for typical schemas
- [ ] Clear error messages for invalid TOML

### Phase 7 Completion (v0.7.0-v1.0.0)
- [ ] All documentation files created
- [ ] 5+ comprehensive examples
- [ ] Getting started tutorial
- [ ] API reference complete
- [ ] Migration guide exists
- [ ] All public APIs documented

### v1.0.0 Release Criteria
- [ ] Zero known critical bugs
- [ ] Full test coverage (>90%)
- [ ] Documentation complete
- [ ] Performance benchmarks published
- [ ] Breaking changes finalized
- [ ] Public API stable
- [ ] Community feedback incorporated

---

## Project Timeline Summary

```
Timeline:
├── [DONE] Phase 1: Core Infrastructure (2-3 weeks)
├── [DONE] Phase 2: Tree Naming (1 week)
├── [DONE] Phase 3: TOML Parser (1 week)
├── [50%] Phase 4: Code Generation (2-3 days remaining)
├── [30%] Phase 5: Macro Implementation (3-4 days remaining)
├── [0%] Phase 7: Documentation (1-2 weeks remaining)
└── [FUTURE] Phase 6: Reverse Generation

Current Status: ~60% complete
Next Milestone: v0.6.0 (TOML macro completion)
ETA to v1.0.0: ~4-6 weeks
```

---

## Contributing

### How to Help

**High Priority Tasks**:
1. Complete code generation in `src/codegen/generator.rs`
2. Extract codegen into `netabase_codegen` crate
3. Implement TOML macros in `netabase_macros/src/lib.rs`
4. Write integration tests for generated code
5. Create examples using TOML macros

**Medium Priority Tasks**:
1. Write documentation (MACRO_USAGE.md, TOML_SCHEMA.md)
2. Create tutorial series
3. Add more test coverage
4. Performance profiling and optimization

**Low Priority Tasks**:
1. Reverse generation tooling
2. IDE integration
3. Schema validation tooling
4. Migration helpers

---

## Notes

### Architectural Decisions

1. **Separate Codegen Crate**: Decided on Option A (separate `netabase_codegen` crate) to avoid circular dependencies between proc-macro crate and main crate.

2. **Standardized Tree Naming**: Chose `{Def}::{Model}::{Type}::{Name}` format for predictability and to avoid collisions in multi-definition scenarios.

3. **Lazy Loading**: Definition stores are loaded on-demand to minimize memory usage for large applications with many definitions.

4. **Compile-Time Permissions**: Permissions are enforced at compile-time using const generics to prevent runtime access violations.

5. **Backend Abstraction**: Maintained clean abstraction between store logic and backend implementations to support multiple backends (Redb, Sled, future IndexedDB).

### Lessons Learned

1. **Macro Limitations**: Proc-macro crates have strict dependency limitations. Extracting shared code into separate library crate is essential.

2. **Code Generation Complexity**: Generating complete, properly-formatted Rust code is more complex than initially anticipated. Need proper quote! usage and formatting tools.

3. **Tree Naming**: Consistent naming convention critical for cross-definition access. Worth investing effort upfront.

4. **Testing Strategy**: Need both unit tests (for individual functions) and integration tests (for generated code compilation and runtime behavior).

---

**Status**: Implementation in progress - 60% complete
**Last Updated**: 2025-12-11
**Next Review**: After v0.6.0 completion
