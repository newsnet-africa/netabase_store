# Getting Started with Netabase Store

## Required Dependencies

Add these to your `Cargo.toml`:

```toml
[dependencies]
# The main library
netabase_store = "0.0.1"
netabase_deps = "0.0.1"

# Required for the macros to work
bincode = { version = "2.0", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
strum = { version = "0.27.2", features = ["derive"] }
derive_more = { version = "2.0.1", features = ["from", "try_into", "into"] }
anyhow = "1.0"  # For error handling
```

**Why so many dependencies?** The macros generate code that uses `bincode`, `serde`, `strum`, and `derive_more`. Due to Rust's macro hygiene rules, these must be in your `Cargo.toml`. The `netabase_deps` crate provides internal dependencies used by the macros.

## Complete Working Example

Here's a minimal, complete example that you can copy and paste:

```rust
use netabase_store::netabase_definition_module;
use netabase_store::traits::model::NetabaseModelTrait;

#[netabase_definition_module(AppDefinition, AppKeys)]
pub mod app {
    use netabase_store::{NetabaseModel, netabase};

    #[derive(NetabaseModel, bincode::Encode, bincode::Decode, Clone, Debug)]
    #[netabase(AppDefinition)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub username: String,
        #[secondary_key]
        pub email: String,
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
    };
    user_tree.put(alice.clone())?;

    // Retrieve by primary key
    let retrieved = user_tree.get(alice.primary_key())?.unwrap();
    println!("Retrieved: {:?}", retrieved);

    // Query by secondary key
    let users = user_tree.get_by_secondary_key(
        alice.secondary_keys().first().unwrap().clone()
    )?;
    println!("Found {} users with that email", users.len());

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

### 3. Missing `netabase_deps`

**Error:**
```
error: failed to resolve: could not find `netabase_deps` in the list of imported crates
```

**Fix:** Add `netabase_deps = "0.0.1"` to your `Cargo.toml`.

### 4. Incorrect iteration pattern

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
