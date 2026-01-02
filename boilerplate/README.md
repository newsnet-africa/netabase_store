# Netabase Store Boilerplate Examples

This directory contains boilerplate examples demonstrating the netabase_store library.

## Structure

### `boilerplate_lib/`
Manual implementation of models and definitions. This is the **production version** used for benchmarks.

**Contains:**
- `models/user.rs` - User model with blobs and relational links
- `models/post.rs` - Post model
- `models/heavy.rs` - Heavy model for stress testing
- `mod.rs` - Definition and DefinitionTwo with Category model

**Definitions:**
- **Definition**: Contains User, Post, and HeavyModel
- **DefinitionTwo**: Contains Category

### `boilerplate_lib_macros/`
Macro-based implementation demonstrating the netabase macro system.

**Status**: Demonstrates successful macro code generation for simple cases. Complex cross-definition references with blobs require additional trait resolution work.

See `MACRO_SYSTEM.md` in the project root for full macro documentation.

## Benchmarks

### CRUD Benchmark (`benches/crud.rs`)
Tests basic create, read, update, delete operations across all models.

Run with:
```bash
cargo bench --bench crud
```

### Stress Test (`benches/stress.rs`)
High-volume operations testing database performance under load.

Run with:
```bash
cargo bench --bench stress
```

## Models Overview

### User
- **Primary Key**: id (String)
- **Secondary Keys**: name, age
- **Relational Links**:
  - partner → Definition::User (self-referential)
  - category → DefinitionTwo::Category (cross-definition)
- **Blobs**: bio (LargeUserFile), another (AnotherLargeUserFile)
- **Subscriptions**: Topic1, Topic2

### Post
- **Primary Key**: id (String)
- **Secondary Keys**: title, author_id
- **Subscriptions**: Topic3, Topic4

### HeavyModel
- **Primary Key**: id (String)
- **Secondary Keys**: field1, field2, field3
- **Blobs**: heavy_blob (HeavyBlob)
- **Subscriptions**: Topic1, Topic2, Topic3, Topic4

### Category (DefinitionTwo)
- **Primary Key**: id (String)
- **Secondary Keys**: name
- **Subscriptions**: General

## Testing

Run all tests:
```bash
cargo test -p netabase_store_examples
```

Run benchmarks:
```bash
cargo bench -p netabase_store_examples
```

## Notes

- The boilerplate demonstrates all netabase features: primary/secondary keys, relational links, blobs, and subscriptions
- Cross-definition links are supported (e.g., User → Category across Definition boundaries)
- The manual implementation serves as the reference for what macros should generate
