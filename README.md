# Netabase Store

A type-safe, high-performance embedded database library for Rust with automatic model migration and compile-time schema validation.

[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

## Features

- [x] **Type-Safe**: Compile-time schema validation with Rust's type system
- [x] **High Performance**: Zero-copy operations with postcard serialization
- [x] **Auto Migration**: Automatic schema versioning and data migration
- [x] **ACID Transactions**: Full transactional integrity with redb
- [x] **Secondary Indexes**: Fast lookups on non-primary fields
- [x] **Relational Links**: Type-safe relationships between models
- [x] **Blob Storage**: Automatic chunking for large data (>60KB)
- [x] **Repository Pattern**: Compile-time access control boundaries

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
netabase_store = "0.1"
netabase_macros = "0.1"
serde = { version = "1.0", features = ["derive"] }
postcard = "1.1"
```

### Define Your Schema

```rust
use netabase_store::prelude::*;
use netabase_store::traits::database::store::NBStore;
use serde::{Serialize, Deserialize};

#[netabase_macros::netabase_definition(MyApp)]
mod my_models {
    use super::*;

    #[derive(
        netabase_macros::NetabaseModel,
        Debug, Clone, Serialize, Deserialize,
        PartialEq, Eq, Hash, PartialOrd, Ord
    )]
    pub struct User {
        #[primary_key]
        pub id: String,
        
        pub name: String,
        
        #[secondary_key]
        pub email: String,
    }

    #[derive(
        netabase_macros::NetabaseModel,
        Debug, Clone, Serialize, Deserialize,
        PartialEq, Eq, Hash, PartialOrd, Ord
    )]
    pub struct Post {
        #[primary_key]
        pub id: String,
        
        pub title: String,
        pub content: String,
        
        #[link(MyApp, User)]
        pub author: String,
    }
}
```

### Use the Database

```rust
use my_models::*;
use netabase_store::relational::RelationalLink;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an in-memory database
    let (store, _temp) = RedbStore::<MyApp>::new_temporary()?;

    // Write data
    let txn = store.begin_write()?;
    txn.create(&User {
        id: UserID("alice".into()),
        name: "Alice Smith".into(),
        email: "alice@example.com".into(),
    })?;
    txn.create(&Post {
        id: PostID("post1".into()),
        title: "Hello World".into(),
        content: "My first post".into(),
        author: RelationalLink::new_dehydrated(UserID("alice".into())),
    })?;
    txn.commit()?;

    // Read data
    let txn = store.begin_read()?;
    let user: Option<User> = txn.read(&UserID("alice".into()))?;
    println!("User: {:?}", user);

    let post: Option<Post> = txn.read(&PostID("post1".into()))?;
    println!("Post: {:?}", post);

    Ok(())
}
```

## Documentation

- **[Examples & Guide](./boilerplate/GUIDE.md)** - Comprehensive beginner's guide with examples
- **[Architecture](./ARCHITECTURE.md)** - Internal design and implementation details
- **[Examples Crate](./boilerplate/README.md)** - Runnable examples and benchmarks

## Core Concepts

### Models

Models are Rust structs decorated with attributes that define their database behavior:

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ProductImage {
    pub data: Vec<u8>,
    pub mime_type: String,
}

#[derive(NetabaseModel, Serialize, Deserialize, ...)]
pub struct Product {
    #[primary_key]
    pub sku: String,           // Unique identifier → ProductID
    
    #[secondary_key]
    pub category: String,      // Indexed for fast lookups
    
    pub name: String,          // Regular field
    pub price: u64,
    
    #[link(MyApp, User)]
    pub seller: String,        // Type-safe foreign key
    
    #[blob]
    pub image: ProductImage,   // Auto-chunked if >60KB
}
```

### Definitions

Definitions group related models into a cohesive schema:

```rust
#[netabase_definition(MyApp)]
mod my_app {
    pub struct User { ... }
    pub struct Post { ... }
    pub struct Comment { ... }
}
```

### Repositories

Repositories enforce access boundaries across multiple definitions:

```rust
#[netabase_repository(MainRepo, definitions(Users, Posts, Comments))]
mod main_repo {}

// Only models from Users, Posts, Comments can be accessed
let store = RedbRepositoryStore::<MainRepo>::new("data.redb")?;
```

### Schema Migration

Version your models and define migration paths:

