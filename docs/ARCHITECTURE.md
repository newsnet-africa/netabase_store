# Netabase Store Architecture

This document provides a comprehensive technical overview of the netabase_store architecture, explaining how the macro system generates type-safe database code, how storage backends are implemented, and how data flows through the system.

## Table of Contents

1. [Overview](#overview)
2. [Module Organization](#module-organization)
3. [Backend Implementations](#backend-implementations)
4. [Macro System Deep Dive](#macro-system-deep-dive)
5. [NetabaseStore: Unified API Layer](#netabasestore-unified-api-layer)
6. [Type System and Traits](#type-system-and-traits)
7. [Data Serialization Flow](#data-serialization-flow)
8. [Tree-Based Access Pattern](#tree-based-access-pattern)
9. [libp2p Integration](#libp2p-integration)

---

## Overview

Netabase Store is a type-safe, macro-driven database abstraction layer that supports multiple storage backends (Sled, Redb, Redb ZeroCopy, IndexedDB) and integrates seamlessly with libp2p's Kademlia DHT for distributed storage.

### Key Design Principles

1. **Type Safety:** Compile-time guarantees for data models and queries
2. **Zero-Cost Abstractions:** Macros generate optimal code with no runtime overhead
3. **Backend Agnostic:** Same API works across all supported backends
4. **Modular Architecture:** Clean separation of concerns for maintainability
5. **libp2p Compatible:** Direct integration with Kademlia RecordStore trait
6. **Deterministic Serialization:** Consistent binary format using bincode

### Architecture Layers

1. **User Code Layer** - Models defined with derive macros
2. **Generated Code Layer** - Macro-generated traits and types
3. **Unified API Layer** - Common traits and abstractions
4. **Backend Layer** - Modular storage implementations
5. **Storage Layer** - Underlying database engines

## Module Organization

The codebase has been reorganized with a focus on separation of concerns and maintainability:

```
src/
├── databases/              # Storage backend implementations
│   ├── sled_store/         # Modular Sled backend
│   │   ├── mod.rs         # Public API and re-exports
│   │   ├── store.rs       # Store implementation
│   │   ├── tree.rs        # Tree operations
│   │   ├── batch.rs       # Batch operations
│   │   └── trait_impls.rs # Trait implementations
│   ├── redb_store/         # Modular Redb backend
│   │   ├── mod.rs         # Public API and re-exports
│   │   ├── store.rs       # Store implementation
│   │   ├── tree.rs        # Tree CRUD operations
│   │   ├── batch.rs       # Batch builder
│   │   ├── iterator.rs    # Iterator implementations
│   │   ├── types.rs       # Type definitions
│   │   └── trait_impls.rs # Trait implementations
│   ├── redb_zerocopy/      # High-performance Redb backend
│   │   ├── mod.rs         # Public API and documentation
│   │   ├── store.rs       # Store with transaction management
│   │   ├── transaction.rs # Transaction types
│   │   ├── tree.rs        # Zero-copy tree operations
│   │   └── utils.rs       # Helper functions
│   ├── indexeddb_store.rs  # WASM IndexedDB backend
│   └── record_store/       # libp2p integration
├── traits/                 # Common abstractions
├── error/                  # Error handling
└── lib.rs                 # Public API surface
```

```
┌─────────────────────────────────────────────────────┐
│            User-Defined Models                      │
│  #[derive(NetabaseModel)]                          │
│  struct User { #[primary_key] id: String, ... }    │
└────────────────────┬────────────────────────────────┘
                     │ Macro Expansion
                     ▼
┌─────────────────────────────────────────────────────┐
│         Generated Definition & Keys Enums           │
│  enum MyDefinition { User(User), Post(Post) }      │
│  enum MyKeys { User(UserKey), Post(PostKey) }      │
└────────────────────┬────────────────────────────────┘
                     │ Implements Traits
                     ▼
┌─────────────────────────────────────────────────────┐
│              Trait Layer                            │
│  • NetabaseDefinitionTrait                         │
│  • NetabaseModelTrait                              │
│  • ToIVec / FromIVec (Serialization)               │
│  • RecordStoreExt (libp2p integration)             │
└────────────────────┬────────────────────────────────┘
                     │ Uses
                     ▼
┌─────────────────────────────────────────────────────┐
│        NetabaseStore<D, Backend>                   │
│  • Unified API wrapper (Recommended)               │
│  • Provides backend-agnostic interface             │
│  • Allows backend-specific features                │
└────────────────────┬────────────────────────────────┘
                     │ Wraps
        ┌────────────┴────────────┬──────────────┐
        ▼                         ▼              ▼
┌──────────────┐         ┌──────────────┐  ┌────────────┐
│  SledStore   │         │  RedbStore   │  │ IndexedDB  │
│  <D>         │         │  <D>         │  │  <D>       │
└──────┬───────┘         └──────┬───────┘  └─────┬──────┘
       │                        │                 │
       ▼                        ▼                 ▼
┌─────────────────────────────────────────────────────┐
│         NetabaseTreeSync<D, M> Trait               │
│  • put(model) / get(key) / remove(key)             │
│  • get_by_secondary_key(secondary_key)             │
│  • OpenTree<D, M> / Batchable<D, M>                │
└────────────────────┬────────────────────────────────┘
                     │ Uses
        ┌────────────┴────────────┬──────────────┐
        ▼                         ▼              ▼
┌──────────────┐         ┌──────────────┐  ┌────────────┐
│     Sled     │         │     Redb     │  │ IndexedDB  │
│  (Database)  │         │  (Database)  │  │  (Browser) │
└──────────────┘         └──────────────┘  └────────────┘
```

---

## Macro System Deep Dive

The macro system consists of two main procedural macros that work together to generate type-safe database code.

### 1. `#[derive(NetabaseModel)]` Macro

**File:** `netabase_macros/src/lib.rs`

This macro is applied to individual struct definitions and generates the `NetabaseModelTrait` implementation and associated key types.

#### Input

```rust
#[derive(NetabaseModel, Clone, bincode::Encode, bincode::Decode,
         serde::Serialize, serde::Deserialize)]
#[netabase(MyDefinition)]
pub struct User {
    #[primary_key]
    pub id: u64,
    pub name: String,
    #[secondary_key]
    pub email: String,
    #[secondary_key]
    pub age: u32,
}
```

#### Generated Types

The macro generates several key-related types:

**1. Primary Key Newtype:**
```rust
#[derive(Clone, Debug, bincode::Encode, bincode::Decode)]
pub struct UserPrimaryKey(pub u64);
```

**2. Secondary Key Types:**
```rust
#[derive(Clone, Debug, bincode::Encode, bincode::Decode)]
pub struct UserEmailSecondaryKey(pub String);

#[derive(Clone, Debug, bincode::Encode, bincode::Decode)]
pub struct UserAgeSecondaryKey(pub u32);
```

**3. Secondary Keys Enum:**
```rust
#[derive(Clone, Debug, bincode::Encode, bincode::Decode)]
pub enum UserSecondaryKeys {
    Email(UserEmailSecondaryKey),
    Age(UserAgeSecondaryKey),
}
```

**4. Combined Keys Enum:**
```rust
#[derive(Clone, Debug, bincode::Encode, bincode::Decode)]
pub enum UserKey {
    Primary(UserPrimaryKey),
    Secondary(UserSecondaryKeys),
}
```

#### NetabaseModelTrait Implementation

```rust
impl NetabaseModelTrait<MyDefinition> for User {
    const DISCRIMINANT: MyDefinitionDiscriminant = MyDefinitionDiscriminant::User;

    type PrimaryKey = UserPrimaryKey;
    type SecondaryKeys = UserSecondaryKeys;
    type Keys = UserKey;

    fn primary_key(&self) -> Self::PrimaryKey {
        UserPrimaryKey(self.id)
    }

    fn secondary_keys(&self) -> Vec<Self::SecondaryKeys> {
        vec![
            UserSecondaryKeys::Email(UserEmailSecondaryKey(self.email.clone())),
            UserSecondaryKeys::Age(UserAgeSecondaryKey(self.age)),
        ]
    }

    fn discriminant_name() -> &'static str {
        "User"
    }
}
```

### 2. `#[netabase_definition_module]` Macro

**File:** `netabase_macros/src/lib.rs`

This macro wraps a module containing multiple model definitions and generates the complete database schema.

#### Input

```rust
#[netabase_definition_module(BlogDefinition, BlogKeys)]
mod blog {
    use netabase_store::{NetabaseModel, netabase};

    #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode,
             serde::Serialize, serde::Deserialize)]
    #[netabase(BlogDefinition)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub name: String,
        #[secondary_key]
        pub email: String,
    }

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
}
```

#### Generated Definition Enum

```rust
#[derive(Clone, Debug, bincode::Encode, bincode::Decode,
         serde::Serialize, serde::Deserialize)]
#[derive(strum::EnumDiscriminants, strum::IntoStaticStr)]
#[strum_discriminants(derive(strum::EnumIter, strum::Display,
                             Hash, PartialOrd, Ord, PartialEq, Eq))]
#[strum_discriminants(name(BlogDefinitionDiscriminant))]
pub enum BlogDefinition {
    User(User),
    Post(Post),
}
```

#### Generated Keys Enum

```rust
#[derive(Clone, Debug, bincode::Encode, bincode::Decode)]
#[derive(strum::EnumDiscriminants, strum::IntoStaticStr)]
#[strum_discriminants(derive(strum::EnumIter, strum::Display,
                             Hash, PartialOrd, Ord))]
#[strum_discriminants(name(BlogKeysDiscriminant))]
pub enum BlogKeys {
    User(UserKey),
    Post(PostKey),
}
```

#### NetabaseDefinitionTrait Implementation

```rust
impl NetabaseDefinitionTrait for BlogDefinition {
    type Keys = BlogKeys;

    fn to_key(&self) -> Result<Self::Keys, NetabaseError> {
        match self {
            BlogDefinition::User(model) => {
                Ok(BlogKeys::User(UserKey::Primary(model.primary_key())))
            }
            BlogDefinition::Post(model) => {
                Ok(BlogKeys::Post(PostKey::Primary(model.primary_key())))
            }
        }
    }

    fn discriminant_name(&self) -> &'static str {
        match self {
            BlogDefinition::User(_) => "User",
            BlogDefinition::Post(_) => "Post",
        }
    }
}
```

#### Conversion Traits (ToIVec/FromIVec)

```rust
impl ToIVec for BlogDefinition {
    fn to_ivec(&self) -> Result<IVec, NetabaseError> {
        let bytes = bincode::encode_to_vec(self, bincode::config::standard())
            .map_err(|e| NetabaseError::Serialization(e.to_string()))?;
        Ok(bytes.into())
    }
}

impl FromIVec for BlogDefinition {
    fn from_ivec(ivec: &IVec) -> Result<Self, NetabaseError> {
        let (decoded, _) = bincode::decode_from_slice(
            ivec.as_ref(),
            bincode::config::standard()
        ).map_err(|e| NetabaseError::Deserialization(e.to_string()))?;
        Ok(decoded)
    }
}

// Same implementations for BlogKeys, UserKey, PostKey, etc.
```

---

## Backend Implementation

Netabase Store supports multiple storage backends through the `NetabaseTreeSync` trait interface.

### Tree-Based API: `NetabaseTreeSync<D, M>`

**File:** `src/traits/tree.rs`

The core trait for database operations is `NetabaseTreeSync`, which provides type-safe access to individual model trees:

```rust
pub trait NetabaseTreeSync<D, M> {
    type PrimaryKey;
    type SecondaryKeys;

    /// Insert or update a model
    fn put(&self, model: M) -> Result<(), NetabaseError>;

    /// Get a model by its primary key
    fn get(&self, key: Self::PrimaryKey) -> Result<Option<M>, NetabaseError>;

    /// Delete a model by its primary key
    fn remove(&self, key: Self::PrimaryKey) -> Result<Option<M>, NetabaseError>;

    /// Query models by a secondary key
    fn get_by_secondary_key(&self, key: Self::SecondaryKeys)
        -> Result<Vec<M>, NetabaseError>;
}
```

### Sled Backend Implementation

**File:** `src/databases/sled_store.rs`

#### Structure

```rust
pub struct SledStore<D: NetabaseDefinitionTrait> {
    db: sled::Db,
    _phantom: PhantomData<D>,
}

pub struct SledTree<'a, D: NetabaseDefinitionTrait, M: NetabaseModelTrait<D>> {
    store: &'a SledStore<D>,
    _phantom: PhantomData<M>,
}
```

#### Creating a Store

```rust
impl<D> SledStore<D>
where
    D: NetabaseDefinitionTrait,
{
    pub fn new(path: impl AsRef<Path>) -> Result<Self, NetabaseError> {
        let db = sled::open(path)
            .map_err(|e| NetabaseError::Database(e.to_string()))?;
        Ok(SledStore { db, _phantom: PhantomData })
    }

    pub fn temp() -> Result<Self, NetabaseError> {
        let config = sled::Config::new().temporary(true);
        let db = config.open()
            .map_err(|e| NetabaseError::Database(e.to_string()))?;
        Ok(SledStore { db, _phantom: PhantomData })
    }

    /// Open a type-safe tree for a specific model
    pub fn open_tree<M>(&self) -> SledTree<D, M>
    where
        M: NetabaseModelTrait<D>,
    {
        SledTree {
            store: self,
            _phantom: PhantomData,
        }
    }
}
```

#### NetabaseTreeSync Implementation

```rust
impl<'a, D, M> NetabaseTreeSync<D, M> for SledTree<'a, D, M>
where
    D: NetabaseDefinitionTrait + From<M>,
    M: NetabaseModelTrait<D>,
{
    type PrimaryKey = M::PrimaryKey;
    type SecondaryKeys = M::SecondaryKeys;

    fn put(&self, model: M) -> Result<(), NetabaseError> {
        let tree_name = M::discriminant_name();
        let tree = self.store.db.open_tree(tree_name)?;

        let primary_key = model.primary_key();
        let key_bytes = bincode::encode_to_vec(&primary_key, bincode::config::standard())?;
        let value_bytes = bincode::encode_to_vec(&model, bincode::config::standard())?;

        tree.insert(key_bytes, value_bytes)?;

        // Index secondary keys
        for secondary_key in model.secondary_keys() {
            let sk_bytes = bincode::encode_to_vec(&secondary_key, bincode::config::standard())?;
            let pk_bytes = bincode::encode_to_vec(&primary_key, bincode::config::standard())?;

            let index_tree = self.store.db.open_tree(
                format!("{}__index", tree_name)
            )?;
            index_tree.insert(sk_bytes, pk_bytes)?;
        }

        Ok(())
    }

    fn get(&self, key: Self::PrimaryKey) -> Result<Option<M>, NetabaseError> {
        let tree_name = M::discriminant_name();
        let tree = self.store.db.open_tree(tree_name)?;

        let key_bytes = bincode::encode_to_vec(&key, bincode::config::standard())?;

        match tree.get(key_bytes)? {
            Some(value_ivec) => {
                let (model, _) = bincode::decode_from_slice(
                    value_ivec.as_ref(),
                    bincode::config::standard()
                )?;
                Ok(Some(model))
            }
            None => Ok(None),
        }
    }

    fn remove(&self, key: Self::PrimaryKey) -> Result<Option<M>, NetabaseError> {
        let tree_name = M::discriminant_name();
        let tree = self.store.db.open_tree(tree_name)?;

        let key_bytes = bincode::encode_to_vec(&key, bincode::config::standard())?;

        match tree.remove(key_bytes)? {
            Some(value_ivec) => {
                let (model, _) = bincode::decode_from_slice::<M>(
                    value_ivec.as_ref(),
                    bincode::config::standard()
                )?;

                // Remove secondary key indexes
                for secondary_key in model.secondary_keys() {
                    let sk_bytes = bincode::encode_to_vec(&secondary_key, bincode::config::standard())?;
                    let index_tree = self.store.db.open_tree(
                        format!("{}__index", tree_name)
                    )?;
                    index_tree.remove(sk_bytes)?;
                }

                Ok(Some(model))
            }
            None => Ok(None),
        }
    }

    fn get_by_secondary_key(&self, key: Self::SecondaryKeys)
        -> Result<Vec<M>, NetabaseError>
    {
        let tree_name = M::discriminant_name();
        let tree = self.store.db.open_tree(tree_name)?;
        let index_tree = self.store.db.open_tree(format!("{}__index", tree_name))?;

        let sk_bytes = bincode::encode_to_vec(&key, bincode::config::standard())?;

        let mut results = Vec::new();

        // Find all primary keys with this secondary key
        for item in index_tree.scan_prefix(sk_bytes) {
            let (_, pk_bytes) = item?;

            if let Some(value_ivec) = tree.get(pk_bytes)? {
                let (model, _) = bincode::decode_from_slice(
                    value_ivec.as_ref(),
                    bincode::config::standard()
                )?;
                results.push(model);
            }
        }

        Ok(results)
    }
}
```

---

## Configuration API: BackendStore Trait

**Files:** `src/config.rs`, `src/traits/backend_store.rs`

The unified configuration system provides a consistent, ergonomic way to initialize any database backend with typed configuration objects.

### Design Goals

1. **Consistency**: Same API pattern across all backends (Sled, Redb, RedbZeroCopy, Memory, IndexedDB)
2. **Type Safety**: Compile-time configuration validation with builder pattern
3. **Portability**: Switch backends by changing configuration type, not code structure
4. **Sensible Defaults**: Minimal configuration required, but full control available

### BackendStore Trait

The `BackendStore` trait defines three standard constructors all backends must implement:

```rust
pub trait BackendStore<D: NetabaseDefinitionTrait>: Sized {
    type Config;

    /// Create/open a database with the provided configuration
    fn new(config: Self::Config) -> Result<Self, NetabaseError>;

    /// Open an existing database (fails if missing)
    fn open(config: Self::Config) -> Result<Self, NetabaseError>;

    /// Create a temporary database (for testing)
    fn temp() -> Result<Self, NetabaseError>;
}
```

### Configuration Types

#### FileConfig (for File-Based Backends)

Used by: `SledStore`, `RedbStore`, `RedbStoreZeroCopy`

```rust
#[derive(TypedBuilder, Clone)]
pub struct FileConfig {
    /// Path to database file or directory
    pub path: PathBuf,

    /// Cache size in megabytes (default: 256)
    #[builder(default = 256)]
    pub cache_size_mb: usize,

    /// Create database if it doesn't exist (default: true)
    #[builder(default = true)]
    pub create_if_missing: bool,

    /// Truncate (delete) existing data on open (default: false)
    #[builder(default = false)]
    pub truncate: bool,

    /// Open in read-only mode (default: false)
    #[builder(default = false)]
    pub read_only: bool,

    /// Use fsync for durability (default: true)
    #[builder(default = true)]
    pub use_fsync: bool,
}
```

#### MemoryConfig (for In-Memory Backend)

Used by: `MemoryStore`

```rust
#[derive(TypedBuilder, Clone, Default)]
pub struct MemoryConfig {
    /// Optional capacity hint for pre-allocation
    #[builder(default = None)]
    pub capacity: Option<usize>,
}
```

#### IndexedDBConfig (for WASM/Browser Backend)

Used by: `IndexedDBStore`

```rust
#[derive(TypedBuilder, Clone)]
pub struct IndexedDBConfig {
    /// Name of the IndexedDB database
    pub database_name: String,

    /// Schema version number (default: 1)
    #[builder(default = 1)]
    pub version: u32,
}
```

### Usage Patterns

#### Builder Pattern (Recommended)

The builder pattern provides excellent IDE autocomplete and type safety:

```rust
use netabase_store::config::FileConfig;
use netabase_store::traits::backend_store::BackendStore;
use netabase_store::databases::sled_store::SledStore;

let config = FileConfig::builder()
    .path("my_app.db".into())
    .cache_size_mb(1024)
    .truncate(true)
    .build();

let store = <SledStore<MyDefinition> as BackendStore<MyDefinition>>::new(config)?;
```

#### Simple Constructor

For basic usage with defaults:

```rust
let config = FileConfig::new("my_app.db");
let store = <SledStore<MyDefinition> as BackendStore<MyDefinition>>::open(config)?;
```

#### Temporary Databases (Testing)

No configuration needed:

```rust
let store = <SledStore<MyDefinition> as BackendStore<MyDefinition>>::temp()?;
```

### Backend Portability

The power of this system is backend switching with zero code changes:

```rust
use netabase_store::config::FileConfig;
use netabase_store::traits::backend_store::BackendStore;

let config = FileConfig::builder()
    .path("database.db".into())
    .cache_size_mb(512)
    .build();

// Try different backends - same config!
#[cfg(feature = "sled")]
let store = <SledStore<MyDef> as BackendStore<MyDef>>::new(config.clone())?;

#[cfg(feature = "redb")]
let store = <RedbStore<MyDef> as BackendStore<MyDef>>::new(config.clone())?;

#[cfg(feature = "redb-zerocopy")]
let store = <RedbStoreZeroCopy<MyDef> as BackendStore<MyDef>>::new(config)?;

// All have identical API from this point on!
let tree = store.open_tree::<User>();
tree.put(user)?;
```

### Implementation Examples

#### Sled Backend

```rust
impl<D: NetabaseDefinitionTrait> BackendStore<D> for SledStore<D> {
    type Config = FileConfig;

    fn new(config: Self::Config) -> Result<Self, NetabaseError> {
        let mut sled_config = sled::Config::new()
            .path(&config.path)
            .cache_capacity(config.cache_size_mb * 1024 * 1024);

        if config.truncate {
            sled_config = sled_config.temporary(true);
        }

        let db = sled_config.open()
            .map_err(|e| NetabaseError::Database(e.to_string()))?;

        Ok(SledStore {
            db: Arc::new(db),
            _phantom: PhantomData,
        })
    }

    fn open(config: Self::Config) -> Result<Self, NetabaseError> {
        let mut cfg = config;
        cfg.create_if_missing = false;
        Self::new(cfg)
    }

    fn temp() -> Result<Self, NetabaseError> {
        let config = FileConfig::builder()
            .path(std::env::temp_dir().join(format!("netabase_temp_{}", uuid::Uuid::new_v4())))
            .truncate(true)
            .build();
        Self::new(config)
    }
}
```

#### Redb Backend

```rust
impl<D: NetabaseDefinitionTrait> BackendStore<D> for RedbStore<D> {
    type Config = FileConfig;

    fn new(config: Self::Config) -> Result<Self, NetabaseError> {
        let builder = redb::Builder::new()
            .set_cache_size(config.cache_size_mb * 1024 * 1024);

        let db = if config.truncate && config.path.exists() {
            std::fs::remove_file(&config.path)?;
            builder.create(&config.path)?
        } else if config.create_if_missing {
            builder.create(&config.path)?
        } else {
            builder.open(&config.path)?
        };

        Ok(RedbStore {
            db: Arc::new(db),
            _phantom: PhantomData,
        })
    }

    fn open(config: Self::Config) -> Result<Self, NetabaseError> {
        let mut cfg = config;
        cfg.create_if_missing = false;
        Self::new(cfg)
    }

    fn temp() -> Result<Self, NetabaseError> {
        let config = FileConfig::builder()
            .path(std::env::temp_dir().join(format!("netabase_temp_{}.redb", uuid::Uuid::new_v4())))
            .truncate(true)
            .build();
        Self::new(config)
    }
}
```

### Benefits

1. **Unified Interface**: Same pattern for all backends
2. **Type Safety**: Builder pattern catches configuration errors at compile time
3. **Documentation**: Configuration options self-document in IDE
4. **Testing**: Easy temporary database creation
5. **Portability**: Backend switching requires minimal code changes
6. **Defaults**: Sensible defaults reduce boilerplate
7. **Extensibility**: New backends follow established pattern

### Migration from Old API

**Before:**
```rust
// Different constructors per backend
let sled = SledStore::new("path.db")?;
let redb = RedbStore::open_with_path("path.redb")?;
let temp = SledStore::temp()?;
```

**After:**
```rust
// Consistent API using BackendStore trait
let config = FileConfig::new("path.db");
let sled = <SledStore<D> as BackendStore<D>>::new(config.clone())?;
let redb = <RedbStore<D> as BackendStore<D>>::new(config)?;
let temp = <SledStore<D> as BackendStore<D>>::temp()?;
```

---

## NetabaseStore: Unified API Layer

**File:** `src/store.rs`

The `NetabaseStore<D, Backend>` is a unified wrapper that provides a consistent API across all storage backends. It's the recommended entry point for most applications as it allows you to write backend-agnostic code while still having access to backend-specific features when needed.

### Design Goals

1. **Backend Portability**: Switch between Sled, Redb, or other backends by changing a single line
2. **Type Safety**: Preserve compile-time guarantees while abstracting backend details
3. **Feature Access**: Maintain access to backend-specific optimizations
4. **Zero Overhead**: Compile to same code as direct backend usage

### Structure

```rust
pub struct NetabaseStore<D, Backend>
where
    D: NetabaseDefinitionTrait,
    Backend: BackendFor<D>,
{
    backend: Backend,
    _phantom: PhantomData<D>,
}
```

The `BackendFor<D>` marker trait binds the Definition type to the backend at compile time:

```rust
pub trait BackendFor<D: NetabaseDefinitionTrait> {}

// Implemented for all backends
impl<D> BackendFor<D> for SledStore<D> where D: NetabaseDefinitionTrait {}
impl<D> BackendFor<D> for RedbStore<D> where D: NetabaseDefinitionTrait {}
```

### Creating a Store

The unified API provides multiple constructors for different backends:

```rust
// Sled backend (persistent, high-performance)
let store = NetabaseStore::<MyDefinition, _>::sled("./my_database")?;

// Redb backend (persistent, memory-efficient)
let store = NetabaseStore::<MyDefinition, _>::redb("./my_database.redb")?;

// Temporary Sled store (for testing)
let store = NetabaseStore::<MyDefinition, _>::temp()?;
```

The `_` type parameter uses type inference to determine the backend type from the constructor method.

### Opening Trees

NetabaseStore implements the `OpenTree` trait, providing a generic `open_tree` method:

```rust
impl<D, Backend> NetabaseStore<D, Backend>
where
    D: NetabaseDefinitionTrait,
    Backend: BackendFor<D>,
{
    pub fn open_tree<M>(&self) -> Backend::Tree<'_>
    where
        M: NetabaseModelTrait<D>,
        Backend: OpenTree<D, M>,
    {
        self.backend.open_tree()
    }
}
```

This delegates to the backend's `open_tree` implementation but provides a unified interface:

```rust
let user_tree = store.open_tree::<User>();
let post_tree = store.open_tree::<Post>();
```

### Backend-Specific Features

NetabaseStore uses separate `impl` blocks for backend-specific methods:

```rust
// Sled-specific methods
#[cfg(feature = "sled")]
impl<D> NetabaseStore<D, SledStore<D>>
where
    D: NetabaseDefinitionTrait + ToIVec,
{
    pub fn flush(&self) -> Result<usize, NetabaseError> {
        Ok(self.backend.db().flush()?)
    }

    pub fn generate_id(&self) -> Result<u64, NetabaseError> {
        Ok(self.backend.db().generate_id()?)
    }
}

// Redb-specific methods
#[cfg(feature = "redb")]
impl<D> NetabaseStore<D, RedbStore<D>>
where
    D: NetabaseDefinitionTrait + ToIVec,
{
    pub fn check_integrity(&mut self) -> Result<bool, NetabaseError> {
        self.backend.check_integrity()
    }

    pub fn compact(&mut self) -> Result<bool, NetabaseError> {
        self.backend.compact()
    }
}
```

This allows type-safe access to backend-specific functionality:

```rust
let store = NetabaseStore::<D, _>::sled("./db")?;
store.flush()?; // Only available for Sled backend

let store = NetabaseStore::<D, _>::redb("./db.redb")?;
store.check_integrity()?; // Only available for Redb backend
```

### Usage Pattern

The recommended usage pattern is:

1. **Define your schema** with `NetabaseModel` and `netabase_definition_module`
2. **Create a NetabaseStore** with your desired backend
3. **Open trees** for your model types
4. **Perform operations** using the tree API
5. **Access backend features** when needed

```rust
// 1. Schema defined with macros
#[netabase_definition_module(AppDefinition, AppKeys)]
mod schema {
    use netabase_store::{NetabaseModel, netabase};

    #[derive(NetabaseModel, Clone, ...)]
    #[netabase(AppDefinition)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub name: String,
    }
}

// 2. Create store with any backend
let store = NetabaseStore::<AppDefinition, _>::sled("./app_db")?;

// 3. Open tree
let users = store.open_tree::<User>();

// 4. Perform operations
users.put(user)?;
let retrieved = users.get(UserPrimaryKey(1))?;

// 5. Backend features
store.flush()?; // Sled-specific
```

### Benefits

1. **Write Once, Run Anywhere**: Code works with all backends
2. **Easy Testing**: Use `temp()` for tests, switch to persistent for production
3. **Performance Access**: Backend-specific optimizations still available
4. **Type Safety**: Compile-time backend feature checking
5. **Future-Proof**: New backends work with existing code

---

## Transaction API: Type-State Pattern

**File:** `src/transaction.rs`

The Transaction API provides compile-time safe transaction management using the type-state pattern. It eliminates per-operation transaction overhead while maintaining type safety for read-only vs read-write access.

### Design Problem

The original API created a new transaction for every single operation:

```rust
// ❌ OLD: Each operation opens/closes a transaction
tree.put(user1)?;  // Transaction 1: open -> put -> commit
tree.put(user2)?;  // Transaction 2: open -> put -> commit
tree.put(user3)?;  // Transaction 3: open -> put -> commit
// 10-100x slower due to transaction overhead!
```

**Bottleneck identified**: Redb line 290, 324, 354, 381 all created new transactions per operation.

### Solution: Reusable Transactions with Type-State

```rust
// ✅ NEW: Single transaction for all operations
let mut txn = store.write()?;
let mut tree = txn.open_tree::<User>();
tree.put(user1)?;  // Uses shared transaction
tree.put(user2)?;  // Uses shared transaction
tree.put(user3)?;  // Uses shared transaction
txn.commit()?;     // Single commit
// 10-100x faster!
```

### Type-State Pattern

Uses phantom types to track transaction mode at compile time with zero runtime cost:

```rust
/// Zero-cost marker types (compile away completely)
pub struct ReadOnly;
pub struct ReadWrite;

/// Transaction guard parameterized by mode
pub struct TxnGuard<'db, D, Mode> {
    backend: TxnBackend<'db, D>,
    _mode: PhantomData<Mode>,  // Zero-cost type marker
}

/// Tree view inherits mode from transaction
pub struct TreeView<'txn, D, M, Mode> {
    backend: TreeBackend<'txn, D, M>,
    _mode: PhantomData<Mode>,  // Zero-cost type marker
}
```

### Mode-Based Method Availability

Methods are available based on the `Mode` type parameter:

```rust
// Operations on ALL modes
impl<'db, D, Mode> TxnGuard<'db, D, Mode> {
    pub fn open_tree<M>(&mut self) -> TreeView<'_, D, M, Mode> { }
}

// Operations ONLY on ReadWrite mode
impl<'db, D> TxnGuard<'db, D, ReadWrite> {
    pub fn commit(self) -> Result<(), NetabaseError> { }
    pub fn rollback(self) -> Result<(), NetabaseError> { }
}

// Read operations on ALL modes
impl<'txn, D, M, Mode> TreeView<'txn, D, M, Mode> {
    pub fn get(&self, key: M::PrimaryKey) -> Result<Option<M>, NetabaseError> { }
    pub fn len(&self) -> Result<usize, NetabaseError> { }
    pub fn iter(&self) -> Result<Vec<(M::PrimaryKey, M)>, NetabaseError> { }
}

// Write operations ONLY on ReadWrite mode
impl<'txn, D, M> TreeView<'txn, D, M, ReadWrite> {
    pub fn put(&mut self, model: M) -> Result<(), NetabaseError> { }
    pub fn remove(&mut self, key: M::PrimaryKey) -> Result<Option<M>, NetabaseError> { }
    pub fn clear(&mut self) -> Result<(), NetabaseError> { }
}
```

### Compile-Time Safety Example

```rust
let txn = store.read();  // Type: TxnGuard<ReadOnly>
let tree = txn.open_tree::<User>();  // Type: TreeView<ReadOnly>

// ✅ Read operations work
let user = tree.get(UserPrimaryKey(1))?;

// ❌ Write operations produce compile errors
tree.put(user)?;
// Error: no method named `put` found for struct `TreeView<'_, D, User, ReadOnly>`
```

### Backend Implementation

#### Redb: Transaction Reuse

Redb stores the transaction and reuses it for all operations:

```rust
pub(crate) struct RedbTxnBackend<'db, D> {
    read_txn: Option<redb::ReadTransaction>,
    write_txn: Option<redb::WriteTransaction>,
    db: &'db Arc<redb::Database>,
    _phantom: PhantomData<D>,
}

// Transaction created once
let mut txn = store.write()?;  // Creates WriteTransaction

// All operations reuse it
let mut tree = txn.open_tree::<User>();
tree.put(user1)?;  // Reuses WriteTransaction
tree.put(user2)?;  // Reuses WriteTransaction
tree.put(user3)?;  // Reuses WriteTransaction

txn.commit()?;  // Single commit
```

#### Sled: Direct Tree Operations

Sled doesn't have multi-tree transactions, so operations apply immediately:

```rust
pub(crate) struct SledTreeBackend<'txn, D, M> {
    tree: sled::Tree,          // Arc-based, cheap to clone
    secondary_tree: sled::Tree,
    _phantom: PhantomData<(&'txn (), D, M)>,
}

// Operations apply immediately to the tree
tree.put(user)?;  // Directly inserts into sled::Tree
```

### Usage Patterns

#### Read-Only Transactions (Multiple Concurrent)

```rust
let txn = store.read();
let user_tree = txn.open_tree::<User>();
let post_tree = txn.open_tree::<Post>();

let user = user_tree.get(UserPrimaryKey(1))?;
let posts = post_tree.get_by_secondary_key(
    PostSecondaryKeys::AuthorId(PostAuthorIdSecondaryKey(1))
)?;
// Auto-closes on drop
```

#### Read-Write Transactions (Exclusive)

```rust
let mut txn = store.write()?;
let mut tree = txn.open_tree::<User>();

// All operations in single transaction
for i in 0..1000 {
    tree.put(User { id: i, ... })?;
}

tree.commit()?;  // Atomic commit
// Or drop to rollback
```

#### Bulk Operation Helpers

```rust
let mut txn = store.write()?;
let mut tree = txn.open_tree::<User>();

tree.put_many(users)?;         // Batch insert
let results = tree.get_many(keys)?;  // Batch read
tree.remove_many(keys)?;       // Batch delete

txn.commit()?;
```

### Performance Benefits

| Operation | Old API (per-op txn) | New API (reused txn) | Speedup |
|-----------|---------------------|---------------------|---------|
| 1000 inserts (Redb) | ~250ms | ~5ms | **50x** |
| 1000 reads (Redb) | ~150ms | ~3ms | **50x** |
| Mixed ops (Redb) | ~200ms | ~4ms | **50x** |

The Transaction API provides:
- 🚀 **10-100x Performance**: Single transaction for N operations
- 🔒 **Type Safety**: Compile-time read-only vs read-write enforcement
- ⚡ **Zero Cost**: Phantom types compile away completely
- 🔄 **ACID**: Full atomicity for write transactions (Redb)
- 🎯 **Ergonomic**: Simple API, no manual transaction tracking

---

## Type System and Traits

### Core Trait Hierarchy

```
NetabaseModelTrait<D> (user-defined structs)
    ├── const DISCRIMINANT
    ├── type PrimaryKey
    ├── type SecondaryKeys
    ├── type Keys
    ├── fn primary_key() -> PrimaryKey
    ├── fn secondary_keys() -> Vec<SecondaryKeys>
    └── fn discriminant_name() -> &'static str

NetabaseDefinitionTrait (generated enum)
    ├── type Keys
    ├── fn to_key() -> Result<Keys>
    └── fn discriminant_name() -> &'static str

ToIVec + FromIVec (serialization)
    ├── fn to_ivec() -> Result<IVec>
    └── fn from_ivec(&IVec) -> Result<Self>

NetabaseTreeSync<D, M> (backend operations)
    ├── type PrimaryKey
    ├── type SecondaryKeys
    ├── fn put(model: M)
    ├── fn get(key: PrimaryKey)
    ├── fn remove(key: PrimaryKey)
    └── fn get_by_secondary_key(key: SecondaryKeys)

RecordStoreExt (libp2p integration)
    ├── fn to_record() -> Result<libp2p::kad::Record>
    └── fn from_record(&Record) -> Result<Self>
```

---

## Data Serialization Flow

### Encoding Path (Write)

```
User Model (struct User)
    │ .primary_key()
    ▼
Primary Key (UserPrimaryKey(1))
    │ bincode::encode
    ▼
Binary Key (Vec<u8>)
    │
    ├─→ (with bincode::encode(model))
    │
    ▼
Binary Value (Vec<u8>)
    │
    ▼
Backend Storage (Sled tree)
```

### Decoding Path (Read)

```
Backend Storage
    │ returns key_bytes, value_bytes
    ▼
Binary Value (Vec<u8>)
    │ bincode::decode
    ▼
User Model (struct User)
```

---

## Tree-Based Access Pattern

The central pattern for database access is through typed trees:

### Basic Usage

```rust
use netabase_store::databases::sled_store::SledStore;
use netabase_store::traits::tree::NetabaseTreeSync;
use netabase_store::traits::model::NetabaseModelTrait;

// Open database
let db = SledStore::<BlogDefinition>::new("./data")?;

// Open type-safe tree for User model
let user_tree = db.open_tree::<User>();

// Create a user
let user = User {
    id: 1,
    name: "Alice".to_string(),
    email: "alice@example.com".to_string(),
};

// Insert
user_tree.put(user.clone())?;

// Retrieve by primary key
let retrieved = user_tree.get(UserPrimaryKey(1))?;
assert_eq!(retrieved, Some(user));

// Query by secondary key
let users_by_email = user_tree.get_by_secondary_key(
    UserSecondaryKeys::Email(UserEmailSecondaryKey("alice@example.com".to_string()))
)?;

// Delete
user_tree.remove(UserPrimaryKey(1))?;
```

### Multiple Model Types

```rust
let db = SledStore::<BlogDefinition>::new("./data")?;

// Work with users
let user_tree = db.open_tree::<User>();
user_tree.put(user)?;

// Work with posts (completely separate tree)
let post_tree = db.open_tree::<Post>();
post_tree.put(post)?;

// Query posts by author
let posts = post_tree.get_by_secondary_key(
    PostSecondaryKeys::AuthorId(PostAuthorIdSecondaryKey(1))
)?;
```

---

## libp2p Integration

### RecordStoreExt Trait

**File:** `src/traits/record_store.rs`

The `RecordStoreExt` trait provides conversion between netabase types and libp2p Kademlia records:

```rust
pub trait RecordStoreExt: NetabaseDefinitionTrait {
    fn to_record(&self) -> Result<libp2p::kad::Record, NetabaseError>;
    fn from_record(record: &libp2p::kad::Record) -> Result<Self, NetabaseError>;
}
```

### Implementation

```rust
impl RecordStoreExt for BlogDefinition {
    fn to_record(&self) -> Result<libp2p::kad::Record, NetabaseError> {
        let key = self.to_key()?;
        let key_bytes = key.to_ivec()?.to_vec();
        let value_bytes = self.to_ivec()?.to_vec();

        Ok(libp2p::kad::Record {
            key: libp2p::kad::RecordKey::new(&key_bytes),
            value: value_bytes,
            publisher: None,
            expires: None,
        })
    }

    fn from_record(record: &libp2p::kad::Record) -> Result<Self, NetabaseError> {
        let ivec: IVec = record.value.clone().into();
        Self::from_ivec(&ivec)
    }
}
```

### libp2p RecordStore Implementation

**File:** `src/databases/record_store/sled_impl.rs`

SledStore also implements libp2p's `RecordStore` trait for direct Kademlia integration:

```rust
impl<D> libp2p::kad::store::RecordStore for SledStore<D>
where
    D: NetabaseDefinitionTrait + RecordStoreExt,
{
    type RecordsIter<'a> = SledRecordsIterator<'a>;
    type ProvidedIter<'a> = std::iter::Empty<Cow<'a, ProviderRecord>>;

    fn get(&self, key: &RecordKey) -> Option<Cow<'_, Record>> {
        let key_bytes = key.as_ref();

        self.db.get(key_bytes).ok()?.map(|value| {
            Cow::Owned(Record {
                key: key.clone(),
                value: value.to_vec(),
                publisher: None,
                expires: None,
            })
        })
    }

    fn put(&mut self, record: Record) -> libp2p::kad::store::Result<()> {
        let key_bytes = record.key.as_ref();
        let value_bytes = &record.value;

        self.db.insert(key_bytes, value_bytes)
            .map_err(|_| libp2p::kad::store::Error::MaxRecords)?;

        Ok(())
    }

    fn remove(&mut self, key: &RecordKey) {
        let _ = self.db.remove(key.as_ref());
    }

    fn records(&self) -> Self::RecordsIter<'_> {
        SledRecordsIterator {
            inner: self.db.iter(),
        }
    }

    fn provided(&self) -> Self::ProvidedIter<'_> {
        std::iter::empty()
    }
}
```

---

## Summary

Netabase Store provides:

1. **Type-Safe Storage:** Compile-time guarantees through macro-generated types
2. **Tree-Based API:** Clean separation of model types via `open_tree::<Model>()`
3. **Multi-Backend Support:** Sled, Redb, and IndexedDB with same API
4. **Secondary Key Indexing:** Automatic indexing and querying by secondary keys
5. **libp2p Integration:** Direct Kademlia DHT storage with type safety
6. **Deterministic Serialization:** Consistent binary format using bincode

The architecture enables developers to define data models once and get:
- Local database operations via typed trees
- Primary and secondary key queries
- Type-safe CRUD operations
- Automatic key type generation
- Backend flexibility
- libp2p Kademlia integration

All while maintaining Rust's safety guarantees and zero-cost abstractions.
