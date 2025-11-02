# Netabase Macro Guide

## Table of Contents
- [Overview](#overview)
- [The `netabase_definition_module` Macro](#the-netabase_definition_module-macro)
- [Generated Code Structure](#generated-code-structure)
- [The `NetabaseModel` Derive Macro](#the-netabasemodel-derive-macro)
- [Secondary Key Generation](#secondary-key-generation)
- [RecordStoreExt Implementation](#recordstoreext-implementation)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

---

## Overview

Netabase uses procedural macros to generate type-safe database code. This guide explains **what happens behind the scenes** when you use these macros, **what code gets generated**, and **why**.

### Why Macros?

Netabase macros provide:
- **Type Safety**: All database operations are checked at compile time
- **Zero-Cost Abstractions**: Generated code compiles to efficient machine code
- **Boilerplate Reduction**: Eliminates hundreds of lines of repetitive code
- **Automatic Trait Implementation**: Generates all necessary trait impls for storage and networking

---

## The `netabase_definition_module` Macro

The `netabase_definition_module` macro is the primary entry point. It wraps a module containing your data models and generates all the infrastructure needed for type-safe database operations.

### Basic Usage

```rust
use netabase_store::*;

#[netabase_definition_module(BlogDefinition, BlogKeys)]
mod blog {
    use netabase_store::{NetabaseModel, netabase};

    #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize)]
    #[netabase(BlogDefinition)]
    pub struct Post {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,
        #[secondary_key]
        pub author_id: u64,
    }

    #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize)]
    #[netabase(BlogDefinition)]
    pub struct Comment {
        #[primary_key]
        pub id: u64,
        pub post_id: u64,
        pub text: String,
        #[secondary_key]
        pub author_id: u64,
    }
}
```

### What Gets Generated

The macro generates:

1. **Definition Enum** - A tagged union of all your models
2. **Keys Enum** - A union of all primary key types
3. **Secondary Key Types** - Newtype wrappers for type-safe queries
4. **Trait Implementations** - All necessary trait impls
5. **Helper Functions** - Utility functions for serialization and routing

---

## Generated Code Structure

### 1. Definition Enum

The Definition enum is the core type that wraps all your models:

```rust
// GENERATED CODE (simplified for clarity)
#[derive(Debug, Clone, strum::EnumDiscriminants, derive_more::From, derive_more::TryInto,
         bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize, strum::Display)]
#[strum_discriminants(derive(Hash, strum::EnumIter, strum::EnumString, strum::Display,
                             strum::AsRefStr, bincode::Encode, bincode::Decode))]
pub enum BlogDefinition {
    Post(Post),
    Comment(Comment),
}
```

**Why this exists:**
- Provides a single type that can represent any of your models
- Enables dynamic dispatch while maintaining type safety
- The discriminant (enum variant name) becomes the "table name" in storage
- `EnumDiscriminants` generates `BlogDefinitionDiscriminants` for pattern matching without data

### 2. Keys Enum

The Keys enum wraps all primary key types:

```rust
// GENERATED CODE
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, strum::EnumDiscriminants,
         derive_more::From, derive_more::TryInto, bincode::Encode, bincode::Decode)]
#[strum_discriminants(derive(Hash, strum::EnumString, strum::AsRefStr, strum::Display,
                             bincode::Encode, bincode::Decode))]
pub enum BlogKeys {
    Post(PostPrimaryKey),
    Comment(CommentPrimaryKey),
}
```

**Why this exists:**
- Allows generic operations over any primary key type
- Used for cross-tree operations and batching
- Maintains the relationship between keys and their models

### 3. Secondary Key Types

For each `#[secondary_key]` field, the macro generates a newtype wrapper:

```rust
// GENERATED CODE for Post.author_id
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
         derive_more::From, derive_more::Into, bincode::Encode, bincode::Decode)]
pub struct PostAuthorIdSecondaryKey(pub u64);

// GENERATED CODE for Comment.author_id
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
         derive_more::From, derive_more::Into, bincode::Encode, bincode::Decode)]
pub struct CommentAuthorIdSecondaryKey(pub u64);
```

**Important: Model Name Prefixing**

Notice that secondary keys are prefixed with the model name:
- `Post.author_id` → `PostAuthorIdSecondaryKey`
- `Comment.author_id` → `CommentAuthorIdSecondaryKey`

This prevents naming conflicts when multiple models have fields with the same name.

**Why newtypes exist:**
- Type safety: Can't accidentally query with the wrong key type
- Clear APIs: `get_by_secondary_key(PostSecondaryKeys::AuthorId(PostAuthorIdSecondaryKey(5)))`
- Zero runtime cost: Newtypes are compile-time only

### 4. Secondary Key Enums

For each model with secondary keys, an enum is generated:

```rust
// GENERATED CODE
#[strum_discriminants(derive(strum::Display, strum::AsRefStr))]
pub enum PostSecondaryKeys {
    AuthorId(PostAuthorIdSecondaryKey),
}

#[strum_discriminants(derive(strum::Display, strum::AsRefStr))]
pub enum CommentSecondaryKeys {
    AuthorId(CommentAuthorIdSecondaryKey),
}
```

**Why this exists:**
- Provides a type-safe API for secondary key queries
- The discriminant becomes the index name in storage
- Extensible: Easy to add new secondary keys

### 5. Trait Implementations

The macro implements several essential traits:

#### NetabaseDefinitionTrait

```rust
// GENERATED CODE
impl NetabaseDefinitionTrait for BlogDefinition {
    type Keys = BlogKeys;
}
```

**Why:** Establishes the relationship between the definition and its keys.

#### RecordStoreExt (when `libp2p` feature is enabled)

```rust
// GENERATED CODE (simplified)
#[cfg(feature = "libp2p")]
impl RecordStoreExt for BlogDefinition {
    fn handle_sled_put(&self, store: &SledStore<Self>) -> libp2p::kad::store::Result<()> {
        match self {
            Self::Post(model) => {
                let tree = store.open_tree::<Post>();
                tree.put_raw(model.clone())?;
                Ok(())
            }
            Self::Comment(model) => {
                let tree = store.open_tree::<Comment>();
                tree.put_raw(model.clone())?;
                Ok(())
            }
        }
    }

    fn handle_sled_get(store: &SledStore<Self>, key: &RecordKey) -> Option<(Self, Record)> {
        let (discriminant, key_bytes) = decode_record_key::<BlogDefinition>(key)?;
        match discriminant {
            disc if disc.to_string() == "Post" => {
                let tree = store.open_tree::<Post>();
                let primary_key: PostPrimaryKey = bincode::decode_from_slice(&key_bytes, config).ok()?.0;
                let model = tree.get_raw(primary_key)?;
                let definition = BlogDefinition::Post(model);
                let record = /* create libp2p Record */;
                Some((definition, record))
            }
            disc if disc.to_string() == "Comment" => { /* similar */ }
            _ => None
        }
    }

    // Similar for remove, and for redb/memory/indexeddb backends
}
```

**Why this exists:**
- Enables libp2p Kademlia DHT integration
- Routes network operations to the correct storage tree
- Handles serialization/deserialization of network records

#### ToIVec

```rust
// GENERATED CODE
impl ToIVec for BlogDefinition {}
impl ToIVec for BlogKeys {}
```

**Why:** Enables conversion to/from byte vectors for storage backends.

### 6. Helper Functions

The macro generates helper functions used internally:

```rust
// GENERATED CODE
#[cfg(feature = "libp2p")]
fn decode_record_key<D>(
    key: &libp2p::kad::RecordKey
) -> Option<(D::Discriminant, Vec<u8>)>
where
    D: strum::IntoDiscriminant,
    D::Discriminant: bincode::Decode<()>,
{
    let bytes = key.to_vec();
    let separator_pos = bytes.iter().position(|&b| b == b':')?;

    // Decode discriminant (determines which model/tree)
    let disc_bytes = &bytes[..separator_pos];
    let (discriminant, _) = bincode::decode_from_slice(disc_bytes, bincode::config::standard()).ok()?;

    // Extract primary key bytes
    let key_bytes = bytes[separator_pos + 1..].to_vec();
    Some((discriminant, key_bytes))
}
```

**Why:** Network record keys use the format `<discriminant>:<primary_key>` to enable routing without decoding the entire value.

---

## The `NetabaseModel` Derive Macro

The `NetabaseModel` macro is applied to individual structs within your definition module.

### Usage

```rust
#[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode,
         serde::Serialize, serde::Deserialize)]
#[netabase(BlogDefinition)]
pub struct Post {
    #[primary_key]
    pub id: u64,
    pub title: String,
    #[secondary_key]
    pub author_id: u64,
}
```

### What Gets Generated

#### 1. Primary Key Type

```rust
// GENERATED CODE
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
         derive_more::From, derive_more::Into, bincode::Encode, bincode::Decode)]
pub struct PostPrimaryKey(pub u64);
```

#### 2. NetabaseModelTrait Implementation

```rust
// GENERATED CODE
impl NetabaseModelTrait<BlogDefinition> for Post {
    type PrimaryKey = PostPrimaryKey;
    type SecondaryKeys = PostSecondaryKeys;

    fn primary_key(&self) -> Self::PrimaryKey {
        PostPrimaryKey(self.id)
    }
}
```

**Why:** Provides a consistent interface for all models, enabling generic code.

#### 3. OpenTree Implementation

```rust
// GENERATED CODE
impl OpenTree<Post> for SledStore<BlogDefinition> {
    fn open_tree(&self) -> SledStoreTree<BlogDefinition, Post> {
        let discriminant = BlogDefinitionDiscriminants::Post;
        let tree_name = discriminant.as_ref();  // Returns "Post"
        let tree = self.db().open_tree(tree_name).unwrap();
        SledStoreTree::new(tree, self.db())
    }
}

// Similar impls for RedbStore, MemoryStore, etc.
```

**Why:** Enables `store.open_tree::<Post>()` syntax, using the discriminant as the tree name.

---

## Secondary Key Generation

### How It Works

When you mark a field with `#[secondary_key]`:

```rust
pub struct Post {
    #[primary_key]
    pub id: u64,
    #[secondary_key]
    pub author_id: u64,  // This field
}
```

The macro:

1. **Extracts the field name**: `author_id`
2. **Converts to PascalCase**: `AuthorId`
3. **Prefixes with model name**: `PostAuthorId`
4. **Suffixes with "SecondaryKey"**: `PostAuthorIdSecondaryKey`

### Storage Structure

Secondary keys are stored as separate indexes. For SledDB:

```
Primary tree: "Post"
  - Key: bincode(PostPrimaryKey(1))
  - Value: bincode(Post { id: 1, title: "...", author_id: 5 })

Secondary tree: "Post:AuthorId"
  - Key: bincode((PostAuthorIdSecondaryKey(5), PostPrimaryKey(1)))
  - Value: empty (the key IS the data)
```

**Why this structure:**
- Efficient range queries on secondary keys
- Multiple records can share the same secondary key value
- Minimal storage overhead (no value duplication)

### Query Example

```rust
let tree = store.open_tree::<Post>();

// Query by secondary key
let posts_by_author = tree.get_by_secondary_key(
    PostSecondaryKeys::AuthorId(PostAuthorIdSecondaryKey(5))
)?;

// Returns Vec<Post> containing all posts where author_id == 5
```

---

## RecordStoreExt Implementation

When the `libp2p` feature is enabled, the macro generates a `RecordStoreExt` implementation for use with libp2p's Kademlia DHT.

### Why It's Needed

libp2p's `RecordStore` trait operates on generic key-value pairs:
- Keys: `RecordKey` (opaque bytes)
- Values: `Record` (opaque bytes)

But netabase needs to:
1. Determine which model type (Post, Comment, etc.)
2. Route to the correct storage tree
3. Handle secondary key indexing
4. Maintain type safety

### How It Works

#### Record Key Format

Network keys use this format:
```
<discriminant_bytes>:<primary_key_bytes>
```

Example:
```
bincode(BlogDefinitionDiscriminants::Post) + b':' + bincode(PostPrimaryKey(1))
```

#### Put Operation Flow

```rust
// User code
netabase.put_record(BlogDefinition::Post(post))?;

// Generated code handles routing:
match self {
    BlogDefinition::Post(model) => {
        // 1. Open the correct tree
        let tree = store.open_tree::<Post>();

        // 2. Store in primary tree
        tree.put_raw(model.clone())?;

        // 3. Update secondary indexes automatically
        // (handled by put_raw implementation)

        Ok(())
    }
    // ... other variants
}
```

#### Get Operation Flow

```rust
// User code
let record = netabase.get_record(&record_key)?;

// Generated code handles deserialization:
// 1. Decode discriminant from key
let (discriminant, key_bytes) = decode_record_key(key)?;

// 2. Route to correct tree based on discriminant
match discriminant {
    BlogDefinitionDiscriminants::Post => {
        let tree = store.open_tree::<Post>();
        let primary_key: PostPrimaryKey = bincode::decode_from_slice(&key_bytes, config)?.0;
        let model = tree.get_raw(primary_key)?;
        Some(BlogDefinition::Post(model))
    }
    // ... other variants
}
```

### Feature Flags

The `RecordStoreExt` implementation is only generated when:
- `libp2p` feature is enabled on netabase_store
- Corresponding backend features (`sled`, `redb`) are enabled in the consuming crate

**Important:** The consuming crate must pass through these features:

```toml
# In your application's Cargo.toml
[features]
default = ["sled", "redb"]
sled = ["netabase_store/sled"]
redb = ["netabase_store/redb"]
```

---

## Best Practices

### 1. Always Derive Required Traits

Your models must derive:
```rust
#[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode,
         serde::Serialize, serde::Deserialize)]
```

**Why each is needed:**
- `Clone`: Required for internal operations
- `Debug`: Essential for debugging and error messages
- `bincode::Encode/Decode`: Storage serialization
- `serde::Serialize/Deserialize`: Network serialization (required for RecordStoreExt)

### 2. Choose Appropriate Key Types

Primary keys should be:
- Cheap to copy (use `u64`, `String`, `uuid::Uuid`)
- Unique (obviously!)
- Ideally sortable

Secondary key types should be:
- Cheap to clone
- Frequently queried values
- Have good cardinality (not too many duplicates)

### 3. Naming Conventions

- Models: PascalCase (`Post`, `UserProfile`)
- Fields: snake_case (`author_id`, `created_at`)
- Generated keys: Automatically follow Rust conventions

### 4. Feature Organization

In your application's `Cargo.toml`:

```toml
[features]
default = ["native"]
native = ["sled", "redb", "netabase_store/native"]
sled = ["netabase_store/sled"]
redb = ["netabase_store/redb"]
libp2p = ["netabase_store/libp2p"]
```

---

## Troubleshooting

### "Cannot find type `XxxSecondaryKey` in this scope"

**Problem:** You're using the old non-prefixed naming.

**Solution:** Update to model-prefixed names:
```rust
// OLD (wrong)
PostSecondaryKeys::AuthorId(AuthorIdSecondaryKey(5))

// NEW (correct)
PostSecondaryKeys::AuthorId(PostAuthorIdSecondaryKey(5))
```

### "Trait `RecordStoreExt` is not implemented"

**Problem:** Feature flags not properly configured.

**Solution:** Ensure your crate's `Cargo.toml` has:
```toml
[features]
sled = ["netabase_store/sled"]
redb = ["netabase_store/redb"]
```

### "The trait bound `XxxDiscriminant: NetabaseDiscriminant` is not satisfied"

**Problem:** Missing derives on the enum generated by `strum_discriminants`.

**Solution:** This should be automatic. If it occurs, ensure:
1. You're using the latest version of netabase_store
2. Your models derive all required traits
3. You're not mixing incompatible versions of dependencies

### Compilation Very Slow

**Problem:** Proc macros can slow down compilation.

**Solution:**
- Use `cargo check` for fast feedback during development
- Enable incremental compilation: `cargo build --timings` to identify bottlenecks
- Consider splitting large definition modules into separate crates

---

## Advanced Topics

### Multiple Definition Modules

You can have multiple definition modules in one application:

```rust
#[netabase_definition_module(BlogDefinition, BlogKeys)]
mod blog { /* ... */ }

#[netabase_definition_module(UserDefinition, UserKeys)]
mod users { /* ... */ }

// Use separate stores for each
let blog_store = SledStore::<BlogDefinition>::new("./blog_data")?;
let user_store = SledStore::<UserDefinition>::new("./user_data")?;
```

### Cross-Tree Operations

The generated `Keys` enum enables operations across different model types:

```rust
// Delete related records atomically
let mut batch = store.batch()?;
batch.delete::<Post>(post_key)?;
batch.delete::<Comment>(comment_key)?;
batch.commit()?;
```

### Custom Serialization

While bincode is used by default, you can implement custom serialization by implementing the `ToIVec` trait manually (advanced users only).

---

## Conclusion

Netabase macros generate a significant amount of boilerplate code, but every piece serves a specific purpose:

- **Type safety** through newtypes and enums
- **Efficient storage** through discriminant-based routing
- **Network integration** through RecordStoreExt
- **Developer ergonomics** through derive macros

Understanding what happens behind the scenes helps you:
- Write more efficient code
- Debug issues faster
- Make informed architectural decisions
- Contribute to the project

For more examples, see the [examples directory](./examples/) and [GETTING_STARTED.md](./GETTING_STARTED.md).