```rust
use netabase_store::traits::migration::MigrateFrom;

#[derive(NetabaseModel, Serialize, Deserialize, ...)]
#[netabase_version(family = "User", version = 1)]
#[subscribe(Topic1)]
pub struct UserV1 {
    #[primary_key]
    pub id: String,
    pub name: String,
}

#[derive(NetabaseModel, Serialize, Deserialize, ...)]
#[netabase_version(family = "User", version = 2, current)]
#[subscribe(Topic1)]
pub struct User {
    #[primary_key]
    pub id: String,
    pub first_name: String,
    pub last_name: String,
}

impl MigrateFrom<UserV1> for User {
    fn migrate_from(old: UserV1) -> Self {
        let parts: Vec<&str> = old.name.split_whitespace().collect();
        User {
            id: old.id,
            first_name: parts.get(0).unwrap_or(&"").to_string(),
            last_name: parts.get(1).unwrap_or(&"").to_string(),
            subscriptions: old.subscriptions,
        }
    }
}

// Old data is automatically migrated when read
```

## Advanced Features

### Secondary Key Queries

```rust
// Query by email (secondary key)
let txn = store.begin_read()?;
let users = txn.query_by_secondary_key::<User>(
    &UserSecondaryKeys::Email("alice@example.com".into())
)?;

// Returns all users with that email
for user in users {
    println!("Found user: {:?}", user);
}
```

### Blob Storage

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct ProfilePicture {
    pub data: Vec<u8>,
    pub mime_type: String,
}

#[derive(NetabaseModel, ...)]
pub struct User {
    #[primary_key]
    pub id: String,
    
    #[blob]
    pub avatar: ProfilePicture,  // Automatically chunked
}
```

### Relational Links

```rust
use netabase_store::relational::RelationalLink;

let post = Post {
    id: PostID("post1".into()),
    title: "Hello".into(),
    author: RelationalLink::new_dehydrated(UserID("alice".into())),
};

// Links can be hydrated (loaded from database)
// or owned (contain the actual model)
```

## Performance

Benchmarks from the examples crate (on typical hardware):

- **CRUD Operations**: ~50k-100k ops/sec
- **Secondary Index Lookup**: ~40k-80k ops/sec  
- **Blob Storage/Retrieval**: ~5-10 MB/sec
- **Migration**: ~10k-50k records/sec

See [benches/](./boilerplate/benches/) for detailed benchmarks.

## Examples

The `boilerplate` crate contains comprehensive examples:

```bash
# Run the demonstration
cargo run -p netabase_store_examples

# Run tests
cargo test -p netabase_store_examples

# Run benchmarks
cargo bench -p netabase_store_examples
```

See [boilerplate/GUIDE.md](./boilerplate/GUIDE.md) for a detailed walkthrough.

## Architecture

Netabase Store is built on these key components:

1. **Macro System** (`netabase_macros`) - Generates type-safe boilerplate
2. **Type Registry** (`traits::registery`) - Models, definitions, repositories
3. **Transaction Layer** (`databases::redb::transaction`) - CRUD operations
4. **Migration System** (`traits::migration`) - Version management
5. **Blob Storage** (`blob`) - Large data handling
6. **Relational Links** (`relational`) - Type-safe relationships

See [ARCHITECTURE.md](./ARCHITECTURE.md) for detailed internals.

## Project Structure

```
netabase_store/
├── src/                      # Main library code
│   ├── databases/            # Backend implementations (redb)
│   ├── traits/               # Core trait definitions
│   ├── blob.rs               # Blob storage
│   ├── relational.rs         # Relational links
│   ├── query.rs              # Query types
│   └── ...
├── netabase_macros/          # Procedural macros
├── boilerplate/              # Examples and tests
│   ├── src/                  # Example models
│   ├── tests/                # Integration tests
│   ├── benches/              # Performance benchmarks
│   ├── GUIDE.md              # Beginner's guide
│   └── README.md
├── ARCHITECTURE.md           # Architecture documentation
└── README.md                 # This file
```

## Requirements

- Rust 1.75+ (uses edition 2024)
- Linux, macOS, or Windows
- For WASM: `wasm32-unknown-unknown` target (IndexedDB backend - planned)

## Testing

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test -p netabase_store
cargo test -p netabase_store_examples

# Run with logging
RUST_LOG=debug cargo test

# Run benchmarks
cargo bench -p netabase_store_examples
```

## Contributing

Contributions welcome! Please:

1. Read [ARCHITECTURE.md](./ARCHITECTURE.md) to understand the design
2. Add tests for new features
3. Update documentation
4. Run `cargo fmt` and `cargo clippy`
5. Ensure all tests pass

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

Built on these excellent crates:

- [redb](https://github.com/cberner/redb) - Embedded database engine
- [postcard](https://github.com/jamesmunns/postcard) - Compact serialization
- [serde](https://serde.rs/) - Serialization framework
- [strum](https://github.com/Peternator7/strum) - Enum utilities

## Status

This crate is under active development. APIs may change before 1.0 release.

Current version: 0.1.0 (pre-release)
