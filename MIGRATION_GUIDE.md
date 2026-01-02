# Netabase Migration Guide

This guide explains how to use the migration system in netabase for versioned models.

## Overview

The migration system allows you to:
- Version your models with `#[netabase_version]` attributes
- Automatically migrate data from older versions to newer versions
- Preserve schema history in TOML files for P2P conflict resolution
- Use bincode with version-aware encoding/decoding

## Basic Usage

### Defining Versioned Models

Use the `#[netabase_version]` attribute to define versioned models:

```rust
#[netabase_definition]
mod users {
    use super::*;

    // Version 1 (legacy, kept for migration)
    #[derive(Debug, Clone, NetabaseModel, Encode, Decode)]
    #[netabase_version(family = "User", version = 1)]
    pub struct UserV1 {
        #[primary]
        pub id: u64,
        pub name: String,
    }

    // Version 2 (current)
    #[derive(Debug, Clone, NetabaseModel, Encode, Decode)]
    #[netabase_version(family = "User", version = 2, current)]
    pub struct User {
        #[primary]
        pub id: u64,
        pub name: String,
        pub email: String,  // Added field
    }
}
```

### Implementing Migration Traits

Implement the `From` trait or `MigrateFrom` trait to define how data migrates:

```rust
impl From<UserV1> for User {
    fn from(old: UserV1) -> Self {
        User {
            id: old.id,
            name: old.name,
            email: String::new(), // Default value for new field
        }
    }
}
```

The macros will automatically generate the `MigrateFrom` implementation based on your `From` impl.

## Version Attributes

The `#[netabase_version]` attribute supports these options:

| Option | Type | Required | Description |
|--------|------|----------|-------------|
| `family` | String | Yes | Groups versions of the same model together |
| `version` | u32 | Yes | Version number (must be unique within family) |
| `current` | flag | No | Marks this as the current version |
| `supports_downgrade` | flag | No | Allows encoding to this older format |

### Examples

```rust
// Basic version
#[netabase_version(family = "Product", version = 1)]

// Current version
#[netabase_version(family = "Product", version = 2, current)]

// Version that supports downgrade
#[netabase_version(family = "Product", version = 1, supports_downgrade)]
```

## Migration Chain

When you have multiple versions, the migration system chains `From` implementations:

```rust
// v1 -> v2 -> v3 (current)
impl From<UserV1> for UserV2 { ... }
impl From<UserV2> for User { ... }

// Reading v1 data automatically:
// 1. Decodes as UserV1
// 2. Converts UserV1 -> UserV2
// 3. Converts UserV2 -> User
```

## Wire Format

Versioned data uses a special header format:

```
+--------+--------+---------+----------+
| Magic  | Magic  | Version | Payload  |
| 'N'    | 'V'    | (u32)   | (bincode)|
+--------+--------+---------+----------+
  1 byte   1 byte   4 bytes   variable
```

- Magic bytes `NV` identify versioned data
- Version number allows correct deserialization
- Legacy data (without header) is handled gracefully

## Version Context

Use `VersionContext` for fine-grained control:

```rust
use netabase_store::traits::migration::{VersionContext, VersionedDecode};

let ctx = VersionContext {
    current_version: 2,
    min_supported_version: 1,
    auto_migrate: true,  // Automatically migrate older versions
    strict: false,       // Don't fail on version mismatch
};

let user = User::decode_versioned(&data, &ctx)?;
```

## Database Migration

To migrate an entire database to a new schema:

```rust
use netabase_store::databases::redb::{MigrationOptions, DatabaseMigrator};

let options = MigrationOptions {
    backup_before_migrate: true,
    dry_run: false,
    continue_on_error: false,
    batch_size: 1000,
};

let migrator = DatabaseMigrator::<MyDefinition>::new(database);
let result = migrator.migrate_all(&options)?;

println!("Migrated {} records", result.records_migrated);
if !result.errors.is_empty() {
    eprintln!("Encountered {} errors", result.errors.len());
}
```

## Schema Export with Version History

The TOML schema export includes version history:

```toml
[definition]
name = "users"
hash = "..."

[definition.version_history.User]
current_version = 2
versions = [
    { version = 1, model = "UserV1", added_fields = [], removed_fields = [] },
    { version = 2, model = "User", added_fields = ["email"], removed_fields = [] }
]
```

## P2P Schema Comparison

Use schema comparison for peer-to-peer conflict resolution:

```rust
use netabase_store::traits::migration::FamilyLineage;

let local_lineage = FamilyLineage::from_versions("User", &[1, 2, 3]);
let remote_lineage = FamilyLineage::from_versions("User", &[1, 2, 4]);

match local_lineage.relationship(&remote_lineage) {
    LineageRelationship::Same => println!("Same lineage"),
    LineageRelationship::Ancestor => println!("Local is ancestor"),
    LineageRelationship::Descendant => println!("Local is descendant"),
    LineageRelationship::Divergent { common_ancestor } => {
        println!("Diverged at version {}", common_ancestor);
    }
}
```

## Best Practices

1. **Always increment versions**: Never modify the structure of an existing version
2. **Keep old versions**: Don't remove old version structs until all data is migrated
3. **Test migrations**: Write tests that create old data and verify migration
4. **Add defaults carefully**: New fields need sensible default values
5. **Document changes**: Include comments explaining field additions/removals
6. **Backup before migration**: Use `backup_before_migrate: true` in production

## Error Handling

Migration errors are captured in `MigrationError`:

```rust
pub struct MigrationError {
    pub record_key: String,   // Which record failed
    pub error: String,        // What went wrong
    pub at_version: u32,      // At which version step
}
```

Handle errors gracefully:

```rust
match migrator.migrate_all(&options) {
    Ok(result) if result.errors.is_empty() => {
        println!("Migration successful!");
    }
    Ok(result) => {
        for error in &result.errors {
            eprintln!("Failed to migrate {}: {}", error.record_key, error.error);
        }
    }
    Err(e) => eprintln!("Migration failed: {}", e),
}
```
