# Example Tests

This directory contains integration tests for the example crate, demonstrating and validating various features of `netabase_store`.

## Test Organization

### Sequential Tests

Some tests depend on artifacts created by other tests and must run sequentially:

```bash
# Run sequential tests (requires --test-threads=1)
cargo test --features sequential-tests -- --test-threads=1
```

**Sequential test dependencies:**

1. **`schema_export.rs`** → Creates TOML files
   - `definition_roundtrip_schema.toml`
   - `definition_2_roundtrip_schema.toml`

2. **`schema_import.rs`** → Reads TOML files created by `schema_export`
   - Currently disabled due to orphan rule issues
   - See comments in file for details

### Independent Tests

These tests can run in parallel (default test mode):

- **`macro_test.rs`** - Validates macro expansion and generated code
- **`migration_logic.rs`** - Tests migration trait implementations
- **`schema_migration_export.rs`** - Validates migration metadata export
- **`content_addressed_test.rs`** - Tests content-addressed blob storage

## Running Tests

```bash
# Run all non-sequential tests (default)
cargo test

# Run all tests including sequential ones
cargo test --features sequential-tests -- --test-threads=1

# Run specific test file
cargo test --test macro_test

# Run with output
cargo test -- --nocapture
```

## Test Naming Convention

Sequential tests are prefixed with numbers to indicate execution order:
- `0_schema_export.rs`
- `1_schema_import.rs`

This ensures alphabetical ordering matches dependency order.
