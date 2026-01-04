# Repository TOML Generation

This document explains how to generate and use `repository.toml` files for database replication and schema management.

## Overview

The `repository.toml` file is a complete description of your database schema, including:
- Repository name and version
- All definitions in the repository
- All models in each definition
- Model fields with their types and constraints
- Version history for migrations
- Relational links between models

This file can be used to:
1. **Replicate databases** - Generate identical schema on another system
2. **Version control** - Track schema changes over time
3. **P2P sync** - Compare schemas between distributed nodes
4. **Code generation** - Generate Rust code from schema files

## Generating Repository.toml

### Method 1: From Code (Recommended)

Use the `schema_toml()` method on your repository:

```rust
use netabase_store_examples::MainRepository;

fn main() {
    // Generate TOML string
    let toml = MainRepository::schema_toml();
    println!("{}", toml);
    
    // Or write directly to file
    MainRepository::write_schema_toml("repository.toml").unwrap();
}
```

### Method 2: Using the Schema Generator Binary

```bash
cd boilerplate
cargo run --bin generate_schemas
```

This generates schema files for all repositories in `generated_schemas/`:
- `main_repository.toml`
- `employee_repository.toml` 
- `manager_repository.toml`

## Repository Pattern: External Definitions (Recommended)

For repository.toml generation to work properly, use the **external definitions pattern** with the `definitions()` attribute:

```rust
// Define your definitions FIRST
#[netabase_definition(Definition)]
pub mod definition {
    #[derive(NetabaseModel)]
    pub struct User {
        #[primary_key]
        pub id: String,
        pub name: String,
    }
}

#[netabase_definition(DefinitionTwo)]
pub mod definition_two {
    #[derive(NetabaseModel)]
    pub struct Category {
        #[primary_key]
        pub id: String,
        pub name: String,
    }
}

// Then create repository with external references
#[netabase_repository(MainRepository, definitions(Definition, DefinitionTwo))]
pub mod main_repository {}
```

This pattern ensures:
- ✅ Complete schema export
- ✅ Proper TOML generation
- ✅ All models included
- ✅ Version history preserved

## Repository Structure

The definitions can live in the same module:

```rust
// Single file with all definitions
#[netabase_definition(Def1)]
pub mod def1 { /* models */ }

#[netabase_definition(Def2)]
pub mod def2 { /* models */ }

#[netabase_repository(MyRepo, definitions(Def1, Def2))]
pub mod my_repo {}
```

Or across multiple files:

```rust
// lib.rs
pub mod users;
pub mod posts;

#[netabase_repository(MyRepo, definitions(users::UserDef, posts::PostDef))]
pub mod my_repo {}

// users.rs
#[netabase_definition(UserDef)]
pub mod user_def { /* User model */ }

// posts.rs
#[netabase_definition(PostDef)]
pub mod post_def { /* Post model */ }
```

## Schema Contents

A generated `repository.toml` includes:

```toml
schema_format_version = 2
name = "MainRepository"

[[definitions]]
schema_format_version = 2
name = "Definition"
subscriptions = ["Topic1", "Topic2"]

[[definitions.models]]
name = "User"
is_current = true
family = "User"
version = 2

[[definitions.models.fields]]
name = "id"
type_name = "String"
kind = "Primary"

[[definitions.models.fields]]
name = "partner"
type_name = "String"
kind = "Relational"

[definitions.models.fields.details]
definition = "Definition"
model = "User"

# Version history for migrations
[[definitions.model_history]]
family = "User"
current_version = 2

[[definitions.model_history.versions]]
version = 1
struct_name = "UserV1"
version_hash = "9774471415192068689"
supports_upgrade = true
# ... fields at this version

[[definitions.model_history.versions]]
version = 2
struct_name = "User"
version_hash = "5563019169600077249"
supports_downgrade = true
# ... fields at this version
```

## Using Repository.toml

### Schema Hashing

Compare schemas between systems:

```rust
use netabase_store::traits::database::hash::FastHash;

let local_hash = MainRepository::schema_hash::<FastHash>();
let remote_hash = fetch_remote_schema_hash();

if MainRepository::schemas_match::<FastHash>(remote_hash) {
    println!("Schemas match!");
} else {
    println!("Schema mismatch - migration needed");
}
```

### Loading from TOML

```rust
use netabase_store::traits::registery::definition::schema::RepositorySchema;
use std::fs;

let toml_content = fs::read_to_string("repository.toml")?;
let schema: RepositorySchema = toml::from_str(&toml_content)?;

println!("Repository: {}", schema.name);
println!("Definitions: {}", schema.definitions.len());

for def in &schema.definitions {
    println!("  - {}: {} models", def.name, def.models.len());
}
```

## Schema Versioning

The schema includes version history for all model families:

```toml
[[definitions.model_history]]
family = "Post"
current_version = 2

[[definitions.model_history.versions]]
version = 1
struct_name = "PostV1"
version_hash = "18306954760968958270"
supports_upgrade = false

[[definitions.model_history.versions]]
version = 2
struct_name = "Post"
version_hash = "12940536922446132280"
supports_upgrade = true
supports_downgrade = true

[[definitions.model_history.migration_paths]]
from_version = 1
to_version = 2
may_lose_data = false
```

This enables:
- **Automatic migration** between versions
- **P2P compatibility** checking
- **Schema evolution** tracking

## Best Practices

1. **Version Control**: Commit `repository.toml` to git
2. **Regenerate on Changes**: Run generator after schema changes
3. **Compare Before Deploy**: Check schema_hash before deployment
4. **Document Migrations**: Add migration notes for breaking changes
5. **Use External Pattern**: Always use `definitions()` for reliable generation

## Troubleshooting

### Empty definitions array

**Problem**: `repository.toml` shows `definitions = []`

**Solution**: Use the `definitions()` attribute pattern:

```rust
// ❌ Nested pattern (doesn't generate full schema)
#[netabase_repository(MyRepo)]
pub mod my_repo {
    #[netabase_definition(Def, repos(MyRepo))]
    pub mod def { /* ... */ }
}

// ✅ External pattern (generates complete schema)
#[netabase_definition(Def)]
pub mod def { /* ... */ }

#[netabase_repository(MyRepo, definitions(Def))]
pub mod my_repo {}
```

### Schema hash mismatch

**Problem**: Hashes don't match between systems

**Cause**: Usually due to:
- Different schema versions
- Field order differences  
- Missing models

**Solution**: Compare TOML files directly to identify differences

## Examples

See:
- `boilerplate/src/boilerplate_lib/mod.rs` - External definitions pattern
- `boilerplate/src/bin/generate_schemas.rs` - Schema generator
- `tests/repository_toml_generation.rs` - Comprehensive tests
- `generated_schemas/` - Example output files
