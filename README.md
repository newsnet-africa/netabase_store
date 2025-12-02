![crates.io](https://img.shields.io/crates/v/netabase_store.svg)
![crates.io downloads](https://img.shields.io/crates/d/netabase_store.svg) ![docs.rs](https://docs.rs/netabase_store/badge.svg)

# Netabase Store

A type-safe, multi-backend key-value storage library for Rust with support for native (Sled, Redb) and WASM (IndexedDB) environments, inspired by [`native_db`](https://crates.io/crates/native_db).

> ⚠️ **Early Development**: This crate is still in early development and will change frequently as it stabilizes. It is not advised to use this in a production environment until it stabilizes.

## ✨ Key Features

### 🎯 Core Capabilities
- **🔄 Multi-Backend Support** - Switch between Sled, Redb, or IndexedDB without changing your code
- **🔒 Type-Safe Schema** - Compile-time validation with derive macros
- **⚡ High Performance** - Zero-copy operations, batch processing, and optimized transactions
- **🌐 Cross-Platform** - Works on desktop, server, and WASM/browser environments

### 🗄️ Data Management
- **📇 Primary & Secondary Keys** - Efficient indexing with automatic secondary key management
- **🔗 Relational Links** - Type-safe relationships between models with automatic cascading
- **📊 Subscription System** - Merkle tree-based change tracking for P2P synchronization
- **🔍 Database Introspection** - Query all internal trees, indexes, and statistics

### 🚀 Performance Features
- **⚡ Zero-Copy Reads** - Direct memory access with RedbStoreZeroCopy (10-50x faster)
- **📦 Batch Operations** - Atomic bulk inserts/updates (10-100x faster than individual operations)
- **🔐 ACID Transactions** - Full transactional support across all operations
- **🎨 Flexible Serialization** - Efficient bincode with serde compatibility

### 🔌 Integrations
- **🌍 libp2p Support** - Built-in RecordStore implementation for distributed systems
- **🔄 Async & Sync APIs** - Native sync for Sled/Redb, async for IndexedDB/WASM

## Installation

Add to your `Cargo.toml`:

```toml
[package]
name = "my_project"
version = "0.1.0"
edition = "2024"

# Features must be enabled in your crate for macro-generated code
[features]
default = ["native"]
native = ["netabase_store/native"]

[dependencies]
netabase_store = { version = "0.0.7", features = ["native"] }

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
netabase_store = { version = "0.0.7", default-features = false, features = ["wasm"] }

# For specific backend only:
netabase_store = { version = "0.0.7", features = ["sled"] }

# For zero-copy redb optimization:
netabase_store = { version = "0.0.7", features = ["redb-zerocopy"] }

# For libp2p integration:
netabase_store = { version = "0.0.7", features = ["native", "libp2p"] }
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
- **Sled**: Full support via transactions and batch API
- **Redb**: Full support via transactions and batch API  
- Both backends provide identical API and performance benefits

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
- **10-100x Faster**: Single transaction for many operations (eliminates per-operation overhead)
- **Type-Safe**: Compile-time enforcement of read-only vs read-write access
- **Zero-Cost**: Phantom types compile away completely
- **ACID**: Full atomicity for write transactions (Redb)

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

### 🔗 Relational Links - Type-Safe Relationships

**Problem**: Traditional key-value stores make you manually manage relationships, leading to inconsistent data and boilerplate code.

**Solution**: Netabase Store provides `RelationalLink<D, M>` - a type-safe way to define relationships that automatically handles:
- ✅ **Cascading Inserts** - Insert a Post with an embedded User, both are saved atomically
- ✅ **Lazy Loading** - Store just a reference, load the full entity only when needed
- ✅ **Type Safety** - Compiler ensures relationship targets exist in your schema
- ✅ **Generated Helpers** - Automatic methods for hydration, type checking, and insertion

#### 📝 Defining Relationships

Use the `#[relation(name)]` attribute on fields with `RelationalLink<D, M>` type:

```rust
use netabase_store::{
    NetabaseModel, netabase, netabase_definition_module,
    links::RelationalLink,
};

#[netabase_definition_module(BlogDef, BlogKeys)]
mod models {
    use super::*;

    #[derive(NetabaseModel, Clone, Debug, PartialEq,
             bincode::Encode, bincode::Decode,
             serde::Serialize, serde::Deserialize)]
    #[netabase(BlogDef)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub name: String,
        pub email: String,
    }

    #[derive(NetabaseModel, Clone, Debug, PartialEq,
             bincode::Encode, bincode::Decode,
             serde::Serialize, serde::Deserialize)]
    #[netabase(BlogDef)]
    pub struct Post {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,

        // Define a relation with a custom name
        #[relation(author)]
        pub author: RelationalLink<BlogDef, User>,
    }
}
```

#### 💡 Two Storage Strategies

Think of `RelationalLink` like a smart pointer - it can hold either the full data (Entity) or just a key (Reference):

**🎯 Entity (Eager Loading)** - Embed the full related model:
- **Use When**: You always need the related data together
- **Benefit**: No extra database lookup needed
- **Trade-off**: Slightly larger storage, data duplication

**🔑 Reference (Lazy Loading)** - Store only the primary key:
- **Use When**: Related data is rarely needed or already cached
- **Benefit**: Normalized storage, no duplication
- **Trade-off**: Requires hydration (lookup) to access data

#### 🔨 Using Entity (Eager Loading):
```rust
let post = Post {
    id: 1,
    title: "Hello World".to_string(),
    content: "My first post".to_string(),
    // Embed the full user entity
    author: RelationalLink::Entity(User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    }),
};

// Inserts both the post AND the user atomically
post.insert_with_relations(&store)?;

// ✅ Result: Both Post and User are in the database
```

#### 🔑 Using Reference (Lazy Loading):
```rust
// Insert user first
let user = User { id: 1, /* ... */ };
user_tree.put(user)?;

let post = Post {
    id: 1,
    title: "Hello World".to_string(),
    content: "My first post".to_string(),
    // Store only a reference to the user
    author: RelationalLink::Reference(UserPrimaryKey(1)),
};

// Only inserts the post (user already exists)
post.insert_with_relations(&store)?;

// ✅ Result: Only Post is inserted, references existing User
```

#### 🎨 Generated Helper Methods

The `#[relation(name)]` attribute automatically generates helper methods for every relation:

| Method | Purpose |
|--------|---------|
| `is_{field}_entity()` | Check if it contains embedded data |
| `is_{field}_reference()` | Check if it's just a reference |
| `get_{field}()` | Get the `RelationalLink` itself |
| `hydrate_{field}()` | Load the full entity from a reference |
| `insert_{field}_if_entity()` | Insert only if it's an Entity |

**Example Usage:**

```rust
// Check the variant type
if post.is_author_entity() {
    println!("Author is embedded");
}

if post.is_author_reference() {
    println!("Author is a reference");
}

// Get the relational link
let author_link = post.get_author();

// Hydrate a reference to load the full entity
if let Some(author) = post.hydrate_author(&user_tree)? {
    println!("Author: {}", author.name);
}

// Insert only if it's an entity
post.insert_author_if_entity(&store)?;
```

#### Multiple Relations

Models can have multiple relational links:

```rust
#[derive(NetabaseModel, Clone, Debug, PartialEq,
         bincode::Encode, bincode::Decode,
         serde::Serialize, serde::Deserialize)]
#[netabase(BlogDef)]
pub struct Post {
    #[primary_key]
    pub id: u64,
    pub title: String,

    #[relation(post_author)]
    pub author: RelationalLink<BlogDef, User>,

    #[relation(post_category)]
    pub category: RelationalLink<BlogDef, Category>,
}

// Mix entity and reference storage
let post = Post {
    id: 1,
    title: "Rust Tips".to_string(),
    author: RelationalLink::Entity(user),  // Embedded
    category: RelationalLink::Reference(CategoryPrimaryKey(1)),  // Reference
};

// Handles both types correctly
post.insert_with_relations(&store)?;
```

#### Performance Considerations

- **Entities**: ~10% slower inserts due to serialization overhead, but no hydration needed
- **References**: ~5% slower inserts, minimal overhead, requires hydration for access
- **Hydration**: ~15% of insert time for bulk operations

Use entities for small, frequently-accessed models; use references for large models or normalized patterns.

For more details, see `RELATIONAL_LINKS.md` or `examples/relational_links_showcase.rs`.

### 📊 Subscription System - Change Tracking & Sync

**Problem**: In distributed systems, you need to know what data changed and sync efficiently between nodes.

**Solution**: Netabase Store's subscription system uses Merkle trees to:
- ✅ **Track Changes** - Automatically monitor data modifications by topic
- ✅ **Detect Differences** - Compare Merkle roots to find what's different in O(1)
- ✅ **Sync Efficiently** - Transfer only the changed data between nodes
- ✅ **Topic-Based Organization** - Group related data for selective synchronization

#### 🎯 Key Concepts

1. **Topics** - Logical groups of data (e.g., "Users", "Posts", "Comments")
2. **Merkle Trees** - Each topic has a tree where data hashes form leaves
3. **Merkle Root** - A single hash representing the entire topic's state
4. **Comparison** - Two nodes compare roots; if different, data differs

#### 📝 Defining Topics

Use `#[streams(...)]` to define subscription topics for your schema:

```rust
use netabase_store::{
    NetabaseModel, netabase, netabase_definition_module, streams,
};

#[netabase_definition_module(BlogDef, BlogKeys)]
#[streams(UserTopic, PostTopic, CommentTopic)]  // Define topics
mod models {
    use super::*;

    #[derive(NetabaseModel, Clone, Debug, PartialEq,
             bincode::Encode, bincode::Decode,
             serde::Serialize, serde::Deserialize)]
    #[netabase(BlogDef)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub name: String,
    }

    #[derive(NetabaseModel, Clone, Debug, PartialEq,
             bincode::Encode, bincode::Decode,
             serde::Serialize, serde::Deserialize)]
    #[netabase(BlogDef)]
    pub struct Post {
        #[primary_key]
        pub id: String,
        pub title: String,
        #[secondary_key]
        pub author_id: u64,
    }
}
```

#### 🔧 Using the Subscription Manager

The `#[streams(...)]` attribute automatically generates:
- **`{Definition}Subscriptions` enum** - Type-safe topic identifiers
- **`{Definition}SubscriptionManager` struct** - Manages all topic trees
- Helper methods for subscribing, unsubscribing, and comparing

**Basic Operations:**

```rust
use netabase_store::traits::subscription::{Subscriptions, SubscriptionManager};

// Create a subscription manager
let mut manager = BlogDefSubscriptionManager::new();

// List available topics
for topic in BlogDef::subscriptions() {
    println!("Topic: {:?}", topic);
}

// Add items to subscription trees
let user = User { id: 1, name: "Alice".to_string() };
let user_key = bincode::encode_to_vec(&user.id, bincode::config::standard())?;
let user_data = bincode::encode_to_vec(&user, bincode::config::standard())?;

manager.subscribe_item(
    BlogDefSubscriptions::UserTopic,
    user_key,
    &user_data,
)?;

// Remove items
manager.unsubscribe_item(
    BlogDefSubscriptions::UserTopic,
    &user_key,
)?;
```

#### 🌲 Merkle Tree Synchronization

**How It Works:**
1. Each topic maintains a Merkle tree of all its data
2. Any change (add/remove/update) updates the Merkle root
3. Nodes compare roots - if equal, data is identical
4. If different, compare detailed hashes to find specific differences

**Example - P2P Sync:**

```rust
// Create two managers (e.g., local and remote nodes)
let mut local_manager = BlogDefSubscriptionManager::new();
let mut remote_manager = BlogDefSubscriptionManager::new();

// Add different data to each
local_manager.subscribe_item(
    BlogDefSubscriptions::UserTopic,
    user1_key,
    &user1_data,
)?;

remote_manager.subscribe_item(
    BlogDefSubscriptions::UserTopic,
    user2_key,
    &user2_data,
)?;

// Compare Merkle roots to detect differences
let local_root = local_manager.topic_merkle_root(
    BlogDefSubscriptions::UserTopic
)?;
let remote_root = remote_manager.topic_merkle_root(
    BlogDefSubscriptions::UserTopic
)?;

if local_root != remote_root {
    println!("Trees differ - sync needed");
}

// Get detailed differences
let diffs = local_manager.compare_with(&mut remote_manager)?;
for (topic, diff) in diffs {
    println!("Topic {:?}:", topic);
    println!("  Missing in local: {}", diff.missing_in_self.len());
    println!("  Missing in remote: {}", diff.missing_in_other.len());
    println!("  Different values: {}", diff.different_values.len());
}
```

#### Subscription Statistics

```rust
use netabase_store::traits::subscription::SubscriptionManager;

// Get statistics
let stats = manager.stats();
println!("Total items: {}", stats.total_items);
println!("Active topics: {}", stats.active_topics);
```

#### Generated Types

The `#[streams(...)]` attribute generates:

- **`{Definition}Subscriptions` enum**: All subscription topics
- **`{Definition}SubscriptionManager` struct**: Manager for all topics
- **`Subscriptions` trait impl**: For iterating topics and getting names

Example generated code:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlogDefSubscriptions {
    UserTopic,
    PostTopic,
    CommentTopic,
}

pub struct BlogDefSubscriptionManager {
    // Internal subscription trees for each topic
}
```

#### 💼 Common Use Cases

| Use Case | Description | Benefit |
|----------|-------------|---------|
| **🔄 P2P Sync** | Distributed database nodes sync changes | Only transfer different data, not everything |
| **📱 Offline-First Apps** | Mobile app syncs with server when online | Merkle roots identify what changed while offline |
| **🔍 Change Detection** | Monitor specific data categories for updates | Fast O(1) check if any changes occurred |
| **📊 Selective Replication** | Replicate only specific topics to nodes | Users subscribe only to data they need |
| **📝 Audit Trail** | Track modifications by topic over time | Know what changed and when |
| **⚡ Real-Time Updates** | Notify subscribers when data changes | Efficient change notifications |

**📚 Learn More:**
- `examples/subscription_streams.rs` - Complete working examples
- `examples/subscription_demo.rs` - Basic usage patterns
- `tests/subscription_system_tests.rs` - Test patterns and edge cases

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

## Implementing a backend

It is *technically* possible to implement a backend of your own, but much of this implementation needed to be generated with macros, with a few implementation specific quirks.
This makes it a bit hard for me to track *exactly how* to create reproducable instructions, but you can do one of 2 things:
1. Read the `netabase_macros` to see what gents generated and how things string up
2. Expanding the macro implementations to see how the backend implementations are generated.
But if you are looking to wrap your own backend, or for a simpler model, [`kivis`](https://crates.io/crates/kivis) or [`native_model`](https://crates.io/crates/native_model) are probably better suited.

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

I was mostly trying to abstract a persisten backend for [`libp2p::kad::store`](https://docs.rs/libp2p/latest/libp2p/kad/store/trait.RecordStore.html), and got carried away.

## Contributing

Contributions are welcome! Feel free to leave a PR.

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
