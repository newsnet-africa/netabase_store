# Netabase Store Examples - Beginner's Guide

This guide will walk you through the `netabase_store_examples` crate, which demonstrates how to use the `netabase_store` embedded database library.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Core Concepts](#core-concepts)
3. [Simple Example](#simple-example)
4. [Working with Models](#working-with-models)
5. [Relationships](#relationships)
6. [Blob Storage](#blob-storage)
7. [Schema Migration](#schema-migration)
8. [Repository Pattern](#repository-pattern)
9. [Subscription System](#subscription-system)
10. [Merkle Trees & P2P Sync](#merkle-trees--p2p-sync)
11. [Running Examples](#running-examples)

---

## Quick Start

### Prerequisites

- Rust 1.75+ (uses edition 2024)
- Basic understanding of Rust structs and traits

### Building and Running

```bash
# Build the examples
cargo build -p example

# Run the main example
cargo run -p example --bin example

# Run tests
cargo test -p example

# Run benchmarks
cargo bench -p example

# Run specific examples
cargo run -p example --example merkle_sync
cargo run -p example --example selective_subscriptions
```

---

## Core Concepts

### What is Netabase Store?

`netabase_store` is a type-safe embedded database built on top of [redb](https://github.com/cberner/redb). It provides:

- **Compile-time type safety**: Wrong types = compiler errors
- **Automatic serialization**: Uses [postcard](https://github.com/jamesmunns/postcard) for efficient binary encoding
- **Relational links**: Type-safe foreign key relationships
- **Schema versioning**: Automatic migration between model versions
- **ACID transactions**: Full transactional integrity

### Key Components

1. **Models**: Rust structs that represent your data (like database tables)
2. **Definitions**: Collections of related models (like a database schema)
3. **Repositories**: Access control boundaries for data graphs
4. **Transactions**: Read and write operations on the database

---

## Simple Example

Let's create a simple model and store it using a standalone `RedbStore`:

```rust
use netabase_store::prelude::*;
use netabase_store::traits::database::store::NBStore;
use serde::{Serialize, Deserialize};

// Step 1: Define your models inside a definition module
#[netabase_macros::netabase_definition(MyApp)]
mod my_models {
    use super::*;

    // A simple User model
    #[derive(
        netabase_macros::NetabaseModel,
        Debug, Clone, Serialize, Deserialize,
        PartialEq, Eq, Hash, PartialOrd, Ord
    )]
    pub struct User {
        #[primary_key]
        pub id: String,          // Primary key - unique identifier
        
        pub name: String,        // Regular field
        
        #[secondary_key]
        pub email: String,       // Secondary key - indexed for fast lookup
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use my_models::*;

    // Step 2: Create a store for your definition
    // (using new_temporary for an in-memory database)
    let (store, _temp) = RedbStore::<MyApp>::new_temporary()?;

    // Step 3: Write data
    let txn = store.begin_write()?;
    txn.create(&User {
        id: UserID("alice".into()),
        name: "Alice Smith".into(),
        email: "alice@example.com".into(),
    })?;
    txn.commit()?;

    // Step 4: Read data back
    let txn = store.begin_read()?;
    let user: Option<User> = txn.read(&UserID("alice".into()))?;
    
    println!("Found user: {:?}", user);

    Ok(())
}
```

---

## Working with Models

### Model Attributes

Models use proc macros to generate all necessary boilerplate. Here are the key attributes:

#### `#[primary_key]`

Every model must have exactly one primary key - a unique identifier.

```rust
#[derive(netabase_macros::NetabaseModel, /* ... */)]
pub struct Product {
    #[primary_key]
    pub sku: String,      // Unique product identifier
    pub name: String,
}
```

The macro automatically generates a type alias: `ProductID` wrapping the primary key type.

#### `#[secondary_key]`

Secondary keys create indexes for fast lookups on non-primary fields.

```rust
#[derive(netabase_macros::NetabaseModel, /* ... */)]
pub struct Product {
    #[primary_key]
    pub sku: String,
    
    #[secondary_key]
    pub category: String,  // Can quickly find all products in a category
    
    #[secondary_key]
    pub barcode: String,   // Can quickly find by barcode
    
    pub price: u64,        // Regular field - no index
}
```

#### `#[link(Definition, Model)]`

Links create type-safe foreign key relationships.

```rust
#[derive(netabase_macros::NetabaseModel, /* ... */)]
pub struct Order {
    #[primary_key]
    pub id: String,
    
    #[link(MyApp, User)]    // Links to User model in MyApp definition
    pub customer: String,    // Will be typed as RelationalLink<..., User>
}
```

#### `#[blob]`

Blobs store large binary data efficiently (automatically chunked if > 60KB).

```rust
#[derive(netabase_macros::NetabaseModel, /* ... */)]
pub struct Document {
    #[primary_key]
    pub id: String,
    
    #[blob]
    pub content: Vec<u8>,   // Large file stored separately
}
```

---

## Relationships

Netabase Store supports type-safe relationships through `RelationalLink`.

### Example: Blog Posts and Authors

```rust
#[netabase_macros::netabase_definition(Blog)]
mod blog {
    use super::*;

    #[derive(netabase_macros::NetabaseModel, /* ... */)]
    pub struct Author {
        #[primary_key]
        pub id: String,
        pub name: String,
    }

    #[derive(netabase_macros::NetabaseModel, /* ... */)]
    pub struct Post {
        #[primary_key]
        pub id: String,
        pub title: String,
        pub content: String,
        
        #[link(Blog, Author)]
        pub author: String,    // Type-safe link to Author
    }
}
```

### Using Links

```rust
use blog::*;
use netabase_store::relational::RelationalLink;

// Create an author
let author = Author {
    id: AuthorID("jane".into()),
    name: "Jane Doe".into(),
};

// Create a post that references the author
let post = Post {
    id: PostID("post1".into()),
    title: "Hello World".into(),
    content: "My first post".into(),
    author: RelationalLink::new_dehydrated(AuthorID("jane".into())),
};

// Store both
let txn = store.begin_write()?;
txn.create(&author)?;
txn.create(&post)?;
txn.commit()?;

// Read and follow the link
let txn = store.begin_read()?;
let post: Post = txn.read(&PostID("post1".into()))?.unwrap();
let author_id = post.author.get_primary_key();
let author: Author = txn.read(&author_id)?.unwrap();
```

---

## Blob Storage

Large binary data (images, files, etc.) should use the `#[blob]` attribute.

### Why Blobs?

- **Automatic chunking**: Files > 60KB are split into multiple chunks
- **Efficient storage**: Blobs are stored separately from main record
- **Easy reconstruction**: Automatically reassembled when read

### Example: User Profile Pictures

```rust
// Define a blob type
#[derive(Debug, Clone, Serialize, Deserialize, /* ... */)]
pub struct ProfilePicture {
    pub data: Vec<u8>,
    pub mime_type: String,
}

#[derive(netabase_macros::NetabaseModel, /* ... */)]
pub struct User {
    #[primary_key]
    pub id: String,
    pub name: String,
    
    #[blob]
    pub avatar: ProfilePicture,  // Automatically handled as blob
}
```

### Blob Reconstruction

The blob system handles chunking and reconstruction automatically:

```rust
use netabase_store::blob::NetabaseBlobItem;

let user = User {
    id: UserID("alice".into()),
    name: "Alice".into(),
    avatar: ProfilePicture {
        data: vec![0u8; 500_000],  // 500KB image
        mime_type: "image/png".into(),
    },
};

// When stored, blob is automatically split into chunks
// When read, blob is automatically reconstructed
let txn = store.begin_write()?;
txn.create(&user)?;
txn.commit()?;

let txn = store.begin_read()?;
let user: User = txn.read(&UserID("alice".into()))?.unwrap();
assert_eq!(user.avatar.data.len(), 500_000);  // Fully reconstructed!
```

---

## Schema Migration

As your application evolves, your data models change. Netabase Store handles this with version families and migration logic.

### Version Families

Models belong to "families" and have version numbers:

```rust
// Version 1: Original user model
#[derive(netabase_macros::NetabaseModel, /* ... */)]
#[netabase_version(family = "User", version = 1)]
pub struct UserV1 {
    #[primary_key]
    pub id: String,
    pub name: String,    // Single name field
}

// Version 2: Split name into first/last
#[derive(netabase_macros::NetabaseModel, /* ... */)]
#[netabase_version(family = "User", version = 2, current)]
pub struct UserV2 {
    #[primary_key]
    pub id: String,
    pub first_name: String,
    pub last_name: String,
}
```

### Migration Logic

Implement `MigrateFrom` to define upgrade paths:

```rust
use netabase_store::traits::migration::MigrateFrom;

impl MigrateFrom<UserV1> for User {
    fn migrate_from(old: UserV1) -> Self {
        let parts: Vec<&str> = old.name.split_whitespace().collect();
        User {
            id: old.id,
            first_name: parts.first().unwrap_or(&"").to_string(),
            last_name: parts.get(1).unwrap_or(&"").to_string(),
        }
    }
}
```

### Automatic Migration

When you read old data, it's automatically upgraded:

```rust
// Database contains UserV1 data
let txn = store.begin_read()?;
let user: User = txn.read(&UserID("alice".into()))?.unwrap();
// Automatically migrated from UserV1 to User!
```

---

## Repository Pattern

Repositories define access boundaries - which models can be accessed together. They encapsulate multiple definitions and provide a unified management layer.

### Why Repositories?

- **Encapsulation**: Definitions are grouped logically (e.g., HR, Inventory).
- **Type Safety**: The macro generates a `Stores` struct with typed fields for each definition.
- **Relational Integrity**: Repositories validate that all relational links are contained within the repository boundary.
- **Cross-Definition Hydration**: Repositories provide the context needed to follow links from one definition to another.

### Example: Employee Management System

```rust
#[netabase_macros::netabase_definition(HR)]
mod hr {
    /* Employee models ... */
}

#[netabase_macros::netabase_definition(TimeTracking)]
mod time {
    /* Shift models ... */
}

// Create a repository encapsulating both
#[netabase_macros::netabase_repository(EmployeeRepo, definitions(HR, TimeTracking))]
mod employee_repo {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use employee_repo::EmployeeRepoStores;

    // Initialize all stores in the repository
    let stores = EmployeeRepoStores::new("data/employee_repo")?;

    // Access HR store
    let hr_txn = stores.hr.begin_write()?;
    // Access TimeTracking store
    let time_txn = stores.time_tracking.begin_write()?;
    
    Ok(())
}
```

---

## Subscription System

The subscription system organizes models into topics for efficient querying and P2P synchronization.

### Declaring Subscriptions

```rust
// Define subscription topics at the definition level
#[netabase_macros::netabase_definition(MyApp, subscriptions(News, Tech, Sports))]
mod my_app {
    use super::*;

    // Subscribe User to News and Tech topics
    #[derive(NetabaseModel, ...)]
    #[subscribe(News, Tech)]
    pub struct User {
        #[primary_key]
        pub id: String,
        pub name: String,
    }

    // Subscribe Post to Sports topic
    #[derive(NetabaseModel, ...)]
    #[subscribe(Sports)]
    pub struct Post {
        #[primary_key]
        pub id: String,
        pub title: String,
    }
}
```

### Querying by Subscription

Query returns models with their content hashes:

```rust
let txn = store.begin_read()?;

// Query all Users subscribed to News topic
let results = txn.query_by_subscription::<User, _>(&MyAppSubscriptions::News)?;
// Returns: Vec<ModelHash>

for hash in results {
    println!("User hash: {}", hash.to_hex());
}
```

### Selective Subscription Control

Control which topics a model subscribes to at creation time:

```rust
// Default: Subscribe to all model topics (News + Tech for User)
txn.create(&user)?;

// Selective: Subscribe to specific topics only
let topics = vec![MyAppSubscriptions::News];
txn.create_with_subscriptions(&user, Some(topics))?;
// User is only in News topic, not Tech

// No subscriptions: Don't add to any topic
txn.create_with_subscriptions(&user, Some(vec![]))?;
// User can still be queried by primary key, but won't appear in subscription queries
```

### Use Cases

**Selective Subscriptions are useful for**:
- **Privacy control**: Some users might want to be in public topics, others private
- **Feature flags**: Beta features only for users in "Beta" topic
- **Sharding**: Different instances sync different topic subsets
- **Access control**: Topic-based permissions

**Example**:
```rust
// Premium users get all features
let premium_topics = vec![
    AppSubscriptions::Public,
    AppSubscriptions::Premium,
    AppSubscriptions::Beta,
];
txn.create_with_subscriptions(&premium_user, Some(premium_topics))?;

// Free users get limited access
let free_topics = vec![AppSubscriptions::Public];
txn.create_with_subscriptions(&free_user, Some(free_topics))?;

// Query only premium features
let premium_users = txn.query_by_subscription::<User, _>(
    &AppSubscriptions::Premium
)?;
```

---

## Merkle Trees & P2P Sync

Content-addressed hashing enables efficient peer-to-peer synchronization using Merkle trees.

### Building Merkle Trees

```rust
use netabase_store::subscription_hash::{SubscriptionMerkleTree, ModelHash};

// Get all model hashes in a topic
let txn = store.begin_read()?;
let hashes = txn.query_by_subscription::<User, _>(&MyAppSubscriptions::News)?;

// Build Merkle tree
let tree = SubscriptionMerkleTree::from_hashes(hashes);

// Get root hash for comparison
let root = tree.root().unwrap();
println!("Merkle root: {}", hex::encode(root));
```

### Verifying Proofs

Merkle proofs allow efficient verification that a model is in the tree:

```rust
// Generate proof for a specific hash
let hash = hashes[0];
let proof = tree.proof(&hash).expect("Hash should be in tree");

// Verify the proof
assert!(tree.verify_proof(&hash, &proof));
println!("✓ Proof verified successfully");

// Proof verification is O(log n), not O(n)
```

### Comparing Trees for Sync

Compare local and peer trees to find differences:

```rust
// Build local tree
let local_hashes = local_results; // Already Vec<ModelHash>
let local_tree = SubscriptionMerkleTree::from_hashes(local_hashes);

// Build peer tree (from network)
let peer_tree = SubscriptionMerkleTree::from_hashes(peer_hashes);

// Compare trees
let diff = local_tree.diff(&peer_tree);

if diff.has_differences() {
    println!("Sync needed:");
    println!("  Missing in peer: {} items", diff.missing_in_other.len());
    println!("  Missing locally: {} items", diff.missing_in_self.len());
    
    // Request missing items from peer
    for hash in diff.missing_in_self {
        // request_from_peer(hash);
    }
} else {
    println!("✓ Trees are in sync");
}
```

### P2P Synchronization Workflow

```rust
// 1. Compare roots first (fast check)
let local_root = local_tree.root().unwrap();
let peer_root = peer_tree.root().unwrap();

if local_root == peer_root {
    println!("✓ Already in sync");
} else {
    // 2. Find differences
    let diff = local_tree.diff(&peer_tree);
    
    // 3. Request missing items with proofs
    for hash in diff.missing_in_self {
        // Peer sends: (model_data, merkle_proof)
        // let (model, proof) = request_from_peer(hash);
        
        // 4. Verify proof before accepting
        // if peer_tree.verify_proof(&hash, &proof) {
        //     txn.create(&model)?;
        // }
    }
    
    // 5. Send our missing items to peer
    for hash in diff.missing_in_other {
        // let proof = local_tree.proof(&hash)?;
        // send_to_peer(model, proof);
    }
}
```

### Hash Properties

```rust
// Hashes are deterministic
let hash1 = ModelHash::from_data(&user)?;
let hash2 = ModelHash::from_data(&user)?;
assert_eq!(hash1, hash2);

// Convert to/from hex
let hex_str = hash1.to_hex();
let parsed = ModelHash::from_hex(&hex_str)?;
assert_eq!(hash1, parsed);

// Hashes are sortable (for deterministic ordering)
let mut hashes = vec![hash3, hash1, hash2];
hashes.sort();
```

### Integration with Subscriptions

Every subscription query returns hashes automatically:

```rust
// Hashes are maintained automatically during CRUD
txn.create(&user)?;   // Hash computed and stored
txn.update(&user)?;   // Hash recomputed
txn.delete(&user_id)?; // Hash removed

// Query always returns current hashes
let results = txn.query_by_subscription::<User, _>(&topic)?;
for hash in results {
    // hash is always up-to-date with model content
}
```

See `tests/comprehensive_table_tests.rs::test_merkle_tree_construction` for a complete example.

---

## Running Examples

### Main Example

Shows all core features in action:

```bash
cargo run -p example --bin example
```

### Runnable Examples

```bash
# Merkle tree P2P synchronization
cargo run -p example --example merkle_sync

# Selective subscription control
cargo run -p example --example selective_subscriptions
```

### Tests

```bash
# All tests
cargo test -p example

# Specific test
cargo test -p example schema_export
```

### Benchmarks

Performance benchmarks for CRUD operations:

```bash
# Basic CRUD benchmark
cargo bench -p example --bench crud

# Stress test
cargo bench -p example --bench stress

# Record store benchmark
cargo bench -p example --bench record_store
```

# Record store benchmark
cargo bench -p netabase_store_examples --bench record_store
```

---

## Example Files

- **`src/main.rs`**: Complete demonstration of all features
- **`src/boilerplate_lib/mod.rs`**: Main model definitions with migration examples
- **`src/boilerplate_lib/repository_example.rs`**: Advanced repository pattern
- **`src/boilerplate_lib/simple_repo_example.rs`**: Simplified repository example
- **`tests/`**: Integration tests for schema export, import, and migration
- **`benches/`**: Performance benchmarks

---

## Next Steps

1. Read through `src/main.rs` to see all features in action
2. Explore `src/boilerplate_lib/mod.rs` to understand model definitions
3. Check out the tests in `tests/` for integration examples
4. Review the parent crate's README for API documentation

## Common Patterns

### Pattern 1: Simple CRUD

```rust
// Create
let txn = store.begin_write()?;
txn.create(&user)?;
txn.commit()?;

// Read
let txn = store.begin_read()?;
let user = txn.read(&user_id)?;

// Update
let txn = store.begin_write()?;
txn.update(&updated_user)?;
txn.commit()?;

// Delete
let txn = store.begin_write()?;
txn.delete(&user_id)?;
txn.commit()?;
```

### Pattern 2: Querying by Secondary Key

```rust
// Find all users with a specific email
let txn = store.begin_read()?;
let users = txn.query_by_secondary_key::<User>(
    &UserSecondaryKeys::Email("alice@example.com".into())
)?;

// Process results
for user in users {
    println!("Found user: {}", user.first_name);
}
```

### Pattern 3: Transaction Rollback

```rust
let txn = store.begin_write()?;
txn.create(&user)?;

// Oops, something went wrong!
if error_occurred {
    // Transaction is automatically rolled back on drop
    drop(txn);
} else {
    txn.commit()?;
}
```

---

## Troubleshooting

### Compilation Errors

**Error**: "the trait bound `X: NetabaseModel` is not satisfied"
- **Fix**: Add `#[derive(netabase_macros::NetabaseModel)]` to your struct

**Error**: "conflicting implementations"
- **Fix**: Ensure you're using the correct repository type parameter

### Runtime Errors

**Error**: "Primary key already exists"
- **Fix**: Use `update()` instead of `create()`, or check for existence first

**Error**: "Transaction already committed"
- **Fix**: Don't reuse transactions after `commit()`

---

## Additional Resources

- [Parent Crate Documentation](../README.md)
- [Macro Documentation](../netabase_macros/README.md)
- [redb Documentation](https://docs.rs/redb/)
- [postcard Documentation](https://docs.rs/postcard/)
