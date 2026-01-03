# Blob Query Methods

## Overview

Four new read-only methods have been added to the `RedbModelCrud` trait to enable decentralized network use cases such as parallel fetching and sharded storage of blob data.

## New Methods

### 1. `read_blob_items`

```rust
fn read_blob_items<'a, 'txn>(
    blob_key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Blob,
    tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
) -> NetabaseResult<Vec<<<Self::Keys as NetabaseModelKeys<D, Self>>::Blob as NetabaseModelBlobKey<D, Self>>::BlobItem>>
```

**Purpose**: Read all blob items for a specific blob key.

**Use Case**: Fetch blob data independently of the main model, enabling parallel fetching in decentralized networks. Multiple nodes can fetch different blob keys in parallel.

**Returns**: A vector of all blob items associated with the given key across all blob tables.

### 2. `list_blob_keys`

```rust
fn list_blob_keys<'a, 'txn>(
    table_index: usize,
    tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
) -> NetabaseResult<Vec<<Self::Keys as NetabaseModelKeys<D, Self>>::Blob>>
```

**Purpose**: List all blob keys in a specific blob table.

**Use Case**: Discover what blobs exist, enabling sharded storage where different nodes may store different blob keys. A coordinator node can use this to determine which blobs exist and distribute fetching across nodes.

**Parameters**:
- `table_index`: Index of the blob table (corresponds to blob field order in your model definition)

**Returns**: A vector of all blob keys in that table.

**Note**: Since blob tables are multimaps (multiple values per key), this may return duplicate keys. Callers should deduplicate if needed.

### 3. `count_blob_entries`

```rust
fn count_blob_entries<'txn>(
    tables: &ModelOpenTables<'txn, 'db, D, Self>,
) -> NetabaseResult<u64>
```

**Purpose**: Count total blob entries across all blob tables.

**Use Case**: Storage metrics and load balancing in sharded systems. Helps determine if blob storage should be redistributed across nodes.

**Returns**: Total number of blob entries (key-value pairs) across all blob tables.

### 4. `blob_table_stats`

```rust
fn blob_table_stats<'txn>(
    tables: &ModelOpenTables<'txn, 'db, D, Self>,
) -> NetabaseResult<Vec<(String, u64)>>
```

**Purpose**: Get blob table metadata (table name and entry count) for each blob field.

**Use Case**: Monitoring and debugging blob storage distribution. Helps identify which blob fields have more data and may need special handling in a distributed system.

**Returns**: A vector of `(table_name, entry_count)` tuples for each blob table.

## Implementation Details

### Type Bounds

These methods required adding new type bounds to the `RedbModelCrud` impl to ensure proper conversion from `redb::Value::SelfType<'a>` to the actual types:

```rust
for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: 
    redb::Value<SelfType<'a> = <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob>,
for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: 
    redb::Value<SelfType<'a> = <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem>,
```

These bounds ensure that the `SelfType<'a>` GAT equals the type itself, allowing safe conversion from guard values.

### Table Permission Handling

All methods handle the three table permission types:
- `TablePermission::ReadOnly`
- `TablePermission::ReadWrite` 
- `TablePermission::ReadOnlyWrite`

This ensures the methods work in both read-only and read-write transaction contexts.

## Example Use Cases

### Parallel Blob Fetching

```rust
// Node 1: Fetch keys
let blob_keys = MyModel::list_blob_keys(0, &tables)?;

// Distribute keys to multiple nodes
// Node 2 fetches blob_keys[0..100]
// Node 3 fetches blob_keys[100..200]
// etc.

for key in my_shard_of_keys {
    let items = MyModel::read_blob_items(&key, &tables)?;
    process_items(items);
}
```

### Sharded Storage Monitoring

```rust
// Monitor storage distribution
let stats = MyModel::blob_table_stats(&tables)?;
for (table_name, count) in stats {
    println!("Table {}: {} entries", table_name, count);
}

let total = MyModel::count_blob_entries(&tables)?;
println!("Total blob entries: {}", total);
```

### Load Balancing

```rust
// Check if rebalancing is needed
let stats = MyModel::blob_table_stats(&tables)?;
let max_entries = stats.iter().map(|(_, count)| *count).max().unwrap_or(0);
let min_entries = stats.iter().map(|(_, count)| *count).min().unwrap_or(0);

if max_entries > min_entries * 2 {
    println!("Storage imbalance detected, consider rebalancing");
}
```

## Migration from Bincode to Postcard

These methods were added as part of the migration from bincode to postcard serialization. The postcard format is more suitable for wire transmission and cross-platform compatibility, making it ideal for decentralized network scenarios.
