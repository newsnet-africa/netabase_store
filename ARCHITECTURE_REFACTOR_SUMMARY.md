# Netabase Store Architecture Refactor Summary

## Overview

This document summarizes the completed refactor of the netabase_store architecture, focusing on creating a clean, type-safe, trait-based system for managing relational data with multiple backends.

## Key Architectural Principles

### 1. Avoid Opaque Vec<u8> Types
**Status: ✅ Implemented**

All data is wrapped in type-safe enums and newtypes:
- `DefinitionModelAssociatedTypes` enum wraps all model-related types
- Primary keys, secondary keys, and relational keys use specific newtypes (e.g., `UserId`, `UserEmail`)
- No opaque `Vec<u8>` or `String` types in the public API

### 2. Discriminant-Based Tree Naming
**Status: ✅ Implemented**

Uses `strum` macros for safe discriminant-based naming:
- `DefinitionsDiscriminants` for model tables
- `UserSecondaryKeysDiscriminants` for secondary key tables
- `UserRelationalKeysDiscriminants` for relational key tables
- Automatic `AsRefStr` conversion for tree names

### 3. Transaction-Based API
**Status: ✅ Implemented**

All operations use transactions with a queue-based pattern:
- `ReadTransaction` trait for read operations
- `WriteTransaction` trait for write operations
- Operations are queued and executed in priority order
- Automatic management of secondary and relational key insertions

### 4. Relational Keys with Typed References
**Status: ✅ Implemented**

Relational keys now reference related model's key types:

**Before (Raw Values):**
```rust
pub enum ProductRelationalKeys {
    CreatedBy(ProductCreatedBy(u64)),  // Raw u64
}
```

**After (Typed Keys):**
```rust
pub enum ProductRelationalKeys {
    CreatedBy(UserId),  // References User's primary key type
}
```

This enables:
- Type-safe relationship traversal
- Better IDE support and refactoring
- Clear dependency tracking between models

### 5. RelationalLink Enum for Lazy Hydration
**Status: ✅ Implemented**

The `RelationalLink<M, D>` enum supports lazy loading:
```rust
pub enum RelationalLink<M, D> {
    Unloaded(M::PrimaryKey),  // Just the key
    Loaded(M),                 // Full model
}
```

Methods:
- `key()` - Get the primary key
- `is_loaded()` - Check if hydrated
- `model()` - Get model if loaded
- `unwrap_model()` - Panic if not loaded

### 6. Tree/Table Manager
**Status: ✅ Implemented**

Centralized management via `TreeManager` trait:
- `all_trees()` - Get all registered trees
- `get_tree_name()` - Get main table name
- `get_secondary_tree_names()` - Get secondary key table names
- `get_relational_tree_names()` - Get relational key table names

### 7. Separation of Concerns
**Status: ✅ Implemented**

Traits organized by concern:
- `src/traits/definition/` - Definition-level traits
- `src/traits/model/` - Model-level traits
- `src/traits/model/key.rs` - Key trait
- `src/traits/model/relational.rs` - RelationalLink
- `src/traits/store/` - Store and transaction traits
- `src/traits/store/tree_manager.rs` - Tree management

## New Features

### Secondary Key Lookups
**Status: ✅ Implemented**

Three new methods on `ReadTransaction`:

1. **`get_pk_by_secondary_key()`** - Get primary key from secondary key
   ```rust
   let pk = txn.get_pk_by_secondary_key::<User>(
       UserSecondaryKeys::Email(UserEmail("alice@example.com".into()))
   )?;
   ```

2. **`get_by_secondary_key()`** - Convenience method (lookup + fetch)
   ```rust
   let user = txn.get_by_secondary_key::<User>(
       UserSecondaryKeys::Email(UserEmail("alice@example.com".into()))
   )?;
   ```

3. **`get()`** - Original primary key lookup (unchanged)

### Model Storage Pattern

Each model has:
1. **Main Tree**: `PrimaryKey -> Model`
   - Example: `User` table stores `UserId -> User`

2. **Secondary Key Trees** (one per secondary key type):
   - Example: `User_sec_Email` stores `UserEmail -> UserId`
   - Example: `User_sec_Name` stores `UserName -> UserId`

3. **Relational Key Trees** (one per relationship type):
   - Example: `Product_rel_CreatedBy` stores `UserId -> ProductId`
   - Enables efficient "find all products by user X" queries

4. **Hash Tree** (optional, for deduplication):
   - Example: `User_hash` stores `[u8; 32] -> UserId`

## Queue-Based Transaction Pattern

### How It Works

1. User calls `store.write(|txn| { ... })`
2. Within the closure, user calls `txn.put(model)`
3. Transaction queues operations:
   - Main tree insert
   - Secondary key inserts (one per secondary key)
   - Relational key inserts (one per relational key)
   - Hash tree insert
4. Operations are sorted by priority
5. All operations execute atomically on commit

### Priority Order
```rust
0 - Main tree insert
1 - Secondary key inserts
2 - Relational key inserts
3 - Hash tree insert
4 - Delete operations
```

## Type Safety Through Wrapper Enums

### DefinitionModelAssociatedTypes

