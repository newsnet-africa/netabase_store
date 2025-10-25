# Netabase Store Architecture

This document describes the internal architecture of the Netabase Store type-safe multi-backend storage library.

## Overview

Netabase Store is a type-safe key-value storage abstraction that provides a unified API across multiple database backends (Sled, Redb, IndexedDB). It uses procedural macros to generate compile-time verified schemas with automatic primary and secondary key indexing.

## High-Level Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                   Application Code                            │
│        (User-defined models with derive macros)              │
└─────────────────────┬────────────────────────────────────────┘
                      │
           ┌──────────┴──────────┐
           │ Procedural Macros   │
           │ (Compile Time)      │
           │                     │
           │ - NetabaseModel     │
           │ - Definition Module │
           └──────────┬──────────┘
                      │ (generates)
                      ▼
┌─────────────────────────────────────────────────────────────┐
│           Generated Type-Safe Schema                         │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  DefinitionEnum: MyModel1 | MyModel2 | ...         │   │
│  │  KeysEnum: MyModel1Keys | MyModel2Keys | ...        │   │
│  │  PrimaryKeys: MyModel1PrimaryKey, ...               │   │
│  │  SecondaryKeys: MyModel1SecondaryKeys, ...          │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│                  Trait Layer                                 │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  NetabaseModelTrait                                  │   │
│  │  NetabaseDefinitionTrait                             │   │
│  │  NetabaseTreeSync (native)                           │   │
│  │  NetabaseTreeAsync (WASM)                            │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────┬───────────────────────────────────────┘
                      │
            ┌─────────┴─────────┐
            │                   │
            ▼                   ▼
┌──────────────────┐  ┌──────────────────┐
│   SledStore<D>   │  │   RedbStore<D>   │  [IndexedDBStore<D>]
│                  │  │                  │
│  open_tree()     │  │  open_tree()     │
│  temp()          │  │  temp()          │
│  new(path)       │  │  new(path)       │
└────────┬─────────┘  └────────┬─────────┘
         │                     │
         ▼                     ▼
┌──────────────────┐  ┌──────────────────┐
│ SledStoreTree<M> │  │ RedbStoreTree<M> │
│                  │  │                  │
│  put(model)      │  │  put(model)      │
│  get(key)        │  │  get(key)        │
│  remove(key)     │  │  remove(key)     │
│  get_by_sec_key()│  │  get_by_sec_key()│
│  iter()          │  │  iter()          │
└────────┬─────────┘  └────────┬─────────┘
         │                     │
         ▼                     ▼
┌──────────────────┐  ┌──────────────────┐
│   sled::Db       │  │  redb::Database  │
│  (Embedded DB)   │  │  (Embedded DB)   │
└──────────────────┘  └──────────────────┘
```

## Core Components

### 1. Procedural Macros (`netabase_macros/`)

**Purpose**: Generate type-safe schema code at compile time.

#### `#[netabase_definition_module]` Macro

**Input**:
```rust
#[netabase_definition_module(MyDefinition, MyKeys)]
mod my_schema {
    #[derive(NetabaseModel, ...)]
    #[netabase(MyDefinition)]
    pub struct Model1 {
        #[primary_key]
        pub id: u64,
        #[secondary_key]
        pub name: String,
    }
}
```

**Generates**:
1. **Definition Enum**: Wraps all models
   ```rust
   pub enum MyDefinition {
       Model1(Model1),
       // ... other models
   }
   ```

2. **Keys Enum**: Union of all model keys
   ```rust
   pub enum MyKeys {
       Model1(Model1Key),
       // ... other model keys
   }
   ```

3. **Trait Implementations**: `NetabaseDefinitionTrait`, conversion traits, etc.

#### `#[derive(NetabaseModel)]` Macro

**Generates for each model**:
1. **Primary Key Type**:
   ```rust
   pub struct Model1PrimaryKey(pub u64);
   ```

