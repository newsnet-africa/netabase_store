# Integration Tests

This directory contains integration tests for `netabase_store`. Tests are organized
by feature area and follow Rust naming conventions.

## Test Categories

### Core CRUD Operations
- `integration_crud.rs` - Basic Create, Read, Update, Delete operations
- `integration_list.rs` - List/query operations with pagination
- `integration_indexes.rs` - Secondary index queries

### Blob Storage
- `blob_complex_types.rs` - Complex blob type handling
- `blob_mixed_content.rs` - Mixed blob/non-blob content
- `blob_query_methods.rs` - Blob-specific query methods

### Subscriptions
- `selective_subscriptions.rs` - Subscription-based filtering
- `debug_subscription.rs` - Subscription debugging utilities

### Migration
- `migration_comprehensive.rs` - Full migration test suite
- `migration_doctests.rs` - Migration documentation tests

### Repository
- `repository_comprehensive.rs` - Repository pattern tests
- `repository_toml_generation.rs` - TOML schema generation
- `nested_repository_pattern.rs` - Nested repository support

### Relational
- `relational_performance.rs` - Relational link performance

### Comprehensive Suites
- `comprehensive_functionality.rs` - Full API coverage (51K lines)
- `comprehensive_table_tests.rs` - Table-level operations
- `database_comprehensive.rs` - Database-level operations
- `all_tables_comprehensive.rs` - All table types tested

### Analysis & Debugging
- `storage_size_analysis.rs` - Storage space analysis
- `detailed_size_analysis.rs` - Detailed size breakdown
- `database_growth_analysis.rs` - Database growth patterns

### Definition Tests
- `definition_iter_test.rs` - Definition iteration
- `definition_record_iter_test.rs` - Record iteration

### Other
- `readme_examples.rs` - Examples from README
- `readme_accuracy.rs` - README code accuracy verification
- `auxiliary_query.rs` - Auxiliary query methods
- `secondary_key_query.rs` - Secondary key querying
- `immutable_model_hash.rs` - Content-addressed hashing
- `libp2p_schema_test.rs` - libp2p integration tests

## Shared Test Infrastructure

The `common/` directory contains shared test utilities:

- `mod.rs` - Main test utilities module
  - `create_test_db()` - Create temporary test database
  - `cleanup_test_db()` - Clean up test database
  - `unique_id()` - Generate unique test identifiers
  - `random_bytes()` / `random_string()` - Test data generators

### Shared Test Models

The `common/mod.rs` also defines reusable test models:

- `SimpleDef` - Basic models (User, Item) for CRUD tests
- `IndexedDef` - Models with secondary keys (Product, Document)

## Running Tests

```bash
# Run all tests
cargo test -p netabase_store

# Run specific test file
cargo test --test integration_crud

# Run with verbose output
cargo test -p netabase_store -- --nocapture

# Run tests matching a pattern
cargo test -p netabase_store blob
```

## Test Dependencies

Tests use the `example` crate which contains comprehensive model definitions.
See `example/src/lib.rs` for the test model schemas.
