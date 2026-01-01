# Netabase Store - Quick Reference Guide

A concise reference for common operations. See [TESTING.md](TESTING.md) for detailed examples.

## Setup

```rust
use netabase_store::databases::redb::RedbNetabaseDatabase;
use netabase_store::databases::redb::transaction::RedbModelCrud;
use netabase_store::traits::registery::models::model::RedbNetbaseModel;
use netabase_store::relational::RelationalLink;

// Define your schema
#[netabase_macros::netabase_definition]
struct MyDefinition {
    // ... models ...
}

// Create database
let store = RedbNetabaseDatabase::<MyDefinition>::open("mydb.redb")?;
```

## CRUD Operations

### Create
```rust
let txn = store.begin_transaction()?;
txn.create_redb(&my_model)?;
txn.commit()?;
```

### Read
```rust
let txn = store.begin_transaction()?;
{
    let table_defs = MyModel::table_definitions();
    let tables = txn.open_model_tables(table_defs, None)?;
    
    if let Some(model) = MyModel::read_default(&primary_key, &tables)? {
        // Use model
    }
}
txn.commit()?;
```

### Update
```rust
let txn = store.begin_transaction()?;
txn.update_redb(&modified_model)?;
txn.commit()?;
```

### Delete
```rust
let txn = store.begin_transaction()?;
txn.delete_redb::<MyModel>(&primary_key)?;
txn.commit()?;
```

## Listing & Counting

```rust
let txn = store.begin_transaction()?;
{
    let table_defs = MyModel::table_definitions();
    let tables = txn.open_model_tables(table_defs, None)?;
    
    // Count
    let total = MyModel::count_entries(&tables)?;
    
    // List all
    let all_models: Vec<MyModel> = MyModel::list_default(&tables)?;
}
txn.commit()?;
```

## Relational Links

### Dehydrated (Primary Key Only)
```rust
// Minimal memory, no lifetime constraints
let link = RelationalLink::new_dehydrated(OtherModelID("key".to_string()));
```

### Owned (Full Model)
```rust
// Owns the related model
let link = RelationalLink::new_owned(
    OtherModelID("key".to_string()),
    other_model
);
```

### Hydrated (Reference)
```rust
// Holds a reference to existing model
let link = RelationalLink::new_hydrated(
    OtherModelID("key".to_string()),
    &other_model
);
```

### Borrowed (Database Reference)
```rust
// Reference from database AccessGuard
let link = RelationalLink::new_borrowed(
    OtherModelID("key".to_string()),
    &db_model
);
```

### Conversions
```rust
// All variants can become dehydrated
let dehydrated = link.dehydrate();

// Check variant type
if link.is_dehydrated() { /* ... */ }
if link.is_owned() { /* ... */ }
if link.is_hydrated() { /* ... */ }
if link.is_borrowed() { /* ... */ }

// Get primary key (available on all variants)
let pk = link.get_primary_key();

// Get model reference (if variant has it)
if let Some(model_ref) = link.get_model() {
    // Use model
}
```

## Transactions

### Commit Changes
```rust
let txn = store.begin_transaction()?;
txn.create_redb(&model1)?;
txn.create_redb(&model2)?;
txn.commit()?;  // Both persisted atomically
```

### Rollback (Automatic on Drop)
```rust
{
    let txn = store.begin_transaction()?;
    txn.create_redb(&model)?;
    // Transaction dropped here - changes rolled back
}
```

### Explicit Rollback
```rust
let txn = store.begin_transaction()?;
txn.create_redb(&model)?;
if error_condition {
    drop(txn);  // Rollback
    return Err(...);
}
txn.commit()?;
```

## Batch Operations

```rust
let txn = store.begin_transaction()?;
for item in items {
    txn.create_redb(&item)?;
}
txn.commit()?;  // All items committed together
```

## Blob Storage

Blobs are automatically chunked when > threshold:

