# Netabase Store Architecture

This document provides a comprehensive overview of the netabase_store crate's internal architecture and design decisions.

## Table of Contents

1. [Overview](#overview)
2. [Module Structure](#module-structure)
3. [Type System](#type-system)
4. [Transaction Architecture](#transaction-architecture)
5. [Migration System](#migration-system)
6. [Blob Storage](#blob-storage)
7. [Repository Pattern](#repository-pattern)
8. [Serialization](#serialization)

---

## Overview

Netabase Store is a type-safe embedded database library built on redb. The core philosophy is:

- **Compile-time correctness**: Wrong types = compiler errors
- **Zero-cost abstractions**: Macros generate optimal code
- **Explicit relationships**: Links are type-checked and repository-aware
- **Graceful evolution**: Schema versioning with automatic migration

### Key Design Principles

1. **Type Safety First**: Every ID is a unique type (`UserID`, `PostID`)
2. **Macro-Driven**: Most boilerplate generated via proc macros
3. **Repository Isolation**: Compile-time enforcement of data access boundaries
4. **Migration-Ready**: Built-in versioning from day one

---

## Module Structure

```
netabase_store/
├── src/
│   ├── lib.rs                 # Crate root, feature flags
│   ├── prelude.rs             # Common imports
│   ├── errors.rs              # Error types
│   ├── blob.rs                # Large data handling
│   ├── query.rs               # Query configuration
│   ├── relational.rs          # Relationship types
│   ├── doc_examples.rs        # Documentation examples
│   ├── utils/                 # Utility functions
│   │   └── mod.rs             # Serde helpers
│   ├── databases/             # Backend implementations
│   │   ├── mod.rs
│   │   ├── redb/              # Redb backend (primary)
│   │   │   ├── mod.rs         # Store and schema management
│   │   │   ├── migration.rs   # Data migration logic
│   │   │   ├── repository.rs  # Repository-scoped stores
│   │   │   ├── libp2p.rs      # P2P integration
│   │   │   └── transaction/   # Transaction layer
│   │   │       ├── mod.rs     # Transaction types
│   │   │       ├── crud.rs    # CRUD operations
│   │   │       ├── hydration.rs # Link hydration
│   │   │       ├── options.rs # Query options
│   │   │       ├── tables.rs  # Table management
│   │   │       ├── wrappers.rs # Type wrappers
│   │   │       └── value_wrappers.rs # Value adapters
│   │   └── indexedb/          # Browser backend (future)
│   └── traits/                # Core trait definitions
│       ├── mod.rs
│       ├── database/          # Database abstractions
│       │   ├── mod.rs
│       │   ├── hash.rs        # Content hashing
│       │   ├── store.rs       # Store trait
│       │   └── transaction/   # Transaction traits
│       ├── migration/         # Migration traits
│       │   ├── mod.rs
│       │   ├── traits.rs      # MigrateFrom/To
│       │   ├── chain.rs       # Multi-step migration
│       │   └── context.rs     # Migration context
│       └── registery/         # Type registry system
│           ├── mod.rs
│           ├── models/        # Model traits and types
│           │   ├── mod.rs
│           │   ├── model/     # NetabaseModel trait
│           │   ├── keys/      # Key type traits
│           │   └── treenames.rs # Table naming
│           ├── definition/    # NetabaseDefinition trait
│           │   ├── mod.rs
│           │   ├── schema.rs  # Schema metadata
│           │   ├── redb_definition.rs # Redb-specific
│           │   └── subscription/ # Pub/sub
│           └── repository/    # NetabaseRepository trait
```

---

## Type System

### Three-Layer Hierarchy

1. **Models** - Individual data structures
2. **Definitions** - Collections of related models
3. **Repositories** - Access boundaries across definitions

```rust
// Layer 1: Models
#[derive(NetabaseModel, ...)]
pub struct User {
    #[primary_key]
    pub id: String,  // → UserID(String)
    pub name: String,
}

// Layer 2: Definitions
#[netabase_definition(MyApp, repos(MainRepo, ...))]
mod my_app {
    pub struct User { ... }
    pub struct Post { ... }
}

// Layer 3: Repositories
#[netabase_repository(MainRepo, definitions(MyApp, OtherApp))]
mod main_repo {}
```

### Generated Types

For each model, the macro generates:

- **Primary Key Type**: `UserID` (newtype wrapper)
- **Keys Enum**: `UserKeys` with variants for each key type
- **Blob Keys Enum**: `UserBlobKeys` if model has blobs
- **Tree Names**: Const table name constants

### Key Types

```rust
pub enum UserKeys {
    Primary(UserID),
    Secondary(UserSecondaryKeys),
    Relational(UserRelationalKeys),
    Blob(UserBlobKeys),
    Subscription(UserSubscriptionKeys),
}

pub enum UserSecondaryKeys {
    Email(String),
    Age(u8),
}
```

---

## Transaction Architecture

### Transaction Lifecycle

```
Store → begin_read/write → Transaction → CRUD ops → commit/rollback
```

### Read vs Write Transactions

**Read Transactions**:
- Multiple concurrent readers
- Snapshot isolation
- No blocking
- Automatically rolled back on drop

**Write Transactions**:
- Exclusive access
- Full ACID guarantees
- Must commit explicitly
- Rollback on drop if not committed

### Transaction Implementation

```rust
pub enum RedbTransactionType<'txn, D> {
    Read(redb::ReadTransaction),
    Write(redb::WriteTransaction),
}

pub struct RedbTransaction<'db, D> {
    inner: RedbTransactionInner<'db, D>,
}

impl RedbTransaction {
    // CRUD operations
    pub fn create<M>(&self, model: &M) -> Result<()>
    pub fn read<M>(&self, id: &M::PrimaryKey) -> Result<Option<M>>
    pub fn update<M>(&self, model: &M) -> Result<()>
    pub fn delete<M>(&self, id: &M::PrimaryKey) -> Result<()>
}
```

---

## Migration System

### Version Families

Models are grouped into "families" that evolve together:

```rust
#[netabase_version(family = "User", version = 1)]
pub struct UserV1 { ... }

#[netabase_version(family = "User", version = 2, current)]
pub struct User { ... }
```

### Migration Traits

```rust
/// Upgrade from old version
impl MigrateFrom<UserV1> for User {
    fn migrate_from(old: UserV1) -> Self { ... }
}

/// Downgrade for P2P compatibility (optional)
impl MigrateTo<UserV1> for User {
    fn migrate_to(&self) -> UserV1 { ... }
}
```

### Migration Chain

Multi-version migrations are automatically chained:

```
UserV1 → UserV2 → UserV3 (current)
```

If a database contains V1 data, it's migrated V1→V2→V3 automatically.

### Detection Algorithm

1. **Probe Database**: Try to open tables with version suffixes
2. **Match Families**: Group detections by family name
3. **Compare Versions**: Check if current > detected
4. **Execute Migration**: Apply chain for each outdated family

---

## Blob Storage

### Why Blobs?

Large data (>60KB) degrades database performance. Blobs solve this by:

1. **Chunking**: Split into 60KB pieces
2. **Separate Tables**: Store chunks in dedicated blob tables
3. **Reconstruction**: Reassemble on read

### Implementation

```rust
#[derive(NetabaseBlobItem, Serialize, Deserialize)]
pub struct LargeFile {
    pub data: Vec<u8>,
    pub metadata: String,
}

#[derive(NetabaseModel, ...)]
pub struct Document {
    #[primary_key]
    pub id: String,
    
    #[blob]
    pub file: LargeFile,  // Automatically chunked
}
```

### Chunking Strategy

- **Chunk Size**: 60KB (60,000 bytes)
- **Serialization**: Full struct serialized, then chunked
- **Storage**: Each chunk stored with index: `(id, chunk_index) → bytes` TODO: it should actually be (OwnerID, BlobKey) -> (Bytes, index)
- **Retrieval**: All chunks fetched and concatenated
- **Reconstruction**: Deserialize reassembled bytes

---

## Repository Pattern

### Purpose

Repositories enforce **data graph completeness** at compile time.

### Problem

Without repositories:
```rust
// ❌ This compiles but might break at runtime
let user_link: RelationalLink<UserDef, User>;
let post = Post { author: user_link };  // Post in different definition!
```

### Solution

Repositories group definitions:
```rust
#[netabase_repository(MyRepo, definitions(UserDef, PostDef))]
mod my_repo {}

// ✅ Now links are type-checked
let user_link: RelationalLink<MyRepo, UserDef, UserDef, User>;
let post = Post { author: user_link };  // Verified at compile time
```

### Repository Isolation

Each repository gets its own store type:
```rust
let store: RedbRepositoryStore<MyRepo> = ...;
let txn = store.begin_write()?;
// Can only access models in MyRepo's definitions
```

---

## Serialization

### Why Postcard?

1. **No Schema**: Schema is in Rust types, not wire format
2. **Compact**: ~2-10x smaller than JSON
3. **Fast**: Zero-copy where possible
4. **Stable**: Variant encoding handles version skew
5. Bincode is no longer maintained.

### Serialization Path

```
Model → serde::Serialize → postcard::to_vec → Vec<u8> → redb
redb → &[u8] → postcard::from_bytes → serde::Deserialize → Model
```

### Special Cases

**RelationalLink**: Always serializes as dehydrated (key only)
```rust
// In memory: Owned(Box<User>), Hydrated(&User), or Borrowed(&User)
// On wire: Dehydrated(UserID)
```

**Blobs**: Serialized before chunking
```rust
LargeFile { data: [0u8; 200_000], ... }
→ postcard::to_vec() → Vec<u8>
→ chunk into 60KB pieces
→ store as [(0, bytes[0..60000]), (1, bytes[60000..120000]), ...]
```

---

## Performance Considerations

### Indexing Strategy

- **Primary Key**: B-tree index (always)
- **Secondary Keys**: Additional B-tree indexes
- **Relational Links**: Indexed by target ID for reverse lookups
- **Blobs**: Indexed by (model_id, chunk_index)

### Transaction Batching

Batch operations in a single transaction:
```rust
let txn = store.begin_write()?;
for item in items {
    txn.create(&item)?;  // Amortize transaction overhead
}
txn.commit()?;
```

### Read Optimization

Use secondary key queries when possible:
```rust
// ❌ Slow: Full scan
for user in all_users {
    if user.email == target { ... }
}

// ✅ Fast: Index lookup
let users = txn.query_by_secondary_key(&UserKeys::Secondary(
    UserSecondaryKeys::Email(target)
))?;
```

---

## Testing Strategy

### Unit Tests
- Per-module in `mod tests { }`
- Focus on isolated functionality

### Integration Tests
- In `tests/` directory
- End-to-end scenarios
- Migration paths

### Property Tests
- Fuzzing key operations
- Serialization round-trips

### Benchmark Tests
- In `benches/` (boilerplate crate)
- CRUD performance
- Migration performance
- Blob chunking overhead

---

## Future Directions

### Planned Features
- [ ] Query builder API
- [ ] Async transaction support
- [ ] IndexedDB backend for WASM
- [ ] Compression for blobs
- [ ] Multi-repository queries

### Under Consideration
- [ ] SQL-like query language
- [ ] Automatic index suggestions
- [ ] Distributed consensus
- [ ] Encryption at rest

---

## Contributing

When modifying the crate:

1. **Preserve type safety**: Don't weaken compile-time guarantees
2. **Document comprehensively**: User docs + implementation docs
3. **Add tests**: Unit + integration
4. **Benchmark critical paths**: Use criterion
5. **Update examples**: Keep boilerplate crate current

## See Also

- [User Guide](./boilerplate/GUIDE.md)
- [Examples README](./boilerplate/README.md)
- [API Documentation](https://docs.rs/netabase_store)
