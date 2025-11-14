# Dependency Management in Netabase Store

## Minimal Dependencies Required! 🎉

`netabase_store` minimizes the dependencies you need to add to your `Cargo.toml`. While derive macros require certain dependencies to be present, you only need to add the core serialization libraries.

## Minimal Cargo.toml

Here's what a typical user's `Cargo.toml` looks like:

```toml
[dependencies]
netabase_store = "0.0.3"

# Required for derive macros (bincode's derives use absolute paths)
bincode = { version = "2.0", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
```

You don't need to add:
- ✅ `derive_more` - Re-exported by netabase_store
- ✅ `strum` - Re-exported by netabase_store
- ✅ `blake3` - Re-exported by netabase_store
- ✅ `paxakos` - Re-exported by netabase_store (if using paxos feature)

## Why Do I Need bincode and serde?

Due to how Rust proc macros work, when you use `#[derive(bincode::Encode)]`, the bincode derive macro generates code with absolute paths like `impl ::bincode::Encode`. This means bincode must be in your dependencies even though we re-export it.

This is standard practice in the Rust ecosystem - even popular crates like `tokio` and `serde` require users to add them as dependencies when using their derive macros.

## Using Derive Macros

Simply use the standard derive syntax with `bincode` and `serde`:

```rust
use netabase_store::{netabase_definition_module, NetabaseModel, netabase};

#[netabase_definition_module(MyDefinition, MyKeys)]
mod models {
    use netabase_store::{NetabaseModel, netabase};

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        // bincode and serde must be in your Cargo.toml
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(MyDefinition)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub name: String,
        #[secondary_key]
        pub email: String,
    }
}
```

## How It Works

The dependency strategy uses the `netabase_deps` crate, which:

1. **Bundles common dependencies**: `bincode`, `serde`, `derive_more`, `strum`, etc.
2. **Re-exports utility crates**: `derive_more`, `strum`, `blake3` through `netabase_store`
3. **Macros use fully qualified paths**: Generated code uses `::netabase_store::derive_more::From`
4. **Ensures version compatibility**: We ensure all dependencies work together

## Available Re-exports

From `netabase_store`, you can access (for direct usage in your code):

| Re-export | Usage | Need in Cargo.toml? |
|-----------|-------|---------------------|
| `netabase_store::bincode` | Serialization functions | **Yes** (for derives) |
| `netabase_store::serde` | JSON serialization | **Yes** (for derives) |
| `netabase_store::derive_more` | Derive utilities (From, TryInto) | No - re-exported |
| `netabase_store::strum` | Enum utilities | No - re-exported |
| `netabase_store::blake3` | Hashing | No - re-exported |
| `netabase_store::paxakos` | Consensus | No - re-exported |

## When You Need Extra Dependencies

### 1. Using serde for JSON operations

```toml
[dependencies]
serde_json = "1.0"  # For JSON serialization beyond derive macros
```

### 2. Choosing a specific database backend

```toml
[dependencies]
netabase_store = { version = "0.0.3", features = ["sled"] }
# or
netabase_store = { version = "0.0.3", features = ["redb", "redb-zerocopy"] }
# or (for WASM)
netabase_store = { version = "0.0.3", features = ["wasm"] }
```

### 3. Additional bincode features

```toml
[dependencies]
bincode = { version = "2.0", features = ["derive", "std"] }  # Add extra features if needed
```

## Best Practices

### ✅ DO: Add bincode and serde to Cargo.toml

```toml
[dependencies]
netabase_store = "0.0.3"
bincode = { version = "2.0", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
```

### ✅ DO: Use standard derive syntax

```rust
#[derive(
    NetabaseModel,
    bincode::Encode,
    bincode::Decode,
    serde::Serialize,
    serde::Deserialize,
)]
```

### ✅ DO: Use netabase features to select backends

```toml
netabase_store = { version = "0.0.3", features = ["redb", "redb-zerocopy"] }
```

### ❌ DON'T: Add derive_more or strum

```toml
# DON'T add these - they're re-exported by netabase_store:
derive_more = "2.0"  # ❌ Not needed!
strum = "0.27"       # ❌ Not needed!
```

## Version Management

By requiring `bincode` and `serde` in your `Cargo.toml`:

- **Versions are coordinated**: We specify compatible versions in our documentation
- **No version conflicts**: Your versions match what netabase_store expects
- **Compatibility is guaranteed**: We test against specific versions
- **Updates are clear**: When we update, we document required version changes

## For Library Authors

If you're building a library on top of `netabase_store`:

1. **Don't expose re-exported types in your public API** if you want API stability
2. **Do document which version of netabase_store you support**
3. **Consider re-exporting netabase_store** for your users:

```rust
pub use netabase_store;
```

## Technical Details

The re-export is implemented in three parts:

1. **`netabase_deps/Cargo.toml`**: Declares all dependencies
2. **`netabase_deps/src/lib.rs`**: Re-exports them under `__private` and publicly
3. **`netabase_store/src/lib.rs`**: Re-exports `netabase_deps` for users

```rust
// In netabase_store/src/lib.rs:
pub use netabase_deps;      // Access as netabase_store::netabase_deps::bincode
pub use netabase_deps::*;   // Access as netabase_store::bincode
```

Macros use fully qualified paths:
```rust
// Generated code:
#[derive(::netabase_store::bincode::Encode)]
```

This ensures:
- ✅ Macro hygiene (no user imports needed)
- ✅ No version conflicts
- ✅ Clear dependency tree
- ✅ Minimal user `Cargo.toml`

## Recommended Setup

### Cargo.toml
```toml
[dependencies]
netabase_store = "0.0.3"

# Required for derive macros
bincode = { version = "2.0", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }

# Optional: for JSON serialization
serde_json = "1.0"
```

### Your Models
```rust
use netabase_store::{NetabaseModel, netabase, netabase_definition_module};

#[netabase_definition_module(MyDef, MyKeys)]
mod models {
    use netabase_store::{NetabaseModel, netabase};

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(MyDef)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub name: String,
    }
}
```

**Benefits:**
- Only 2 extra dependencies needed (bincode, serde)
- No need for derive_more, strum, blake3, etc.
- Standard Rust ecosystem practices
- Clear and predictable behavior

## Summary

The dependency strategy in `netabase_store` means:

1. ✅ **Minimal `Cargo.toml`** - Just add `netabase_store`, `bincode`, and `serde`
2. ✅ **Standard derives** - Use `bincode::Encode`, `serde::Serialize` like normal
3. ✅ **Re-exported utilities** - No need for `derive_more`, `strum`, etc.
4. ✅ **Version compatibility** - We ensure all versions work together

This follows Rust ecosystem best practices and provides a great user experience! 🚀
