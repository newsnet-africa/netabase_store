# Getting Started with Netabase Store

## Required Dependencies

Add these to your `Cargo.toml`:

```toml
[dependencies]
# The main library
netabase_store = { version = "0.1", features = ["native"] }

# Required for the macros to work
bincode = "2.0.1"
serde = { version = "1.0", features = ["derive"] }
strum = { version = "0.27.2", features = ["derive"] }
derive_more = "2.0.1"
anyhow = "1.0"  # For error handling
```

For GitHub-hosted versions, use:

```toml
[dependencies]
netabase_store = { git = "https://github.com/newsnet-africa/netabase_store.git", features = ["native"] }
netabase_deps = { git = "https://github.com/newsnet-africa/netabase_store.git" }
bincode = "2.0.1"
serde = { version = "1.0", features = ["derive"] }
strum = { version = "0.27.2", features = ["derive"] }
derive_more = "2.0.1"
anyhow = "1.0"
```

## Required Imports

**IMPORTANT**: Inside your definition module, you MUST import the `netabase` attribute:

```rust
use netabase_store::{netabase_definition_module, NetabaseModel};

#[netabase_definition_module(AppDefinition, AppKeys)]
mod app {
    use super::*;
    use netabase_store::netabase;  // ⚠️  REQUIRED!

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(AppDefinition)]  // Uses the imported attribute
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub username: String,
        #[secondary_key]
        pub email: String,
    }
}

use app::*;
```

## Complete Working Example

```rust
use netabase_store::{netabase_definition_module, NetabaseModel};

#[netabase_definition_module(AppDefinition, AppKeys)]
mod app {
    use super::*;
    use netabase_store::netabase;  // Required!

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(AppDefinition)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub username: String,
        #[secondary_key]
        pub email: String,
        pub age: u32,
    }
}

use app::*;

fn main() -> anyhow::Result<()> {
    use netabase_store::databases::sled_store::SledStore;

    // Create temporary database
    let store = SledStore::<AppDefinition>::temp()?;
    
    // Open a tree for the User model
    let user_tree = store.open_tree::<User>();
    
    // Create and insert a user
    let alice = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        age: 30,
    };
    user_tree.put(alice.clone())?;
    
    // Retrieve by primary key
    let retrieved = user_tree.get(UserPrimaryKey(1))?.unwrap();
    assert_eq!(retrieved, alice);
    
    // Query by secondary key
    let users = user_tree.get_by_secondary_key(
        UserSecondaryKeys::Email(EmailSecondaryKey("alice@example.com".to_string()))
    )?;
    assert_eq!(users.len(), 1);
    
    // Iterate (returns Result<(Key, Model)> tuples)
    for result in user_tree.iter() {
        let (_key, user) = result?;
        println!("User: {} ({})", user.username, user.email);
    }
    
    Ok(())
}
```

## Common Mistakes

### 1. Forgetting to import `netabase` attribute

**Error:**
```
error: cannot find attribute `netabase` in this scope
```

**Fix:** Add `use netabase_store::netabase;` inside your module definition.

### 2. Missing required dependencies

**Error:**
```
error: cannot find derive macro `Encode` in this scope
```

**Fix:** Add all required dependencies to your `Cargo.toml` as shown above.

### 3. Incorrect iteration pattern

**Error:**
```
error[E0609]: no field `age` on type `({integer}, User)`
```

**Fix:** `iter()` returns `Result<(Key, Model)>`, so destructure it:
```rust
for result in tree.iter() {
    let (_key, user) = result?;  // Correct
    // NOT: let user = result?;
}
```

## Next Steps

- Read the [README](./README.md) for full API documentation
- Check [ARCHITECTURE.md](./ARCHITECTURE.md) to understand how it works
- See [examples/](./examples/) for more examples
- Run tests: `cargo test --features native`
