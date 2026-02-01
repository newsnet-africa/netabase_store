# Examples Architecture - Composable Feature Showcase

## Design Philosophy

Examples should be:
1. **Composable** - Build on shared base models
2. **Feature-gated** - Only compile with required features
3. **Incremental** - Show progression from simple to complex
4. **Testable** - Examples double as integration tests

## Structure

```
example/
├── Cargo.toml                    # Feature gates match main crate
├── README.md                     # Guide for users and contributors
│
├── src/
│   ├── lib.rs                    # Common utilities + re-exports
│   │
│   ├── models/                   # Shared model definitions
│   │   ├── mod.rs
│   │   ├── base.rs               # Minimal models (no features)
│   │   ├── secondary.rs          # + secondary_keys
│   │   ├── relational.rs         # + relational_keys
│   │   ├── blob.rs               # + blobs
│   │   ├── subscription.rs       # + subscriptions
│   │   ├── versioned.rs          # + migration
│   │   └── complete.rs           # All features combined
│   │
│   └── utils/                    # Test utilities
│       ├── mod.rs
│       ├── fixtures.rs           # Data generation
│       └── assertions.rs         # Common assertions
│
├── examples/
│   ├── 00_minimal.rs             # No optional features
│   ├── 01_secondary_keys.rs      # #![cfg(feature = "secondary_keys")]
│   ├── 02_relational.rs          # #![cfg(feature = "relational_keys")]
│   ├── 03_blobs.rs               # #![cfg(feature = "blobs")]
│   ├── 04_subscriptions.rs       # #![cfg(feature = "subscriptions")]
│   ├── 05_migration.rs           # #![cfg(feature = "migration")]
│   ├── 06_repository.rs          # #![cfg(feature = "repository")]
│   ├── 07_libp2p.rs              # #![cfg(feature = "libp2p")]
│   │
│   ├── combinations/             # Feature interaction tests
│   │   ├── relational_blob.rs    # Test relational + blob
│   │   ├── migration_repo.rs     # Test migration + repository
│   │   └── full_stack.rs         # All features enabled
│   │
│   └── real_world/               # Practical examples
│       ├── blog.rs               # Blog with posts, comments
│       ├── ecommerce.rs          # Products, orders, inventory
│       └── social.rs             # Users, posts, follows
│
├── benches/
│   ├── base_crud.rs              # No features
│   ├── secondary_lookup.rs       # #[cfg(feature = "secondary_keys")]
│   ├── blob_chunking.rs          # #[cfg(feature = "blobs")]
│   ├── relational_hydration.rs   # #[cfg(feature = "relational_keys")]
│   └── full_featured.rs          # All features
│
└── tests/
    ├── feature_isolation.rs      # Ensure features don't interfere
    ├── macro_expansion.rs        # Verify generated code
    └── schema_roundtrip.rs       # Export/import validation
```

## Example Template

### Base Example (00_minimal.rs)

```rust
//! Minimal example - no optional features required
//!
//! This demonstrates the absolute minimum needed to use netabase_store.
//! Run with: `cargo run --example 00_minimal --no-default-features`

use netabase_store::prelude::*;
use netabase_store::traits::database::store::NBStore;
use serde::{Serialize, Deserialize};

#[netabase_macros::netabase_definition(MinimalDef)]
mod minimal_models {
    use super::*;

    #[derive(
        netabase_macros::NetabaseModel,
        Debug, Clone, Serialize, Deserialize,
        PartialEq, Eq, Hash, PartialOrd, Ord
    )]
    pub struct Counter {
        #[primary_key]
        pub id: String,
        pub value: u64,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use minimal_models::*;

    // Create in-memory database
    let (store, _temp) = RedbStore::<MinimalDef>::new_temporary()?;

    // Write
    let txn = store.begin_write()?;
    txn.create(&Counter {
        id: CounterID("main".into()),
        value: 0,
    })?;
    txn.commit()?;

    // Read
    let txn = store.begin_read()?;
    let counter: Option<Counter> = txn.read(&CounterID("main".into()))?;
    println!("Counter value: {}", counter.unwrap().value);

    Ok(())
}
```

### Feature Example (01_secondary_keys.rs)

