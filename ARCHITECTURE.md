# Netabase Store - Architecture Overview

> **Type-safe embedded database for Rust with automatic migration and P2P synchronization**

## Table of Contents

- [Design Philosophy](#design-philosophy)
- [Core Concepts](#core-concepts)
- [Architecture Layers](#architecture-layers)
- [Type System](#type-system)
- [Feature System](#feature-system)
- [Backend Abstraction](#backend-abstraction)
- [Code Generation](#code-generation)
- [Data Flow](#data-flow)
- [Performance Model](#performance-model)

---

## Design Philosophy

### Principles

1. **Type Safety First**
   - Compile-time schema validation
   - No runtime schema errors
   - Type-safe relational links

2. **Zero-Cost Abstractions**
   - No runtime overhead for unused features
   - Monomorphization over dynamic dispatch
   - Inline-friendly design

3. **Ergonomic API**
   - Derive macros for minimal boilerplate
   - Prelude for common imports
   - Builder patterns for complex operations

4. **Feature Modularity**
   - Pay-only-for-what-you-use
   - Independent feature composition
   - Graceful degradation

5. **Backend Agnostic**
   - Trait-based storage abstraction
   - Multiple backend support
   - Testable without I/O

### Non-Goals

- ❌ Distributed consensus (use raft/paxos)
- ❌ SQL compatibility (use diesel/sqlx)
- ❌ ORM patterns (we're lower-level)
- ❌ Network transport (use libp2p separately)

---

## Core Concepts

### Definition

A **Definition** is a collection of related models that share a schema namespace.

```rust
#[netabase_definition(BlogDef)]
mod blog {
    #[derive(NetabaseModel)]
    pub struct Post { /* ... */ }
    
    #[derive(NetabaseModel)]
    pub struct Comment { /* ... */ }
}
```

**Purpose:**
- Groups related models
- Provides schema versioning boundary
- Enables cross-model queries
- Defines subscription topics

**Generated:**
- `BlogDef` enum wrapping all models
- `BlogDefDiscriminant` for pattern matching
- `BlogDefTreeNames` for table access
- `BlogDefKeys` for unified key handling

### Model

A **Model** is a single data type that can be stored in the database.

```rust
#[derive(NetabaseModel)]
pub struct User {
    #[primary_key]
    pub id: String,
    
    #[secondary_key]
    pub email: String,
    
    pub name: String,
}
```

**Requirements:**
- Exactly one `#[primary_key]`
- Must implement `Serialize + Deserialize`
- Must implement `Clone`

**Generated:**
- Wrapper type for primary key (`UserID`)
- Key enums for indexing
- CRUD trait implementations
- Serialization/deserialization

### Repository

A **Repository** groups definitions for access control.

```rust
#[netabase_repository(AppRepo)]
mod app {
    #[netabase_definition(UserDef, repos(AppRepo))]
    mod users { /* ... */ }
    
    #[netabase_definition(PostDef, repos(AppRepo))]
    mod posts { /* ... */ }
}
```

**Purpose:**
- Enforces compile-time isolation
- Enables cross-definition queries
- Manages migration boundaries

---

## Architecture Layers

```
┌──────────────────────────────────────────────────────────────┐
│                     User Application                         │
│  - Models defined with derive macros                         │
│  - Business logic using type-safe APIs                       │
└──────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────┐
│              Generated Code (netabase_macros)                │
│  - Trait implementations                                     │
│  - Type wrappers (UserID, UserKeys, etc.)                   │
│  - Schema metadata                                           │
└──────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────┐
│                   Core Traits Layer                          │
│  - NetabaseModel<D>                                          │
│  - NetabaseDefinition                                        │
│  - NetabaseRepository (optional)                             │
└──────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────┐
│                   Store Layer (NBStore)                      │
│  - Transaction management                                    │
│  - ACID guarantees                                           │
│  - Query execution                                           │
└──────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────┐
│                  Backend Abstraction                         │
│  - StorageBackend trait                                      │
│  - Read/Write transactions                                   │
│  - Iterator protocols                                        │
└──────────────────────────────────────────────────────────────┘
                           │
            ┌──────────────┼──────────────┐
            ▼              ▼              ▼
    ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
    │    Redb     │ │   Memory    │ │  IndexedDB  │
    │   Backend   │ │   Backend   │ │   Backend   │
    └─────────────┘ └─────────────┘ └─────────────┘
```

### Layer Responsibilities

#### User Application Layer
- Define models and schemas
- Implement business logic
- Handle errors and edge cases

#### Generated Code Layer
- Type safety enforcement
- Boilerplate elimination
- Schema metadata generation

#### Core Traits Layer
- Define contracts for models/definitions
- Enable generic programming
- Feature isolation

#### Store Layer
- Transaction lifecycle
- Query planning and execution
- Index management
- Cache coordination

#### Backend Abstraction Layer
- Unified storage interface
- Serialization/deserialization
- Iterator protocols

#### Backend Implementation Layer
- Actual storage (redb, memory, etc.)
- ACID properties
- Performance optimization

---

## Type System

### Type Flow for a Model

```
User Struct
    ↓
[NetabaseModel derive]
    ↓
Generated Types:
    - UserID(String)              // Primary key wrapper
    - UserEmail(String)           // Secondary key wrapper
    - UserKeys                    // Union of all keys
    - UserSecondaryKeys           // Secondary key enum
    - UserRelationalKeys          // Relational link enum
    - UserBlobKeys                // Blob field enum
    - UserSubscriptionKeys        // Subscription topic enum
    ↓
Trait Implementations:
    - NetabaseModel<BlogDef> for User
    - Serialize/Deserialize for User
    - RedbNetaseModel<BlogDef> for User
```

### Type Safety Guarantees

1. **Primary Key Uniqueness**
   ```rust
   UserID("alice")  ≠  PostID("alice")  // Different types!
   ```

2. **Relational Type Safety**
   ```rust
   #[link(BlogDef, User)]
   pub author: String;  // Must reference UserID
   ```

3. **Feature Availability**
   ```rust
   #[cfg(feature = "secondary_keys")]
   fn query_by_email(&self, email: &str) { }
   ```

4. **Repository Isolation**
   ```rust
   // Compile error: User from UserDef can't link to Post from PostDef
   // unless both in same repository
   ```

---

## Feature System

### Feature Matrix

| Feature          | Crate Flag | Macro Detection | Runtime Cost |
|------------------|------------|-----------------|--------------|
| `secondary_keys` | ✓          | ✓               | Low          |
| `relational_keys`| ✓          | ✓               | Low          |
| `blobs`          | ✓          | ✓               | Medium       |
| `subscriptions`  | ✓          | ✓               | Low          |
| `migration`      | ✓          | ✗               | None         |
| `repository`     | ✓          | ✗               | None         |
| `libp2p`         | ✓          | ✗               | High         |

### Feature Dependencies

```
repository
    └── (no dependencies)

migration
    └── toml (external)

libp2p
    ├── libp2p (external)
    └── subscriptions (optional)

relational_keys
    └── (no dependencies)

secondary_keys
    └── (no dependencies)

blobs
    └── (no dependencies)

subscriptions
    └── (no dependencies)
```

### Feature Interaction

- ✅ `relational_keys` + `blobs` - Links can reference models with blobs
- ✅ `secondary_keys` + `relational_keys` - Indexed foreign keys
- ✅ `migration` + `blobs` - Blobs migrate correctly
- ✅ `subscriptions` + `libp2p` - P2P pub/sub
- ⚠️ `repository` + `migration` - Migrations cross definitions carefully

---

## Backend Abstraction

### Storage Backend Trait

```rust
pub trait StorageBackend {
    type ReadTxn: ReadTransaction;
    type WriteTxn: WriteTransaction;
    
    fn begin_read(&self) -> Result<Self::ReadTxn>;
    fn begin_write(&self) -> Result<Self::WriteTxn>;
}

pub trait ReadTransaction {
    fn get(&self, table: &str, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn range(&self, table: &str, start: &[u8], end: &[u8]) -> Result<RangeIter>;
}

pub trait WriteTransaction: ReadTransaction {
    fn insert(&mut self, table: &str, key: &[u8], value: &[u8]) -> Result<()>;
    fn commit(self) -> Result<()>;
}
```

### Backend Implementations

#### Redb Backend (Production)
- **Pros**: ACID, fast, persistent, zero-copy
- **Cons**: Linux/macOS only, larger binary
- **Use Case**: Production deployments

#### Memory Backend (Testing)
- **Pros**: Fast, portable, deterministic
- **Cons**: Not persistent, no ACID guarantees
- **Use Case**: Unit tests, development

#### IndexedDB Backend (Future)
- **Pros**: Browser support, persistent
- **Cons**: Async API, browser-only
- **Use Case**: Web applications

---

## Code Generation

### Macro Processing Pipeline

```
User Code
    ↓
syn::parse (AST)
    ↓
Visitor Pattern
    ├── ModelVisitor (collects fields, attributes)
    ├── DefinitionVisitor (collects models)
    └── RepositoryVisitor (collects definitions)
    ↓
Feature Detection
    ├── has_secondary_keys?
    ├── has_blobs?
    ├── has_relational_links?
    └── has_subscriptions?
    ↓
Code Generators
    ├── generate_wrapper_types()
    ├── generate_key_enums()
    ├── generate_trait_impls()
    ├── generate_serialization()
    └── generate_crud_impls()
    ↓
quote! { ... } (Generate TokenStream)
    ↓
Rust Compiler
```

### Generated Code Size

**Without Optimization:**
- Simple model: ~500 lines generated
- Complex model: ~2000 lines generated

**With Feature Detection:**
- Simple model: ~200 lines generated (60% reduction)
- Complex model: ~1500 lines generated (25% reduction)

### Conditional Generation Example

```rust
// Input: Model with no secondary keys
#[derive(NetabaseModel)]
pub struct Counter {
    #[primary_key]
    pub id: String,
    pub value: u64,
}

// Generated (optimized):
pub enum CounterSecondaryKeys {
    #[doc(hidden)]
    __NoSecondaryKeys(())  // Minimal placeholder
}

// NOT generated:
// - Secondary key index logic
// - Secondary key query methods
// - Secondary key table definitions
```

---

## Data Flow

### Write Path

```
User Code: txn.create(&user)
    ↓
RedbModelCrud::create()
    ├── Serialize model (postcard)
    ├── Extract keys
    │   ├── Primary key → main table
    │   ├── Secondary keys → index tables
    │   ├── Relational keys → link tables
    │   ├── Blob keys → blob chunks
    │   └── Subscription keys → topic registry
    ├── Insert into tables
    └── txn.commit()
        └── Redb commit (fsync)
```

### Read Path

```
User Code: txn.read(&user_id)
    ↓
RedbModelCrud::read()
    ├── Lookup primary key
    ├── Get value bytes
    ├── Deserialize (postcard)
    ├── Reconstruct model
    │   └── Hydrate relational links (if requested)
    └── Return Option<User>
```

### Query Path

```
User Code: txn.list_with_config(config)
    ↓
Query Execution
    ├── Parse QueryConfig
    ├── Determine scan strategy
    │   ├── Full table scan
    │   ├── Secondary index range
    │   └── Subscription topic filter
    ├── Create iterator
    ├── Apply filters
    ├── Apply pagination
    ├── Deserialize results
    └── Return QueryResult<User>
```

---

## Performance Model

### Operation Costs

| Operation                | Complexity | Cost Factor        |
|--------------------------|------------|--------------------|
| Create (no features)     | O(1)       | 1x (baseline)      |
| Create (+ sec. index)    | O(log n)   | 1.5x               |
| Create (+ blob)          | O(n/chunk) | 2-10x (size-dep.)  |
| Read by primary key      | O(log n)   | 1x                 |
| Read by secondary key    | O(log n)   | 1.2x               |
| List (full scan)         | O(n)       | 1x per record      |
| List (indexed)           | O(k log n) | 0.8x per record    |
| Update (in-place)        | O(log n)   | 1.5x               |
| Delete (cascade)         | O(d log n) | 1.5x (d = depth)   |

### Memory Usage

**Minimal Model:**
```rust
struct Counter {
    id: String,      // 24 bytes (String)
    value: u64,      // 8 bytes
}
// Total: ~32 bytes + heap (id length)
```

**Full-Featured Model:**
```rust
struct User {
    id: String,                  // 24 bytes
    email: String,               // 24 bytes (secondary index)
    company: RelationalLink,     // 32 bytes
    avatar: Vec<u8>,             // 24 bytes + heap (blob)
    topics: Vec<String>,         // 24 bytes + heap (subscriptions)
}
// Total: ~128 bytes + heap
// On-disk: ~200 bytes + blob chunks
```

### Storage Overhead

- **No features**: ~1.1x serialized size
- **Secondary indexes**: +0.3x per index
- **Blobs**: +0.05x metadata (chunks separate)
- **Relational links**: +0.2x for link tables
- **Subscriptions**: +0.1x for topic registry

---

## Implementation Patterns

### Pattern: Type-Safe Keys

```rust
// Problem: String keys are error-prone
txn.read("alice")?;  // Is this a user or a post?

// Solution: Wrapper types
txn.read(&UserID("alice".into()))?;  // Clear!
```

### Pattern: Builder for Queries

```rust
// Instead of many parameters:
txn.list(Some(10), Some(offset), Some(filter), /* ... */)?;

// Use builder:
txn.list_with_config(
    QueryConfig::new()
        .limit(10)
        .offset(offset)
        .filter(filter)
)?;
```

### Pattern: Relational Hydration

```rust
// Dehydrated (just ID):
post.author  // RelationalLink<UserID>

// Hydrate when needed:
let author: User = post.author.hydrate(&txn)?;

// Or batch hydrate:
let posts = txn.list()?;
let with_authors = txn.hydrate_links(posts)?;
```

### Pattern: Migration Chain

```rust
// Version 1
#[netabase_version(family = "User", version = 1)]
struct UserV1 { /* ... */ }

// Version 2
#[netabase_version(family = "User", version = 2)]
struct UserV2 { /* ... */ }

impl MigrateFrom<UserV1> for UserV2 {
    fn migrate_from(old: UserV1) -> Result<Self> {
        // Transform data
    }
}
```

---

## Testing Strategy

### Unit Tests
- Individual trait implementations
- Serialization/deserialization
- Key extraction logic

### Integration Tests
- Full CRUD workflows
- Multi-table operations
- Feature combinations

### Property Tests
- Roundtrip serialization
- Query consistency
- Index correctness

### Benchmark Tests
- CRUD performance
- Query performance
- Feature overhead

### Example-Based Tests
- Documentation examples
- Real-world scenarios
- Migration paths

---

## Future Directions

### Planned Features

1. **Async Backend Support**
   - Tokio-based async transactions
   - IndexedDB backend
   - Network-backed stores

2. **Advanced Queries**
   - JOINs across models
   - Aggregations (COUNT, SUM, etc.)
   - Full-text search

3. **Replication**
   - Master-slave replication
   - CRDTs for conflict resolution
   - Merkle tree synchronization

4. **Compression**
   - zstd compression for blobs
   - Dictionary compression for models
   - Transparent decompression

5. **Encryption**
   - At-rest encryption
   - Field-level encryption
   - Key management

### Research Areas

- **Zero-copy deserialization** - Direct mmap access
- **Concurrent transactions** - MVCC or optimistic locking
- **Query optimization** - Cost-based query planning
- **Schema evolution** - Backward compatibility

---

## References

- [Redb Documentation](https://docs.rs/redb/)
- [Postcard Format](https://docs.rs/postcard/)
- [Serde Documentation](https://serde.rs/)
- [Rust Procedural Macros](https://doc.rust-lang.org/reference/procedural-macros.html)

---

## Glossary

- **Definition**: Logical grouping of related models
- **Model**: Single data type stored in database
- **Repository**: Access control boundary for definitions
- **Primary Key**: Unique identifier for model instance
- **Secondary Key**: Indexed field for fast lookups
- **Relational Link**: Type-safe reference to another model
- **Blob**: Large binary data stored separately
- **Subscription**: Topic-based pub/sub mechanism
- **Hydration**: Loading referenced models from links
- **Migration**: Transforming data between schema versions
- **Backend**: Storage implementation (redb, memory, etc.)
- **Transaction**: ACID-compliant database operation

---

*Last Updated: 2026-02-01*
*Version: 0.1.0*
