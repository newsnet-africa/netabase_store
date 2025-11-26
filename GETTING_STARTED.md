# Getting Started with Netabase Store

This guide will walk you through using `netabase_store` from the very beginning.

## Table of Contents

1. [Installation](#installation)
2. [Understanding the Basics](#understanding-the-basics)
3. [Your First Model](#your-first-model)
4. [CRUD Operations](#crud-operations)
5. [Secondary Keys](#secondary-keys)
6. [Understanding Generated Types](#understanding-generated-types)
7. [Common Mistakes](#common-mistakes)
8. [Next Steps](#next-steps)

## Installation

Add these dependencies to your `Cargo.toml`:

```toml
[package]
name = "my_project"
version = "0.1.0"
edition = "2021"


[dependencies]
netabase_store = { version = "0.0.6", features = ["native", "redb"] }

# Required dependencies
bincode = { version = "2.0", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
strum = { version = "0.27.2", features = ["derive"] }
derive_more = { version = "2.0.1", features = ["from", "try_into", "into"] }
libp2p = "0.56"
anyhow = "1.0"
```


## Understanding the Basics

Netabase Store provides three key concepts:

1. **Models** - Your data structures (like `User`, `Post`, `Product`)
2. **Definition** - A schema that groups related models together
3. **Store** - The database backend (Sled, Redb, or IndexedDB for WASM)

Think of it like this:
- **Models** = Tables in traditional databases
- **Definition** = Database schema
- **Store** = The database itself

## Your First Model

Let's create a simple user model:

```rust
use netabase_store::{netabase_definition_module, NetabaseModel, netabase};

// Step 1: Define a schema module
#[netabase_definition_module(AppSchema, AppKeys)]
mod app {
    use netabase_store::{NetabaseModel, netabase};

    // Step 2: Define your model
    #[derive(
        NetabaseModel,           // The netabase_store derive
        Clone,                   // Required: for internal operations
        Debug,                   // Recommended: for debugging
        bincode::Encode,         // Required: for serialization
        bincode::Decode,         // Required: for deserialization
        serde::Serialize,        // REQUIRED: for macro-generated code
        serde::Deserialize,      // REQUIRED: for macro-generated code
    )]
    #[netabase(AppSchema)]       // Link to the schema
    pub struct User {
        #[primary_key]           // Every model needs exactly ONE primary key
        pub id: u64,
        pub name: String,
        pub email: String,
        pub age: u32,
    }
}

// Step 3: Import the generated types
use app::*;
```

### What Just Happened?

The macros generated several types for you:

- `AppSchema` - An enum that can hold any model (User, Post, etc.)
- `AppKeys` - An enum for all model keys
- `UserPrimaryKey` - A newtype wrapper for the user's ID
- `UserKeys` - An enum combining primary and secondary keys
- And several trait implementations

## CRUD Operations

Now let's use our model with a database:

```rust
use netabase_store::databases::sled_store::SledStore;
use netabase_store::traits::tree::NetabaseTreeSync;
use netabase_store::traits::model::NetabaseModelTrait;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Create a store (database)
    let store = SledStore::<AppSchema>::temp()?;  // Temporary for testing

    // Step 2: Open a tree for User models
    let user_tree = store.open_tree::<User>();

    // CREATE: Add a new user
    let alice = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        age: 30,
    };
    user_tree.put(alice.clone())?;
    println!("✓ Created user: {}", alice.name);

    // READ: Get the user back
    let retrieved = user_tree.get(UserPrimaryKey(1))?;
    if let Some(user) = retrieved {
        println!("✓ Retrieved user: {} ({})", user.name, user.email);
    }

    // Or use the generated primary key type:
    // let retrieved = user_tree.get(UserPrimaryKey(1))?;

    // UPDATE: Modify the user
    let mut alice_updated = alice.clone();
    alice_updated.age = 31;
    user_tree.put(alice_updated.clone())?;
    println!("✓ Updated age to 31");

    // DELETE: Remove the user
    user_tree.remove(alice_updated.primary_key())?;
    println!("✓ Deleted user");

    // Verify deletion
    assert!(user_tree.get(UserPrimaryKey(1))?.is_none());

    Ok(())
}
```

## Secondary Keys

Secondary keys allow you to query models by fields other than the primary key:

```rust
#[netabase_definition_module(AppSchema, AppKeys)]
mod app {
    use netabase_store::{NetabaseModel, netabase};

    #[derive(
        NetabaseModel, Clone, Debug,
        bincode::Encode, bincode::Decode,
        serde::Serialize, serde::Deserialize,  // REQUIRED
    )]
    #[netabase(AppSchema)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub name: String,

        #[secondary_key]        // Add this!
        pub email: String,

        #[secondary_key]        // You can have multiple
        pub department: String,
    }
}

use app::*;
use app::AsUserEmail;  // Generated trait for ergonomic queries
use app::AsUserDepartment;

fn query_examples() -> Result<(), Box<dyn std::error::Error>> {
    let store = SledStore::<AppSchema>::temp()?;
    let user_tree = store.open_tree::<User>();

    // Add some users
    user_tree.put(User {
        id: 1,
        name: "Alice".into(),
        email: "alice@example.com".into(),
        department: "Engineering".into(),
    })?;

    user_tree.put(User {
        id: 2,
        name: "Bob".into(),
        email: "bob@example.com".into(),
        department: "Engineering".into(),
    })?;

    // Query by email (ergonomic way)
    let users = user_tree.get_by_secondary_key(
        "alice@example.com".as_user_email_key()
    )?;
    println!("Found {} user(s) with that email", users.len());

    // Query by department
    let engineers = user_tree.get_by_secondary_key(
        "Engineering".as_user_department_key()
    )?;
    println!("Found {} engineers", engineers.len());

    // Or use the verbose syntax:
    let users_verbose = user_tree.get_by_secondary_key(
        UserSecondaryKeys::Email(UserEmailSecondaryKey("alice@example.com".to_string()))
    )?;

    Ok(())
}
```

## Understanding Generated Types

When you write:

```rust
#[derive(NetabaseModel, Clone, bincode::Encode, bincode::Decode)]
#[netabase(AppSchema)]
pub struct User {
    #[primary_key]
    pub id: u64,

    #[secondary_key]
    pub email: String,
}
```

The macros generate:

### 1. Primary Key Type
```rust
pub struct UserPrimaryKey(pub u64);
```
- Type-safe wrapper for the primary key
- Prevents accidentally using a `PostPrimaryKey` with a `User` tree

### 2. Secondary Key Types
```rust
pub struct UserEmailSecondaryKey(pub String);

pub enum UserSecondaryKeys {
    Email(UserEmailSecondaryKey),
}
```
- Each secondary key gets its own newtype
- Prefixed with model name to avoid conflicts

### 3. Extension Traits (for ergonomics)
```rust
pub trait AsUserEmail {
    fn as_user_email_key(&self) -> UserSecondaryKeys;
}

// Implemented for String, &str, &String
impl AsUserEmail for String { ... }
impl AsUserEmail for &str { ... }
```
- Makes querying more ergonomic
- Converts `"email@example.com"` to the correct secondary key type

### 4. Combined Keys Enum
```rust
pub enum UserKey {
    Primary(UserPrimaryKey),
    Secondary(UserSecondaryKeys),
}
```
- Used internally for batch operations


## Next Steps

- **[README.md](./README.md)** - Advanced features and API reference
- **[Architecture Documentation](./docs/ARCHITECTURE.md)** - Detailed technical design
- **Examples**: Check the `examples/` directory for working code
