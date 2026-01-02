# Netabase Store

A type-safe, high-performance embedded database library for Rust with support for multiple backends (redb, sled, IndexedDB for WASM), automatic model migration, and compile-time schema validation.

[![Crates.io](https://img.shields.io/crates/v/netabase_store.svg)](https://crates.io/crates/netabase_store)
[![Documentation](https://docs.rs/netabase_store/badge.svg)](https://docs.rs/netabase_store)
[![License](https://img.shields.io/crates/l/netabase_store.svg)](LICENSE)

## Features

- 🔒 **Type-Safe**: Compile-time schema validation with Rust's type system
- ⚡ **High Performance**: Zero-copy operations with bincode serialization
- 🔄 **Auto Migration**: Automatic schema versioning and data migration
- 🎯 **Multiple Backends**: Redb, Sled, or IndexedDB (WASM)
- 📦 **Unified API**: Same code works across all backends
- 🔍 **Secondary Indexes**: Fast lookups on non-primary fields
- 💾 **Transactions**: ACID-compliant read/write transactions
- 🌐 **Cross-Platform**: Native (Linux, macOS, Windows) and WASM support
- 📚 **Rich Query API**: Builder pattern with pagination, filtering, and ordering
- 🔔 **Subscriptions**: Real-time change notifications (optional feature)

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Core Concepts](#core-concepts)
  - [Models](#models)
  - [Definitions](#definitions)
  - [Stores and Backends](#stores-and-backends)
- [Complete Feature Guide](#complete-feature-guide)
  - [Defining Models](#defining-models)
  - [Primary and Secondary Keys](#primary-and-secondary-keys)
  - [Database Operations](#database-operations)
  - [Transactions](#transactions)
  - [Queries and Pagination](#queries-and-pagination)
  - [Model Migration](#model-migration)
  - [Backend-Specific Features](#backend-specific-features)
- [Advanced Topics](#advanced-topics)
  - [Version Migration](#version-migration)
  - [Batch Operations](#batch-operations)
  - [Custom Serialization](#custom-serialization)
  - [Performance Tuning](#performance-tuning)
- [Examples](#examples)
- [API Reference](#api-reference)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
netabase_store = "0.1"
bincode = "2.0"
serde = { version = "1.0", features = ["derive"] }

# For procedural macros
netabase_macros = "0.1"
```

## Quick Start

Here's a complete example that demonstrates the basics:

```rust
use netabase_store::{netabase_definition, NetabaseModel};
use netabase_store::databases::redb::RedbStore;
use netabase_store::traits::database::transaction::{
    NetabaseRwTransaction, NetabaseRoTransaction
};
use bincode::{Encode, Decode};

// Step 1: Define your models
#[netabase_definition]
mod blog {
    use super::*;

    #[derive(Debug, Clone, PartialEq, NetabaseModel, Encode, Decode)]
    pub struct User {
        #[primary]
        pub id: u64,
        pub username: String,
        #[secondary]
        pub email: String,
    }

    #[derive(Debug, Clone, PartialEq, NetabaseModel, Encode, Decode)]
    pub struct Post {
        #[primary]
        pub id: u64,
        pub title: String,
        pub content: String,
        #[secondary]
        pub author_id: u64,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Step 2: Open a database
    let store = RedbStore::<blog::Blog>::open("my_blog.db")?;

    // Step 3: Create records in a write transaction
    {
        let txn = store.begin_write()?;
        
        let user = blog::User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        };
        
        txn.create(&user)?;
        
        let post = blog::Post {
            id: 1,
            title: "Hello World".to_string(),
            content: "My first post!".to_string(),
            author_id: 1,
        };
        
        txn.create(&post)?;
        txn.commit()?;
    }

    // Step 4: Query records in a read transaction
    {
        let txn = store.begin_read()?;
        
        // Read by primary key
        let user: Option<blog::User> = txn.read(&1u64)?;
        println!("User: {:?}", user);
        
        // Read by secondary key (email)
        let users_by_email = txn.read_by_secondary::<blog::User, _>(&"alice@example.com")?;
        println!("Found {} users with that email", users_by_email.len());
        
        // Query posts by author
        let posts = txn.read_by_secondary::<blog::Post, _>(&1u64)?;
        println!("User has {} posts", posts.len());
    }

    Ok(())
}
```

## Core Concepts

### Models

Models are your data structures, defined as Rust structs with the `#[derive(NetabaseModel)]` attribute. Every model must:

1. Derive `NetabaseModel`, `Clone`, `bincode::Encode`, and `bincode::Decode`
2. Have exactly **one** field marked with `#[primary]`
3. Optionally have fields marked with `#[secondary]` for indexed lookups

```rust
#[derive(NetabaseModel, Clone, Encode, Decode)]
pub struct Product {
    #[primary]
    pub sku: String,           // Primary key - must be unique
    
    pub name: String,
    pub price: u64,
    
    #[secondary]
    pub category: String,      // Indexed for fast lookups
    
    #[secondary]
    pub manufacturer: String,  // Multiple secondary indexes allowed
}
```

### Definitions

A definition is a module that groups related models together, creating a type-safe database schema:

```rust
#[netabase_definition]
mod ecommerce {
    use super::*;
    
    #[derive(NetabaseModel, Clone, Encode, Decode)]
    pub struct Product { /* ... */ }
    
    #[derive(NetabaseModel, Clone, Encode, Decode)]
    pub struct Order { /* ... */ }
    
    #[derive(NetabaseModel, Clone, Encode, Decode)]
    pub struct Customer { /* ... */ }
}

// This generates:
// - ecommerce::Ecommerce (the definition type)
// - ecommerce::Product, ecommerce::Order, ecommerce::Customer (your models)
```

### Stores and Backends

Netabase Store supports multiple storage backends with a unified API:

| Backend | Platform | Use Case | Features |
|---------|----------|----------|----------|
| **Redb** | Native | Production databases | ACID, MVCC, zero-copy reads |
| **Sled** | Native | High-write workloads | Embedded B-tree, fast writes |
| **IndexedDB** | WASM | Browser storage | Async API, persistent |

Creating a store:

```rust
use netabase_store::databases::redb::RedbStore;

// Redb backend
let store = RedbStore::<MyDefinition>::open("data.db")?;

// Sled backend
use netabase_store::databases::sled::SledStore;
let store = SledStore::<MyDefinition>::open("data_dir")?;

// Temporary database (for testing)
let store = RedbStore::<MyDefinition>::temporary()?;
```

## Complete Feature Guide

### Defining Models

#### Basic Model Structure

```rust
#[derive(NetabaseModel, Clone, Encode, Decode)]
pub struct User {
    #[primary]
    pub id: u64,        // Primary key - any type that is Ord + Clone + Encode + Decode
    
    pub username: String,
    pub email: String,
    pub created_at: u64,
}
```

#### Supported Primary Key Types

```rust
// Numeric types
#[primary] pub id: u64;
#[primary] pub id: i32;
#[primary] pub id: u128;

// String types
#[primary] pub uuid: String;
#[primary] pub key: &'static str;

// Tuples (for composite keys)
#[primary] pub id: (u64, String);
#[primary] pub composite: (String, u32, bool);

// Custom types (must implement Ord + Clone + Encode + Decode)
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Encode, Decode)]
pub struct CustomId(u64);

#[primary] pub id: CustomId;
```

#### Field Attributes

```rust
#[derive(NetabaseModel, Clone, Encode, Decode)]
pub struct Article {
    // Primary key - exactly one required
    #[primary]
    pub id: u64,
    
    // Secondary indexes - create indexed lookups (multiple allowed)
    #[secondary]
    pub category: String,
    
    #[secondary]
    pub author_id: u64,
    
    #[secondary]
    pub published_date: u64,
    
    // Regular fields - no special indexing
    pub title: String,
    pub content: String,
    pub views: u32,
}
```

### Primary and Secondary Keys

#### Primary Keys

Every model **must** have exactly one primary key:

```rust
// ✅ CORRECT - One primary key
#[derive(NetabaseModel, Clone, Encode, Decode)]
pub struct User {
    #[primary]
    pub id: u64,
    pub name: String,
}

// ❌ ERROR - No primary key
#[derive(NetabaseModel, Clone, Encode, Decode)]
pub struct InvalidModel {
    pub id: u64,  // Missing #[primary]
    pub name: String,
}

// ❌ ERROR - Multiple primary keys
#[derive(NetabaseModel, Clone, Encode, Decode)]
pub struct InvalidModel {
    #[primary]
    pub id: u64,
    #[primary]  // Error: only one primary key allowed
    pub uuid: String,
}
```

#### Secondary Keys (Indexes)

Secondary keys create indexes for fast lookups:

```rust
#[netabase_definition]
mod library {
    use super::*;
    
    #[derive(NetabaseModel, Clone, Encode, Decode)]
    pub struct Book {
        #[primary]
        pub isbn: String,
        
        pub title: String,
        
        #[secondary]
        pub author: String,      // Index: find books by author
        
        #[secondary]
        pub genre: String,       // Index: find books by genre
        
        #[secondary]
        pub year: u32,           // Index: find books by year
        
        pub pages: u32,
        pub available: bool,
    }
}

// Usage:
let txn = store.begin_read()?;

// Query by secondary key
let scifi_books = txn.read_by_secondary::<library::Book, _>(&"Science Fiction")?;
let books_2024 = txn.read_by_secondary::<library::Book, _>(&2024u32)?;
let asimov_books = txn.read_by_secondary::<library::Book, _>(&"Isaac Asimov")?;
```

### Database Operations

#### Create (Insert)

```rust
let txn = store.begin_write()?;

let user = User {
    id: 1,
    username: "alice".to_string(),
    email: "alice@example.com".to_string(),
    created_at: 1234567890,
};

// Create record
txn.create(&user)?;

// Attempting to create duplicate primary key will fail
let duplicate = User { id: 1, /* ... */ };
txn.create(&duplicate)?;  // Error: key already exists

txn.commit()?;
```

#### Read (Query)

```rust
let txn = store.begin_read()?;

// Read by primary key - returns Option<T>
let user: Option<User> = txn.read(&1u64)?;

match user {
    Some(u) => println!("Found user: {}", u.username),
    None => println!("User not found"),
}

// Read by secondary key - returns Vec<T>
let users: Vec<User> = txn.read_by_secondary::<User, _>(&"alice@example.com")?;

// Read all records of a type
let all_users: Vec<User> = txn.read_all::<User>()?;
```

#### Update (Modify)

```rust
let txn = store.begin_write()?;

// Read existing record
let mut user: User = txn.read(&1u64)?.expect("User not found");

// Modify fields
user.username = "alice_updated".to_string();
user.email = "new_email@example.com".to_string();

// Save changes
txn.update(&user)?;

txn.commit()?;
```

#### Delete (Remove)

```rust
let txn = store.begin_write()?;

// Delete by primary key
txn.delete::<User>(&1u64)?;

// Attempting to delete non-existent key is a no-op (succeeds)
txn.delete::<User>(&999u64)?;  // OK - does nothing

txn.commit()?;
```

#### Batch Operations

```rust
let txn = store.begin_write()?;

// Batch create
for i in 1..=100 {
    let user = User {
        id: i,
        username: format!("user{}", i),
        email: format!("user{}@example.com", i),
        created_at: i * 1000,
    };
    txn.create(&user)?;
}

// Batch update
for i in 1..=50 {
    let mut user: User = txn.read(&i)?.unwrap();
    user.username = format!("updated_{}", user.username);
    txn.update(&user)?;
}

// Batch delete
for i in 51..=100 {
    txn.delete::<User>(&i)?;
}

txn.commit()?;
```

### Transactions

#### Read Transactions

Read transactions provide a consistent snapshot view:

```rust
// Start read transaction
let txn = store.begin_read()?;

// Multiple reads see consistent state
let user1 = txn.read::<User>(&1)?;
let user2 = txn.read::<User>(&2)?;
let all_posts = txn.read_all::<Post>()?;

// Read transactions don't need explicit commit
// They automatically close when dropped
```

#### Write Transactions

Write transactions are ACID-compliant:

```rust
let txn = store.begin_write()?;

// Multiple operations
txn.create(&user1)?;
txn.create(&user2)?;
txn.update(&user3)?;
txn.delete::<Post>(&123)?;

// Commit to persist all changes atomically
txn.commit()?;

// If commit() is not called, all changes are rolled back when txn drops
```

#### Transaction Isolation

```rust
// Create initial data
{
    let txn = store.begin_write()?;
    txn.create(&User { id: 1, username: "alice".into(), /* ... */ })?;
    txn.commit()?;
}

// Start a write transaction but don't commit yet
let write_txn = store.begin_write()?;
let mut user = write_txn.read::<User>(&1)?.unwrap();
user.username = "alice_modified".into();
write_txn.update(&user)?;
// Not committed yet

// Read transactions see the old committed state
{
    let read_txn = store.begin_read()?;
    let user = read_txn.read::<User>(&1)?.unwrap();
    assert_eq!(user.username, "alice");  // Sees old value
}

// Commit the write
write_txn.commit()?;

// Now reads see the new state
{
    let read_txn = store.begin_read()?;
    let user = read_txn.read::<User>(&1)?.unwrap();
    assert_eq!(user.username, "alice_modified");  // Sees new value
}
```

#### Transaction Rollback

```rust
let txn = store.begin_write()?;

txn.create(&user1)?;
txn.create(&user2)?;

// Oops, error occurred
if some_error_condition {
    // Don't call commit() - changes are automatically rolled back
    drop(txn);  // Explicit drop (optional - happens automatically)
    return Err("Operation failed".into());
}

txn.commit()?;  // Only commits if we reach here
```

### Queries and Pagination

#### Query Configuration

The `QueryConfig` type provides a builder API for complex queries:

```rust
use netabase_store::query::QueryConfig;

let txn = store.begin_read()?;

// Basic query - fetch all
let config = QueryConfig::all();
let results = txn.query::<User>(config)?;

// With limit
let config = QueryConfig::default().with_limit(10);
let results = txn.query::<User>(config)?;

// With pagination
let config = QueryConfig::default()
    .with_limit(20)
    .with_offset(40);  // Skip first 40, take next 20
let results = txn.query::<User>(config)?;

// Count only (no data fetching)
let config = QueryConfig::default().count_only();
let count = txn.query::<User>(config)?.count().unwrap();

// Reversed order
let config = QueryConfig::default().reversed();
let results = txn.query::<User>(config)?;

// Combined
let config = QueryConfig::default()
    .with_limit(50)
    .with_offset(100)
    .reversed()
    .no_blobs();  // Exclude large fields
let results = txn.query::<User>(config)?;
```

#### Range Queries

```rust
// Query by primary key range
let config = QueryConfig::new(100u64..200u64);
let users = txn.query::<User>(config)?;

// Open-ended ranges
let config = QueryConfig::new(100u64..);  // From 100 to end
let config = QueryConfig::new(..200u64);  // From start to 200

// Inspection helpers
let config = QueryConfig::inspect_range(0u64..10u64);  // Includes all data
let config = QueryConfig::dump_all();  // Dump entire database
let config = QueryConfig::first();  // Get just first record
```

#### Query Results

```rust
use netabase_store::query::QueryResult;

let result = txn.query::<User>(config)?;

match result {
    QueryResult::Single(Some(user)) => {
        println!("Found one user: {:?}", user);
    }
    QueryResult::Single(None) => {
        println!("No user found");
    }
    QueryResult::Multiple(users) => {
        println!("Found {} users", users.len());
        for user in users {
            println!("  - {}", user.username);
        }
    }
    QueryResult::Count(n) => {
        println!("Total count: {}", n);
    }
}

// Convenience methods
let vec = result.into_vec();  // Convert to Vec<T>
let len = result.len();       // Get count
let is_empty = result.is_empty();

// For testing/assertions
let single = result.unwrap_single();  // Panics if not Single(Some(_))
let single = result.expect_single("should have value");
let single_ref = result.as_single();  // Returns Option<&T>
let multi_ref = result.as_multiple();  // Returns Option<&Vec<T>>
```

### Model Migration

Netabase Store provides automatic schema migration when your models evolve.

#### Versioning Models

Mark model versions with the `#[netabase_version]` attribute:

```rust
#[netabase_definition]
mod users {
    use super::*;

    // Version 1 - initial schema
    #[derive(NetabaseModel, Clone, Encode, Decode)]
    #[netabase_version(family = "User", version = 1)]
    pub struct UserV1 {
        #[primary]
        pub id: u64,
        pub name: String,
    }

    // Version 2 - added email field
    #[derive(NetabaseModel, Clone, Encode, Decode)]
    #[netabase_version(family = "User", version = 2)]
    pub struct UserV2 {
        #[primary]
        pub id: u64,
        pub name: String,
        pub email: String,
    }

    // Version 3 - current version (added age)
    #[derive(NetabaseModel, Clone, Encode, Decode)]
    #[netabase_version(family = "User", version = 3, current)]
    pub struct User {
        #[primary]
        pub id: u64,
        pub name: String,
        pub email: String,
        pub age: u32,
    }
}

// Define migration logic
impl From<users::UserV1> for users::UserV2 {
    fn from(old: users::UserV1) -> Self {
        users::UserV2 {
            id: old.id,
            name: old.name,
            email: String::from("unknown@example.com"),  // Default value
        }
    }
}

impl From<users::UserV2> for users::User {
    fn from(old: users::UserV2) -> Self {
        users::User {
            id: old.id,
            name: old.name,
            email: old.email,
            age: 0,  // Default value
        }
    }
}
```

#### Automatic Migration on Read

When you read old versioned data, it automatically migrates:

```rust
// Old database has V1 data
let store = RedbStore::<users::Users>::open("old_database.db")?;

let txn = store.begin_read()?;

// Automatically migrates V1 → V2 → V3
let user: users::User = txn.read(&1u64)?.expect("User not found");
// user now has all V3 fields with appropriate defaults
```

#### Version Attributes

```rust
// Basic version
#[netabase_version(family = "Product", version = 1)]

// Mark as current version
#[netabase_version(family = "Product", version = 2, current)]

// Allow downgrade (for P2P compatibility)
#[netabase_version(family = "Product", version = 1, supports_downgrade)]
```

#### Migration Context

Control migration behavior:

```rust
use netabase_store::traits::migration::{VersionContext, VersionedDecode};

// Automatic migration (default)
let ctx = VersionContext::new(3).with_auto_migrate(true);

// Strict mode - fail on version mismatch
let ctx = VersionContext::strict(3);

// Custom decoding with context
let user = users::User::decode_versioned(&data, &ctx)?;
```

### Backend-Specific Features

#### Redb Backend

```rust
use netabase_store::databases::redb::RedbStore;

let store = RedbStore::<MyDef>::open("data.db")?;

// Compact database to reclaim space
store.compact()?;

// Check database integrity
store.check_integrity()?;

// Get database statistics
let stats = store.stats()?;
println!("Number of tables: {}", stats.tree_count);
println!("Database size: {} bytes", stats.size_bytes);
```

#### Sled Backend

```rust
use netabase_store::databases::sled::SledStore;

let store = SledStore::<MyDef>::open("data_dir")?;

// Flush to disk immediately
store.flush()?;

// Get size on disk
let size = store.size_on_disk()?;
println!("Database uses {} bytes", size);

// Configure cache size
let config = sled::Config::new()
    .cache_capacity(1024 * 1024 * 100)  // 100 MB cache
    .path("data_dir");
let store = SledStore::<MyDef>::with_config(config)?;
```

#### IndexedDB Backend (WASM)

```rust
#[cfg(target_arch = "wasm32")]
use netabase_store::databases::indexeddb::IndexedDbStore;

// Async API for WASM
let store = IndexedDbStore::<MyDef>::open("my_db").await?;

let txn = store.begin_write().await?;
txn.create(&user).await?;
txn.commit().await?;
```

## Advanced Topics

### Version Migration

See [MIGRATION_GUIDE.md](MIGRATION_GUIDE.md) for comprehensive migration documentation.

Key features:
- Automatic chain migration (V1 → V2 → V3)
- Version header wire format
- Database migration utilities
- P2P schema comparison

### Batch Operations

For high-throughput scenarios:

```rust
let txn = store.begin_write()?;

// Insert 10,000 records
for i in 0..10_000 {
    let record = MyModel {
        id: i,
        data: format!("Record {}", i),
    };
    txn.create(&record)?;
}

txn.commit()?;  // Single commit for all operations
```

### Custom Serialization

Netabase uses bincode for serialization. You can customize it:

```rust
use bincode::{Encode, Decode, config};

#[derive(Encode, Decode)]
struct CustomType {
    data: Vec<u8>,
}

// Use custom bincode configuration
let config = config::standard()
    .with_big_endian()
    .with_fixed_int_encoding();

let bytes = bincode::encode_to_vec(&value, config)?;
```

### Performance Tuning

#### Redb Performance

```rust
// Use read transactions for read-only operations
let txn = store.begin_read()?;  // Faster, allows concurrent reads

// Batch writes in single transaction
let txn = store.begin_write()?;
for item in items {
    txn.create(&item)?;
}
txn.commit()?;  // One commit is faster than many small commits
```

#### Sled Performance

```rust
use sled::Config;

let config = Config::new()
    .path("data")
    .cache_capacity(1024 * 1024 * 1024)  // 1 GB cache
    .flush_every_ms(Some(1000))  // Flush every second
    .mode(sled::Mode::HighThroughput);

let store = SledStore::<MyDef>::with_config(config)?;
```

## Examples

### Complete E-Commerce Example

```rust
use netabase_store::{netabase_definition, NetabaseModel};
use netabase_store::databases::redb::RedbStore;
use netabase_store::traits::database::transaction::{
    NetabaseRwTransaction, NetabaseRoTransaction
};
use bincode::{Encode, Decode};

#[netabase_definition]
mod shop {
    use super::*;

    #[derive(Debug, Clone, NetabaseModel, Encode, Decode)]
    pub struct Customer {
        #[primary]
        pub id: u64,
        pub name: String,
        #[secondary]
        pub email: String,
        pub created_at: u64,
    }

    #[derive(Debug, Clone, NetabaseModel, Encode, Decode)]
    pub struct Product {
        #[primary]
        pub sku: String,
        pub name: String,
        pub price: u64,
        #[secondary]
        pub category: String,
        pub stock: u32,
    }

    #[derive(Debug, Clone, NetabaseModel, Encode, Decode)]
    pub struct Order {
        #[primary]
        pub id: u64,
        #[secondary]
        pub customer_id: u64,
        pub product_sku: String,
        pub quantity: u32,
        pub total_price: u64,
        pub timestamp: u64,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = RedbStore::<shop::Shop>::open("shop.db")?;

    // Add customers
    {
        let txn = store.begin_write()?;
        
        txn.create(&shop::Customer {
            id: 1,
            name: "Alice Johnson".into(),
            email: "alice@example.com".into(),
            created_at: 1704153600,
        })?;
        
        txn.commit()?;
    }

    // Add products
    {
        let txn = store.begin_write()?;
        
        txn.create(&shop::Product {
            sku: "LAPTOP-001".into(),
            name: "Professional Laptop".into(),
            price: 129999,
            category: "Electronics".into(),
            stock: 50,
        })?;
        
        txn.create(&shop::Product {
            sku: "MOUSE-001".into(),
            name: "Wireless Mouse".into(),
            price: 2999,
            category: "Accessories".into(),
            stock: 200,
        })?;
        
        txn.commit()?;
    }

    // Place an order
    {
        let txn = store.begin_write()?;
        
        // Check stock
        let mut product: shop::Product = txn.read(&"LAPTOP-001".to_string())?
            .expect("Product not found");
        
        if product.stock < 1 {
            return Err("Out of stock".into());
        }
        
        // Create order
        txn.create(&shop::Order {
            id: 1,
            customer_id: 1,
            product_sku: "LAPTOP-001".into(),
            quantity: 1,
            total_price: product.price,
            timestamp: 1704240000,
        })?;
        
        // Update stock
        product.stock -= 1;
        txn.update(&product)?;
        
        txn.commit()?;
    }

    // Query customer orders
    {
        let txn = store.begin_read()?;
        
        let customer_orders = txn.read_by_secondary::<shop::Order, _>(&1u64)?;
        println!("Customer has {} orders", customer_orders.len());
        
        for order in customer_orders {
            let product: shop::Product = txn.read(&order.product_sku)?
                .expect("Product not found");
            println!("  - {} x {} = ${}", 
                order.quantity, 
                product.name, 
                order.total_price as f64 / 100.0
            );
        }
    }

    // Find all electronics
    {
        let txn = store.begin_read()?;
        
        let electronics = txn.read_by_secondary::<shop::Product, _>(&"Electronics")?;
        println!("\nElectronics catalog:");
        for product in electronics {
            println!("  - {}: ${} ({} in stock)",
                product.name,
                product.price as f64 / 100.0,
                product.stock
            );
        }
    }

    Ok(())
}
```

### Migration Example

See [MIGRATION_GUIDE.md](MIGRATION_GUIDE.md) and [tests/migration_comprehensive.rs](tests/migration_comprehensive.rs).

### More Examples

Check the [examples/](examples/) directory:
- `basic_store.rs` - Basic CRUD operations
- `batch_operations.rs` - Batch inserts and updates
- `secondary_indexes.rs` - Using secondary key queries
- `redb_basic.rs` - Redb-specific features
- `wasm_example.rs` - WASM/IndexedDB usage

## API Reference

### Core Traits

#### `NetabaseModel`

Derive macro for model structs:

```rust
#[derive(NetabaseModel, Clone, Encode, Decode)]
pub struct MyModel {
    #[primary]
    pub id: u64,
    pub data: String,
}
```

#### `NetabaseRwTransaction`

Write transaction operations:

```rust
trait NetabaseRwTransaction {
    fn create<M: NetabaseModel>(&self, model: &M) -> Result<()>;
    fn read<M: NetabaseModel>(&self, key: &M::Key) -> Result<Option<M>>;
    fn update<M: NetabaseModel>(&self, model: &M) -> Result<()>;
    fn delete<M: NetabaseModel>(&self, key: &M::Key) -> Result<()>;
    fn commit(self) -> Result<()>;
}
```

#### `NetabaseRoTransaction`

Read-only transaction operations:

```rust
trait NetabaseRoTransaction {
    fn read<M: NetabaseModel>(&self, key: &M::Key) -> Result<Option<M>>;
    fn read_by_secondary<M, K>(&self, key: &K) -> Result<Vec<M>>;
    fn read_all<M: NetabaseModel>(&self) -> Result<Vec<M>>;
    fn query<M: NetabaseModel>(&self, config: QueryConfig) -> Result<QueryResult<M>>;
}
```

### QueryConfig API

```rust
impl QueryConfig {
    // Constructors
    fn default() -> Self;
    fn new<R>(range: R) -> QueryConfig<R>;
    fn all() -> QueryConfig<RangeFull>;
    fn first() -> QueryConfig<RangeFull>;
    fn dump_all() -> QueryConfig<RangeFull>;
    fn inspect_range<R>(range: R) -> QueryConfig<R>;
    
    // Builders
    fn with_limit(self, limit: usize) -> Self;
    fn with_offset(self, offset: usize) -> Self;
    fn with_range<NewR>(self, range: NewR) -> QueryConfig<NewR>;
    fn reversed(self) -> Self;
    fn count_only(self) -> Self;
    fn no_blobs(self) -> Self;
    fn with_blobs(self, include: bool) -> Self;
    fn with_hydration(self, depth: usize) -> Self;
    fn no_hydration(self) -> Self;
}
```

### QueryResult API

```rust
impl<T> QueryResult<T> {
    fn into_vec(self) -> Vec<T>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn count(&self) -> Option<u64>;
    
    // For testing
    fn unwrap_single(self) -> T;
    fn expect_single(self, msg: &str) -> T;
    fn as_single(&self) -> Option<&T>;
    fn as_multiple(&self) -> Option<&Vec<T>>;
}
```

## Testing

Run all tests:

```bash
# All tests
cargo test

# Integration tests only
cargo test --test '*'

# Doctests only
cargo test --doc

# Specific test file
cargo test --test migration_comprehensive

# With output
cargo test -- --nocapture
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Documentation

- [Getting Started Guide](GETTING_STARTED.md)
- [Migration Guide](MIGRATION_GUIDE.md)
- [Architecture Overview](docs/ARCHITECTURE.md)
- [API Documentation](https://docs.rs/netabase_store)
- [Test Coverage Report](TEST_COVERAGE.md)

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history and release notes.
