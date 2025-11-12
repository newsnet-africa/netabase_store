![crates.io](https://img.shields.io/crates/v/netabase_store.svg)
![crates.io downloads](https://img.shields.io/crates/d/netabase_store.svg) ![docs.rs](https://docs.rs/netabase_store/badge.svg)

# Netabase Store

A type-safe, multi-backend key-value storage library for Rust with support for native (Sled, Redb) and WASM (IndexedDB) environments.

> ⚠️ **Early Development**: This crate is still in early development and will change frequently as it stabilizes. It is not advised to use this in a production environment until it stabilizes.

## Features

### ✨ Core Features

- **🗄️ Multi-Backend Support**:
  - **Sled**: High-performance embedded database for native platforms
  - **Redb**: Memory-efficient embedded database with ACID guarantees
  - **IndexedDB**: Browser-based storage for WASM applications
  - **In-Memory**: Fast in-memory storage for testing and caching

- **🔒 Type-Safe Schema Definition**:
  - Derive macros for automatic schema generation
  - Primary and secondary key support
  - Compile-time type checking for all database operations
  - Zero-cost abstractions with trait-based design

- **🌍 Cross-Platform**:
  - Unified API across native and WASM targets
  - Feature flags for platform-specific backends
  - Seamless switching between backends

- **⚡ High Performance**:
  - Transaction API with type-state pattern (10-100x faster for bulk ops)
  - Batch operations for bulk inserts/updates
  - Efficient secondary key indexing
  - Minimal overhead (<5-10%) over raw backend operations
  - Zero-copy deserialization where possible

- **🔍 Secondary Key Indexing**:
  - Fast lookups using secondary keys
  - Multiple secondary keys per model
  - Automatic index management

- **🔄 Iteration Support**:
  - Efficient iteration over stored data
  - Type-safe iterators with proper error handling

- **🔗 libp2p Integration** (Optional):
  - Record store implementation for distributed systems
  - Compatible with libp2p DHT
  - Enable via `record-store` feature

- **🧪 Testing Utilities**:
  - Comprehensive test suite
  - Benchmarking tools included
  - WASM test support via wasm-pack

### 🔌 Extensibility

- **Unified Trait-Based API**:
  - `NetabaseTreeSync` for synchronous operations (native)
  - `NetabaseTreeAsync` for asynchronous operations (WASM)
  - Easy to implement custom backends
  - Full compatibility with existing code

- **Batch Processing**:
  - `Batchable` trait for atomic bulk operations
  - Significantly faster than individual operations
  - Backend-specific optimizations

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
netabase_store = "0.0.3"

# Required dependencies for macros to work
bincode = { version = "2.0", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
strum = { version = "0.27.2", features = ["derive"] }
derive_more = { version = "2.0.1", features = ["from", "try_into", "into"] }
anyhow = "1.0"  # Optional, for error handling

# For WASM support
[target.'cfg(target_arch = "wasm32")'.dependencies]
netabase_store = { version = "0.0.2", default-features = false, features = ["wasm"] }
```

### Feature Flags

- `native` (default): Enable Sled and Redb backends
- `sled`: Enable Sled backend only
- `redb`: Enable Redb backend only
- `wasm`: Enable IndexedDB backend for WASM
- `libp2p`: Enable libp2p integration
- `record-store`: Enable RecordStore trait (requires `libp2p`)

## Quick Start

### 1. Define Your Schema

```rust
use netabase_store::netabase_definition_module;
use netabase_store::traits::model::NetabaseModelTrait;

#[netabase_definition_module(BlogDefinition, BlogKeys)]
pub mod blog_schema {
    use netabase_store::{NetabaseModel, netabase};

    #[derive(NetabaseModel, bincode::Encode, bincode::Decode, Clone, Debug)]
    #[netabase(BlogDefinition)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub username: String,
        #[secondary_key]
        pub email: String,
    }

    #[derive(NetabaseModel, bincode::Encode, bincode::Decode, Clone, Debug)]
    #[netabase(BlogDefinition)]
    pub struct Post {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,
        #[secondary_key]
        pub author_id: u64,
    }
}

use blog_schema::*;
```

### 2. Use with NetabaseStore (Recommended)

The unified `NetabaseStore` provides a consistent API across all backends:

```rust
use netabase_store::NetabaseStore;

fn main() -> anyhow::Result<()> {
    // Create a store with any backend - easily switch by changing one line!

    // Option 1: Sled backend (high-performance)
    let store = NetabaseStore::<BlogDefinition, _>::sled("./my_db")?;

    // Option 2: Redb backend (memory-efficient, ACID)
    // let store = NetabaseStore::<BlogDefinition, _>::redb("./my_db.redb")?;

    // Option 3: Temporary store for testing
    // let store = NetabaseStore::<BlogDefinition, _>::temp()?;

    // Open a tree for users - works identically across all backends
    let user_tree = store.open_tree::<User>();

    // Insert a user
    let user = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    user_tree.put(user.clone())?;

    // Get by primary key
    let retrieved = user_tree.get(UserPrimaryKey(1))?.unwrap();
    assert_eq!(retrieved.username, "alice");

    // Query by secondary key
    let users_by_email = user_tree.get_by_secondary_key(
        UserSecondaryKeys::Email(UserEmailSecondaryKey("alice@example.com".to_string()))
    )?;
    assert_eq!(users_by_email.len(), 1);

    // Iterate over all users
    for result in user_tree.iter() {
        let (_key, user) = result?;
        println!("User: {} - {}", user.username, user.email);
    }

    // Access backend-specific features when needed
    store.flush()?; // Sled-specific method

    Ok(())
}
```

### 3. Direct Backend Usage (Advanced)

You can also use backends directly for backend-specific features:

```rust
use netabase_store::databases::sled_store::SledStore;
use netabase_store::databases::redb_store::RedbStore;

// Direct Sled usage
let sled_store = SledStore::<BlogDefinition>::temp()?;
let user_tree = sled_store.open_tree::<User>();

// Direct Redb usage
let redb_store = RedbStore::<BlogDefinition>::new("my_database.redb")?;
let user_tree = redb_store.open_tree::<User>();

// Both have identical APIs via NetabaseTreeSync trait
```

### 4. Use with IndexedDB (WASM)

```rust
use netabase_store::databases::indexeddb_store::IndexedDBStore;
use netabase_store::traits::tree::NetabaseTreeAsync;

#[cfg(target_arch = "wasm32")]
async fn wasm_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a store in the browser
    let store = IndexedDBStore::<BlogDefinition>::new("my_database").await?;

    // Note: WASM uses async API
    let user_tree = store.open_tree::<User>();

    let user = User {
        id: 1,
        username: "charlie".to_string(),
        email: "charlie@example.com".to_string(),
    };

    // All operations are async
    user_tree.put(user.clone()).await?;
    let retrieved = user_tree.get(user.primary_key()).await?;

    Ok(())
}
```

## Advanced Usage

### Batch Operations

For high-performance bulk operations:

```rust
use netabase_store::NetabaseStore;
use netabase_store::traits::batch::Batchable;

let store = NetabaseStore::<BlogDefinition, _>::temp()?;
let user_tree = store.open_tree::<User>();

// Create a batch
let mut batch = user_tree.create_batch()?;

// Add many operations
for i in 0..1000 {
    let user = User {
        id: i,
        username: format!("user{}", i),
        email: format!("user{}@example.com", i),
    };
    batch.put(user)?;
}

// Commit atomically - all or nothing
batch.commit()?;
```

Batch operations are:
- ⚡ **Faster**: 10-100x faster than individual operations
- 🔒 **Atomic**: All succeed or all fail
- 📦 **Efficient**: Reduced I/O and locking overhead

### Transactions (New!)

For maximum performance and atomicity, use the transaction API to reuse a single transaction across multiple operations:

```rust
use netabase_store::NetabaseStore;

let store = NetabaseStore::<BlogDefinition, _>::sled("./my_db")?;

// Read-only transaction - multiple concurrent reads allowed
let txn = store.read();
let user_tree = txn.open_tree::<User>();
let user = user_tree.get(UserPrimaryKey(1))?;
// Transaction auto-closes on drop

// Read-write transaction - exclusive access, atomic commit
let mut txn = store.write()?;
let mut user_tree = txn.open_tree::<User>();

// All operations share the same transaction
for i in 0..1000 {
    let user = User {
        id: i,
        username: format!("user{}", i),
        email: format!("user{}@example.com", i),
    };
    user_tree.put(user)?;
}

// Bulk helpers also work within transactions
user_tree.put_many(more_users)?;

// Commit all changes atomically
txn.commit()?;
// Or drop without committing to rollback
```

**Transaction Benefits:**
- 🚀 **10-100x Faster**: Single transaction for many operations (eliminates per-operation overhead)
- 🔒 **Type-Safe**: Compile-time enforcement of read-only vs read-write access
- ⚡ **Zero-Cost**: Phantom types compile away completely
- 🔄 **ACID**: Full atomicity for write transactions (Redb)

**Compile-Time Safety:**
```rust
let txn = store.read();  // ReadOnly transaction
let tree = txn.open_tree::<User>();
tree.put(user)?;  // ❌ Compile error: put() not available on ReadOnly!
```

### Secondary Keys

Secondary keys enable efficient lookups on non-primary fields:

```rust
#[derive(NetabaseModel, Clone, bincode::Encode, bincode::Decode)]
#[netabase(BlogDefinition)]
pub struct Article {
    #[primary_key]
    pub id: u64,
    pub title: String,
    #[secondary_key]
    pub category: String,
    #[secondary_key]
    pub published: bool,
}

// Query by category
let tech_articles = article_tree
    .get_by_secondary_key(
        ArticleSecondaryKeys::Category(
            ArticleCategorySecondaryKey("tech".to_string())
        )
    )?;

// Query by published status
let published = article_tree
    .get_by_secondary_key(
        ArticleSecondaryKeys::Published(
            ArticlePublishedSecondaryKey(true)
        )
    )?;
```

### Multiple Models in One Store

```rust
use netabase_store::NetabaseStore;

let store = NetabaseStore::<BlogDefinition, _>::sled("blog_db")?;

// Different trees for different models
let user_tree = store.open_tree::<User>();
let post_tree = store.open_tree::<Post>();

// Each tree is independent but shares the same underlying database
user_tree.put(user)?;
post_tree.put(post)?;
```

### Temporary Store for Testing

```rust
use netabase_store::NetabaseStore;

// Perfect for unit tests - no I/O, no cleanup needed
let store = NetabaseStore::<BlogDefinition, _>::temp()?;
let user_tree = store.open_tree::<User>();

user_tree.put(user)?;
```

## Custom Backend Implementation

Netabase Store's trait-based design makes it easy to implement custom storage backends. Here's what you need to know:

### Required Traits

To create a custom backend, implement one of these traits depending on your backend's characteristics:

#### 1. **`NetabaseTreeSync`** - For Synchronous Backends

Use this for native, blocking I/O backends (like SQLite, file systems, etc.):

```rust
use netabase_store::traits::tree::NetabaseTreeSync;
use netabase_store::traits::model::NetabaseModelTrait;
use netabase_store::traits::definition::NetabaseDefinitionTrait;
use netabase_store::error::NetabaseError;

pub struct MyCustomBackend<D, M> {
    // Your backend state (connection, file handles, etc.)
}

impl<D, M> NetabaseTreeSync<D, M> for MyCustomBackend<D, M>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + From<M>,
    M: NetabaseModelTrait<D> + TryFrom<D> + Clone,
    M::PrimaryKey: bincode::Encode + bincode::Decode<()> + Clone,
    M::SecondaryKeys: bincode::Encode + bincode::Decode<()>,
    // Add discriminant bounds...
{
    type PrimaryKey = M::PrimaryKey;
    type SecondaryKeys = M::SecondaryKeys;

    // Required: Insert or update a model
    fn put(&self, model: M) -> Result<(), NetabaseError> {
        // 1. Get primary key: model.primary_key()
        // 2. Get secondary keys: model.secondary_keys()
        // 3. Serialize model to bytes (use bincode or ToIVec trait)
        // 4. Store in your backend
        // 5. Create secondary key indexes
        todo!()
    }

    // Required: Retrieve by primary key
    fn get(&self, key: Self::PrimaryKey) -> Result<Option<M>, NetabaseError> {
        // 1. Serialize key to bytes
        // 2. Look up in your backend
        // 3. Deserialize bytes to model
        // 4. Return Some(model) or None
        todo!()
    }

    // Required: Delete by primary key
    fn remove(&self, key: Self::PrimaryKey) -> Result<Option<M>, NetabaseError> {
        // 1. Get the model (for cleanup)
        // 2. Delete primary key entry
        // 3. Delete secondary key indexes
        // 4. Return the deleted model
        todo!()
    }

    // Required: Query by secondary key
    fn get_by_secondary_key(
        &self,
        secondary_key: Self::SecondaryKeys
    ) -> Result<Vec<M>, NetabaseError> {
        // 1. Look up secondary key index
        // 2. Get all matching primary keys
        // 3. Retrieve all models with those keys
        // 4. Return vector of models
        todo!()
    }

    // Required: Check if empty
    fn is_empty(&self) -> Result<bool, NetabaseError> {
        Ok(self.len()? == 0)
    }

    // Required: Get count
    fn len(&self) -> Result<usize, NetabaseError> {
        todo!()
    }

    // Required: Delete all entries
    fn clear(&self) -> Result<(), NetabaseError> {
        todo!()
    }
}
```

#### 2. **`NetabaseTreeAsync`** - For Asynchronous Backends

Use this for async backends (remote databases, web APIs, etc.):

```rust
use netabase_store::traits::tree::NetabaseTreeAsync;
use netabase_store::error::NetabaseError;
use std::future::Future;

pub struct MyAsyncBackend<D, M> {
    // Your async backend state
}

impl<D, M> NetabaseTreeAsync<D, M> for MyAsyncBackend<D, M>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + From<M>,
    M: NetabaseModelTrait<D> + TryFrom<D> + Clone,
    // ... same bounds as sync version
{
    type PrimaryKey = M::PrimaryKey;
    type SecondaryKeys = M::SecondaryKeys;

    fn put(&self, model: M) -> impl Future<Output = Result<(), NetabaseError>> {
        async move {
            // Your async implementation
            todo!()
        }
    }

    fn get(
        &self,
        key: Self::PrimaryKey
    ) -> impl Future<Output = Result<Option<M>, NetabaseError>> {
        async move {
            todo!()
        }
    }

    // ... implement other methods with async
}
```

#### 3. **`OpenTree`** - For Store-Level API

Implement this on your store type to allow opening trees:

```rust
use netabase_store::traits::store_ops::OpenTree;

pub struct MyStore<D> {
    // Store state
}

impl<D> OpenTree<D> for MyStore<D>
where
    D: NetabaseDefinitionTrait,
{
    type Tree<M> = MyCustomBackend<D, M>
    where
        M: NetabaseModelTrait<D>;

    fn open_tree<M>(&self) -> Self::Tree<M>
    where
        M: NetabaseModelTrait<D> + TryFrom<D> + Into<D>,
    {
        // Create and return a tree instance for model M
        MyCustomBackend {
            // Initialize with store's connection/state
        }
    }
}
```

#### 4. **`Batchable`** (Optional) - For Batch Operations

If your backend supports atomic batching:

```rust
use netabase_store::traits::batch::{Batchable, BatchBuilder};

impl<D, M> Batchable<D, M> for MyCustomBackend<D, M>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + From<M>,
    M: NetabaseModelTrait<D> + TryFrom<D> + Clone,
{
    type Batch = MyBatch<D, M>;

    fn batch(&self) -> Self::Batch {
        MyBatch::new(/* ... */)
    }
}

pub struct MyBatch<D, M> {
    // Accumulate operations
}

impl<D, M> BatchBuilder<D, M> for MyBatch<D, M> {
    fn put(&mut self, model: M) -> Result<(), NetabaseError> {
        // Queue the operation
        todo!()
    }

    fn remove(&mut self, key: M::PrimaryKey) -> Result<(), NetabaseError> {
        // Queue the operation
        todo!()
    }

    fn commit(self) -> Result<(), NetabaseError> {
        // Execute all queued operations atomically
        todo!()
    }
}
```

### Implementation Tips

1. **Serialization**: Use `bincode` for efficient serialization:
   ```rust
   use bincode::{encode_to_vec, decode_from_slice, config::standard};

   let bytes = encode_to_vec(&model, standard())?;
   let (model, _) = decode_from_slice(&bytes, standard())?;
   ```

2. **Secondary Key Indexing**: Store composite keys:
   ```rust
   // Create composite key: secondary_key_bytes + primary_key_bytes
   let mut composite = secondary_key_bytes;
   composite.extend_from_slice(&primary_key_bytes);
   ```

3. **Error Handling**: Convert your backend errors to `NetabaseError`:
   ```rust
   use netabase_store::error::NetabaseError;

   my_backend_op().map_err(|e|
       NetabaseError::Storage(format!("Backend error: {}", e))
   )?;
   ```

4. **Iterator Support**: Implement iterators for efficient traversal:
   ```rust
   fn iter(&self) -> impl Iterator<Item = Result<(M::PrimaryKey, M), NetabaseError>> {
       // Return an iterator over all entries
   }
   ```

### Complete Example

See the existing backends for reference:
- **`sled_store.rs`**: Example of sync backend with batch support
- **`redb_store.rs`**: Example of transactional backend
- **`indexeddb_store.rs`**: Example of async WASM backend
- **`memory_store.rs`**: Simple in-memory implementation

All existing code will work with your custom backend once you implement the traits!

## Performance

Netabase Store is designed for high performance while maintaining type safety. The library provides a unified API across multiple backends, with carefully measured overhead characteristics.

### Performance Characteristics

#### Redb Backend

The Redb backend uses bincode re-serialization on read paths due to Rust's type system limitations with Generic Associated Types (GATs). This affects read operations but not writes:

| Operation | Wrapper Time (100 items) | Raw Time (100 items) | Overhead | Impact |
|-----------|-------------------------|---------------------|----------|---------|
| **Insert** | 2.21 ms | 343 µs | ~6.4x | Write operations include transaction overhead |
| **Get** | 68.2 µs | 10.3 µs | ~6.6x | Read operations include deserialize+serialize |
| **Iteration** | 15.0 µs | 8.5 µs | ~1.8x | Iteration has lower relative overhead |
| **Secondary Key** | 14.1 µs | 2.6 µs | ~5.4x | Secondary lookups include index traversal |

**Key Points:**
- Write operations (`put`, `remove`) use zero-copy serialization
- Read operations (`get`, `iter`) include re-serialization overhead for type safety
- The overhead is a safety tradeoff: we prioritize type correctness over unsafe transmutes
- For bulk operations, use transactions to amortize overhead across many operations

#### Sled Backend

The Sled backend has significantly lower overhead due to different internal data structures:

| Operation | Wrapper Time (100 items) | Raw Time (100 items) | Overhead | Impact |
|-----------|-------------------------|---------------------|----------|---------|
| **Insert** | 1.73 ms | 474 µs | ~3.6x | Transaction and index management overhead |
| **Get** | 23.8 µs | 19.6 µs | ~1.2x | Minimal deserialization overhead |
| **Iteration** | 21.9 µs | 17.7 µs | ~1.2x | Very efficient iteration |

**Key Points:**
- Sled has much lower overhead than Redb for read operations (~1.2x vs ~6.6x)
- Write overhead is moderate due to secondary index management
- Excellent choice for read-heavy workloads

### Performance Optimization Tips

1. **Use Transactions for Bulk Operations** (10-100x speedup):
   ```rust
   let mut txn = store.write()?;
   let mut tree = txn.open_tree::<User>();
   for i in 0..1000 {
       tree.put(user)?;  // Shares single transaction
   }
   txn.commit()?;  // Atomic commit
   ```

2. **Choose Backend Based on Workload**:
   - **Sled**: Best for read-heavy workloads (1.2x read overhead)
   - **Redb**: Best for write-heavy workloads with ACID guarantees

3. **Batch Operations**: Use when atomicity is needed:
   ```rust
   let mut batch = tree.create_batch()?;
   batch.put_many(users)?;
   batch.commit()?;
   ```

### Benchmarks

Run benchmarks to measure performance on your hardware:

```bash
# Sled benchmarks
cargo bench --bench sled_wrapper_overhead --features "sled,libp2p"

# Redb benchmarks
cargo bench --bench redb_wrapper_overhead --features "redb,libp2p"
```

Benchmark categories:
- Insert performance (with secondary index management)
- Get performance (by primary key)
- Iteration performance (full table scan)
- Secondary key lookup performance

### Technical Note: Why Re-serialization?

The read-path overhead in Redb comes from a Rust type system limitation with Generic Associated Types (GATs). While we define `type SelfType<'a> = Self` in our `redb::Value` implementations, the compiler cannot prove at call sites that `<T as Value>::SelfType<'_>` equals `T`. This prevents us from using `.clone()` or safe zero-cost coercion.

We explored several alternatives:
- **Unsafe transmute**: Rejected for safety reasons
- **Custom trait bounds**: Failed due to trait composition issues
- **Direct `.clone()` calls**: Compiler cannot prove type equality

The current approach prioritizes safety and correctness over raw performance. For applications where this overhead is significant, we recommend:
1. Using the Sled backend (1.2x read overhead instead of 6.6x)
2. Using transactions to amortize overhead across many operations
3. Profiling your specific workload to verify if the overhead matters in practice

Future work may explore nightly features or upstream redb API changes to eliminate this limitation.

## Testing

```bash
# Run all tests
cargo test --all-features

# Run native tests only
cargo test --features native

# Run WASM tests (requires wasm-pack and Firefox)
wasm-pack test --headless --firefox --features wasm
```

## Architecture

See [ARCHITECTURE.md](./ARCHITECTURE.md) for a deep dive into the library's design.

### High-Level Overview

```
┌───────────────────────────────────────────────────┐
│         Your Application Code                     │
│      (Type-safe models with macros)              │
└───────────────────────────────────────────────────┘
                      │
                      ↓
┌───────────────────────────────────────────────────┐
│         NetabaseStore<D, Backend>                 │
│    (Unified API layer - Recommended)             │
└───────────────────────────────────────────────────┘
                      │
        ┌─────────────┼─────────────┐
        ↓             ↓             ↓
┌─────────────┐ ┌─────────────┐ ┌─────────────┐
│  SledStore  │ │  RedbStore  │ │IndexedDBStore│
│   <D>       │ │   <D>       │ │    <D>       │
└─────────────┘ └─────────────┘ └─────────────┘
        │             │             │
        ↓             ↓             ↓
┌─────────────────────────────────────────────────┐
│         Trait Layer                             │
│  (NetabaseTreeSync, NetabaseTreeAsync)         │
│  (OpenTree, Batchable, StoreOps)               │
└─────────────────────────────────────────────────┘
        │             │             │
        ↓             ↓             ↓
    ┌─────┐      ┌──────┐     ┌─────────┐
    │ Sled│      │ Redb │     │IndexedDB│
    └─────┘      └──────┘     └─────────┘
    Native       Native        WASM
```

## Roadmap

### For 1.0.0

- [x] Transaction support across multiple operations (**COMPLETED**)
- [ ] Zero-copy reads for redb backend (via `redb-zerocopy` feature) - **Phase 1 Complete**
- [ ] Allow modules to define more than one definition for flexible organization
- [ ] Migration utilities for schema changes
- [ ] Query builder for complex queries
- [ ] Range queries on ordered keys
- [ ] Compression support
- [ ] Encryption at rest
- [ ] Improved documentation and examples

### Future Plans

- [ ] Distributed systems support with automatic sync
- [ ] CRDT-based conflict resolution
- [ ] WebRTC backend for peer-to-peer storage
- [ ] SQL-like query language
- [ ] GraphQL integration

## Examples

See the [`test_netabase_store_usage`](../test_netabase_store_usage) crate for a complete working example.

Additional examples in the repository:
- `examples/basic_store.rs` - Basic CRUD operations
- `examples/unified_api.rs` - Working with multiple backends
- `tests/wasm_tests.rs` - WASM usage patterns

## Why Netabase Store?

### Problem

Working with different database backends in Rust typically means:
- Learning different APIs for each backend
- No type safety for keys and values
- Manual serialization/deserialization
- Difficulty switching backends
- Complex secondary indexing

### Solution

Netabase Store provides:
- ✅ Single unified API across all backends
- ✅ Compile-time type safety for everything
- ✅ Automatic serialization with bincode
- ✅ Seamless backend switching
- ✅ Automatic secondary key management
- ✅ Cross-platform support (native + WASM)

## Contributing

Contributions are welcome! Please:

1. Open an issue to discuss major changes
2. Follow the existing code style
3. Add tests for new features
4. Update documentation

## License

This project is licensed under the GPL-3.0-only License - see the LICENSE file for details.

## Links

- [Documentation](https://docs.rs/netabase_store)
- [Crates.io](https://crates.io/crates/netabase_store)
- [Repository](https://github.com/newsnet-africa/netabase_store)
- [Issue Tracker](https://github.com/newsnet-africa/netabase_store/issues)

## Acknowledgments

Built with:
- [Sled](https://github.com/spacejam/sled) - Embedded database
- [Redb](https://github.com/cberner/redb) - Embedded database
- [IndexedDB](https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API) - Browser storage
- [Bincode](https://github.com/bincode-org/bincode) - Binary serialization
