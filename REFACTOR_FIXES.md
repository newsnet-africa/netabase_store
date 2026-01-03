# Refactor Fixes - Implementation Guide

## 1. Replace Deprecated API Calls

### Status: PARTIALLY COMPLETE
The following files still need deprecated API calls replaced:

**Files Fixed:**
- ✅ `tests/integration_list.rs` - create_redb → create (2 locations)
- ✅ `tests/integration_crud.rs` - Most instances fixed
- ✅ `tests/comprehensive_functionality.rs` - Some instances fixed

**Remaining Work:**
Search and replace in all test files:
- `create_redb` → `create`
- `update_redb` → `update` 
- `delete_redb` → `delete`

Use this command to find remaining instances:
```bash
grep -r "create_redb\|update_redb\|delete_redb" tests/ boilerplate/tests/
```

## 2. Configure Test Ordering

### Boilerplate Tests
In `boilerplate/Cargo.toml`, add a test harness configuration to ensure schema_export runs before schema_import:

```toml
[[test]]
name = "schema_export"
path = "tests/schema_export.rs"

[[test]]
name = "schema_import"
path = "tests/schema_import.rs"
harness = true
```

Tests run in alphabetical order by default, so renaming helps:
- `schema_export.rs` → `00_schema_export.rs`
- `schema_import.rs` → `01_schema_import.rs`

Alternative: Use test dependencies with `#[test]` attributes:
```rust
// In schema_import.rs
#[test]
#[cfg_attr(not(feature = "schema_export_run"), ignore)]
fn test_definition_roundtrip() {
    // Test code...
}
```

### Main Tests
Same approach for main workspace tests if needed.

## 3. Use QueryConfig Properly

### Current State
There are TWO QueryConfig implementations:
1. `src/query.rs` - Full-featured with range, pagination, fetch options
2. `src/databases/redb/transaction/options.rs` - Simplified version

### Fix Required
The deprecated `QueryConfig` in `src/databases/mod.rs` and `src/databases/redb/transaction/options.rs` should be removed or marked deprecated.

All code should use `src/query.rs::QueryConfig<R>` which provides:
- Range support (full, bounded, unbounded)
- Pagination (limit/offset)
- Fetch options (blobs, hydration depth)
- Count-only mode
- Reversal

### Example Usage
```rust
use netabase_store::query::QueryConfig;

// Fetch all with blobs
let config = QueryConfig::default();

// Paginated query
let config = QueryConfig::default()
    .with_limit(10)
    .with_offset(20);

// Count only
let config = QueryConfig::default().count_only();

// Range query
let config = QueryConfig::new(start_key..end_key);

// Without blobs
let config = QueryConfig::default().no_blobs();
```

### Migration Steps
1. Update `src/databases/redb/transaction/crud.rs` to use `src/query::QueryConfig`
2. Remove or deprecate `src/databases/redb/transaction/options.rs::QueryConfig`
3. Update all test files to import from `netabase_store::query::QueryConfig`

## 4. Complete Migration Implementation

### Current State
Migration infrastructure is partially implemented:
- ✅ VersionHeader encoding/decoding  
- ✅ VersionContext tracking
- ✅ Schema comparison (DefinitionSchema::compare)
- ✅ MigrationPath calculation
- ❌ **Actual migration execution is stubbed out**

### What's Missing

#### 4.1 Metadata Table Implementation
File: `src/databases/redb/mod.rs`

The `SCHEMA_META_TABLE` constant exists but isn't used. Need to:

```rust
const SCHEMA_META_TABLE: redb::TableDefinition<&str, &[u8]> = 
    redb::TableDefinition::new("__netabase_schema_meta__");

impl<D: RedbDefinition> RedbStore<D> {
    fn new<P: AsRef<Path>>(path: P) -> NetabaseResult<Self> {
        // ... existing code ...
        
        // Read metadata from database
        let txn = db.begin_read()?;
        let stored_schema = if let Ok(table) = txn.open_table(SCHEMA_META_TABLE) {
            if let Some(schema_bytes) = table.get("schema")? {
                toml::from_str(std::str::from_utf8(schema_bytes.value())?).ok()
            } else {
                None
            }
        } else {
            None
        };
        
        // Write current schema to metadata table
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(SCHEMA_META_TABLE)?;
            let toml = D::export_toml();
            table.insert("schema", toml.as_bytes())?;
            table.insert("version", &D::VERSION.to_le_bytes())?;
        }
        write_txn.commit()?;
        
        // ... rest of initialization ...
    }
}
```

