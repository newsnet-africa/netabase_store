# Netabase Store

A type-safe, multi-backend key-value storage library for Rust with support for native (Sled, Redb) and WASM (IndexedDB) environments.

## Features

### Current Features

- **Multi-Backend Support**:
  - **Sled**: High-performance embedded database for native platforms
  - **Redb**: Memory-efficient embedded database with ACID guarantees
  - **IndexedDB**: Browser-based storage for WASM applications

- **Type-Safe Schema Definition**:
  - Derive macros for automatic schema generation
  - Primary and secondary key support
  - Compile-time type checking for all database operations

- **Cross-Platform**:
  - Unified API across native and WASM targets
  - Feature flags for platform-specific backends

- **Zero-Copy Deserialization**: Efficient data access with minimal overhead

- **Secondary Key Indexing**: Fast lookups using secondary keys

- **Iterators**: Efficient iteration over stored data

- **Benchmarking**: Comprehensive benchmarks for performance analysis

- **libp2p Integration**: Optional record store for distributed systems (via `record-store` feature)

### TODO for 1.0.0

- [ ] **Default Profiles/Modes**:
  - Simple mode: Pre-configured for common use cases with sensible defaults
  - Performance mode: Optimized for high-throughput applications
  - Compact mode: Optimized for minimal storage footprint

- [ ] **Migration Tools**:
  - Schema migration utilities
  - Data import/export functionality
  - Backend conversion tools

- [ ] **Query Builder**:
  - Fluent API for complex queries
  - Compound secondary key queries
  - Range queries on ordered keys

- [ ] **Transaction Support**:
  - Multi-operation ACID transactions
  - Batch operations for improved performance

- [ ] **Async API**:
  - Fully async/await compatible operations
  - Non-blocking I/O for all backends

- [ ] **Compression**:
  - Optional transparent compression
  - Configurable compression algorithms

- [ ] **Encryption**:
  - At-rest encryption support
  - Transparent encryption/decryption

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
netabase_store = { version = "0.1", features = ["native"] }

# Required dependencies for macros to work
bincode = "2.0.1"
serde = { version = "1.0", features = ["derive"] }
strum = { version = "0.27.2", features = ["derive"] }
derive_more = "2.0.1"
anyhow = "1.0"  # For error handling

# For WASM
# netabase_store = { version = "0.1", features = ["wasm"] }
```

## Quick Start

### Define Your Schema

```rust
use netabase_store::*;

#[netabase_definition_module(BlogDefinition, BlogKeys)]
mod blog_schema {
    use netabase_deps::{bincode, serde};
    use netabase_macros::NetabaseModel;
    use netabase_store::netabase;

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(BlogDefinition)]
    pub struct User {
        #[primary_key]
        pub id: String,
        pub username: String,
        pub email: String,
        #[secondary_key]
        pub age: u32,
    }

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(BlogDefinition)]
    pub struct Post {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,
        #[secondary_key]
        pub author_id: String,
    }
}

use blog_schema::*;
```

### Use with Sled (Native)

```rust
use netabase_store::databases::sled_store::SledStore;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a store
    let store = SledStore::<BlogDefinition>::new("my_database")?;

    // Open a tree for users
    let user_tree = store.open_tree::<User>();

    // Insert a user
    let user = User {
        id: "user123".to_string(),
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        age: 30,
    };
    user_tree.put(user.clone())?;

    // Get by primary key
    let retrieved = user_tree.get(UserPrimaryKey(user.id.clone()))?.unwrap();
    assert_eq!(retrieved.username, "alice");

    // Query by secondary key
    let users_age_30 = user_tree.get_by_secondary_key(UserSecondaryKeys::Age(AgeSecondaryKey(30)))?;
    assert_eq!(users_age_30.len(), 1);

    // Iterate over all users (iter() returns Result tuples)
    for result in user_tree.iter() {
        let (_key, user) = result?;
        println!("User: {} - {}", user.username, user.email);
    }

    Ok(())
}
```

### Use with Redb (Native)

```rust
use netabase_store::databases::redb_store::RedbStore;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a store
    let store = RedbStore::<BlogDefinition>::new("my_database.redb")?;

    // API is identical to SledStore
    let user_tree = store.open_tree::<User>();

    let user = User {
        id: "user456".to_string(),
        username: "bob".to_string(),
        email: "bob@example.com".to_string(),
        age: 25,
    };
    user_tree.put(user)?;

    Ok(())
}
```

### Use with IndexedDB (WASM)

```rust
#[cfg(target_arch = "wasm32")]
use netabase_store::databases::indexed_db::IndexedDbStore;

