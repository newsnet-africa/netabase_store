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
  - **RedbZeroCopy**: Zero-copy variant for maximum performance (10-54x faster for bulk ops)
  - **IndexedDB**: Browser-based storage for WASM applications

- **⚙️ Unified Configuration API**:
  - `FileConfig`, `IndexedDBConfig` with builder pattern
  - Consistent initialization across all backends
  - Switch backends by changing one line of code
  - Type-safe configuration with sensible defaults

- **🔒 Type-Safe Schema Definition**:
  - Derive macros for automatic schema generation
  - Primary and secondary key support
  - Compile-time type checking for all database operations
  - Zero-cost abstractions with trait-based design

- **🌍 Cross-Platform**:
  - Unified API across native and WASM targets
  - Feature flags for platform-specific backends
  - Seamless switching between backends with same configuration

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
[package]
name = "my_project"
version = "0.1.0"
edition = "2021"

# Features must be enabled in your crate for macro-generated code
[features]
default = ["native"]
native = ["netabase_store/native"]

[dependencies]
netabase_store = { version = "0.0.6", features = ["native"] }

# Required dependencies
bincode = { version = "2.0", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
strum = { version = "0.27.2", features = ["derive"] }
derive_more = { version = "2.0.1", features = ["from", "try_into", "into"] }
libp2p = "0.56" # Optional, if you would like to use this as a persistent backend for [`libp2p-kad` RecordStore implementation](https://docs.rs/libp2p/latest/libp2p/kad/index.html)
anyhow = "1.0"
```

### Feature Flags

**The default feature set includes `native` which provides both Sled and Redb backends. You can customize features for your specific needs.**

#### Backend Features:
- `native` - **(Recommended)** Enables both Sled and Redb backends for desktop/server
- `sled` - Sled backend only (high-performance embedded database)
- `redb` - Redb backend only (memory-efficient, ACID compliant)
- `redb-zerocopy` - Zero-copy Redb variant (maximum performance, requires `redb`)
- `wasm` - IndexedDB backend for browser/WASM applications


#### Integration Features:
- `libp2p` - Enable libp2p integration for distributed systems
- `record-store` - Enable RecordStore trait (requires `libp2p`)

#### Common Configurations:

```toml
# For desktop/server applications (recommended):
netabase_store = { version = "0.0.6", features = ["native"] }

# For WASM/browser applications:
[target.'cfg(target_arch = "wasm32")'.dependencies]
netabase_store = { version = "0.0.6", default-features = false, features = ["wasm"] }

# For specific backend only:
netabase_store = { version = "0.0.6", features = ["sled"] }

# For zero-copy redb optimization:
netabase_store = { version = "0.0.6", features = ["redb-zerocopy"] }

# For libp2p integration:
netabase_store = { version = "0.0.6", features = ["native", "libp2p"] }
```

## Quick Start

### 1. Define Your Schema

```rust
use netabase_store::netabase_definition_module;
use netabase_store::traits::model::NetabaseModelTrait;

#[netabase_definition_module(BlogDefinition, BlogKeys)]
pub mod blog_schema {
    use netabase_store::{NetabaseModel, netabase};

    #[derive(NetabaseModel, bincode::Encode, bincode::Decode, Clone, Debug, serde::Serialize, serde::Deserialize)]
    #[netabase(BlogDefinition)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub username: String,
        #[secondary_key]
        pub email: String,
    }

    #[derive(NetabaseModel, bincode::Encode, bincode::Decode, Clone, Debug, serde::Serialize, serde::Deserialize)]
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

    // Get by primary key - use the generated type
    let retrieved = user_tree.get(UserPrimaryKey(1))?.unwrap();
    assert_eq!(retrieved.username, "alice");

    // Alternative: use primary_key() method
    let retrieved2 = user_tree.get(user.primary_key())?.unwrap();
    assert_eq!(retrieved2.username, "alice");

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

### Backend Comparison & Selection Guide

All backends share the same core API through traits, but have different characteristics:

#### 🔹 **Sled** (Recommended for Most Cases)
- **Type**: Native, persistent, sync
- **Use Cases**: General-purpose applications, desktop apps, servers
- **Strengths**: 
  - Excellent write performance
  - Crash-safe with recovery
  - Mature and battle-tested
  - Simple file-based storage
- **API Example**: 
  ```rust
  let store = NetabaseStore::<BlogDefinition, _>::sled("./my_db")?;
  let tree = store.open_tree::<User>();
  tree.put(user)?; // Synchronous operations
  ```

#### 🔹 **Redb** (Optimized for Reads)
- **Type**: Native, persistent, sync
- **Use Cases**: Read-heavy workloads, embedded databases
- **Strengths**:
  - Zero-copy reads (fastest read performance)
  - Memory-efficient
  - ACID compliant
  - Smaller file size than Sled
- **API Example**:
  ```rust
  let store = NetabaseStore::<BlogDefinition, _>::redb("./my_db.redb")?;
  let tree = store.open_tree::<User>();
  tree.put(user)?; // Synchronous operations
  ```

#### 🔹 **RedbZeroCopy** (Advanced, Explicit Transactions)
- **Type**: Native, persistent, explicit transactions
- **Use Cases**: Maximum performance, explicit transaction control
- **Strengths**:
  - Zero-copy reads
  - Explicit transaction API for fine-grained control
  - Best performance for bulk operations
- **API Difference**: Requires explicit transaction management
  ```rust
  use netabase_store::databases::redb_zerocopy::RedbStoreZeroCopy;
  let store = RedbStoreZeroCopy::<BlogDefinition>::new("./my_db.redb")?;
  
  // Must use explicit transactions
  let mut txn = store.begin_write()?;
  let mut tree = txn.open_tree::<User>()?;
  tree.put(user)?;
  drop(tree);
  txn.commit()?; // Must explicitly commit
  ```



#### 🔹 **IndexedDB** (Browser/WASM)
- **Type**: WASM, persistent, async
- **Use Cases**: Web applications, browser-based storage
- **Strengths**:
  - Native browser storage
  - Persistent across sessions
  - Standard web API
- **API Difference**: All operations are async
  ```rust
  let store = IndexedDBStore::<BlogDefinition>::new("my_db").await?;
  let tree = store.open_tree::<User>();
  tree.put(user).await?; // Note: async operations
  let result = tree.get(user.primary_key()).await?;
  ```

#### Quick Selection Guide

| Backend | Persistence | Async | Best For | Avoid If |
|---------|-------------|-------|----------|----------|
| **Sled** | ✅ Disk | ❌ Sync | General purpose, high writes | WASM target |
| **Redb** | ✅ Disk | ❌ Sync | Read-heavy, low memory | Need fastest writes |
| **RedbZeroCopy** | ✅ Disk | ❌ Sync | Bulk ops, transaction control | Want simple API |
| **IndexedDB** | ✅ Browser | ✅ Async | Web/WASM apps | Native targets |

**Performance Notes**:
- All backends support batch operations (10-100x faster for bulk inserts)
- All backends support secondary key queries
- All backends (except RedbZeroCopy) support the transaction API: `store.read()` and `store.write()`
- Sled and Redb have similar performance, with Redb slightly faster for reads
- For testing, use temp() methods for fast, isolated tests

**API Compatibility**:
- ✅ All sync backends (Sled, Redb) have identical APIs
- ✅ RedbZeroCopy requires explicit transaction management
- ⚠️ IndexedDB uses async (`.await`) for all operations
- ✅ All backends support the same data models and secondary keys

## Advanced Usage

### Configuration API

The new unified configuration system provides consistent backend initialization across all database types:

#### FileConfig - For File-Based Backends

```rust
use netabase_store::config::FileConfig;
use netabase_store::traits::backend_store::BackendStore;
use netabase_store::databases::sled_store::SledStore;

// Method 1: Builder pattern (recommended)
let config = FileConfig::builder()
    .path("app_data.db".into())
    .cache_size_mb(1024)
    .truncate(true)
    .build();

let store = <SledStore<BlogDefinition> as BackendStore<BlogDefinition>>::new(config)?;

// Method 2: Simple constructor
let config = FileConfig::new("app_data.db");
let store = <SledStore<BlogDefinition> as BackendStore<BlogDefinition>>::open(config)?;

// Method 3: Temporary database
let store = <SledStore<BlogDefinition> as BackendStore<BlogDefinition>>::temp()?;
```

#### Switching Backends with Same Config

The power of the configuration API is that you can switch backends without changing your code:

```rust
use netabase_store::config::FileConfig;
use netabase_store::traits::backend_store::BackendStore;

let config = FileConfig::builder()
    .path("my_app.db".into())
    .cache_size_mb(512)
    .build();

// Try different backends - same config!
#[cfg(feature = "sled")]
let store = <SledStore<BlogDefinition> as BackendStore<BlogDefinition>>::new(config.clone())?;

#[cfg(feature = "redb")]
let store = <RedbStore<BlogDefinition> as BackendStore<BlogDefinition>>::new(config.clone())?;

#[cfg(feature = "redb-zerocopy")]
let store = <RedbStoreZeroCopy<BlogDefinition> as BackendStore<BlogDefinition>>::new(config)?;

// All have the same API from this point on!
let user_tree = store.open_tree::<User>();
```

#### Configuration Options Reference

**FileConfig** (for Sled, Redb, RedbZeroCopy):
- `path: PathBuf` - Database file/directory path
- `cache_size_mb: usize` - Cache size in megabytes (default: 256)
- `create_if_missing: bool` - Create if doesn't exist (default: true)
- `truncate: bool` - Delete existing data (default: false)
- `read_only: bool` - Open read-only (default: false)
- `use_fsync: bool` - Fsync for durability (default: true)



**IndexedDBConfig** (for WASM):
- `database_name: String` - IndexedDB database name
- `version: u32` - Schema version (default: 1)

### Bulk Operations with Transactions

For high-performance bulk operations, use the **transaction API** (10-100x faster than individual operations):

```rust
use netabase_store::NetabaseStore;

let store = NetabaseStore::<BlogDefinition, _>::sled("./my_db")?;

// Create a write transaction for bulk operations
// NOTE: write() returns TxnGuard directly, not a Result
let mut txn = store.write();
let mut user_tree = txn.open_tree::<User>();

// Bulk insert - 8-9x faster than individual puts!
let users: Vec<User> = (0..1000)
    .map(|i| User {
        id: i,
        username: format!("user{}", i),
        email: format!("user{}@example.com", i),
    })
    .collect();

// All inserts in a single transaction
user_tree.put_many(users)?;

// Bulk read within transaction
let keys: Vec<UserPrimaryKey> = (0..100).map(UserPrimaryKey).collect();
let users: Vec<Option<User>> = user_tree.get_many(keys)?;

// Commit all changes atomically
txn.commit()?;
```

**Transaction Methods:**
- `put_many(Vec<M>)` - Insert multiple models in one transaction
- `get_many(Vec<M::PrimaryKey>)` - Read multiple models in one transaction

**Backend Support:**
- **Sled**: ✅ Full support via transactions and batch API
- **Redb**: ✅ Full support via transactions and batch API  
- Both backends provide identical API and performance benefits

**Performance Benefits:**
- ⚡ **10-100x faster** than individual operations
- 🔒 **Atomic**: All succeed or all fail
- 📦 **Efficient**: Single transaction reduces overhead

For more examples, see `examples/batch_operations_all_backends.rs`

**Or use the batch API for more control:**

```rust
use netabase_store::traits::batch::Batchable;

// Create a batch
let mut batch = user_tree.create_batch()?;

// Add many operations
for i in 0..1000 {
    batch.put(User { /* ... */ })?;
}

// Commit atomically - all or nothing
batch.commit()?;
```

The batch API provides fine-grained control and is supported on both sync backends (Sled, Redb).

### Transactions (New!)

For maximum performance and atomicity, use the transaction API to reuse a single transaction across multiple operations:

```rust
use netabase_store::NetabaseStore;

let store = NetabaseStore::<BlogDefinition, _>::sled("./my_db")?;

// Read-only transaction - multiple concurrent reads allowed
// NOTE: read() and write() return guards directly, not Results
let txn = store.read();
let user_tree = txn.open_tree::<User>();
let user = user_tree.get(UserPrimaryKey(1))?;
// Transaction auto-closes on drop

// Read-write transaction - exclusive access, atomic commit
let mut txn = store.write();
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

**Backend Support Notes:**
- **Sled, Redb**: Full support for `store.read()` and `store.write()` transaction API
- **RedbZeroCopy**: Uses explicit `store.begin_write()` and `txn.commit()` pattern (different API)
- **IndexedDB**: Async operations, transactions handled internally by browser
- See `examples/transactions.rs` for detailed examples

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

// Query by single secondary key
let tech_articles = article_tree
    .get_by_secondary_key(
        ArticleSecondaryKeys::Category(
            ArticleCategorySecondaryKey("tech".to_string())
        )
    )?;

// Bulk query multiple secondary keys (2-3x faster!)
let keys = vec![
    ArticleSecondaryKeys::Category(ArticleCategorySecondaryKey("tech".to_string())),
    ArticleSecondaryKeys::Category(ArticleCategorySecondaryKey("science".to_string())),
];
let results: Vec<Vec<Article>> = article_tree.get_many_by_secondary_keys(keys)?;
// results[0] = tech articles, results[1] = science articles
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
All existing code will work with your custom backend once you implement the traits!

## Performance

Netabase Store is designed for high performance while maintaining type safety. The library provides multiple APIs optimized for different use cases, with comprehensive benchmarking and profiling support.

### API Options for Performance

The library offers three APIs with different performance characteristics:

1. **Standard Wrapper API**: Simple, ergonomic API with auto-transaction per operation
2. **Bulk Methods**: `put_many()`, `get_many()`, `get_many_by_secondary_keys()` - single transaction for multiple items
3. **ZeroCopy API**: Explicit transaction management for maximum control

### Benchmark Results

Comprehensive benchmarks comparing all implementations across multiple dataset sizes (10, 100, 500, 1000, 5000 items):

#### Insert Performance (1000 items)

| Implementation | Time | vs Raw | Notes |
|----------------|------|--------|-------|
| Raw Redb (baseline) | 1.42 ms | 0% | Single transaction, manual index management |
| Wrapper Redb (bulk) | 3.10 ms | +118% | `put_many()` - single transaction |
| Wrapper Redb (loop) | 27.3 ms | +1,822% | Individual `put()` calls - creates N transactions |
| ZeroCopy (bulk) | 3.51 ms | +147% | `put_many()` with explicit transaction |
| ZeroCopy (loop) | 4.34 ms | +206% | Loop with single explicit transaction |

**Key Insights:**
- **Bulk methods provide 8-9x speedup** over loop-based insertion (27.3ms → 3.10ms)
- Bulk wrapper API approaches raw performance (118% overhead vs 1,822% for loops)
- Transaction overhead dominates when creating N transactions vs 1 transaction

#### Read Performance (1000 items)

| Implementation | Time | vs Raw | Notes |
|----------------|------|--------|-------|
| Raw Redb (baseline) | 164 µs | 0% | Single transaction |
| Wrapper Redb (bulk) | 382 µs | +133% | `get_many()` - single transaction |
| Wrapper Redb (loop) | 895 µs | +446% | Individual `get()` calls - creates N transactions |
| ZeroCopy (single txn) | 692 µs | +322% | Explicit read transaction |

**Key Insights:**
- **Bulk `get_many()` provides 2.3x speedup** over individual gets (895µs → 382µs)
- Transaction reuse is critical for read performance
- Even bulk methods have overhead due to transaction and deserialization costs

#### Secondary Key Queries (10 queries)

| Implementation | Time | vs Raw | Notes |
|----------------|------|--------|-------|
| Raw Redb (baseline) | 291 µs | 0% | 10 transactions, manual index traversal |
| Wrapper Redb (bulk) | 470 µs | +61% | `get_many_by_secondary_keys()` - single transaction |
| Wrapper Redb (loop) | 1.02 ms | +248% | 10 separate `get_by_secondary_key()` calls |
| ZeroCopy (single txn) | 5.41 µs | **-98%** | Single transaction, optimized index access |

**Key Insights:**
- **ZeroCopy API is 54x faster** than raw redb for secondary queries (291µs → 5.4µs)
- Bulk secondary query method provides 2.2x speedup over loops
- Single transaction + efficient index access = dramatic performance gains

### Performance Optimization Guide

#### 1. Use Bulk Methods for Standard API (8-9x faster)

```rust
// ❌ Slow: Creates 1000 transactions
for user in users {
    tree.put(user)?;  // Each call = new transaction
}

// ✅ Fast: Single transaction
tree.put_many(users)?;  // 8-9x faster!
```

**Available Bulk Methods:**
- `put_many(Vec<M>)` - Bulk insert
- `get_many(Vec<M::Keys>)` - Bulk read
- `get_many_by_secondary_keys(Vec<SecondaryKey>)` - Bulk secondary queries

#### 2. Use Explicit Transactions for Maximum Control

```rust
// For write-heavy workloads
// NOTE: write() returns TxnGuard directly, not a Result
let mut txn = store.write();
let mut tree = txn.open_tree::<User>();

for user in users {
    tree.put(user)?;  // All share same transaction
}

txn.commit()?;  // Single atomic commit
```

#### 3. Choose the Right API for Your Use Case

| Use Case | Recommended API | Reason |
|----------|----------------|--------|
| Simple CRUD, few operations | Standard wrapper | Simplest API, auto-commit |
| Bulk inserts/reads (100+ items) | Bulk methods | 8-9x faster than loops |
| Complex transactions | Explicit transactions | Full control, atomic commits |
| Read-heavy queries | ZeroCopy API | Up to 54x faster for secondary queries |

### Profiling Support

The benchmarks include full profiling support via pprof and flamegraphs:

```bash
# Run benchmarks with profiling
cargo bench --bench cross_store_comparison --features native

# Analyze profiling data
./scripts/analyze_profiling.sh

# View flamegraphs (SVG files in target/criterion/)
firefox target/criterion/cross_store_insert/wrapper_redb_bulk/profile/flamegraph.svg
```

**Flamegraphs show:**
- Function call stacks and time distribution
- Serialization overhead (bincode operations)
- Transaction costs (redb internal operations)
- Memory allocation patterns
- Lock contention (if any)

### Running Benchmarks

```bash
# Cross-store comparison (all backends, multiple sizes)
cargo bench --bench cross_store_comparison --features native

# Generate visualizations
uv run scripts/generate_benchmark_charts.py

# View results
open docs/benchmarks/insert_comparison_bars.png
open docs/benchmarks/overhead_percentages.png
open docs/benchmarks/bulk_api_speedup.png
```

### Backend Comparison

#### Redb
- **Best for**: Write-heavy workloads, ACID guarantees
- **Wrapper overhead**: 118-133% for bulk operations
- **Strengths**: Excellent write performance, full ACID compliance, efficient storage
- **Use when**: Data integrity is critical, write performance matters

#### Sled
- **Best for**: Read-heavy workloads
- **Wrapper overhead**: ~20% for read operations
- **Strengths**: Very low read overhead, battle-tested
- **Use when**: Read performance is critical, workload is read-heavy

### Technical Notes

#### Why Transaction Overhead Matters

Creating a new transaction has fixed costs:
- Lock acquisition
- MVCC snapshot creation
- Internal state setup

When you call `put()` in a loop, you pay these costs N times. Using `put_many()` or explicit transactions, you pay once.

#### Type Safety vs Performance

The wrapper APIs prioritize type safety and ergonomics. For applications where the overhead is significant:
1. **Use bulk methods first** - often solves the problem
2. **Use explicit transactions** - full control with same safety
3. **Profile your workload** - measure before optimizing
4. **Consider ZeroCopy API** - for specialized high-performance scenarios

#### Serialization Overhead

The read-path overhead in Redb comes from type system limitations with Generic Associated Types (GATs). We prioritize safety over unsafe transmutes. For applications where this matters:
- Use bulk methods to amortize overhead
- Use explicit transactions for better performance
- Consider Sled backend for read-heavy workloads

See benchmark results and visualizations in `docs/benchmarks/` for detailed performance analysis.

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

### Core Examples
- `examples/basic_store.rs` - Basic CRUD operations with Sled
- `examples/unified_api.rs` - Working with the NetabaseStore unified API
- `examples/config_api_showcase.rs` - Configuration system and backend switching



### Single-Backend Examples
- `examples/batch_operations.rs` - Batch operations (Sled-focused)
- `examples/transactions.rs` - Transaction API (Sled-focused)
- `examples/redb_basic.rs` - Redb-specific features
- `examples/redb_zerocopy.rs` - RedbZeroCopy explicit transaction API
- `examples/subscription_demo.rs` - Change notification system
- `examples/subscription_streams.rs` - Advanced streaming examples

### Test Examples
- `tests/backend_crud_tests.rs` - Comprehensive CRUD tests for all backends
- `tests/wasm_tests.rs` - WASM/IndexedDB usage patterns
- `tests/comprehensive_store_tests.rs` - Full test suite

### Running Examples

```bash
# Run multi-backend examples (automatically includes available backends)
cargo run --example batch_operations_all_backends --features native
cargo run --example transactions_all_backends --features native

# Run with specific backends
cargo run --example batch_operations_all_backends --features "sled,redb"
cargo run --example transactions_all_backends --features "sled"

# Run single-backend examples
cargo run --example basic_store --features native
cargo run --example redb_zerocopy --features redb-zerocopy
```

**Note**: Examples work with both Sled and Redb backends. Switch backends by changing the initialization method.

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
