# Netabase Store Architecture

This document provides a comprehensive technical overview of the netabase_store architecture, explaining how the macro system generates type-safe database code, how storage backends are implemented, and how data flows through the system.

## Table of Contents

1. [Overview](#overview)
2. [Macro System Deep Dive](#macro-system-deep-dive)
3. [Backend Implementation](#backend-implementation)
4. [Type System and Traits](#type-system-and-traits)
5. [Data Serialization Flow](#data-serialization-flow)
6. [Tree-Based Access Pattern](#tree-based-access-pattern)
7. [libp2p Integration](#libp2p-integration)

---

## Overview

Netabase Store is a type-safe, macro-driven database abstraction layer that supports multiple storage backends (Sled, Redb, IndexedDB) and integrates seamlessly with libp2p's Kademlia DHT for distributed storage.

### Key Design Principles

1. **Type Safety:** Compile-time guarantees for data models and queries
2. **Zero-Cost Abstractions:** Macros generate optimal code with no runtime overhead
3. **Backend Agnostic:** Same API works across Sled, Redb, and IndexedDB
4. **libp2p Compatible:** Direct integration with Kademlia RecordStore trait
5. **Deterministic Serialization:** Consistent binary format using bincode

### Architecture Layers

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
│         NetabaseTreeSync<D, M> Trait               │
│  • put(model) / get(key) / remove(key)             │
│  • get_by_secondary_key(secondary_key)             │
└────────────────────┬────────────────────────────────┘
                     │ Implemented by
        ┌────────────┴────────────┬──────────────┐
        ▼                         ▼              ▼
┌──────────────┐         ┌──────────────┐  ┌────────────┐
│  SledStore   │         │  RedbStore   │  │ IndexedDB  │
│  (Native)    │         │  (Native)    │  │   (WASM)   │
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