2. **Secondary Keys Enum**:
   ```rust
   pub enum Model1SecondaryKeys {
       Name(NameSecondaryKey),
       // ... other secondary keys
   }
   ```

3. **Model Key Enum**:
   ```rust
   pub enum Model1Key {
       Primary(Model1PrimaryKey),
       Secondary(Model1SecondaryKeys),
   }
   ```

4. **Trait Implementations**: `NetabaseModelTrait` with methods like:
   - `primary_key()` - Extract primary key
   - `secondary_keys()` - Extract all secondary keys
   - `DISCRIMINANT` - Model identifier

### 2. Trait Layer (`src/traits/`)

**Purpose**: Define common interfaces for all storage operations.

#### Core Traits

**`NetabaseModelTrait<D>`**:
- Associated types for keys
- Methods to extract primary/secondary keys
- Model discriminant (type identifier)

**`NetabaseDefinitionTrait`**:
- Schema-level enum trait
- Keys enum association
- Conversion methods

**`NetabaseTreeSync<D, M>`** (Native):
- `put(model)` - Insert or update
- `get(key)` - Retrieve by primary key
- `remove(key)` - Delete
- `get_by_secondary_key(key)` - Query by secondary key
- `iter()` - Iterate all records
- `len()`, `is_empty()`, `clear()` - Utility methods

**`NetabaseTreeAsync<D, M>`** (WASM):
- Same methods as `NetabaseTreeSync` but async

**`ToIVec`**:
- Conversion to/from byte vectors
- Used for database serialization

### 3. Storage Backends (`src/databases/`)

#### Sled Backend (`sled_store.rs`)

**SledStore<D>**:
- Wrapper around `sled::Db`
- Generic over definition type `D`
- Methods:
  - `new(path)` - Open database at path
  - `temp()` - Create temporary in-memory database
  - `open_tree::<M>()` - Get type-safe tree for model
  - `tree_names()` - List all discriminants
  - `db()` - Access underlying sled::Db
  - `flush()` - Persist to disk

**SledStoreTree<D, M>**:
- Type-safe wrapper around `sled::Tree`
- Generic over definition `D` and model `M`
- Implements `NetabaseTreeSync<D, M>`
- Manages:
  - Primary key → model mapping
  - Secondary key → primary key index
- Methods handle serialization automatically

**Key Storage Structure**:
```
Tree: "<ModelDiscriminant>"
  ├─ primary:<primary_key> → <serialized_model>
  ├─ secondary:email:<email> → <primary_key>
  └─ secondary:age:<age> → <primary_key>
```

#### Redb Backend (`redb_store.rs`)

**BincodeWrapper<T>**:
- Implements redb's `Key` and `Value` traits
- Uses bincode for serialization
- Allows arbitrary Rust types as keys/values

**RedbStore<D>**:
- Wrapper around `redb::Database`
- Similar API to `SledStore`
- Uses `Arc<Database>` for thread-safety

**RedbStoreTree<D, M>**:
- Type-safe wrapper for redb tables
- Similar to `SledStoreTree`
- Uses transactions for writes

**Differences from Sled**:
- ACID guarantees with full transaction support
- More memory-efficient
- Different trade-offs (slightly slower writes, better consistency)

#### IndexedDB Backend (`indexeddb_store.rs`) - WASM Only

**IndexedDBStore<D>**:
- Wrapper around browser's IndexedDB API
- Async operations (browser requirement)
- Uses `indexed_db_futures` crate

**IndexedDBStoreTree<D, M>**:
- Implements `NetabaseTreeAsync<D, M>`
- All operations are async
- Uses IndexedDB object stores

### 4. libp2p Integration (`record_store.rs`)

**Purpose**: Allow stores to be used as Kademlia DHT record stores.

**Implementation**:
- `SledStore` and `RedbStore` implement libp2p's `RecordStore` trait
- Stores DHT records as special entries in the database
- Manages provider records for content discovery
- Separate from model storage (different key namespace)

