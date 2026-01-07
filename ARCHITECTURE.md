# Netabase Store Architecture

This document provides a comprehensive technical overview of the netabase_store crate's internal architecture, implementation details, and design decisions.

## Table of Contents

1. [Overview](#overview)
2. [Module Structure](#module-structure)
3. [Type System](#type-system)
4. [CRUD Operations & Table Interactions](#crud-operations--table-interactions)
5. [Transaction Architecture](#transaction-architecture)
6. [Feature Implementations](#feature-implementations)
   - [Primary Keys](#primary-keys)
   - [Secondary Keys](#secondary-keys)
   - [Relational Links](#relational-links)
   - [Blob Storage](#blob-storage)
   - [Subscriptions](#subscriptions)
   - [Migration System](#migration-system)
7. [Repository Pattern](#repository-pattern)
8. [Serialization](#serialization)
9. [Macro System](#macro-system)
10. [Performance Considerations](#performance-considerations)

---

## Overview

Netabase Store is a type-safe embedded database library built on [redb](https://github.com/cberner/redb). The core philosophy is:

- **Compile-time correctness**: Wrong types = compiler errors, not runtime panics
- **Zero-cost abstractions**: Macros generate optimal code with no runtime overhead
- **Explicit relationships**: Links are type-checked and repository-aware
- **Graceful evolution**: Schema versioning with automatic migration paths
- **Embedded-first**: No external database server required

### Key Design Principles

1. **Type Safety First**: Every ID is a unique type (`UserID`, `PostID`) preventing ID confusion
2. **Macro-Driven**: Procedural macros generate type-safe boilerplate at compile time
3. **Repository Isolation**: Compile-time enforcement of data access boundaries
4. **Migration-Ready**: Built-in versioning from day one with automatic data migration
5. **Zero Configuration**: Schemas defined in code, no external config files
6. **Transaction-Oriented**: All operations are transactional (ACID guarantees)

### Architecture Goals

- **Performance**: Minimize serialization overhead, use efficient indexing
- **Safety**: Impossible to mix incompatible model versions or definitions
- **Ergonomics**: Simple API that feels natural in Rust
- **Decentralization Ready**: Schema export/import for P2P synchronization

---

## Module Structure

```
netabase_store/
├── src/
│   ├── lib.rs                 # Crate root, re-exports, feature flags
│   ├── prelude.rs             # Common imports for users
│   ├── errors.rs              # Error types (NetabaseError enum)
│   ├── blob.rs                # Large data handling (NetabaseBlobItem trait)
│   ├── query.rs               # Query configuration and results
│   ├── relational.rs          # Relationship types (RelationalLink)
│   ├── doc_examples.rs        # Inline documentation examples
│   ├── utils/                 # Utility functions
│   │   └── mod.rs             # Serde helpers, hashing utilities
│   ├── databases/             # Backend implementations
│   │   ├── mod.rs             # Database abstraction layer
│   │   ├── redb/              # Redb backend (primary implementation)
│   │   │   ├── mod.rs         # RedbStore, schema management
│   │   │   ├── migration.rs   # Data migration execution
│   │   │   ├── repository.rs  # Repository-scoped stores
│   │   │   ├── libp2p.rs      # P2P schema export/import
│   │   │   └── transaction/   # Transaction layer
│   │   │       ├── mod.rs     # Transaction types and lifecycle
│   │   │       ├── crud.rs    # Create, Read, Update, Delete ops
│   │   │       ├── hydration.rs # RelationalLink hydration
│   │   │       ├── options.rs # Query configuration
│   │   │       ├── tables.rs  # Table opening and management
│   │   │       ├── wrappers.rs # ReadableTable/WritableTable
│   │   │       └── value_wrappers.rs # Value type adapters
│   │   └── indexedb/          # Browser backend (future)
│   └── traits/                # Core trait definitions
│       ├── mod.rs             # Trait re-exports
│       ├── database/          # Database abstractions
│       │   ├── mod.rs         # Core database traits
│       │   ├── hash.rs        # Content-addressable hashing
│       │   ├── store.rs       # NBStore trait (main store interface)
│       │   └── transaction/   # Transaction trait definitions
│       │       ├── mod.rs     # NBTransaction trait
│       │       ├── crud.rs    # CRUD operation traits
│       │       ├── hydrate.rs # Link hydration traits
│       │       └── iter.rs    # Iterator traits for models
│       ├── migration/         # Migration system traits
│       │   ├── mod.rs         # Migration trait re-exports
│       │   ├── traits.rs      # MigrateFrom/MigrateTo traits
│       │   ├── chain.rs       # Multi-step migration chains
│       │   └── context.rs     # VersionContext, VersionHeader
│       └── registery/         # Type registry system (core of type safety)
│           ├── mod.rs         # Registry trait re-exports
│           ├── models/        # Model-level traits and types
│           │   ├── mod.rs     # NetabaseModel trait
│           │   ├── model/     # Model trait implementation
│           │   │   ├── mod.rs # Model documentation and trait
│           │   │   └── schema.rs # ModelSchema (runtime metadata)
│           │   ├── keys/      # Key type traits
│           │   │   ├── mod.rs # Key trait definitions
│           │   │   ├── primary.rs # PrimaryKey trait
│           │   │   ├── secondary.rs # SecondaryKey trait
│           │   │   ├── relational.rs # RelationalKey trait
│           │   │   ├── blob.rs # BlobKey trait
│           │   │   └── subscription.rs # SubscriptionKey trait
│           │   └── treenames.rs # Table naming strategy
│           ├── definition/    # Definition-level traits
│           │   ├── mod.rs     # NetabaseDefinition trait
│           │   ├── schema.rs  # DefinitionSchema (metadata export)
│           │   ├── redb_definition.rs # Redb-specific definition
│           │   └── subscription/ # Pub/sub topic management
│           │       ├── mod.rs # Subscription traits
│           │       └── topic.rs # Topic enum generation
│           └── repository/    # Repository-level traits
│               ├── mod.rs     # NetabaseRepository trait
│               └── standalone.rs # Standalone (default repository)

netabase_macros/              # Procedural macro crate
├── src/
│   ├── lib.rs                # Macro entry points
│   ├── macros/               # Macro implementations
│   │   ├── netabase.rs       # #[netabase(...)] (multi-definition)
│   │   ├── netabase_repository.rs # #[netabase_repository(...)]
│   │   ├── netabase_definition.rs # #[netabase_definition(...)]
│   │   ├── netabase_model.rs # #[derive(NetabaseModel)]
│   │   └── netabase_blob_item.rs # #[derive(NetabaseBlobItem)]
│   ├── parsers/              # Attribute parsing
│   │   ├── repository.rs     # Parse repository attributes
│   │   ├── definition.rs     # Parse definition attributes
│   │   ├── model.rs          # Parse model field attributes
│   │   └── link.rs           # Parse #[link(...)] attributes
│   ├── visitors/             # AST visiting and transformation
│   │   ├── definition/       # Definition visitor
│   │   └── model/            # Model visitor
│   │       ├── collector.rs  # Collect field information
│   │       └── mutator.rs    # Transform field types
│   └── generators/           # Code generation
│       ├── structure.rs      # Generate struct definitions
│       ├── model/            # Model code generation
│       │   ├── keys.rs       # Generate key enums
│       │   ├── impl_model.rs # NetabaseModel impl
│       │   └── treenames.rs  # Table name constants
│       ├── definition/       # Definition code generation
│       │   ├── schema.rs     # DefinitionSchema impl
│       │   └── subscription.rs # Subscription enum
│       └── repository/       # Repository code generation
│           ├── discriminant.rs # Definition/Model enums
│           ├── marker.rs     # Marker trait impl
│           ├── store.rs      # Repository store struct
│           └── trait_impl.rs # NetabaseRepository impl
```

### Module Responsibilities

- **`src/traits/`**: Define the contract - what types must implement
- **`src/databases/redb/`**: Fulfill the contract - how redb implements it
- **`netabase_macros/`**: Generate the glue - boilerplate connecting user code to traits

---

## Type System

### Three-Layer Hierarchy

The type system enforces a strict hierarchy at compile time:

1. **Models** - Individual data structures (structs with `#[derive(NetabaseModel)]`)
2. **Definitions** - Collections of related models (modules with `#[netabase_definition(...)]`)
3. **Repositories** - Access boundaries across definitions (modules with `#[netabase_repository(...)]`)

```rust
// Layer 1: Models - Individual entities
#[derive(NetabaseModel, Serialize, Deserialize, Clone, ...)]
pub struct User {
    #[primary_key]
    pub id: String,      // Becomes UserID(String) newtype
    
    #[secondary_key]
    pub email: String,   // Creates UserSecondaryKeys::Email index
    
    pub name: String,    // Regular field, no special treatment
}

// Layer 2: Definitions - Logical grouping
#[netabase_definition(MyApp, subscriptions(UserEvents, PostEvents))]
mod my_app {
    pub struct User { ... }
    pub struct Post { ... }
    pub struct Comment { ... }
}

// Layer 3: Repositories - Access boundaries
#[netabase_repository(MainRepo, definitions(MyApp, OtherApp))]
mod main_repo {}

// Usage: Repository type parameter constrains what you can access
let store = RedbRepositoryStore::<MainRepo>::new("data.redb")?;
let txn = store.begin_write()?;
// Can ONLY access models from MyApp and OtherApp definitions
```

### Generated Types for Each Model

For a model named `User`, the macro generates:

#### 1. Primary Key Newtype
```rust
// Generated from: #[primary_key] pub id: String
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UserID(pub String);

impl From<String> for UserID { ... }
impl Display for UserID { ... }
impl PrimaryKey for UserID {
    type Inner = String;
    fn inner(&self) -> &Self::Inner { &self.0 }
}
```

#### 2. Keys Enum (Discriminated Union)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserKeys {
    Primary(UserID),                          // For primary key operations
    Secondary(UserSecondaryKeys),             // For secondary index queries
    Relational(UserRelationalKeys),           // For relationship lookups
    Blob(UserBlobKeys),                       // For blob chunk access
    Subscription(UserSubscriptionKeys),       // For pub/sub filtering
}
```

#### 3. Secondary Keys Enum
```rust
// One variant per #[secondary_key] field
pub enum UserSecondaryKeys {
    Email(String),    // From #[secondary_key] pub email: String
    Age(u8),          // From #[secondary_key] pub age: u8
}
```

#### 4. Relational Keys Enum
```rust
// One variant per #[link(...)] field
pub enum UserRelationalKeys {
    Partner(UserID),   // From #[link(MyApp, User)] pub partner: String
    Company(CompanyID), // From #[link(OtherDef, Company)] pub company: String
}
```

#### 5. Blob Keys Enum
```rust
// One variant per #[blob] field, includes chunk index
pub enum UserBlobKeys {
    Avatar { index: u8 },       // From #[blob] pub avatar: ProfilePic
    Document { index: u8 },     // From #[blob] pub document: LargeFile
}
```

#### 6. Table Name Constants
```rust
pub const USER_TABLE: &str = "MyApp::User";
pub const USER_EMAIL_INDEX: &str = "MyApp::User::Email";
pub const USER_AGE_INDEX: &str = "MyApp::User::Age";
pub const USER_PARTNER_LINK: &str = "MyApp::User::Partner";
pub const USER_BLOB_TABLE: &str = "MyApp::User::Blobs";
```

### Type Safety Guarantees

1. **No ID Confusion**: `UserID` ≠ `PostID` (compile error if mixed)
2. **Repository Constraints**: Can't access models outside repository's definitions
3. **Link Type Checking**: Links must reference models in the same repository
4. **Version Safety**: Can't mix incompatible model versions in same query

---

## CRUD Operations & Table Interactions

### Database Table Layout

For each model, up to 6 tables are created in redb:

```
Model: User (in definition MyApp)

Tables created:
┌─────────────────────────────────────────────────────────────────┐
│ 1. Primary Table: "MyApp::User"                                 │
│    Key: UserID → Value: Serialized User struct                  │
│    Purpose: Store the complete model data                       │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ 2. Secondary Index: "MyApp::User::Email"                        │
│    Key: String (email) → Value: UserID                          │
│    Purpose: Fast lookups by email                               │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ 3. Secondary Index: "MyApp::User::Age"                          │
│    Key: u8 (age) → Value: UserID                                │
│    Purpose: Fast lookups by age                                 │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ 4. Relational Link: "MyApp::User::Partner"                      │
│    Key: UserID (partner) → Value: UserID (owner)                │
│    Purpose: Reverse lookup - find users whose partner is X      │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ 5. Blob Table: "MyApp::User::Blobs"                             │
│    Key: (UserID, BlobKey) → Value: Vec<u8> (chunk data)         │
│    Purpose: Store large binary data separately                  │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ 6. Subscription Index: "MyApp::User::Subscriptions"             │
│    Key: SubscriptionTopic → Value: UserID                       │
│    Purpose: Find models subscribed to specific topics           │
└─────────────────────────────────────────────────────────────────┘
```

### CREATE Operation

**What happens when you call `txn.create(&user)`:**

```rust
let user = User {
    id: UserID("alice".into()),
    email: "alice@example.com".into(),
    age: 30,
    partner: RelationalLink::new_dehydrated(UserID("bob".into())),
    avatar: ProfilePic { data: vec![...] },
    subscriptions: vec![MyAppSubscriptions::UserEvents],
};

txn.create(&user)?;
```

**Step-by-step execution:**

1. **Serialize Main Model**
   ```rust
   let serialized = postcard::to_allocvec(&user)?;
   // User { id: "alice", email: "alice@...", age: 30, ... } → bytes
   ```

2. **Write to Primary Table**
   ```rust
   primary_table.insert(
       UserID("alice"),     // Key
       serialized_bytes     // Value
   )?;
   // Table "MyApp::User": "alice" → <serialized user>
   ```

3. **Update Secondary Indexes**
   ```rust
   // For each #[secondary_key] field:
   email_index.insert("alice@example.com", UserID("alice"))?;
   // Table "MyApp::User::Email": "alice@example.com" → "alice"
   
   age_index.insert(30, UserID("alice"))?;
   // Table "MyApp::User::Age": 30 → "alice"
   ```

4. **Update Relational Link Indexes**
   ```rust
   // For each #[link(...)] field:
   partner_link.insert(
       UserID("bob"),      // Target ID
       UserID("alice")     // Owner ID
   )?;
   // Table "MyApp::User::Partner": "bob" → "alice"
   // Meaning: "alice"'s partner is "bob"
   ```

5. **Store Blob Chunks**
   ```rust
   // For each #[blob] field:
   let chunks = avatar.split_into_blobs();  // Split into 60KB chunks
   for (chunk_index, chunk_data) in chunks.into_iter().enumerate() {
       blob_table.insert(
           (UserID("alice"), UserBlobKeys::Avatar { index: chunk_index as u8 }),
           chunk_data
       )?;
   }
   // Table "MyApp::User::Blobs": ("alice", Avatar{0}) → <chunk 0>
   //                              ("alice", Avatar{1}) → <chunk 1>
   ```

6. **Update Subscription Indexes**
   ```rust
   // For each topic in subscriptions field:
   for topic in user.subscriptions {
       subscription_index.insert(topic, UserID("alice"))?;
   }
   // Table "MyApp::User::Subscriptions": UserEvents → "alice"
   ```

**Result:** 1 CREATE operation → up to 6 table modifications

---

### READ Operation

**What happens when you call `txn.read(&user_id)`:**

```rust
let user: Option<User> = txn.read(&UserID("alice".into()))?;
```

**Step-by-step execution:**

1. **Open Primary Table**
   ```rust
   let table = txn.open_table::<UserID, Vec<u8>>(USER_TABLE)?;
   ```

2. **Lookup by Primary Key**
   ```rust
   let serialized = table.get(&UserID("alice"))?;
   // Retrieves: <serialized user bytes>
   ```

3. **Deserialize Main Model**
   ```rust
   let mut user: User = postcard::from_bytes(&serialized)?;
   // bytes → User { id, email, age, partner, avatar, subscriptions }
   ```

4. **Reconstruct Blob Fields**
   ```rust
   // For each #[blob] field:
   let blob_table = txn.open_blob_table()?;
   let mut chunks = Vec::new();
   let mut chunk_index = 0u8;
   
   loop {
       let key = (UserID("alice"), UserBlobKeys::Avatar { index: chunk_index });
       match blob_table.get(&key)? {
           Some(chunk) => chunks.push(chunk),
           None => break,  // No more chunks
       }
       chunk_index += 1;
   }
   
   user.avatar = ProfilePic::reconstruct_from_blobs(chunks);
   ```

5. **Return Deserialized Model**
   ```rust
   Ok(Some(user))
   ```

**Secondary Key Queries:**

```rust
let users = txn.query_by_secondary_key(&UserKeys::Secondary(
    UserSecondaryKeys::Email("alice@example.com".into())
))?;
```

1. Open secondary index table: `"MyApp::User::Email"`
2. Lookup email → get `Vec<UserID>`
3. For each ID, call `txn.read(&id)` to get full model
4. Return `QueryResult::Multiple(Vec<User>)`

---

### UPDATE Operation

**What happens when you call `txn.update(&user)`:**

```rust
let mut user: User = txn.read(&UserID("alice".into()))?.unwrap();
user.email = "newemail@example.com".into();  // Changed
user.age = 31;                                // Changed
txn.update(&user)?;
```

**Step-by-step execution:**

1. **Read Old Version** (for comparison)
   ```rust
   let old_user: User = txn.read(&user.id())?.expect("Must exist");
   ```

2. **Update Primary Table**
   ```rust
   let serialized = postcard::to_allocvec(&user)?;
   primary_table.insert(user.id(), serialized)?;
   // Overwrites old data
   ```

3. **Update Changed Secondary Indexes**
   ```rust
   // Remove old email entry
   if old_user.email != user.email {
       email_index.remove(&old_user.email)?;
       // Insert new email entry
       email_index.insert(&user.email, &user.id())?;
   }
   
   // Update age if changed
   if old_user.age != user.age {
       age_index.remove(&old_user.age)?;
       age_index.insert(&user.age, &user.id())?;
   }
   ```

4. **Update Changed Relational Links**
   ```rust
   if old_user.partner.id() != user.partner.id() {
       // Remove old link
       partner_link.remove(&old_user.partner.id())?;
       // Insert new link
       partner_link.insert(&user.partner.id(), &user.id())?;
   }
   ```

5. **Update Changed Blobs**
   ```rust
   if old_user.avatar != user.avatar {
       // Delete old chunks
       for chunk_index in 0..255 {
           let key = (user.id(), UserBlobKeys::Avatar { index: chunk_index });
           blob_table.remove(&key)?;
       }
       // Insert new chunks
       let chunks = user.avatar.split_into_blobs();
       for (index, chunk) in chunks.into_iter().enumerate() {
           blob_table.insert(
               (user.id(), UserBlobKeys::Avatar { index: index as u8 }),
               chunk
           )?;
       }
   }
   ```

**Optimization**: Only modified indexes/blobs are updated, not all of them.

---

### DELETE Operation

**What happens when you call `txn.delete::<User>(&user_id)`:**

```rust
txn.delete::<User>(&UserID("alice".into()))?;
```

**Step-by-step execution:**

1. **Read Model First** (to get indexed values)
   ```rust
   let user: User = txn.read(&UserID("alice"))?.expect("Must exist to delete");
   ```

2. **Delete from Primary Table**
   ```rust
   primary_table.remove(&UserID("alice"))?;
   // Table "MyApp::User": "alice" → (deleted)
   ```

3. **Delete from Secondary Indexes**
   ```rust
   email_index.remove(&user.email)?;
   // Table "MyApp::User::Email": "alice@example.com" → (deleted)
   
   age_index.remove(&user.age)?;
   // Table "MyApp::User::Age": 30 → (deleted)
   ```

4. **Delete from Relational Link Indexes**
   ```rust
   partner_link.remove(&user.partner.id())?;
   // Table "MyApp::User::Partner": "bob" → (deleted)
   ```

5. **Delete Blob Chunks**
   ```rust
   for chunk_index in 0..255 {
       let key = (UserID("alice"), UserBlobKeys::Avatar { index: chunk_index });
       blob_table.remove(&key).ok();  // Ignore if not found
   }
   // Table "MyApp::User::Blobs": all ("alice", *) → (deleted)
   ```

6. **Delete from Subscription Indexes**
   ```rust
   for topic in user.subscriptions {
       subscription_index.remove(&topic, &user.id())?;
   }
   // Table "MyApp::User::Subscriptions": UserEvents → (deleted)
   ```

**Result:** 1 DELETE operation → cleans up all related index/blob entries

---

## Transaction Architecture

### Transaction Lifecycle

```
┌──────────┐
│  Store   │
└────┬─────┘
     │ begin_read() or begin_write()
     ▼
┌──────────────┐
│ Transaction  │
└──────┬───────┘
       │ CRUD operations
       │ Queries
       │ Hydration
       ▼
┌──────────────┐
│commit() or   │
│rollback()    │
└──────────────┘
```

### Read vs Write Transactions

**Read Transactions** (`begin_read()`):
- **Concurrency**: Multiple concurrent readers allowed
- **Isolation**: Snapshot isolation - sees database state at transaction start
- **Blocking**: Never blocks writers or other readers
- **Lifetime**: Automatically rolled back on drop
- **Use Case**: All read-only operations, queries, hydration

**Write Transactions** (`begin_write()`):
- **Concurrency**: Exclusive - only one writer at a time
- **Isolation**: Full ACID guarantees
- **Blocking**: Blocks other writers (not readers)
- **Lifetime**: Must call `commit()` explicitly, rollback on drop if not committed
- **Use Case**: All create/update/delete operations

### Transaction Types

```rust
// Core transaction enum wraps redb types
pub enum RedbTransactionType<'txn> {
    Read(redb::ReadTransaction),
    Write(redb::WriteTransaction),
}

// User-facing transaction type
pub struct RedbTransaction<'db, D: NetabaseDefinition> {
    inner: RedbTransactionInner<'db, D>,
    _phantom: PhantomData<D>,  // Constrains what models can be accessed
}

impl<'db, D> RedbTransaction<'db, D> {
    pub fn create<M>(&self, model: &M) -> Result<()>
    where
        M: NetabaseModel<Definition = D>,  // Type-safe: M must be in D
    {
        // Implementation in src/databases/redb/transaction/crud.rs
    }
    
    pub fn read<M>(&self, id: &M::PrimaryKey) -> Result<Option<M>>
    where
        M: NetabaseModel<Definition = D>,
    {
        // Implementation in src/databases/redb/transaction/crud.rs
    }
    
    // ... update, delete, query methods
}
```

### ACID Guarantees

- **Atomicity**: All operations in a transaction succeed or all fail
- **Consistency**: Database moves from one valid state to another
- **Isolation**: Concurrent transactions don't see each other's uncommitted changes
- **Durability**: Committed changes survive crashes (via redb's durability)

### Transaction Best Practices

1. **Keep transactions short**: Lock contention on write transactions
2. **Batch operations**: Multiple creates in one transaction is faster
3. **Read then write**: Use separate read tx to check, then write tx to modify
4. **Handle errors**: Always check transaction results

---

## Feature Implementations

This section details how each feature is implemented, covering both the core runtime implementation and the macro-generated code.

---

### Primary Keys

Primary keys uniquely identify each model instance. Every model must have exactly one primary key.

#### Core Implementation

**Trait Definition** (`src/traits/registery/models/keys/primary.rs`):
```rust
pub trait PrimaryKey: 
    Serialize + DeserializeOwned + Clone + Debug + 
    PartialEq + Eq + Hash + PartialOrd + Ord + Display 
{
    type Inner: Serialize + DeserializeOwned + Clone;
    
    fn inner(&self) -> &Self::Inner;
    fn into_inner(self) -> Self::Inner;
}
```

**redb Integration** (`src/databases/redb/transaction/crud.rs`):
```rust
impl redb::Key for UserID {
    fn compare(data1: &[u8], data2: &[u8]) -> Ordering {
        // Postcard-deserialized comparison
        let id1: UserID = postcard::from_bytes(data1).unwrap();
        let id2: UserID = postcard::from_bytes(data2).unwrap();
        id1.cmp(&id2)
    }
}

impl redb::Value for UserID {
    type SelfType<'a> = UserID;
    type AsBytes<'a> = Vec<u8>;
    
    fn fixed_width() -> Option<usize> { None }  // Variable size
    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a> {
        postcard::from_bytes(data).unwrap()
    }
    fn as_bytes<'a, 'b>(&'a self) -> Self::AsBytes<'b> {
        postcard::to_allocvec(self).unwrap()
    }
}
```

#### Macro Implementation

**Input Code:**
```rust
#[derive(NetabaseModel, ...)]
pub struct User {
    #[primary_key]
    pub id: String,
    pub name: String,
}
```

**Generated Code** (`netabase_macros/src/generators/model/keys.rs`):

1. **Primary Key Newtype:**
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, 
         Serialize, Deserialize, Display, From, TryInto)]
pub struct UserID(pub String);

impl PrimaryKey for UserID {
    type Inner = String;
    fn inner(&self) -> &Self::Inner { &self.0 }
    fn into_inner(self) -> Self::Inner { self.0 }
}
```

2. **Model Implementation:**
```rust
impl NetabaseModel for User {
    type PrimaryKey = UserID;
    
    fn id(&self) -> &Self::PrimaryKey {
        &self.id  // Direct field access
    }
    
    // ... other trait methods
}
```

3. **Field Type Transformation:**
```rust
// Before macro expansion:
pub struct User {
    #[primary_key]
    pub id: String,  // ← Original type
    pub name: String,
}

// After macro expansion:
pub struct User {
    pub id: UserID,  // ← Wrapped in newtype
    pub name: String,
}
```

**Table Usage:**
- Primary table: `"MyApp::User"` with key type `UserID`
- Used for: Direct lookups, ensuring uniqueness

---

### Secondary Keys

Secondary keys create additional indexes for fast non-primary key lookups.

#### Core Implementation

**Trait Definition** (`src/traits/registery/models/keys/secondary.rs`):
```rust
pub trait SecondaryKey: Serialize + DeserializeOwned + Clone + Debug {
    // Marker trait - implementation generated by macro
}
```

**Index Table Structure:**
```
Table: "MyApp::User::Email"
Key: String (email value) → Value: Vec<UserID> (list of users with this email)

Example entries:
"alice@example.com" → [UserID("user1")]
"bob@example.com"   → [UserID("user2"), UserID("user3")]  // Multiple users possible
```

**Query Implementation** (`src/databases/redb/transaction/crud.rs`):
```rust
pub fn query_by_secondary_key<M, K>(
    &self,
    key: &M::Keys
) -> Result<QueryResult<M>>
where
    M: NetabaseModel,
    K: SecondaryKey,
{
    match key {
        M::Keys::Secondary(sec_key) => {
            // Open the secondary index table
            let table_name = sec_key.table_name();  // e.g., "MyApp::User::Email"
            let table = self.open_table::<K, Vec<M::PrimaryKey>>(table_name)?;
            
            // Lookup in index
            let ids = table.get(&sec_key.value())?.unwrap_or_default();
            
            // Fetch all matching models
            let mut results = Vec::new();
            for id in ids {
                if let Some(model) = self.read::<M>(&id)? {
                    results.push(model);
                }
            }
            
            Ok(QueryResult::Multiple(results))
        }
        _ => Err(NetabaseError::InvalidKeyType),
    }
}
```

#### Macro Implementation

**Input Code:**
```rust
#[derive(NetabaseModel, ...)]
pub struct User {
    #[primary_key]
    pub id: String,
    
    #[secondary_key]
    pub email: String,
    
    #[secondary_key]
    pub age: u8,
    
    pub name: String,
}
```

**Generated Code:**

1. **Secondary Keys Enum:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserSecondaryKeys {
    Email(String),
    Age(u8),
}

impl UserSecondaryKeys {
    pub fn table_name(&self) -> &'static str {
        match self {
            UserSecondaryKeys::Email(_) => "MyApp::User::Email",
            UserSecondaryKeys::Age(_) => "MyApp::User::Age",
        }
    }
    
    pub fn value_bytes(&self) -> Vec<u8> {
        match self {
            UserSecondaryKeys::Email(v) => postcard::to_allocvec(v).unwrap(),
            UserSecondaryKeys::Age(v) => postcard::to_allocvec(v).unwrap(),
        }
    }
}
```

2. **Table Name Constants:**
```rust
pub const USER_EMAIL_INDEX: &str = "MyApp::User::Email";
pub const USER_AGE_INDEX: &str = "MyApp::User::Age";
```

3. **NetabaseModel Implementation:**
```rust
impl NetabaseModel for User {
    fn secondary_keys(&self) -> Vec<UserSecondaryKeys> {
        vec![
            UserSecondaryKeys::Email(self.email.clone()),
            UserSecondaryKeys::Age(self.age),
        ]
    }
}
```

**CRUD Impact:**
- **CREATE**: Inserts entry in each secondary index table
- **READ**: Can query by secondary key to find matching IDs, then read full models
- **UPDATE**: If secondary key value changes, removes old index entry and adds new one
- **DELETE**: Removes all secondary index entries for the model

---

### Relational Links

Relational links create type-safe relationships between models with compile-time validation.

#### Core Implementation

**RelationalLink Type** (`src/relational.rs`):
```rust
pub enum RelationalLink<'a, Repo, OwnerDef, TargetDef, Target>
where
    Repo: NetabaseRepository,
    OwnerDef: InRepository<Repo>,
    TargetDef: InRepository<Repo>,
    Target: NetabaseModel<Definition = TargetDef>,
{
    /// Link with just the ID (most common in database)
    Dehydrated(Target::PrimaryKey),
    
    /// Link with borrowed reference (during queries)
    Hydrated(&'a Target),
    
    /// Link with borrowed mutable reference (rare)
    HydratedMut(&'a mut Target),
    
    /// Link with owned data (after hydration)
    Owned(Box<Target>),
    
    /// Link with full ownership, non-boxed
    Borrowed(&'a Target),
}

impl<'a, Repo, OwnerDef, TargetDef, Target> RelationalLink<'a, Repo, OwnerDef, TargetDef, Target> {
    pub fn new_dehydrated(id: Target::PrimaryKey) -> Self {
        RelationalLink::Dehydrated(id)
    }
    
    pub fn id(&self) -> &Target::PrimaryKey {
        match self {
            RelationalLink::Dehydrated(id) => id,
            RelationalLink::Hydrated(model) => model.id(),
            RelationalLink::Owned(model) => model.id(),
            RelationalLink::Borrowed(model) => model.id(),
            RelationalLink::HydratedMut(model) => model.id(),
        }
    }
    
    pub fn hydrate<'txn, D>(
        &mut self,
        txn: &'txn RedbTransaction<D>,
    ) -> Result<()>
    where
        D: NetabaseDefinition,
    {
        if let RelationalLink::Dehydrated(id) = self {
            let model = txn.read::<Target>(id)?.ok_or(NetabaseError::NotFound)?;
            *self = RelationalLink::Owned(Box::new(model));
        }
        Ok(())
    }
}
```

**Serialization** (`src/relational.rs`):
```rust
impl<...> Serialize for RelationalLink<...> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Always serialize as just the ID
        self.id().serialize(serializer)
    }
}

impl<...> Deserialize for RelationalLink<...> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Always deserialize as Dehydrated
        let id = Target::PrimaryKey::deserialize(deserializer)?;
        Ok(RelationalLink::Dehydrated(id))
    }
}
```

**Link Index Table Structure:**
```
Table: "MyApp::Post::Author"
Key: UserID (target) → Value: Vec<PostID> (owners that link to this target)

Example entries:
UserID("alice") → [PostID("post1"), PostID("post2"), PostID("post3")]
UserID("bob")   → [PostID("post4")]

Purpose: Reverse lookup - find all posts where author is alice
```

#### Macro Implementation

**Input Code:**
```rust
#[netabase_definition(MyApp)]
mod my_app {
    #[derive(NetabaseModel, ...)]
    pub struct User {
        #[primary_key]
        pub id: String,
        pub name: String,
    }
    
    #[derive(NetabaseModel, ...)]
    pub struct Post {
        #[primary_key]
        pub id: String,
        pub title: String,
        
        #[link(MyApp, User)]
        pub author: String,  // ← Will be transformed
    }
}
```

**Generated Code:**

1. **Field Type Transformation:**
```rust
// Before:
pub struct Post {
    #[link(MyApp, User)]
    pub author: String,  // Input type
}

// After:
pub struct Post {
    pub author: RelationalLink<
        'static,
        Standalone,  // Default repository
        MyApp,       // Owner definition
        MyApp,       // Target definition
        User         // Target model
    >,
}
```

2. **Relational Keys Enum:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PostRelationalKeys {
    Author(UserID),  // Target type's primary key
}

impl PostRelationalKeys {
    pub fn table_name(&self) -> &'static str {
        match self {
            PostRelationalKeys::Author(_) => "MyApp::Post::Author",
        }
    }
}
```

3. **Table Name Constants:**
```rust
pub const POST_AUTHOR_LINK: &str = "MyApp::Post::Author";
```

4. **NetabaseModel Implementation:**
```rust
impl NetabaseModel for Post {
    fn relational_keys(&self) -> Vec<PostRelationalKeys> {
        vec![
            PostRelationalKeys::Author(self.author.id().clone()),
        ]
    }
}
```

**Compile-Time Validation:**
```rust
// ✅ Valid: Both User and Post in same repository
#[link(MyApp, User)]
pub author: String,

// ❌ Compile Error: OtherDef not in same repository
#[link(OtherDef, Company)]  // Error: OtherDef not InRepository<CurrentRepo>
pub company: String,
```

**CRUD Impact:**
- **CREATE**: Adds entry to relational link index (target ID → owner ID)
- **READ**: Link is dehydrated (just ID). Call `hydrate()` to load target model
- **UPDATE**: If link target changes, removes old index entry, adds new one
- **DELETE**: Removes link index entry

**Hydration Example:**
```rust
let post: Post = txn.read(&PostID("post1".into()))?.unwrap();
// post.author is Dehydrated(UserID("alice"))

let mut post = post;
post.author.hydrate(&txn)?;  // Loads User model from database
// post.author is now Owned(Box<User { id: "alice", ... }>)

// Access the hydrated model
if let RelationalLink::Owned(user) = &post.author {
    println!("Author name: {}", user.name);
}
```

---

## Blob Storage

### Why Blobs?

Large data (>60KB) degrades database performance. Blobs solve this by:

1. **Chunking**: Split into 60KB pieces
2. **Separate Tables**: Store chunks in dedicated blob tables
3. **Reconstruction**: Reassemble on read

### Implementation

```rust
#[derive(NetabaseBlobItem, Serialize, Deserialize)]
pub struct LargeFile {
    pub data: Vec<u8>,
    pub metadata: String,
}

#[derive(NetabaseModel, ...)]
pub struct Document {
    #[primary_key]
    pub id: String,
    
    #[blob]
    pub file: LargeFile,  // Automatically chunked
}
```

### Chunking Strategy

- **Chunk Size**: 60KB (60,000 bytes)
- **Serialization**: Full struct serialized, then chunked
- **Storage**: Each chunk stored with index: `(id, chunk_index) → bytes` TODO: it should actually be (OwnerID, BlobKey) -> (Bytes, index)
- **Retrieval**: All chunks fetched and concatenated
- **Reconstruction**: Deserialize reassembled bytes

---

## Repository Pattern

### Purpose

Repositories enforce **data graph completeness** at compile time.

### Problem

Without repositories:
```rust
// ❌ This compiles but might break at runtime
let user_link: RelationalLink<UserDef, User>;
let post = Post { author: user_link };  // Post in different definition!
```

### Solution

Repositories group definitions:
```rust
#[netabase_repository(MyRepo, definitions(UserDef, PostDef))]
mod my_repo {}

// ✅ Now links are type-checked
let user_link: RelationalLink<MyRepo, UserDef, UserDef, User>;
let post = Post { author: user_link };  // Verified at compile time
```

### Repository Isolation

Each repository gets its own store type:
```rust
let store: RedbRepositoryStore<MyRepo> = ...;
let txn = store.begin_write()?;
// Can only access models in MyRepo's definitions
```

---

## Serialization

### Why Postcard?

1. **No Schema**: Schema is in Rust types, not wire format
2. **Compact**: ~2-10x smaller than JSON
3. **Fast**: Zero-copy where possible
4. **Stable**: Variant encoding handles version skew
5. Bincode is no longer maintained.

### Serialization Path

```
Model → serde::Serialize → postcard::to_vec → Vec<u8> → redb
redb → &[u8] → postcard::from_bytes → serde::Deserialize → Model
```

### Special Cases

**RelationalLink**: Always serializes as dehydrated (key only)
```rust
// In memory: Owned(Box<User>), Hydrated(&User), or Borrowed(&User)
// On wire: Dehydrated(UserID)
```

**Blobs**: Serialized before chunking
```rust
LargeFile { data: [0u8; 200_000], ... }
→ postcard::to_vec() → Vec<u8>
→ chunk into 60KB pieces
→ store as [(0, bytes[0..60000]), (1, bytes[60000..120000]), ...]
```

---

## Performance Considerations

### Indexing Strategy

- **Primary Key**: B-tree index (always)
- **Secondary Keys**: Additional B-tree indexes
- **Relational Links**: Indexed by target ID for reverse lookups
- **Blobs**: Indexed by (model_id, chunk_index)

### Transaction Batching

Batch operations in a single transaction:
```rust
let txn = store.begin_write()?;
for item in items {
    txn.create(&item)?;  // Amortize transaction overhead
}
txn.commit()?;
```

### Read Optimization

Use secondary key queries when possible:
```rust
// ❌ Slow: Full scan
for user in all_users {
    if user.email == target { ... }
}

// ✅ Fast: Index lookup
let users = txn.query_by_secondary_key(&UserKeys::Secondary(
    UserSecondaryKeys::Email(target)
))?;
```

---


## Repository Pattern

Repositories enforce **data graph completeness** at compile time by grouping related definitions together.

### Purpose

Without repositories, links between models could reference data that doesn't exist in the current database scope, leading to runtime errors. Repositories solve this by:

1. **Compile-Time Validation**: Only models in the same repository can link to each other
2. **Data Locality**: All linked data is guaranteed to be in the same database
3. **Access Control**: Repositories act as isolation boundaries
4. **Type Safety**: `RelationalLink` type parameters enforce repository membership

### Problem: Cross-Definition Links Without Repositories

```rust
// Without repositories - compiles but unsafe!
#[netabase_definition(UserDef)]
mod user_def {
    pub struct User { ... }
}

#[netabase_definition(PostDef)]
mod post_def {
    pub struct Post {
        #[link(UserDef, User)]  // ❌ Links to external definition
        pub author: String,      //    What if UserDef isn't loaded?
    }
}
```

### Solution: Repository Grouping

```rust
// Define which definitions belong together
#[netabase_repository(BlogRepo, definitions(UserDef, PostDef))]
mod blog_repo {}

// Now RelationalLink enforces the repository constraint
pub struct Post {
    pub author: RelationalLink<
        BlogRepo,   // ← Repository type parameter
        PostDef,    // ← Owner definition
        UserDef,    // ← Target definition (must be in BlogRepo)
        User        // ← Target model
    >,
}

// ✅ Valid: Both UserDef and PostDef are in BlogRepo
// ❌ Invalid: If UserDef not in BlogRepo, compile error
```

### Standalone Repository

Every definition automatically gets a `Standalone` repository if no repository is specified:

```rust
// Equivalent to:
#[netabase_repository(Standalone, definitions(MyApp))]
#[netabase_definition(MyApp)]
mod my_app {
    pub struct User { ... }
    pub struct Post {
        #[link(MyApp, User)]  // Links within same definition
        pub author: String,
    }
}
```

---

## Serialization

### Why Postcard?

Netabase Store uses [postcard](https://github.com/jamesmunns/postcard) for all serialization:

1. **No Schema Files**: Schema is defined by Rust types, not external files
2. **Compact**: 2-10x smaller than JSON
3. **Fast**: Zero-copy deserialization where possible
4. **Stable**: Variant-based encoding handles version skew gracefully
5. **Maintained**: Active development, unlike bincode
6. **Deterministic**: Same data always serializes to same bytes

### Serialization Path

```
Model (Rust) → serde::Serialize → postcard::to_allocvec() → Vec<u8> → redb
redb → &[u8] → postcard::from_bytes → serde::Deserialize → Model
```

### Special Cases

**RelationalLink - Always Dehydrated:**
```rust
// Always serializes as just the ID
impl Serialize for RelationalLink {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.id().serialize(serializer)
    }
}
```

**Blobs - Pre-Serialized Then Chunked:**
```rust
// Step 1: Serialize entire struct
let serialized = postcard::to_allocvec(&blob)?;
// Step 2: Chunk the bytes
let chunks = serialized.chunks(60000);
// Step 3: Store each chunk
```

---

## Performance Considerations

### Transaction Batching

```rust
// ❌ Slow: 1000 transactions
for item in items {
    let txn = store.begin_write()?;
    txn.create(&item)?;
    txn.commit()?;
}

// ✅ Fast: 1 transaction
let txn = store.begin_write()?;
for item in items {
    txn.create(&item)?;
}
txn.commit()?;
```

### Index Strategy

- Only index fields that are frequently queried
- Each index doubles write time for that field
- Read queries on indexed fields are O(log n) instead of O(n)

---

## Testing Strategy

### Test Types

- **Unit Tests**: Per-module `mod tests { }` blocks
- **Integration Tests**: `tests/` directory for end-to-end scenarios
- **README Validation**: `tests/readme_accuracy.rs` ensures docs are correct
- **Benchmarks**: `boilerplate/benches/` for performance metrics

---

## See Also

- [User Guide](./boilerplate/GUIDE.md)
- [Examples](./boilerplate/README.md)  
- [README](./README.md)

---

**Architecture Version**: 0.1.0