```rust
//! Secondary key indexing example
//!
//! This builds on the minimal example by adding indexed lookups.
//! Run with: `cargo run --example 01_secondary_keys --features secondary_keys`

#![cfg(feature = "secondary_keys")]

use netabase_store::prelude::*;
use netabase_store::traits::database::store::NBStore;
use netabase_store::databases::redb::transaction::RedbModelCrud;
use serde::{Serialize, Deserialize};

// Import base models for reference
use example::models::base::*;

#[netabase_macros::netabase_definition(IndexedDef)]
mod indexed_models {
    use super::*;

    #[derive(
        netabase_macros::NetabaseModel,
        Debug, Clone, Serialize, Deserialize,
        PartialEq, Eq, Hash, PartialOrd, Ord
    )]
    pub struct User {
        #[primary_key]
        pub id: String,
        
        #[secondary_key]  // Enable indexed lookup
        pub email: String,
        
        pub name: String,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use indexed_models::*;

    let (store, _temp) = RedbStore::<IndexedDef>::new_temporary()?;

    // Create users
    let txn = store.begin_write()?;
    txn.create(&User {
        id: UserID("u1".into()),
        email: "alice@example.com".into(),
        name: "Alice".into(),
    })?;
    txn.commit()?;

    // Lookup by secondary key (email)
    let txn = store.begin_read()?;
    let users = txn.read_by_secondary_key(
        &UserEmail("alice@example.com".into())
    )?;
    
    println!("Found user by email: {:?}", users);

    Ok(())
}
```

### Combination Example (combinations/relational_blob.rs)

```rust
//! Test interaction between relational links and blob storage
//!
//! Ensures that blobs work correctly in models with relational fields.
//! Run with: `cargo run --example relational_blob --features relational_keys,blobs`

#![cfg(all(feature = "relational_keys", feature = "blobs"))]

use netabase_store::prelude::*;
use serde::{Serialize, Deserialize};

#[netabase_macros::netabase_definition(MediaDef)]
mod media_models {
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
        
        #[blob]
        pub avatar: Vec<u8>,  // Blob storage
    }

    #[derive(
        netabase_macros::NetabaseModel,
        Debug, Clone, Serialize, Deserialize,
        PartialEq, Eq, Hash, PartialOrd, Ord
    )]
    pub struct Post {
        #[primary_key]
        pub id: String,
        
        #[link(MediaDef, User)]  // Relational link
        pub author: String,
        
        #[blob]
        pub image: Option<Vec<u8>>,  // Optional blob
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use media_models::*;
    
    // Test that blobs and relations work together
    let (store, _temp) = RedbStore::<MediaDef>::new_temporary()?;
    
    let txn = store.begin_write()?;
    
    // Create user with avatar blob
    txn.create(&User {
        id: UserID("u1".into()),
        name: "Alice".into(),
        avatar: vec![0xFF; 100_000],  // Large blob
    })?;
    
    // Create post linked to user, with image blob
    txn.create(&Post {
        id: PostID("p1".into()),
        author: RelationalLink::new_dehydrated(UserID("u1".into())),
        image: Some(vec![0xAA; 50_000]),
    })?;
    
    txn.commit()?;
    
    // Verify both blobs stored correctly
    let txn = store.begin_read()?;
    let user: User = txn.read(&UserID("u1".into()))?.unwrap();
    let post: Post = txn.read(&PostID("p1".into()))?.unwrap();
    
    assert_eq!(user.avatar.len(), 100_000);
    assert_eq!(post.image.unwrap().len(), 50_000);
    
    // Test hydration with blobs
    let hydrated_author = post.author.hydrate(&txn)?;
    assert_eq!(hydrated_author.name, "Alice");
    
    println!("✓ Relational links and blobs work together");
    
    Ok(())
}
```

## Cargo.toml for Examples

```toml
[package]
name = "example"
version = "0.1.0"
edition = "2024"

[features]
# Match parent crate features exactly
default = ["secondary_keys", "relational_keys", "blobs", "repository", "migration", "libp2p", "subscriptions"]
secondary_keys = ["netabase_store/secondary_keys"]
relational_keys = ["netabase_store/relational_keys"]
blobs = ["netabase_store/blobs"]
repository = ["netabase_store/repository"]
migration = ["netabase_store/migration"]
libp2p = ["netabase_store/libp2p"]
subscriptions = ["netabase_store/subscriptions"]

[dependencies]
netabase_store = { path = "..", default-features = false }
netabase_macros = { path = "../netabase_macros" }
serde = { version = "1.0", features = ["derive"] }

[lib]
name = "example"
path = "src/lib.rs"

# Examples are automatically discovered from examples/
# Run specific example: cargo run --example 00_minimal --no-default-features
# Run with features: cargo run --example 01_secondary_keys --features secondary_keys

[[bench]]
name = "base_crud"
harness = false

[[bench]]
name = "secondary_lookup"
harness = false
required-features = ["secondary_keys"]

[[bench]]
name = "blob_chunking"
harness = false
required-features = ["blobs"]
```