**RecordStore Methods**:
- `get(key)` - Retrieve a DHT record
- `put(record)` - Store a DHT record
- `remove(key)` - Delete a DHT record
- `records()` - Iterate all DHT records
- `add_provider()` - Add a provider record
- `providers(key)` - Get providers for a key
- `provided()` - Get all locally provided records
- `remove_provider()` - Remove a provider record

**Storage Layout**:
```
Tree: "__libp2p_records__"
  ├─ record:<key> → <Record>
  ├─ provider:<key>:<peer_id> → <ProviderRecord>
  └─ provided:<peer_id>:<key> → <ProviderRecord>
```

## Data Flow

### Put Operation

```
1. Application: tree.put(model)
        ↓
2. Tree: Extract primary key from model
        ↓
3. Tree: Serialize model with bincode
        ↓
4. Tree: Store primary_key → serialized_model
        ↓
5. Tree: For each secondary key:
        Extract key value
        Store secondary_key → primary_key
        ↓
6. Backend: Persist to disk/IndexedDB
        ↓
7. Return: Result<()>
```

### Get by Primary Key

```
1. Application: tree.get(primary_key)
        ↓
2. Tree: Convert key to bytes
        ↓
3. Backend: Lookup in database
        ↓
4. Tree: Deserialize if found
        ↓
5. Return: Result<Option<Model>>
```

### Get by Secondary Key

```
1. Application: tree.get_by_secondary_key(secondary_key)
        ↓
2. Tree: Convert secondary key to bytes
        ↓
3. Backend: Lookup secondary_key → primary_key
        ↓
4. Tree: For each primary key found:
        Lookup primary_key → model
        Deserialize model
        ↓
5. Return: Result<Vec<Model>>
```

### Remove Operation

```
1. Application: tree.remove(primary_key)
        ↓
2. Tree: Get model by primary key (to extract secondary keys)
        ↓
3. Tree: Remove primary_key → model mapping
        ↓
4. Tree: For each secondary key in the model:
        Remove secondary_key → primary_key mapping
        ↓
5. Backend: Persist changes
        ↓
6. Return: Result<Option<Model>> (the removed model)
```

## Secondary Key Indexing

### Index Maintenance

**On Insert/Update**:
1. If updating, get old model and remove old secondary key entries
2. For each `#[secondary_key]` field:
   - Generate index key: `secondary:<field_name>:<value>`
   - Store: index_key → primary_key
3. This allows reverse lookups

**On Delete**:
1. Get model to extract secondary keys
2. Remove all secondary key → primary key mappings
3. Remove primary key → model mapping

### Index Structure

```
Model:
  User { id: 1, email: "alice@example.com", age: 30 }

Storage:
  primary:1 → User{id:1, email:"alice@example.com", age:30}
  secondary:email:alice@example.com → 1
  secondary:age:30 → 1
```

### Query Performance

- **Primary Key**: O(log n) - Direct lookup
- **Secondary Key**: O(log n + m) where m = matching records
  1. O(log n) to find secondary key index
  2. O(m) to fetch m matching primary keys
  3. O(m log n) to fetch m models
- **Full Iteration**: O(n) - Scan all records

## Type Safety Mechanism

### Compile-Time Guarantees

1. **Key Type Matching**: Can't use Model1's key with Model2's tree
   ```rust
   let user_tree = store.open_tree::<User>();
   let post_tree = store.open_tree::<Post>();

   user_tree.get(UserPrimaryKey(1)); // ✓ OK
   user_tree.get(PostPrimaryKey(1)); // ✗ Compile error!
   ```

2. **Secondary Key Existence**: Can only query keys that exist
   ```rust
   user_tree.get_by_secondary_key(UserSecondaryKeys::Email(...)); // ✓ OK
   user_tree.get_by_secondary_key(UserSecondaryKeys::Phone(...)); // ✗ Compile error if Phone not defined!
   ```

3. **Definition Matching**: Trees must match store definition
   ```rust
   let store = SledStore::<BlogDefinition>::new("db")?;
   let user_tree = store.open_tree::<User>(); // ✓ OK if User in BlogDefinition
   let article_tree = store.open_tree::<Article>(); // ✗ Compile error if Article not in BlogDefinition!
   ```