Wraps all model-specific types in a single enum:
```rust
pub enum DefinitionModelAssociatedTypes {
    // User types
    UserPrimaryKey(UserId),
    UserModel(User),
    UserSecondaryKey(UserSecondaryKeys),
    UserRelationalKey(UserRelationalKeys),
    UserSecondaryKeyDiscriminant(UserSecondaryKeysDiscriminants),
    UserRelationalKeyDiscriminant(UserRelationalKeysDiscriminants),

    // Product types
    ProductPrimaryKey(ProductId),
    ProductModel(Product),
    // ... etc
}
```

### Benefits
- No `Vec<u8>` in operation queues
- Pattern matching for type-safe operations
- Clear conversion points at API boundaries
- Easy to extend with new models

## Example: Complete Model Definition

See `examples/boilerplate.rs` for a complete example showing:
- Model definition with primary, secondary, and relational keys
- Typed relational keys referencing other models
- redb `Key` and `Value` trait implementations
- `NetabaseModelTrait` and `RedbNetabaseModelTrait` implementations
- `ModelAssociatedTypesExt` implementation for type conversions
- Complete CRUD operations with relationships

## Implementation Status

### ✅ Completed
- [x] RelationalLink enum for lazy hydration
- [x] Typed relational keys (no raw values)
- [x] Transaction-based API with operation queue
- [x] Wrapper enums for type safety
- [x] Tree manager trait
- [x] Secondary key lookup methods
- [x] Trait-first architecture
- [x] Separation of concerns into modules
- [x] Discriminant-based naming

### 🚧 To Be Implemented (Future Work)
- [ ] `get_with_relations()` - Automatic relation hydration
- [ ] Batch operations with relation loading
- [ ] Range queries on secondary keys
- [ ] Relational queries (find all related models)
- [ ] Migration system for schema changes
- [ ] Additional backends (sled, memory, IndexedDB)

## API Usage Examples

### Basic CRUD
```rust
// Create store
let store = RedbStore::<Definitions>::new("db.redb")?;

// Insert model (automatically inserts all keys)
store.put_one(user)?;

// Get by primary key
let user = store.get_one::<User>(UserId(1))?;

// Get by secondary key
store.read(|txn| {
    let user = txn.get_by_secondary_key::<User>(
        UserSecondaryKeys::Email(UserEmail("alice@example.com".into()))
    )?;
    Ok(user)
})?;
```

### Batch Operations
```rust
// Insert many models
store.put_many(vec![user1, user2, user3])?;

// Get many by primary keys
let users = store.get_many::<User>(vec![
    UserId(1), UserId(2), UserId(3)
])?;
```

### Custom Transactions
```rust
store.write(|txn| {
    // Multiple operations in single transaction
    txn.put(user1)?;
    txn.put(product1)?;

    // All operations are atomic
    Ok(())
})?;
```

## Performance Considerations

### Operation Queue
- Operations are queued in memory before execution
- Allows for deduplication and optimization
- Sorted by priority to maintain referential integrity

### Secondary Key Lookups
- Two table lookups: secondary key table → main table
- Consider caching for frequently accessed data

### Relational Keys
- Enable efficient relationship queries
- Trade-off: extra storage for faster queries
- Suitable for many-to-one relationships

## Testing

The `examples/boilerplate.rs` file includes comprehensive tests:
- Primary key access
- Secondary key enumeration
- Relational key relationships
- Hash computation
- Discriminant enumeration
- Tree naming
- Serialization roundtrips
- Real database operations
- Batch operations

Run tests with:
```bash
cargo run --example boilerplate
```

## Migration Guide

If you have existing code using raw values in relational keys:

### Before
```rust
pub enum ProductRelationalKeys {
    CreatedBy(ProductCreatedBy(u64)),
}

impl NetabaseModelKeyTrait<Definitions, Product> for ProductKeys {
    fn relational_keys(model: &Product) -> Vec<Self::RelationalEnum> {
        vec![
            ProductRelationalKeys::CreatedBy(
                ProductCreatedBy(model.created_by)
            ),
        ]
    }
}
```

### After
```rust
pub enum ProductRelationalKeys {
    CreatedBy(UserId),  // Use the related model's key type
}

impl NetabaseModelKeyTrait<Definitions, Product> for ProductKeys {
    fn relational_keys(model: &Product) -> Vec<Self::RelationalEnum> {
        vec![
            ProductRelationalKeys::CreatedBy(UserId(model.created_by)),
        ]
    }
}
```

## Summary

This refactor achieves all the key goals:
1. ✅ Type-safe operations with no opaque types
2. ✅ Discriminant-based tree naming
3. ✅ Transaction-based API with queues
4. ✅ Typed relational keys
5. ✅ Trait-first, composable architecture
6. ✅ Separation of concerns
7. ✅ Secondary key lookups
8. ✅ Comprehensive boilerplate example

The architecture is now ready for:
- Multiple backend implementations
- Advanced querying features
- Relationship traversal and hydration
- Production use cases

## Next Steps

Recommended priorities for future development:
1. Implement `get_with_relations()` for automatic hydration
2. Add range queries on secondary keys
3. Implement relational queries (find all related)
4. Add more backends (memory, sled)
5. Create derive macros to reduce boilerplate
6. Add migration system for schema evolution