#[cfg(target_arch = "wasm32")]
async fn wasm_example() -> Result<(), Box<dyn std::error::Error>> {
    // Create a store
    let store = IndexedDbStore::<BlogDefinition>::new("my_database").await?;

    // API is identical to native backends
    let user_tree = store.open_tree::<User>();

    let user = User {
        id: "user789".to_string(),
        username: "charlie".to_string(),
        email: "charlie@example.com".to_string(),
        age: 28,
    };
    user_tree.put(user).await?;

    Ok(())
}
```

## Advanced Usage

### Secondary Keys

Secondary keys enable efficient lookups on non-primary fields:

```rust
// Define a model with secondary keys
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

// Query by secondary key
let published_articles = article_tree
    .get_by_secondary_key(ArticleSecondaryKeys::Published(PublishedSecondaryKey(true)))?;

let tech_articles = article_tree
    .get_by_secondary_key(ArticleSecondaryKeys::Category(CategorySecondaryKey("tech".to_string())))?;
```

### Multiple Models in One Store

```rust
let store = SledStore::<BlogDefinition>::new("blog_db")?;

// Different trees for different models
let user_tree = store.open_tree::<User>();
let post_tree = store.open_tree::<Post>();

// Each tree is independent but shares the same underlying database
user_tree.put(user)?;
post_tree.put(post)?;
```

## Benchmarks

Run benchmarks to compare backend performance:

```bash
# Sled benchmarks
cargo bench --bench sled_wrapper_overhead

# Redb benchmarks
cargo bench --bench redb_wrapper_overhead
```

Benchmark categories:
- Insert performance
- Get performance
- Iteration performance
- Secondary key lookup performance

## Architecture

### Backend Abstraction

Each backend implements a common set of operations:
- `put(model)`: Insert or update a model
- `get(primary_key)`: Retrieve a model by primary key
- `remove(primary_key)`: Delete a model
- `iter()`: Iterate over all models
- `get_by_secondary_key(key)`: Query by secondary key
- `len()`: Get count of stored models
- `is_empty()`: Check if store is empty
- `clear()`: Remove all models

### Type Safety

The library uses Rust's type system to ensure:
- Keys match their models
- Secondary keys exist for the model
- Backend stores only hold their defined models
- Compile-time verification of all operations

## Feature Flags

- `native` (default): Enable sled and redb backends
- `wasm`: Enable IndexedDB backend
- `libp2p`: Enable libp2p integration
- `record-store`: Enable record store for distributed systems (requires `libp2p`)

## Examples

See the `examples/` directory for complete examples:
- `basic_store.rs`: Basic CRUD operations
- More examples coming in 1.0.0

## Testing

```bash
# Run all tests
cargo test --all-features

# Run native tests only
cargo test --features native

# Run WASM tests (requires wasm-pack)
wasm-pack test --node --features wasm
```

## Performance

Netabase Store is designed for high performance:
- Minimal overhead over raw backend operations (typically <5%)
- Zero-copy deserialization where possible
- Efficient secondary key indexing
- Batch operation support (coming in 1.0.0)

### Performance Considerations

**Abstraction Overhead**: While netabase_store provides a powerful type-safe abstraction layer over multiple database backends, this abstraction does come with some overhead. The cost primarily comes from:
- Type conversions between models and the underlying storage format
- The `RecordStore` trait implementation which adds indirection
- Automatic secondary key indexing which requires additional storage operations

For most applications, this overhead is minimal (typically <5-10%) and well worth the benefits of type safety, portability, and developer ergonomics. However, for extremely performance-critical applications that need to squeeze out every last bit of performance, direct use of the underlying backends (Sled, Redb) without the abstraction layer may be preferred.

**Future Optimizations**: We plan to reduce this overhead in future releases through:
- **Direct backend integration for Redb**: Currently in research phase. The challenge is maintaining the `RecordStore` trait abstraction while allowing for more direct access patterns.
- **Compile-time optimization**: Leveraging Rust's zero-cost abstractions more aggressively
- **Lazy secondary key updates**: Optional deferred indexing for write-heavy workloads
- **Custom serialization strategies**: Per-backend optimized serialization paths

### Distributed Systems Support

**P2P Networking Plans**: We are actively developing enhanced support for decentralized peer-to-peer applications:
- **Configurable profiles and modes**: Simple presets for different use cases (local-only, DHT-backed, full replication, etc.)
- **Network protocols abstraction**: Making it easier to use netabase_store over libp2p, WebRTC, or other transport layers
- **Distributed RecordStore**: Currently in development - will provide automatic record distribution and discovery across peers
- **Conflict resolution strategies**: Pluggable CRDT and last-write-wins implementations for eventual consistency

The vision is to make it trivial to build distributed applications where your local database automatically synchronizes with peers without complex manual synchronization logic.

## License

This project is licensed under the GNU Apache License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please see CONTRIBUTING.md for guidelines (coming in 1.0.0).

## Links

- [Netabase (networking layer)](../netabase)
- [GDELT Fetcher](../gdelt_fetcher)
- [Example Usage](../test_netabase)