### Runtime Invariants

1. **Primary Key Uniqueness**: Enforced by database backend
2. **Index Consistency**: Secondary key indices always match stored models
3. **Serialization**: All models must be bincode-serializable
4. **Tree Isolation**: Different model types use different database trees

## Error Handling

### Error Types

**`NetabaseError`**:
- `Sled(sled::Error)` - Sled backend errors
- `Redb(redb::Error)` - Redb backend errors
- `IndexedDB(String)` - IndexedDB errors (WASM)
- `Encoding(EncodingDecodingError)` - Serialization errors
- `StoreError(StoreError)` - General store errors

**`EncodingDecodingError`**:
- `BincodeEncode` - Failed to serialize
- `BincodeDecode` - Failed to deserialize

### Error Propagation

```
Backend Error
    ↓
NetabaseError wrapper
    ↓
Tree method Result
    ↓
Application
```

## Performance Optimizations

### Zero-Copy Where Possible

- Sled allows zero-copy reads with `IVec`
- Bincode provides efficient serialization
- Secondary key lookups minimize data copying

### Caching

- Backends (Sled, Redb) have their own caching
- No additional caching layer (keep it simple)

### Batch Operations

- Not yet implemented, but planned
- Will use backend-specific transaction support

## Testing Strategy

### Unit Tests

- `tests/backend_crud_tests.rs`: Comprehensive CRUD tests for all backends
- `tests/sled_store_tests.rs`: Sled-specific tests
- `tests/record_store_tests.rs`: libp2p integration tests
- `tests/cross_store_compat_tests.rs`: Cross-backend compatibility

### Integration Tests

- Real database operations
- Multi-model scenarios
- Secondary key queries

### Benchmarks

- `benches/sled_wrapper_overhead.rs`: Measure overhead vs raw Sled
- `benches/redb_wrapper_overhead.rs`: Measure overhead vs raw Redb
- Comparison of different backends

## Cross-Platform Support

### Native (Rust std)

- Full feature set
- Synchronous API (`NetabaseTreeSync`)
- Sled and Redb backends

### WASM (Browser)

- Async API only (`NetabaseTreeAsync`)
- IndexedDB backend
- Limited to browser capabilities

### Feature Flags

```toml
[features]
default = ["sled"]
native = ["sled", "redb"]
wasm = ["indexed_db_futures"]
libp2p = ["libp2p-kad"]
```

## Future Enhancements

1. **Transactions**: Multi-operation ACID transactions
2. **Range Queries**: Query ranges of ordered keys
3. **Composite Keys**: Multiple-field primary keys
4. **Migrations**: Schema version management
5. **Compression**: Optional transparent compression
6. **Encryption**: At-rest encryption support
7. **Query Builder**: Fluent API for complex queries
8. **Async Native**: Async API for native platforms

## Debugging Tips

### Inspect Database Structure

```rust
// List all trees
let trees = store.tree_names();
println!("Trees: {:?}", trees);

// Count records in a tree
let count = user_tree.len()?;
println!("Users: {}", count);

// Iterate and inspect
for result in user_tree.iter() {
    let (key, user) = result?;
    println!("{:?} => {:?}", key, user);
}
```

### Check Secondary Keys

```rust
// Query by secondary key
let users = user_tree.get_by_secondary_key(
    UserSecondaryKeys::Age(AgeSecondaryKey(30))
)?;
println!("Users with age 30: {:?}", users);
```

### Backend-Specific Tools

**Sled**:
```bash
# View database with sled command-line tool
sled dump ./my_database
```

**Redb**:
- Use redb's built-in inspection tools

## Related Documentation

- [README.md](./README.md): User guide and examples
- [netabase/ARCHITECTURE.md](../netabase/ARCHITECTURE.md): Networking layer
- [examples/](./examples/): Working code examples
