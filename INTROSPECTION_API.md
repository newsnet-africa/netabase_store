# Database Introspection API

## Overview

A comprehensive introspection API has been added to netabase_store, allowing users to inspect all internal database trees, including user-defined model trees, secondary indexes, and system trees.

## API Documentation

### Core Trait: `DatabaseIntrospection<D>`

Located in `src/traits/introspection.rs`, this trait provides methods to inspect database internals.

#### Methods

```rust
pub trait DatabaseIntrospection<D: NetabaseDefinitionTrait> {
    /// List all trees in the database (models, indexes, system)
    fn list_all_trees(&self) -> Result<Vec<TreeInfo>, NetabaseError>;

    /// List only user-defined model trees
    fn list_model_trees(&self) -> Result<Vec<TreeInfo>, NetabaseError>;

    /// List only secondary index trees
    fn list_secondary_trees(&self) -> Result<Vec<TreeInfo>, NetabaseError>;

    /// List only system trees (libp2p, etc.)
    fn list_system_trees(&self) -> Result<Vec<TreeInfo>, NetabaseError>;

    /// Get the number of entries in a specific tree
    fn tree_entry_count(&self, tree_name: &str) -> Result<usize, NetabaseError>;

    /// Get all keys in a tree as raw bytes
    fn tree_keys_raw(&self, tree_name: &str) -> Result<Vec<Vec<u8>>, NetabaseError>;

    /// Get all key-value pairs in a tree as raw bytes
    fn tree_contents_raw(&self, tree_name: &str) -> Result<Vec<(Vec<u8>, Vec<u8>)>, NetabaseError>;

    /// Check if a tree exists
    fn tree_exists(&self, tree_name: &str) -> Result<bool, NetabaseError>;

    /// Get aggregate database statistics
    fn database_stats(&self) -> Result<DatabaseStats, NetabaseError>;
}
```

### Data Types

#### `TreeInfo`
```rust
pub struct TreeInfo {
    pub name: String,              // Tree name in database
    pub tree_type: TreeType,       // Category of tree
    pub entry_count: Option<usize>, // Number of entries
    pub size_bytes: Option<u64>,   // Size in bytes (if available)
}
```

#### `TreeType`
```rust
pub enum TreeType {
    PrimaryModel,      // User-defined model tree
    SecondaryIndex,    // Secondary key index
    LibP2PProviders,   // libp2p provider records
    LibP2PProvided,    // libp2p provided keys
    Subscription,      // Subscription/sync trees
    System,            // Unknown/system tree
}
```

#### `DatabaseStats`
```rust
pub struct DatabaseStats {
    pub total_trees: usize,
    pub model_trees: usize,
    pub secondary_trees: usize,
    pub system_trees: usize,
    pub total_entries: usize,
    pub total_size_bytes: u64,
}
```

## Implementation Status

### ✅ Fully Implemented

- **SledStore**: Complete introspection support
- **RedbStore**: Complete introspection support
- **RedbStoreZeroCopy**: Complete introspection support

### Internal Trees Exposed

For each model in your definition, the following trees are tracked:

1. **Primary Tree**: `{ModelName}` - Stores model instances by primary key
2. **Secondary Index Tree**: `{ModelName}_secondary` - Maps secondary keys to primary keys

Additional system trees (when features enabled):
3. **libp2p Trees** (with `libp2p` feature):
   - `__libp2p_providers` - Provider records
   - `__libp2p_provided` - Provided keys

## Usage Examples

### Basic Introspection

```rust
use netabase_store::databases::sled_store::SledStore;
use netabase_store::traits::introspection::DatabaseIntrospection;

let store = SledStore::<MyDefinition>::temp()?;

// List all trees
for tree in store.list_all_trees()? {
    println!("{}: {} entries ({:?})",
        tree.name,
        tree.entry_count.unwrap_or(0),
        tree.tree_type
    );
}

// Get stats
let stats = store.database_stats()?;
println!("Total entries: {}", stats.total_entries);
```

### Model Tree Inspection

```rust
// List only user-defined models
let model_trees = store.list_model_trees()?;
for tree in model_trees {
    println!("Model tree: {}", tree.name);
}

// Check specific tree
let user_count = store.tree_entry_count("User")?;
println!("User tree has {} entries", user_count);
```

### Secondary Index Inspection

```rust
// List all secondary indexes
let secondary_trees = store.list_secondary_trees()?;
for tree in secondary_trees {
    println!("Secondary index: {} ({} entries)",
        tree.name,
        tree.entry_count.unwrap_or(0)
    );
}
```

### Raw Data Access

```rust
// Get all keys from a tree
let keys = store.tree_keys_raw("User")?;
println!("Found {} keys", keys.len());

// Get all key-value pairs
let contents = store.tree_contents_raw("User")?;
for (key, value) in contents {
    println!("Key: {} bytes, Value: {} bytes", key.len(), value.len());
}
```

## Testing Framework

A comprehensive testing framework has been created in `tests/comprehensive/` with:

- **Backend-agnostic utilities** (`utils/mod.rs`)
- **State verification helpers** - Capture and compare DB state before/after operations
- **Test categories**:
  - CRUD operations
  - Secondary key queries
  - Batch operations
  - Transactions
  - Relations/links
  - Subscriptions
  - Introspection API

### State Verification Pattern

```rust
use tests::comprehensive::utils::DatabaseState;

// Capture state before
let before = DatabaseState::capture(&store)?;

// Perform operation
tree.put(user)?;

// Capture state after
let after = DatabaseState::capture(&store)?;

// Verify changes
let diff = before.diff(&after);
assert_eq!(diff.entry_count_change, 2); // 1 primary + 1 secondary
```

## Use Cases

### 1. Testing and Verification
Verify database state after operations to ensure correctness.

### 2. Debugging
Inspect internal state when troubleshooting issues.

### 3. Monitoring
Track database size and entry counts over time.

### 4. Migration Tools
Build tools to migrate data between backends or versions.

### 5. Backup and Recovery
Create comprehensive backups that include all internal state.

### 6. Performance Analysis
Identify which trees are growing and analyze access patterns.

## Performance Considerations

- `list_all_trees()` is relatively lightweight - it queries metadata, not full content
- `tree_contents_raw()` can be expensive for large trees - use sparingly
- `tree_keys_raw()` is more efficient than `tree_contents_raw()` when you only need keys
- Consider caching `DatabaseStats` for frequently accessed statistics

## Future Enhancements

Potential future additions:
- IndexedDB backend introspection (WASM support)
- Tree-level statistics (min/max key, avg value size, etc.)
- Real-time monitoring hooks
- Export/import utilities using introspection API
- Diff tools to compare two database states

## Notes

- The API exposes internal implementation details - use carefully in production
- Raw byte access bypasses type safety - deserialize manually if needed
- Some backends may not provide all statistics (e.g., `size_bytes` may be None)
- System trees are implementation details and may change between versions
