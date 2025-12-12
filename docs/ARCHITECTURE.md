# Netabase Store Architecture

**Last Updated**: 2025-12-11
**Version**: 0.5.0

## Table of Contents

1. [Overview](#overview)
2. [Fundamental Architecture](#fundamental-architecture)
3. [Trees and Tree Abstractions](#trees-and-tree-abstractions)
4. [Transaction API](#transaction-api)
5. [Data Flow and Process Flow](#data-flow-and-process-flow)
6. [Backend Abstraction Layer](#backend-abstraction-layer)
7. [Multi-Definition Management](#multi-definition-management)
8. [Type System and Trait Architecture](#type-system-and-trait-architecture)
9. [Code Generation System](#code-generation-system)
10. [Performance Characteristics](#performance-characteristics)

---

## Overview

Netabase Store is a **type-safe, embedded database abstraction layer** for Rust that provides a strongly-typed interface for storing and querying structured data. It abstracts over multiple backend storage engines (currently redb and sled) while providing:

- **Compile-time type safety** for all database operations
- **Automatic index management** (secondary indices, relational keys)
- **Multi-definition stores** with lazy loading and cross-definition transactions
- **Subscription trees** for event streaming and data synchronization
- **Zero-copy reads** where possible
- **Macro-based code generation** to eliminate boilerplate

### Key Design Principles

1. **Type Safety First**: All operations are type-checked at compile time
2. **Backend Agnostic**: Clean separation between API and storage implementation
3. **Zero Overhead Abstractions**: No runtime cost for type safety
4. **Declarative Schema**: Define models, get everything else generated
5. **Lazy Evaluation**: Load data and databases only when needed

---

## Fundamental Architecture

### The Three-Layer Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    APPLICATION LAYER                             │
│  - User code                                                     │
│  - Type-safe API calls                                           │
│  - Model definitions                                             │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                    ABSTRACTION LAYER                             │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   Models     │  │ Definitions  │  │   Managers   │          │
│  │   (Traits)   │  │   (Enums)    │  │  (Multi-DB)  │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              RedbStore<D> / SledStore<D>                 │  │
│  │  - Transactions (Read/Write)                             │  │
│  │  - Tree Management                                        │  │
│  │  - Type routing                                           │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                     BACKEND LAYER                                │
│                                                                  │
│  ┌──────────────────┐           ┌──────────────────┐           │
│  │  RedbBackend     │           │  SledBackend     │           │
│  │  - redb crate    │           │  - sled crate    │           │
│  │  - Zero-copy     │           │  - Bincode       │           │
│  │  - Typed tables  │           │  - Byte-based    │           │
│  └──────────────────┘           └──────────────────┘           │
│                                                                  │
│              Common Backend Trait Interface                      │
│  - BackendStore, BackendTransaction, BackendTable              │
└─────────────────────────────────────────────────────────────────┘
```

### Core Concepts

#### 1. **Models**
User-defined structs that represent data entities. Each model:
- Has a primary key
- Can have secondary indices (for fast lookups)
- Can have relational keys (foreign key relationships)
- Can subscribe to event topics
- Computes a content hash for synchronization

```rust
#[derive(NetabaseModel, Clone, Debug)]
pub struct User {
    #[primary_key]
    pub id: u64,

    #[secondary_key]
    pub email: String,

    pub name: String,
    pub age: u32,
}
```

#### 2. **Definitions**
Enums that wrap all model types in a namespace. A Definition:
- Routes operations to the correct model type
- Provides discriminants for type identification
- Manages tree names and table structures
- Enables multi-model operations

```rust
#[derive(EnumDiscriminants)]
pub enum MyDefinitions {
    User(User),
    Product(Product),
    Order(Order),
}
```

#### 3. **Stores**
Typed database instances that provide CRUD operations:
- `RedbStore<Definition>` - High-performance, typed backend
- `SledStore<Definition>` - Lightweight, bincode-based backend

#### 4. **Managers**
Coordinate multiple definitions with lazy loading:
- `DefinitionManager<R, D, P, B>` - Multi-definition coordinator
- Loads definition stores on-demand
- Manages cross-definition transactions
- Enforces compile-time permissions

---

## Trees and Tree Abstractions

### What is a Tree?

In Netabase, a **tree** is an ordered key-value store (similar to a B-tree). Each tree stores a specific type of data for a model:

1. **Main Tree**: Primary key → Full model data
2. **Secondary Index Trees**: Secondary key → Primary key
3. **Relational Trees**: Foreign key → List of primary keys
4. **Subscription Trees**: Primary key → Content hash
5. **Hash Tree** (future): Content hash → List of primary keys

### Tree Naming Convention

All trees follow the standardized naming pattern:

```
{DefinitionName}::{ModelName}::{TreeType}::{TreeName}
```

**Examples**:
```
User::User::Main                          # Primary key tree
User::User::Secondary::Email              # Email secondary index
User::User::Secondary::Username           # Username secondary index
Product::Product::Relational::CreatedBy   # Foreign key to User
User::User::Subscription::Updates         # Updates event stream
User::User::Hash                          # Content hash lookup
```

### Why This Naming?

1. **Namespace Isolation**: `User::` prevents collisions with `UserProfile::`
2. **Predictable**: Given definition + model + type, tree name is deterministic
3. **Cross-Definition Lookup**: Easy to locate trees across definitions
4. **Multi-Model Support**: Each model namespaced within definition

### Tree Management Trait

The `TreeManager<D>` trait provides tree name resolution:

```rust
pub trait TreeManager<D>
where
    D: IntoDiscriminant,
    D::Discriminant: IntoEnumIterator + Hash + Eq + Debug + DiscriminantName + Clone,
{
    /// Returns all trees for this definition
    fn all_trees() -> AllTrees<D>;

    /// Get main tree name for a model
    fn get_tree_name(model_discriminant: &D::Discriminant) -> Option<TreeName>;

    /// Get all secondary index tree names
    fn get_secondary_tree_names(model_discriminant: &D::Discriminant) -> Vec<TreeName>;

    /// Get all relational tree names
    fn get_relational_tree_names(model_discriminant: &D::Discriminant) -> Vec<TreeName>;

    /// Get all subscription tree names
    fn get_subscription_tree_names(model_discriminant: &D::Discriminant) -> Vec<TreeName>;
}
```

### Tree Structure Examples

#### Main Tree (Primary Storage)
```
Tree: User::User::Main
Type: PrimaryKey -> Model

Structure:
UserId(1) -> User { id: 1, email: "alice@example.com", name: "Alice" }
UserId(2) -> User { id: 2, email: "bob@example.com", name: "Bob" }
UserId(3) -> User { id: 3, email: "carol@example.com", name: "Carol" }
```

#### Secondary Index Tree
```
Tree: User::User::Secondary::Email
Type: SecondaryKey -> PrimaryKey

Structure:
UserEmail("alice@example.com") -> UserId(1)
UserEmail("bob@example.com")   -> UserId(2)
UserEmail("carol@example.com") -> UserId(3)

Note: For unique indices, one secondary key maps to one primary key.
      For non-unique indices, implementation may vary by backend.
```

#### Relational Tree (Foreign Keys)
```
Tree: Product::Product::Relational::CreatedBy
Type: ForeignKey -> PrimaryKey

Structure:
UserId(1) -> ProductId(101)  # Alice created product 101
UserId(1) -> ProductId(102)  # Alice created product 102
UserId(2) -> ProductId(103)  # Bob created product 103

Note: Multiple entries for same foreign key enable one-to-many relationships.
```

#### Subscription Tree (Event Streams)
```
Tree: User::User::Subscription::Updates
Type: PrimaryKey -> ContentHash

Structure:
UserId(1) -> [blake3_hash_of_user_1]
UserId(2) -> [blake3_hash_of_user_2]
UserId(3) -> [blake3_hash_of_user_3]

Purpose: Enable efficient sync by XORing all hashes for order-independent accumulator.
```

### Tree Lifecycle

1. **Creation**: Trees are created automatically on first write
2. **Access**: Trees are opened per-transaction
3. **Indexing**: Secondary/relational trees updated atomically with main tree
4. **Deletion**: Trees can be manually dropped (not automatic)

---

## Transaction API

### Transaction Types

Netabase provides two transaction types:

1. **Read Transactions** (`ReadTransaction<D>`)
   - Provide consistent, isolated read access
   - Multiple concurrent read transactions allowed
   - Lightweight, can be held for extended periods
   - Never block writes (MVCC in redb, optimistic in sled)

2. **Write Transactions** (`WriteTransaction<D>`)
   - Provide exclusive write access
   - Include read capabilities
   - Atomic: all changes commit or none do
   - Automatically maintain indices

### Transaction Trait API

#### ReadTransaction Trait

```rust
pub trait ReadTransaction<D: NetabaseDefinition> {
    /// Get a model by primary key
    fn get<M: NetabaseModelTrait<D>>(
        &self,
        key: M::PrimaryKey,
    ) -> NetabaseResult<Option<M>>;

    /// Get primary key by secondary key
    fn get_pk_by_secondary_key<M: NetabaseModelTrait<D>>(
        &self,
        secondary_key: <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum,
    ) -> NetabaseResult<Option<M::PrimaryKey>>;

    /// Get model by secondary key (convenience method)
    fn get_by_secondary_key<M: NetabaseModelTrait<D>>(
        &self,
        secondary_key: <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum,
    ) -> NetabaseResult<Option<M>>;

    /// Get subscription accumulator (XORed hash + count)
    fn get_subscription_accumulator<M: NetabaseModelTrait<D>>(
        &self,
        subscription_discriminant: <<M as NetabaseModelTrait<D>>::SubscriptionEnum as IntoDiscriminant>::Discriminant,
    ) -> NetabaseResult<([u8; 32], u64)>;

    /// Get all primary keys in a subscription
    fn get_subscription_keys<M: NetabaseModelTrait<D>>(
        &self,
        subscription_discriminant: <<M as NetabaseModelTrait<D>>::SubscriptionEnum as IntoDiscriminant>::Discriminant,
    ) -> NetabaseResult<Vec<M::PrimaryKey>>;
}
```

#### WriteTransaction Trait

```rust
pub trait WriteTransaction<D: NetabaseDefinition>: ReadTransaction<D> {
    /// Insert or update a model
    fn put<M: NetabaseModelTrait<D>>(&mut self, model: M) -> NetabaseResult<()>;

    /// Delete a model by primary key
    fn delete<M: NetabaseModelTrait<D>>(
        &mut self,
        key: M::PrimaryKey,
    ) -> NetabaseResult<()>;

    /// Commit the transaction
    fn commit(self) -> NetabaseResult<()>;
}
```

### Store-Level Transaction API

The `StoreTrait<D>` provides convenience methods:

```rust
pub trait StoreTrait<D: NetabaseDefinition> {
    type ReadTxn<'a>: ReadTransaction<D> where Self: 'a;
    type WriteTxn: WriteTransaction<D>;

    /// Execute a closure in a read transaction
    fn read<F, R>(&self, f: F) -> NetabaseResult<R>
    where
        F: FnOnce(&Self::ReadTxn<'_>) -> NetabaseResult<R>;

    /// Execute a closure in a write transaction (auto-commits)
    fn write<F, R>(&self, f: F) -> NetabaseResult<R>
    where
        F: FnOnce(&mut Self::WriteTxn) -> NetabaseResult<R>;
}
```

### Example Usage

#### Simple Read

```rust
let user = store.read(|txn| {
    txn.get::<User>(UserId(1))
})?;
```

#### Secondary Key Lookup

```rust
let user = store.read(|txn| {
    txn.get_by_secondary_key::<User>(
        UserSecondaryKeys::Email(UserEmail("alice@example.com".into()))
    )
})?;
```

#### Write with Automatic Index Updates

```rust
store.write(|txn| {
    let user = User {
        id: 1,
        email: "alice@example.com".into(),
        name: "Alice".into(),
        age: 30,
    };

    txn.put(user)?;  // Automatically updates all indices
    Ok(())
})?;
```

#### Cross-Model Transaction

```rust
store.write(|txn| {
    // Create user
    let user = User { id: 1, email: "alice@example.com".into(), name: "Alice".into() };
    txn.put(user)?;

    // Create product linked to user
    let product = Product {
        id: 101,
        name: "Widget".into(),
        created_by: UserId(1),  // Foreign key
    };
    txn.put(product)?;

    Ok(())
})?;
```

---

## Data Flow and Process Flow

### Put Operation Flow

When you call `txn.put(model)`, here's what happens:

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. User Code: txn.put(user)                                     │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│ 2. WriteTransaction::put<User>(user)                            │
│    - Extract primary key: user.primary_key()                    │
│    - Extract secondary keys: user.get_secondary_keys()          │
│    - Extract relational keys: user.get_relational_keys()        │
│    - Compute hash: user.compute_hash()                          │
│    - Get subscriptions: user.get_subscriptions()                │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│ 3. Queue Operations (Ordered by Priority)                       │
│    Priority 0: MainTreeInsert                                   │
│      - table: "User::User::Main"                                │
│      - key: UserId(1)                                           │
│      - value: User { ... }                                      │
│                                                                  │
│    Priority 1: SecondaryKeyInsert (for each secondary key)      │
│      - table: "User::User::Secondary::Email"                    │
│      - key: UserEmail("alice@example.com")                      │
│      - value: UserId(1)                                         │
│                                                                  │
│    Priority 2: RelationalKeyInsert (for each relational key)    │
│      - table: "Product::Product::Relational::CreatedBy"         │
│      - key: UserId(1)                                           │
│      - value: ProductId(101)                                    │
│                                                                  │
│    Priority 3: SubscriptionTreeInsert (for each subscription)   │
│      - table: "User::User::Subscription::Updates"               │
│      - key: UserId(1)                                           │
│      - value: blake3_hash([user data])                          │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│ 4. Execute Operations (In Priority Order)                       │
│    for operation in queue.sorted_by_priority():                 │
│        operation.execute(backend_txn)                           │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│ 5. Backend Layer                                                │
│    RedbBackend::open_table_mut(table_name)                      │
│      -> BackendWritableTable<K, V>                              │
│    table.insert(key, value)                                     │
│      -> Serialize and write to B-tree                           │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│ 6. Commit Transaction                                           │
│    txn.commit()                                                 │
│      -> Flush all changes to disk                               │
│      -> Make changes visible to subsequent reads                │
└─────────────────────────────────────────────────────────────────┘
```

#### Detailed Steps

1. **User Calls `put(model)`**
   - Model instance with all fields populated
   - Type-safe: compiler ensures model implements `NetabaseModelTrait<D>`

2. **Extract Model Data**
   ```rust
   let pk = model.primary_key();                    // UserId(1)
   let secondary_keys = model.get_secondary_keys(); // Iterator of secondary keys
   let relational_keys = model.get_relational_keys(); // Iterator of foreign keys
   let hash = model.compute_hash();                 // Blake3 hash
   let subscriptions = model.get_subscriptions();   // Vec of topics
   ```

3. **Queue Operations**
   - Operations queued in priority order (not executed immediately)
   - Priority ensures main tree insert happens before index updates
   - This allows rollback if any operation fails

4. **Execute Operations**
   ```rust
   for operation in queue.sorted_by_priority() {
       match operation {
           MainTreeInsert { table_name, key, value, .. } => {
               let mut table = txn.open_table_mut(table_name)?;
               table.insert(key, value)?;
           }
           SecondaryKeyInsert { tree_name, key_data, primary_key_ref } => {
               let mut table = txn.open_table_mut(tree_name)?;
               table.insert(key_data, primary_key_ref)?;
           }
           // ... other operations
       }
   }
   ```

5. **Backend Serialization**
   - Keys and values converted to bytes
   - Backend-specific serialization (redb uses its own format, sled uses bincode)
   - Written to B-tree structure on disk

6. **Commit**
   - All changes flushed to disk
   - Transaction log updated
   - Changes become visible to new read transactions

### Get Operation Flow

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. User Code: txn.get::<User>(UserId(1))                       │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│ 2. ReadTransaction::get::<User>(key)                            │
│    - Resolve tree name: User::main_tree_name()                  │
│    - Type info: TableDefinition<UserId, User>                   │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│ 3. Backend Layer                                                │
│    table = txn.open_table("User::User::Main")?                  │
│    result = table.get(UserId(1))?                               │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│ 4. Deserialize (if found)                                       │
│    bytes -> User struct                                         │
│    Zero-copy where possible (redb)                              │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│ 5. Return to User                                               │
│    Ok(Some(User { id: 1, email: "alice@...", ... }))           │
└─────────────────────────────────────────────────────────────────┘
```

### Secondary Key Lookup Flow

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. User: txn.get_by_secondary_key::<User>(email_key)           │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│ 2. Get PK by Secondary Key                                      │
│    - Get discriminant from secondary key enum                   │
│    - Resolve secondary tree name                                │
│    - Look up: UserEmail -> UserId                               │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│ 3. Backend: Query Secondary Index Tree                          │
│    table = txn.open_table("User::User::Secondary::Email")?      │
│    primary_key = table.get(UserEmail("alice@example.com"))?     │
│    // Returns: Some(UserId(1))                                  │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│ 4. Get Full Model by Primary Key                               │
│    (Same as regular get operation)                              │
│    txn.get::<User>(UserId(1))                                   │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│ 5. Return Full Model                                            │
│    Ok(Some(User { ... }))                                       │
└─────────────────────────────────────────────────────────────────┘
```

### Delete Operation Flow

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. User: txn.delete::<User>(UserId(1))                         │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│ 2. Fetch Model First (to get keys for index cleanup)           │
│    model = txn.get::<User>(UserId(1))?                          │
│    if None: return Ok(())  // Already deleted                   │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│ 3. Extract Keys from Model                                      │
│    - secondary_keys = model.get_secondary_keys()                │
│    - relational_keys = model.get_relational_keys()              │
│    - subscriptions = model.get_subscriptions()                  │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│ 4. Delete from All Trees                                        │
│    - Main tree: remove(UserId(1))                               │
│    - Secondary indices: remove(UserEmail("..."))                │
│    - Relational trees: remove entries                           │
│    - Subscription trees: remove(UserId(1))                      │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│ 5. Commit Transaction                                           │
│    All deletions atomic                                         │
└─────────────────────────────────────────────────────────────────┘
```

---

## Backend Abstraction Layer

### Backend Trait Hierarchy

```
BackendStore
  ├── begin_read() -> ReadTransaction
  ├── begin_write() -> WriteTransaction
  ├── read(closure)
  └── write(closure)

BackendReadTransaction
  ├── open_table<K, V>(name) -> ReadableTable<K, V>
  └── table_exists(name) -> bool

BackendWriteTransaction : BackendReadTransaction
  ├── open_table_mut<K, V>(name) -> WritableTable<K, V>
  ├── commit()
  └── abort()

BackendReadableTable<K, V>
  ├── get(key) -> Option<V>
  ├── iter() -> Iterator<(K, V)>
  ├── len() -> Option<u64>
  └── is_empty() -> bool

BackendWritableTable<K, V> : BackendReadableTable<K, V>
  ├── insert(key, value)
  └── remove(key) -> bool
```

### Backend Implementations

#### Redb Backend

**Characteristics**:
- **Zero-copy reads**: Models deserialized directly from mmap'd pages
- **Typed tables**: Compile-time type safety with `TableDefinition<K, V>`
- **MVCC**: Multiple concurrent readers, single writer
- **ACID**: Full transaction support with crash recovery

**Key Types**:
```rust
impl BackendStore for RedbBackendStore {
    type ReadTransaction<'a> = redb::ReadTransaction;
    type WriteTransaction = redb::WriteTransaction;

    fn begin_read(&self) -> Result<Self::ReadTransaction<'_>, ...> {
        self.db.begin_read()
    }

    fn begin_write(&self) -> Result<Self::WriteTransaction, ...> {
        self.db.begin_write()
    }
}
```

**Serialization**: Built-in `redb::Key` and `redb::Value` traits

#### Sled Backend

**Characteristics**:
- **Bincode serialization**: Simple, efficient binary encoding
- **Optimistic concurrency**: Lock-free data structure
- **Embedded**: Pure Rust, no C dependencies
- **Simpler**: Less features but easier to embed

**Key Types**:
```rust
impl BackendStore for SledBackendStore {
    type ReadTransaction<'a> = SledReadTransaction<'a>;
    type WriteTransaction = SledWriteTransaction;

    fn begin_read(&self) -> Result<Self::ReadTransaction<'_>, ...> {
        Ok(SledReadTransaction { db: &self.db })
    }

    fn begin_write(&self) -> Result<Self::WriteTransaction, ...> {
        Ok(SledWriteTransaction { db: self.db.clone(), batch: sled::Batch::default() })
    }
}
```

**Serialization**: `bincode::Encode` and `bincode::Decode` traits

### Adding a New Backend

To add a new backend (e.g., IndexedDB for WASM):

1. **Implement `BackendStore`**
   ```rust
   pub struct IndexedDbBackend {
       db: web_sys::IdbDatabase,
   }

   impl BackendStore for IndexedDbBackend {
       type ReadTransaction<'a> = IndexedDbReadTxn;
       type WriteTransaction = IndexedDbWriteTxn;
       // ... implement methods
   }
   ```

2. **Implement Transaction Traits**
   ```rust
   impl BackendReadTransaction for IndexedDbReadTxn { ... }
   impl BackendWriteTransaction for IndexedDbWriteTxn { ... }
   ```

3. **Implement Table Traits**
   ```rust
   impl<K, V> BackendReadableTable<K, V> for IndexedDbTable<K, V> { ... }
   impl<K, V> BackendWritableTable<K, V> for IndexedDbTable<K, V> { ... }
   ```

4. **Create Store Wrapper**
   ```rust
   pub struct IndexedDbStore<D: NetabaseDefinition> {
       backend: IndexedDbBackend,
       _marker: PhantomData<D>,
   }

   impl<D: NetabaseDefinition> StoreTrait<D> for IndexedDbStore<D> { ... }
   ```

---

## Multi-Definition Management

### The Problem

Large applications have multiple logical domains (Users, Products, Orders, etc.). Each domain may need its own database for:
- **Isolation**: Clear boundaries between domains
- **Performance**: Smaller databases, less contention
- **Scalability**: Distribute databases across machines

But they also need:
- **Cross-domain queries**: Products reference Users
- **Atomic transactions**: Create Order with OrderItems
- **Lazy loading**: Don't load unused domains

### The Solution: DefinitionManager

```rust
pub struct DefinitionManager<R, D, P, B>
where
    R: DefinitionManagerTrait<Definition = D, Permissions = P, Backend = B>,
    D: NetabaseDefinition,
    P: PermissionEnumTrait,
{
    root_path: PathBuf,
    stores: HashMap<D::Discriminant, DefinitionStoreLink<D, B>>,
    warm_on_access: HashSet<D::Discriminant>,
    accessed_in_transaction: HashSet<D::Discriminant>,
}
```

### Definition Store Link (Lazy Loading)

```rust
pub enum DefinitionStoreLink<D, B> {
    /// Not yet loaded
    Unloaded(D::Discriminant),

    /// Currently loaded in memory
    Loaded(B),  // B is RedbStore<D> or SledStore<D>
}

impl<D, B> DefinitionStoreLink<D, B> {
    pub fn is_loaded(&self) -> bool {
        matches!(self, Self::Loaded(_))
    }

    pub fn load(&mut self, root_path: &Path) -> NetabaseResult<&mut B> {
        if let Self::Unloaded(disc) = self {
            let db_path = root_path.join(disc.name()).join("store.db");
            let store = B::new(db_path)?;
            *self = Self::Loaded(store);
        }
        self.get_store_mut()
    }

    pub fn unload(&mut self) {
        if let Self::Loaded(_) = self {
            *self = Self::Unloaded(/* ... */);
        }
    }
}
```

### Directory Structure

```
root_path/
├── User/
│   └── store.db          # User definition database
├── Product/
│   └── store.db          # Product definition database
├── Order/
│   └── store.db          # Order definition database
└── Review/
    └── store.db          # Review definition database
```

### Manager Lifecycle

1. **Initialization**
   ```rust
   let manager = DefinitionManager::<
       EcommerceManager,
       EcommerceDefinitions,
       EcommercePermissions,
       RedbStore<EcommerceDefinitions>
   >::new("./data")?;

   // All definitions start as Unloaded
   assert_eq!(manager.loaded_definitions().len(), 0);
   ```

2. **First Access (Lazy Load)**
   ```rust
   manager.write(Permission::Admin, |txn| {
       // First access to User definition -> loads database
       let user_txn = txn.definition_txn_mut::<User, true>(
           &EcommerceDefinitionsDiscriminants::User
       )?;

       user_txn.put(user)?;
       Ok(())
   })?;

   // User definition now loaded
   assert!(manager.is_loaded(&EcommerceDefinitionsDiscriminants::User));
   ```

3. **Cross-Definition Transaction**
   ```rust
   manager.write(Permission::Admin, |txn| {
       // Access User definition
       let user_txn = txn.definition_txn_mut::<User, true>(
           &EcommerceDefinitionsDiscriminants::User
       )?;
       user_txn.put(user)?;

       // Access Product definition (auto-loads if needed)
       let product_txn = txn.definition_txn_mut::<Product, true>(
           &EcommerceDefinitionsDiscriminants::Product
       )?;
       product_txn.put(product)?;

       // Both definitions accessed in this transaction
       Ok(())
   })?;
   ```

4. **Auto-Unload After Transaction**
   ```rust
   // After transaction, unused definitions can be unloaded
   let unloaded = manager.unload_unused();

   // Definitions marked warm_on_access are never unloaded
   manager.add_warm_on_access(EcommerceDefinitionsDiscriminants::User);
   ```

### Permission System

```rust
pub enum EcommercePermissions {
    Admin,        // Read + Write all definitions
    Manager,      // Read all, Write Product + Order
    Customer,     // Read Product only
    Support,      // Read User + Order, Write Order
}

impl PermissionEnumTrait for EcommercePermissions {
    fn can_read<D: NetabaseDefinition>(
        &self,
        definition: &D::Discriminant,
    ) -> bool {
        match self {
            Self::Admin => true,
            Self::Manager => true,
            Self::Customer => definition.name() == "Product",
            Self::Support => matches!(definition.name(), "User" | "Order"),
        }
    }

    fn can_write<D: NetabaseDefinition>(
        &self,
        definition: &D::Discriminant,
    ) -> bool {
        match self {
            Self::Admin => true,
            Self::Manager => matches!(definition.name(), "Product" | "Order"),
            Self::Customer => false,
            Self::Support => definition.name() == "Order",
        }
    }
}
```

**Compile-Time Permission Checking**:

```rust
// This compiles: Admin has write access
manager.write(EcommercePermissions::Admin, |txn| {
    let user_txn = txn.definition_txn_mut::<User, true>(&disc)?;
    user_txn.put(user)?;
    Ok(())
})?;

// This compiles: Customer has read access to Product
manager.read(EcommercePermissions::Customer, |txn| {
    let product_txn = txn.definition_txn::<Product>(&disc)?;
    product_txn.get(ProductId(1))?;
    Ok(())
})?;

// This would fail at runtime: Customer cannot write
manager.write(EcommercePermissions::Customer, |txn| {
    let product_txn = txn.definition_txn_mut::<Product, true>(&disc)?;
    // Runtime error: insufficient permissions
    product_txn.put(product)?;
    Ok(())
})?;
```

---

## Type System and Trait Architecture

### The Trait Hierarchy

```
NetabaseDefinitionTrait
  ├── type Keys
  ├── type ModelAssociatedTypes
  ├── type Permissions
  └── TreeManager<Self>

NetabaseModelTrait<D>
  ├── type Keys: NetabaseModelKeyTrait
  ├── type PrimaryKey
  ├── type SecondaryKeys: Iterator
  ├── type RelationalKeys: Iterator
  ├── type SubscriptionEnum
  ├── type Hash
  ├── const MODEL_TREE_NAME
  ├── fn primary_key(&self) -> PrimaryKey
  ├── fn get_secondary_keys(&self) -> SecondaryKeys
  ├── fn get_relational_keys(&self) -> RelationalKeys
  ├── fn get_subscriptions(&self) -> Vec<SubscriptionEnum>
  └── fn compute_hash(&self) -> Hash

NetabaseModelKeyTrait<D, M>
  ├── type PrimaryKey
  ├── type SecondaryEnum
  ├── type RelationalEnum
  └── type SubscriptionEnum

StoreTrait<D>
  ├── type ReadTxn<'a>: ReadTransaction<D>
  ├── type WriteTxn: WriteTransaction<D>
  ├── fn read<F, R>(&self, f: F) -> Result<R>
  └── fn write<F, R>(&self, f: F) -> Result<R>
```

### Associated Types Deep Dive

#### ModelAssociatedTypes

The `ModelAssociatedTypes` is a unified enum that wraps all model-specific types:

```rust
pub enum MyDefinitionModelAssociatedTypes {
    // Primary keys
    UserPrimaryKey(UserId),
    ProductPrimaryKey(ProductId),

    // Models
    UserModel(User),
    ProductModel(Product),

    // Secondary key discriminants
    UserSecondaryKeyDiscriminant(UserSecondaryKeysDiscriminants),
    ProductSecondaryKeyDiscriminant(ProductSecondaryKeysDiscriminants),

    // Relational key discriminants
    ProductRelationalKeyDiscriminant(ProductRelationalKeysDiscriminants),

    // Secondary key data
    UserSecondaryKeyData(UserSecondaryKeys),
    ProductSecondaryKeyData(ProductSecondaryKeys),

    // Relational key data
    ProductRelationalKeyData(ProductRelationalKeys),

    // Subscription key discriminants
    UserSubscriptionKeyDiscriminant(UserSubscriptionKeysDiscriminants),
}
```

**Why?** This eliminates opaque `Vec<u8>` and `String` types, providing full type safety throughout the stack.

#### Key Enums

Each model has three key enums:

1. **SecondaryKeys Enum**
   ```rust
   #[derive(EnumDiscriminants)]
   pub enum UserSecondaryKeys {
       Email(UserEmail),
       Username(UserUsername),
   }

   // Discriminant enum (generated by strum)
   pub enum UserSecondaryKeysDiscriminants {
       Email,
       Username,
   }
   ```

2. **RelationalKeys Enum**
   ```rust
   #[derive(EnumDiscriminants)]
   pub enum ProductRelationalKeys {
       CreatedBy(UserId),  // Foreign key to User
   }

   pub enum ProductRelationalKeysDiscriminants {
       CreatedBy,
   }
   ```

3. **SubscriptionKeys Enum**
   ```rust
   #[derive(EnumDiscriminants)]
   pub enum UserSubscriptionKeys {
       Updates,
       Authentication,
   }

   pub enum UserSubscriptionKeysDiscriminants {
       Updates,
       Authentication,
   }
   ```

### Discriminants and strum

We use [strum](https://docs.rs/strum/) extensively for discriminant handling:

- `IntoDiscriminant`: Convert enum to discriminant
- `IntoEnumIterator`: Iterate over all variants
- `AsRefStr`: Convert discriminant to string name

**Example**:
```rust
let key = UserSecondaryKeys::Email(UserEmail("alice@example.com".into()));
let disc = key.discriminant();  // UserSecondaryKeysDiscriminants::Email

for disc in UserSecondaryKeysDiscriminants::iter() {
    println!("Secondary key: {}", disc.name());
}
// Prints: "Email", "Username"
```

### Phantom Types and Zero-Sized Types

We use phantom types extensively to maintain type safety without runtime cost:

```rust
pub struct RedbStore<D: NetabaseDefinition> {
    db: redb::Database,
    _marker: PhantomData<D>,  // Zero-sized, compile-time only
}
```

This allows:
- Type-safe API: `RedbStore<UserDefinitions>` vs `RedbStore<ProductDefinitions>`
- No runtime overhead: `PhantomData<D>` is optimized away
- Compile-time checks: Can't mix definitions by accident

---

## Code Generation System

### The Boilerplate Problem

For a single model with 2 secondary keys, manual implementation requires:

- Model struct: ~10 lines
- Primary key wrapper: ~10 lines
- Secondary key wrappers: ~20 lines (2 keys)
- Secondary keys enum: ~15 lines
- Relational keys enum: ~15 lines
- Subscription keys enum: ~10 lines
- NetabaseModelTrait impl: ~100 lines
- NetabaseModelKeyTrait impl: ~50 lines
- Backend-specific impls: ~50 lines
- Tree name constants: ~20 lines

**Total: ~300 lines per model**

For the example `examples/boilerplate.rs` with 6 models: **3,846 lines**

### The Solution: Procedural Macros

#### Approach 1: Manual Definition with Macros (Current, Working)

```rust
#[netabase_definition_module(EcommerceDefinitions, EcommerceKeys, subscriptions(Updates))]
pub mod ecommerce {
    #[derive(NetabaseModel, Clone, Debug)]
    #[subscribe(Updates)]
    pub struct User {
        #[primary_key]
        pub id: u64,

        #[secondary_key]
        pub email: String,

        #[secondary_key]
        pub username: String,

        pub name: String,
        pub age: u32,
    }

    #[derive(NetabaseModel, Clone, Debug)]
    pub struct Product {
        #[primary_key]
        pub id: u64,

        #[secondary_key]
        pub name: String,

        #[relation]
        pub created_by: UserId,  // Foreign key to User

        pub price: f64,
    }
}

// Generates ~3800 lines of boilerplate
```

**94% code reduction**: 50 lines user code → 3,850 lines generated

#### Approach 2: TOML-Based Generation (In Progress)

**schemas/User.netabase.toml**:
```toml
[definition]
name = "User"
version = "1"

[model]
fields = [
    { name = "id", type = "u64" },
    { name = "email", type = "String" },
    { name = "username", type = "String" },
    { name = "name", type = "String" },
    { name = "age", type = "u32" },
]

[keys.primary]
field = "id"

[[keys.secondary]]
name = "Email"
field = "email"
unique = true

[[keys.secondary]]
name = "Username"
field = "username"
unique = true

[[subscriptions]]
name = "Updates"
description = "All user updates"
```

**Usage**:
```rust
netabase_definition_from_toml!("schemas/User.netabase.toml");

// Generates complete definition
```

**Manager-Level Generation**:

**ecommerce.root.netabase.toml**:
```toml
[manager]
name = "EcommerceManager"
root_path = "./data"

[[definitions]]
name = "User"
schema_file = "schemas/User.netabase.toml"

[[definitions]]
name = "Product"
schema_file = "schemas/Product.netabase.toml"

[[permissions.roles]]
name = "Admin"
level = "ReadWrite"
definitions = ["*"]

[[permissions.roles]]
name = "Customer"
read = ["Product"]
write = []
```

**Usage**:
```rust
netabase_manager_from_toml!("ecommerce.root.netabase.toml");

// Generates:
// - All definitions (User, Product)
// - Manager enum (EcommerceManager)
// - Permission enum (EcommercePermissions)
// - All trait implementations
```

### Code Generation Pipeline

```
TOML Schema
    │
    ▼
┌──────────────┐
│ TOML Parser  │  (netabase_codegen::toml_parser)
│ - Parse      │
│ - Validate   │
└──────┬───────┘
       │
       ▼
┌──────────────────────┐
│ Intermediate Schema  │  (netabase_codegen::toml_types)
│ - DefinitionSchema   │
│ - ManagerSchema      │
└──────┬───────────────┘
       │
       ▼
┌──────────────┐
│  Generator   │  (netabase_codegen::generator)
│ - gen models │
│ - gen keys   │
│ - gen traits │
└──────┬───────┘
       │
       ▼
┌──────────────────┐
│  Rust Code       │  (proc_macro2::TokenStream)
│ - Formatted      │
│ - Type-checked   │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│  User Code       │
│ - Compiles       │
│ - Type-safe      │
└──────────────────┘
```

### Generated Code Example

From this TOML:
```toml
[definition]
name = "User"

[model]
fields = [
    { name = "id", type = "u64" },
    { name = "email", type = "String" },
]

[keys.primary]
field = "id"

[[keys.secondary]]
name = "Email"
field = "email"
unique = true
```

Generates:
```rust
// Model struct
#[derive(Debug, Clone, PartialEq, bincode::Encode, bincode::Decode)]
pub struct User {
    pub id: u64,
    pub email: String,
}

// Primary key wrapper
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserId(pub u64);

// Secondary key wrapper
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserEmail(pub String);

// Secondary keys enum
#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(name(UserSecondaryKeysDiscriminants))]
pub enum UserSecondaryKeys {
    Email(UserEmail),
}

// NetabaseModelTrait implementation
impl NetabaseModelTrait<UserDefinitions> for User {
    type Keys = UserKeys;
    type PrimaryKey = UserId;
    type SecondaryKeys = std::iter::Empty<UserSecondaryKeys>;  // or actual iterator
    type RelationalKeys = std::iter::Empty<UserRelationalKeys>;
    type SubscriptionEnum = UserSubscriptionKeys;
    type Hash = [u8; 32];

    const MODEL_TREE_NAME: UserDefinitionsDiscriminants = UserDefinitionsDiscriminants::User;

    fn primary_key(&self) -> Self::PrimaryKey {
        UserId(self.id)
    }

    fn get_secondary_keys(&self) -> Self::SecondaryKeys {
        // Generate iterator over secondary keys
    }

    // ... other methods
}

// Tree name constants
impl User {
    pub const MAIN_TREE_NAME: &'static str = "User::User::Main";
    pub const SECONDARY_TREE_NAMES: &'static [&'static str] = &["User::User::Secondary::Email"];
}

// Backend-specific implementations
impl RedbValue for User { ... }
impl RedbKey for UserId { ... }
// ... etc
```

---

## Performance Characteristics

### Read Performance

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| Get by primary key | O(log n) | B-tree lookup |
| Get by secondary key | O(log n + log m) | Two B-tree lookups |
| Iterate all models | O(n) | Sequential scan |
| Subscription accumulator | O(n) | Full tree scan, XOR accumulation |

**Optimization Tips**:
- Use primary key lookups when possible (single B-tree access)
- Secondary keys are fast for unique lookups
- Keep frequently-accessed definitions warm (avoid lazy load overhead)

### Write Performance

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| Insert model | O(k * log n) | k = number of indices |
| Update model | O(k * log n) | Must update all indices |
| Delete model | O(k * log n) | Must clean up all indices |
| Batch insert | O(m * k * log n) | m = batch size |

**Optimization Tips**:
- Minimize secondary indices (each adds write overhead)
- Use batch operations when inserting many models
- Keep write transactions short to minimize lock contention

### Memory Usage

**Per-Model Overhead**:
- Model data: Size of struct
- Primary key: Size of key wrapper
- Secondary keys: k * (key size + pointer)
- Relational keys: r * (foreign key size + pointer)

**Manager Overhead**:
- Loaded definitions: Full database handle per definition
- Unloaded definitions: ~8 bytes (discriminant)
- Warm set: ~24 bytes per warmed definition

**Optimization Tips**:
- Lazy load definitions (don't load all upfront)
- Use `unload_unused()` after transactions
- Warm frequently-accessed definitions only

### Disk Usage

**Tree Storage**:
```
Main Tree: n * (key_size + model_size)
Secondary Indices: k * n * (index_key_size + primary_key_size)
Relational Trees: r * n * (foreign_key_size + primary_key_size)
Subscription Trees: s * n * (primary_key_size + 32 bytes)
```

Where:
- n = number of models
- k = number of secondary indices
- r = number of relational keys per model
- s = number of subscriptions

**Example**: User model with 1M users
```
Model: 100 bytes
Primary key: 8 bytes

Main tree: 1M * 108 bytes = 103 MB
Email index: 1M * (40 + 8) bytes = 46 MB
Username index: 1M * (30 + 8) bytes = 36 MB
Updates subscription: 1M * (8 + 32) bytes = 38 MB

Total: ~223 MB
```

**Optimization Tips**:
- Limit secondary indices to frequently-queried fields
- Use relational trees for sparse relationships
- Consider archiving old subscription data

### Concurrency

**Redb Backend**:
- Multiple concurrent readers (MVCC)
- Single writer at a time
- Writers don't block readers
- Readers see consistent snapshot

**Sled Backend**:
- Lock-free data structure
- Optimistic concurrency
- May require retries on conflicts
- Generally good for read-heavy workloads

**Manager Concurrency**:
- Multiple definitions can be accessed in parallel
- Each definition has independent locks
- Cross-definition transactions acquire locks in order

---

## Summary

Netabase Store provides a comprehensive, type-safe database abstraction with:

1. **Tree-Based Storage**: Organized, predictable data layout
2. **Type-Safe Transactions**: Compile-time guarantees
3. **Automatic Index Management**: Secondary and relational keys
4. **Multi-Definition Support**: Lazy loading and cross-database transactions
5. **Backend Abstraction**: Support multiple storage engines
6. **Code Generation**: 94% boilerplate reduction
7. **High Performance**: Efficient operations with predictable characteristics

The architecture prioritizes:
- **Developer Experience**: Minimal boilerplate, clear APIs
- **Type Safety**: Compile-time checks everywhere
- **Flexibility**: Multiple backends, extensible design
- **Performance**: Zero-cost abstractions, efficient operations

---

**Further Reading**:
- [Getting Started Guide](./getting_started.md)
- [Cross-Definition Access Plan](./CROSS_DEFINITION_PLAN.md)
- [API Documentation](../netabase_store/src/lib.rs)
- [Examples](../examples/)