## Shared Models Library (src/models/)

```rust
// src/models/mod.rs
pub mod base;

#[cfg(feature = "secondary_keys")]
pub mod secondary;

#[cfg(feature = "relational_keys")]
pub mod relational;

#[cfg(feature = "blobs")]
pub mod blob;

#[cfg(feature = "subscriptions")]
pub mod subscription;

#[cfg(feature = "migration")]
pub mod versioned;

// Always available for combination examples
pub mod complete;
```

```rust
// src/models/base.rs
//! Minimal models with no optional features

use serde::{Serialize, Deserialize};

#[netabase_macros::netabase_definition(BaseDef)]
pub mod base_models {
    use super::*;

    #[derive(
        netabase_macros::NetabaseModel,
        Debug, Clone, Serialize, Deserialize,
        PartialEq, Eq, Hash, PartialOrd, Ord
    )]
    pub struct Counter {
        #[primary_key]
        pub id: String,
        pub value: u64,
    }
}
```

```rust
// src/models/relational.rs
#![cfg(feature = "relational_keys")]

//! Models demonstrating relational links

use serde::{Serialize, Deserialize};

#[netabase_macros::netabase_definition(RelationalDef)]
pub mod relational_models {
    use super::*;

    #[derive(
        netabase_macros::NetabaseModel,
        Debug, Clone, Serialize, Deserialize,
        PartialEq, Eq, Hash, PartialOrd, Ord
    )]
    pub struct Author {
        #[primary_key]
        pub id: String,
        pub name: String,
    }

    #[derive(
        netabase_macros::NetabaseModel,
        Debug, Clone, Serialize, Deserialize,
        PartialEq, Eq, Hash, PartialOrd, Ord
    )]
    pub struct Book {
        #[primary_key]
        pub id: String,
        pub title: String,
        
        #[link(RelationalDef, Author)]
        pub author: String,
    }
}
```

## Testing Feature Combinations

```rust
// tests/feature_isolation.rs
//! Verify that features don't interfere with each other

#[test]
fn minimal_compiles() {
    // This test exists to ensure no-default-features works
    // The fact that it compiles is the test
}

#[cfg(all(feature = "secondary_keys", feature = "relational_keys"))]
#[test]
fn secondary_and_relational_coexist() {
    // Test that both features work together
}

#[cfg(all(feature = "blobs", feature = "migration"))]
#[test]
fn blobs_migrate_correctly() {
    // Ensure blob storage survives migration
}
```

## Running Examples

```bash
# Minimal (no features)
cargo run --example 00_minimal --no-default-features

# Single feature
cargo run --example 01_secondary_keys --features secondary_keys

# Multiple features
cargo run --example relational_blob --features relational_keys,blobs

# All features (default)
cargo run --example full_stack

# Run all examples with their required features
cargo build --examples

# Run benchmarks with features
cargo bench --features secondary_keys --bench secondary_lookup
```

## Benefits

1. **Progressive Learning** - Users see features incrementally
2. **Feature Testing** - Examples validate feature combinations
3. **Documentation** - Examples serve as living documentation
4. **CI/CD** - Can test each feature combination independently
5. **Performance** - Benchmarks isolated by feature

## CI Integration

```yaml
# .github/workflows/examples.yml
name: Examples

on: [push, pull_request]

jobs:
  examples:
    strategy:
      matrix:
        features:
          - ""
          - "secondary_keys"
          - "relational_keys"
          - "blobs"
          - "secondary_keys,relational_keys"
          - "relational_keys,blobs"
          # ... all combinations
    
    steps:
      - uses: actions/checkout@v2
      - name: Build example
        run: |
          cd example
          cargo build --examples --no-default-features --features "${{ matrix.features }}"
```