```rust
// Derive NetabaseBlobItem for large data types
#[derive(NetabaseBlobItem)]
struct LargeData {
    data: Vec<u8>,  // Automatically chunked if large
    metadata: String,
}

let model = MyModel {
    // ... fields ...
    large_field: LargeData {
        data: vec![0; 200_000],  // 200KB - will be chunked
        metadata: "description".to_string(),
    },
};

// Store (chunking is automatic)
txn.create_redb(&model)?;

// Retrieve (reassembly is automatic)
let read_model = MyModel::read_default(&id, &tables)?;
// read_model.large_field.data is complete
```

## Subscriptions

```rust
// Define in model
#[netabase_macros::netabase_model]
struct MyModel {
    #[primary_key]
    id: MyModelID,
    // ... fields ...
    #[index]
    subscriptions: Vec<MyDefinitionSubscriptions>,
}

// Create with subscriptions
let model = MyModel {
    id: MyModelID("id".to_string()),
    subscriptions: vec![
        MyDefinitionSubscriptions::Topic1,
        MyDefinitionSubscriptions::Topic2,
    ],
    // ... other fields ...
};

txn.create_redb(&model)?;
```

## Error Handling

Operations return `NetabaseResult<T>`:

```rust
match MyModel::read_default(&id, &tables) {
    Ok(Some(model)) => { /* Found */ },
    Ok(None) => { /* Not found - not an error */ },
    Err(e) => { /* Actual error */ },
}
```

Common patterns:
```rust
// Read with default fallback
let model = MyModel::read_default(&id, &tables)?
    .unwrap_or_else(|| MyModel::default());

// Safe delete (idempotent)
txn.delete_redb::<MyModel>(&id)?;  // Succeeds even if doesn't exist
```

## Repository Isolation

```rust
// Standalone repository (default - can link across definitions)
#[netabase_macros::netabase_model]
struct User {
    #[primary_key]
    id: UserID,
    partner: RelationalLink<Standalone, Definition, Definition, User>,
    category: RelationalLink<Standalone, Definition, Definition, Category>,
}

// Explicit repository (isolated)
#[netabase_macros::netabase_model(repos(SecureRepo))]
struct SecureModel {
    // Can only link to other SecureRepo models
}
```

## Best Practices

1. **Transactions**: Keep transactions short, commit promptly
2. **Batch Operations**: Use single transaction for multiple related changes
3. **Error Handling**: Always handle `NetabaseResult` properly
4. **Links**: Use `Dehydrated` for storage, upgrade to `Owned`/`Hydrated` when needed
5. **Blobs**: Let the system handle chunking automatically
6. **Cleanup**: Use `drop(txn)` for explicit rollback

## Testing

See comprehensive examples in `tests/comprehensive_functionality.rs`:
```bash
cargo test --test comprehensive_functionality
```

## Common Patterns

### Create and Immediately Read
```rust
let txn = store.begin_transaction()?;
txn.create_redb(&model)?;
txn.commit()?;

let txn = store.begin_transaction()?;
{
    let table_defs = MyModel::table_definitions();
    let tables = txn.open_model_tables(table_defs, None)?;
    let verified = MyModel::read_default(&model.id, &tables)?;
}
txn.commit()?;
```

### Update with Verification
```rust
// Read original
let txn = store.begin_transaction()?;
let original = {
    let table_defs = MyModel::table_definitions();
    let tables = txn.open_model_tables(table_defs, None)?;
    MyModel::read_default(&id, &tables)?.unwrap()
};
txn.commit()?;

// Modify
let modified = MyModel {
    id: original.id,
    field: new_value,
    ..original
};

// Update
let txn = store.begin_transaction()?;
txn.update_redb(&modified)?;
txn.commit()?;
```

### Conditional Create
```rust
let txn = store.begin_transaction()?;
{
    let table_defs = MyModel::table_definitions();
    let tables = txn.open_model_tables(table_defs, None)?;
    
    if MyModel::read_default(&id, &tables)?.is_none() {
        drop(tables);  // Release read lock
        txn.create_redb(&model)?;
    }
}
txn.commit()?;
```

## See Also

- [TESTING.md](TESTING.md) - Comprehensive examples and documentation
- [TEST_SUMMARY.md](TEST_SUMMARY.md) - Test suite overview
- API documentation: `cargo doc --open`