#### 4.2 Model Family Migration Execution
File: `src/databases/redb/migration.rs`

The `DatabaseMigrator::run()` method is stubbed. Needs implementation:

```rust
pub fn run(&self) -> NetabaseResult<DatabaseMigrationResult> {
    let paths = self.get_migration_paths();
    
    if paths.is_empty() {
        return Ok(DatabaseMigrationResult { /* ... */ });
    }
    
    let mut family_results = Vec::new();
    let mut total_records = 0;
    let mut has_errors = false;
    
    for path in &paths {
        // For each model family needing migration:
        // 1. Open the main table for this family
        let table_name = format!("{}_{}", path.family, "main");
        
        // 2. Read all records
        let read_txn = self.db.begin_read()?;
        let records: Vec<(Vec<u8>, Vec<u8>)> = {
            let table = read_txn.open_table(table_name)?;
            table.iter()?.collect()
        };
        
        // 3. Migrate each record
        let mut migrated_records = Vec::new();
        let mut errors = Vec::new();
        
        for (key, value) in records {
            match self.migrate_record(&path, &value) {
                Ok(migrated) => migrated_records.push((key, migrated)),
                Err(e) => {
                    errors.push(e);
                    if !self.options.continue_on_error {
                        break;
                    }
                }
            }
        }
        
        // 4. Write back migrated records
        if !self.options.dry_run {
            let write_txn = self.db.begin_write()?;
            {
                let mut table = write_txn.open_table(table_name)?;
                for (key, value) in migrated_records {
                    table.insert(&key, &value)?;
                }
            }
            write_txn.commit()?;
        }
        
        total_records += migrated_records.len();
        has_errors |= !errors.is_empty();
        
        family_results.push((
            path.family.to_string(),
            MigrationResult {
                records_migrated: migrated_records.len(),
                records_failed: errors.len(),
                errors,
                path: path.clone(),
            },
        ));
    }
    
    Ok(DatabaseMigrationResult {
        tables_migrated: paths.len(),
        total_records,
        family_results,
        has_errors,
        dry_run: self.options.dry_run,
    })
}
```

#### 4.3 Per-Model Migration Chain
This requires macro-generated code. Each model with `#[netabase_version]` should generate:

```rust
// Generated by macro for User model family
pub struct MigrationChain_User;

impl MigrationChainExecutor for MigrationChain_User {
    type Current = User; // Current version
    const VERSIONS: &'static [u32] = &[1, 2, 3];
    
    fn migrate_bytes(from_version: u32, bytes: &[u8]) -> Result<Self::Current, MigrationError> {
        match from_version {
            1 => {
                let v1: UserV1 = bincode::decode_from_slice(bytes, config)?;
                let v2: UserV2 = v1.into();
                let v3: User = v2.into();
                Ok(v3)
            }
            2 => {
                let v2: UserV2 = bincode::decode_from_slice(bytes, config)?;
                let v3: User = v2.into();
                Ok(v3)
            }
            3 => {
                let v3: User = bincode::decode_from_slice(bytes, config)?;
                Ok(v3)
            }
            _ => Err(MigrationError {
                record_key: String::new(),
                error: format!("Unsupported version: {}", from_version),
                at_version: from_version,
            })
        }
    }
}
```

### Testing Migration
Create a test that:
1. Creates a database with v1 schema
2. Inserts records
3. Updates code to v2 schema
4. Opens database and verifies migration runs
5. Checks all records migrated correctly

## Summary

### Priority Order
1. **CRITICAL**: Replace deprecated API calls (prevents warnings, improves code clarity)
2. **HIGH**: Configure test ordering (prevents import tests from failing)
3. **MEDIUM**: Use QueryConfig properly (removes duplicate code, improves API)
4. **LOW**: Complete migration (feature is partially implemented, not blocking)

### Quick Wins
Run these commands to find and fix deprecated calls:
```bash
# Find all deprecated calls
grep -rn "create_redb\|update_redb\|delete_redb" tests/

# Simple sed replacement (backup first!)
find tests -name "*.rs" -exec sed -i.bak 's/\.create_redb(/.create(/g; s/\.update_redb(/.update(/g; s/\.delete_redb(/.delete(/g' {} +
```
