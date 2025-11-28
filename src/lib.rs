//! # Netabase Store - Type-Safe Multi-Backend Key-Value Storage
//!
//! `netabase_store` is a type-safe, multi-backend key-value storage library that provides
//! a unified API across different database backends (Sled, Redb, IndexedDB). It uses procedural
//! macros to generate type-safe schemas and supports primary and secondary key indexing.
//!
//! ## Key Features
//!
//! - **Multi-Backend Support**: Choose between Sled (embedded DB), Redb (ACID compliant), or IndexedDB (browser)
//! - **Type-Safe Schema**: Derive macros generate compile-time checked schemas
//! - **Primary & Secondary Keys**: Efficient indexing with automatic secondary key management
//! - **Cross-Platform**: Supports both native and WASM targets
//! - **Zero-Copy Operations**: Efficient serialization with bincode
//! - **libp2p Integration**: Optional integration for distributed systems
//!
//! ## Quick Start
//!
//! ### Define Your Data Models
//!
//! ```rust
//! use netabase_store::{netabase_definition_module, NetabaseModel};
//!
//! // Define your schema with the definition module macro
//! #[netabase_definition_module(BlogDefinition, BlogKeys)]
//! mod blog {
//!     use super::*;
//!     use netabase_store::{netabase, NetabaseModel};
//!
//!     /// User model with primary and secondary keys
//!     #[derive(
//!         NetabaseModel,
//!         Clone,
//!         Debug,
//!         PartialEq,
//!         bincode::Encode,
//!         bincode::Decode,
//!         serde::Serialize,
//!         serde::Deserialize,
//!     )]
//!     #[netabase(BlogDefinition)]
//!     pub struct User {
//!         #[primary_key]
//!         pub id: u64,
//!         pub username: String,
//!         #[secondary_key]
//!         pub email: String,
//!         pub age: u32,
//!     }
//!
//!     /// Post model associated with the same definition
//!     #[derive(
//!         NetabaseModel,
//!         Clone,
//!         Debug,
//!         PartialEq,
//!         bincode::Encode,
//!         bincode::Decode,
//!         serde::Serialize,
//!         serde::Deserialize,
//!     )]
//!     #[netabase(BlogDefinition)]
//!     pub struct Post {
//!         #[primary_key]
//!         pub id: String,
//!         pub title: String,
//!         pub author_id: u64,
//!         #[secondary_key]
//!         pub published: bool,
//!     }
//! }  // end mod blog
//!
//! use blog::*;
//! ```
//!
//! ### Using the Unified NetabaseStore (Recommended)
//!
//! ```rust
//! use netabase_store::{NetabaseStore, netabase_definition_module, NetabaseModel, netabase};
//! use netabase_store::traits::tree::NetabaseTreeSync;
//! use netabase_store::traits::model::NetabaseModelTrait;
//!
//! // Define your schema
//! #[netabase_definition_module(BlogDefinition, BlogKeys)]
//! mod blog {
//!     use netabase_store::{NetabaseModel, netabase};
//!
//!     #[derive(NetabaseModel, Clone, Debug, PartialEq,
//!              bincode::Encode, bincode::Decode,
//!              serde::Serialize, serde::Deserialize)]
//!     #[netabase(BlogDefinition)]
//!     pub struct User {
//!         #[primary_key]
//!         pub id: u64,
//!         pub username: String,
//!         #[secondary_key]
//!         pub email: String,
//!         pub age: u32,
//!     }
//! }
//! use blog::*;
//!
//! # fn main() -> Result<(), netabase_store::error::NetabaseError> {
//! // Create a store with any backend - Sled example (using temp for doctest)
//! let store = NetabaseStore::<BlogDefinition, _>::temp()?;
//!
//! // Or use Redb (in production):
//! // let store = NetabaseStore::<BlogDefinition, _>::redb("./my_db.redb")?;
//!
//! // Open a tree for the User model - works with any backend!
//! let user_tree = store.open_tree::<User>();
//!
//! // Standard operations work the same across all backends
//! let alice = User {
//!     id: 1,
//!     username: "alice".to_string(),
//!     email: "alice@example.com".to_string(),
//!     age: 30,
//! };
//!
//! user_tree.put(alice.clone())?;
//! let retrieved = user_tree.get(alice.primary_key())?.unwrap();
//! assert_eq!(retrieved, alice);
//!
//! // Backend-specific features still available
//! store.flush()?; // Sled-specific
//! # Ok(())
//! # }
//! ```
//!
//! ### Using Sled Backend (Direct)
//!
//! ```rust
//! use netabase_store::{netabase_definition_module, NetabaseModel, netabase};
//! use netabase_store::databases::sled_store::SledStore;
//! use netabase_store::traits::tree::NetabaseTreeSync;
//! use netabase_store::traits::model::NetabaseModelTrait;
//!
//! // Define schema for this example
//! #[netabase_definition_module(BlogDefinition, BlogKeys)]
//! mod blog {
//!     use super::*;
//!     use netabase_store::{NetabaseModel, netabase};
//!
//!     #[derive(NetabaseModel, Clone, Debug, PartialEq,
//!              bincode::Encode, bincode::Decode,
//!              serde::Serialize, serde::Deserialize)]
//!     #[netabase(BlogDefinition)]
//!     pub struct User {
//!         #[primary_key]
//!         pub id: u64,
//!         pub username: String,
//!         #[secondary_key]
//!         pub email: String,
//!         pub age: u32,
//!     }
//! }
//! use blog::*;
//!
//! // Open a database
//! let store = SledStore::<BlogDefinition>::temp().unwrap();
//!
//! // Get a type-safe tree for the User model
//! let user_tree = store.open_tree::<User>();
//!
//! // Create a user
//! let alice = User {
//!     id: 1,
//!     username: "alice".to_string(),
//!     email: "alice@example.com".to_string(),
//!     age: 30,
//! };
//!
//! // Insert the user
//! user_tree.put(alice.clone()).unwrap();
//!
//! // Retrieve by primary key
//! let retrieved = user_tree.get(alice.primary_key()).unwrap().unwrap();
//! assert_eq!(retrieved, alice);
//!
//! // Query by secondary key (email) using convenience function
//! use blog::AsUserEmail;
//! let users_by_email = user_tree
//!     .get_by_secondary_key("alice@example.com".as_user_email_key())
//!     .unwrap();
//! assert_eq!(users_by_email.len(), 1);
//!
//! // Update the user
//! let mut alice_updated = alice.clone();
//! alice_updated.age = 31;
//! user_tree.put(alice_updated).unwrap();
//!
//! // Remove the user
//! user_tree.remove(alice.primary_key()).unwrap();
//! ```
//!
//! ### Using Redb Backend (Native)
//!
//! ```rust
//! use netabase_store::{netabase_definition_module, NetabaseModel, netabase};
//!
//! // Define your schema with the definition module macro
//! #[netabase_definition_module(BlogDefinition, BlogKeys)]
//! mod blog {
//!     use super::*;
//!     use netabase_store::{netabase, NetabaseModel};
//!
//!     /// User model with primary and secondary keys
//!     #[derive(
//!         NetabaseModel,
//!         Clone,
//!         Debug,
//!         PartialEq,
//!         bincode::Encode,
//!         bincode::Decode,
//!         serde::Serialize,
//!         serde::Deserialize,
//!     )]
//!     #[netabase(BlogDefinition)]
//!     pub struct User {
//!         #[primary_key]
//!         pub id: u64,
//!         pub username: String,
//!         #[secondary_key]
//!         pub email: String,
//!         pub age: u32,
//!     }
//!
//!     /// Post model associated with the same definition
//!     #[derive(
//!         NetabaseModel,
//!         Clone,
//!         Debug,
//!         PartialEq,
//!         bincode::Encode,
//!         bincode::Decode,
//!         serde::Serialize,
//!         serde::Deserialize,
//!     )]
//!     #[netabase(BlogDefinition)]
//!     pub struct Post {
//!         #[primary_key]
//!         pub id: String,
//!         pub title: String,
//!         pub author_id: u64,
//!         #[secondary_key]
//!         pub published: bool,
//!     }
//! }  // end mod blog
//!
//! use blog::*;
//! use netabase_store::databases::redb_store::RedbStore;
//! use netabase_store::traits::tree::NetabaseTreeSync;
//! use netabase_store::traits::model::NetabaseModelTrait;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Open a database with Redb backend (using temp file for doctest)
//! let temp_dir = tempfile::tempdir()?;
//! let db_path = temp_dir.path().join("test.redb");
//! let store = RedbStore::<BlogDefinition>::new(db_path)?;
//!
//! // API is identical to SledStore
//! let user_tree = store.open_tree::<User>();
//!
//! let user = User {
//!     id: 1,
//!     username: "bob".to_string(),
//!     email: "bob@example.com".to_string(),
//!     age: 25,
//! };
//! user_tree.put(user)?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Using IndexedDB Backend (WASM)
//!
//! IndexedDB backend provides persistent storage in web browsers.
//! This example requires the `wasm` feature and wasm32 target:
//!
//! ```
//! # #[cfg(all(target_arch = "wasm32", feature = "wasm"))]
//! # {
//! use netabase_store::{netabase_definition_module, NetabaseModel, netabase};
//! use netabase_store::databases::indexeddb_store::IndexedDBStore;
//! use netabase_store::traits::tree::NetabaseTreeAsync;
//! use netabase_store::traits::model::NetabaseModelTrait;
//!
//! #[netabase_definition_module(AppDefinition, AppKeys)]
//! mod app {
//!     use super::*;
//!     use netabase_store::{NetabaseModel, netabase};
//!
//!     #[derive(NetabaseModel, Clone, Debug, PartialEq,
//!              bincode::Encode, bincode::Decode,
//!              serde::Serialize, serde::Deserialize)]
//!     #[netabase(AppDefinition)]
//!     pub struct User {
//!         #[primary_key]
//!         pub id: u64,
//!         pub username: String,
//!         #[secondary_key]
//!         pub email: String,
//!     }
//! }
//! use app::*;
//!
//! async fn wasm_example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Open a database in the browser (persists across page reloads)
//!     let store = IndexedDBStore::<AppDefinition>::new("my_app_db").await?;
//!
//!     // Get an async tree - note WASM uses async API
//!     let user_tree = store.open_tree::<User>();
//!
//!     // Create a user
//!     let alice = User {
//!         id: 1,
//!         username: "alice".into(),
//!         email: "alice@example.com".into()
//!     };
//!
//!     // All operations are async in WASM
//!     user_tree.put(alice.clone()).await?;
//!     let retrieved = user_tree.get(alice.primary_key()).await?;
//!
//!     // Query by secondary key using convenience function
//!     use app::AsUserEmail;
//!     let users_by_email = user_tree
//!         .get_by_secondary_key("alice@example.com".as_user_email_key())
//!         .await?;
//!
//!     Ok(())
//! }
//! # }
//! ```
//!
//! ## Batch Operations
//!
//! For high-performance bulk operations, use batch processing to reduce overhead.
//! This example requires the `native` feature:
//!
//! ```rust
//! use netabase_store::{netabase_definition_module, NetabaseModel, netabase};
//! use netabase_store::databases::sled_store::SledStore;
//! use netabase_store::traits::batch::{Batchable, BatchOperations};
//! use netabase_store::traits::model::NetabaseModelTrait;
//!
//! #[netabase_definition_module(AppDefinition, AppKeys)]
//! mod app {
//!     use super::*;
//!     use netabase_store::{NetabaseModel, netabase};
//!
//!     #[derive(NetabaseModel, Clone, Debug,
//!              bincode::Encode, bincode::Decode,
//!              serde::Serialize, serde::Deserialize)]
//!     #[netabase(AppDefinition)]
//!     pub struct User {
//!         #[primary_key]
//!         pub id: u64,
//!         pub name: String,
//!     }
//! }
//! use app::*;
//!
//! # fn main() -> Result<(), netabase_store::error::NetabaseError> {
//! let store = SledStore::<AppDefinition>::temp()?;
//! let user_tree = store.open_tree::<User>();
//!
//! // Prepare batch of users
//! let users: Vec<User> = (0..1000)
//!     .map(|i| User { id: i, name: format!("User {}", i) })
//!     .collect();
//!
//! // Bulk insert - much faster than individual puts
//! user_tree.put_batch(users)?;
//! # Ok(())
//! # }
//! ```
//!
//! Batch operations are atomic and significantly faster than individual operations
//! when working with many records.
//!
//! ## Zero-Copy Redb Backend (High Performance)
//!
//! For maximum performance with the Redb backend, use the zero-copy API that provides
//! explicit transaction control and zero-copy reads.
//!
//! ### Enabling Zero-Copy
//!
//! Add both `redb` and `redb-zerocopy` features:
//!
//! ```toml
//! [dependencies]
//! netabase_store = { version = "*", features = ["redb", "redb-zerocopy"] }
//! ```
//!
//! ### Performance Characteristics
//!
//! | Operation | Regular API | Zero-Copy API | Speedup |
//! |-----------|------------|---------------|---------|
//! | Bulk insert (1000 items) | ~50ms | ~5ms | **10x faster** |
//! | Secondary key query | ~5.4ms | ~100μs | **54x faster** |
//! | Single read | ~100ns | ~100ns | Similar |
//!
//! ### Usage Example
//!
//! ```no_run
//! // This example requires the `redb-zerocopy` feature
//! use netabase_store::{netabase_definition_module, NetabaseModel};
//! use netabase_store::databases::redb_zerocopy::RedbStoreZeroCopy;
//! use netabase_store::traits::model::NetabaseModelTrait;
//!
//! #[netabase_definition_module(AppDef, AppKeys)]
//! mod app {
//!     use netabase_store::{NetabaseModel, netabase};
//!     #[derive(NetabaseModel, Clone, Debug, PartialEq,
//!              bincode::Encode, bincode::Decode,
//!              serde::Serialize, serde::Deserialize)]
//!     #[netabase(AppDef)]
//!     pub struct User {
//!         #[primary_key]
//!         pub id: u64,
//!         pub name: String,
//!         #[secondary_key]
//!         pub email: String,
//!     }
//! }
//! use app::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let temp_dir = tempfile::tempdir()?;
//! // Create a zero-copy store
//! let store = RedbStoreZeroCopy::<AppDef>::new(temp_dir.path().join("app.redb"))?;
//!
//! // Write transaction - batch multiple operations
//! let mut write_txn = store.begin_write()?;
//! let mut tree = write_txn.open_tree::<User>()?;
//!
//! // Batch insert 1000 users in one transaction
//! for i in 0..1000 {
//!     tree.put(User {
//!         id: i,
//!         name: format!("User {}", i),
//!         email: format!("user{}@example.com", i),
//!     })?;
//! }
//! drop(tree);
//! write_txn.commit()?;  // All 1000 inserts committed atomically
//!
//! // Read transaction - efficient queries
//! let read_txn = store.begin_read()?;
//! let tree = read_txn.open_tree::<User>()?;
//! let user = tree.get(&UserPrimaryKey(42))?.unwrap();
//! assert_eq!(user.name, "User 42");
//! # Ok(())
//! # }
//! ```
//!
//! ### When to Use Zero-Copy API
//!
//! **Use zero-copy when:**
//! - You need to batch multiple operations (bulk inserts/updates)
//! - Performance is critical
//! - You want explicit transaction control
//! - You're doing many secondary key queries
//!
//! **Use regular API when:**
//! - Simplicity is more important than performance
//! - Single-operation transactions are fine
//! - You want the simplest possible code
//!
//! ### Available Types
//!
//! When `redb-zerocopy` feature is enabled, these types are re-exported at the crate root:
//! - `RedbStoreZeroCopy` - The zero-copy store
//! - `RedbWriteTransactionZC` - Write transaction handle
//! - `RedbReadTransactionZC` - Read transaction handle
//! - `RedbTreeMut` - Mutable tree for write transactions
//! - `RedbTree` - Immutable tree for read transactions
//! - `BorrowedGuard` - Guard for zero-copy borrowed data
//! - `BorrowedIter` - Iterator for zero-copy iteration
//!
//! ## Custom Backend Implementations
//!
//! Netabase Store provides a unified API through traits, making it easy to implement
//! custom storage backends:
//!
//! - **`NetabaseTreeSync`**: For synchronous backends (native)
//! - **`NetabaseTreeAsync`**: For asynchronous backends (WASM, remote databases)
//! - **`OpenTree`**: For creating tree instances from stores
//! - **`Batchable`**: For batch operation support
//!
//! Implement these traits to add support for new databases while maintaining
//! compatibility with all existing code using netabase_store.
//!
//! ## Architecture
//!
//! ### Core Components
//!
//! 1. **Definition Module**: Groups related models into a schema
//!    - Created with `#[netabase_definition_module]` macro
//!    - Generates an enum containing all models
//!    - Generates a keys enum for type-safe queries
//!
//! 2. **Models**: Individual data structures
//!    - Derived with `#[derive(NetabaseModel)]`
//!    - Must have one `#[primary_key]`
//!    - Can have multiple `#[secondary_key]` fields
//!
//! 3. **Storage Backends**:
//!    - **SledStore**: Fast embedded database, native only
//!    - **RedbStore**: ACID-compliant embedded DB, native only
//!    - **IndexedDBStore**: Browser storage, WASM only
//!
//! 4. **Traits**:
//!    - `NetabaseTreeSync`: Synchronous CRUD operations (native)
//!    - `NetabaseTreeAsync`: Asynchronous CRUD operations (WASM)
//!    - `NetabaseModelTrait`: Core model trait
//!    - `NetabaseDefinitionTrait`: Schema trait
//!
//! ## Secondary Key Queries
//!
//! Secondary keys enable efficient querying by non-primary fields:
//!
//! ```rust
//! use netabase_store::{netabase_definition_module, NetabaseModel, netabase};
//! use netabase_store::databases::sled_store::SledStore;
//! use netabase_store::traits::tree::NetabaseTreeSync;
//! use netabase_store::traits::model::NetabaseModelTrait;
//!
//! #[netabase_definition_module(BlogDefinition, BlogKeys)]
//! mod blog {
//!     use super::*;
//!     use netabase_store::{NetabaseModel, netabase};
//!     #[derive(NetabaseModel, Clone, Debug, PartialEq,
//!              bincode::Encode, bincode::Decode,
//!              serde::Serialize, serde::Deserialize)]
//!     #[netabase(BlogDefinition)]
//!     pub struct User {
//!         #[primary_key]
//!         pub id: u64,
//!         pub username: String,
//!         #[secondary_key]
//!         pub email: String,
//!     }
//! }
//! use blog::*;
//!
//! let store = SledStore::<BlogDefinition>::temp().unwrap();
//! let user_tree = store.open_tree::<User>();
//!
//! let user = User { id: 1, username: "alice".into(), email: "alice@example.com".into() };
//! user_tree.put(user.clone()).unwrap();
//!
//! // Query by email (secondary key) using convenience function
//! use blog::AsUserEmail;
//! let users = user_tree
//!     .get_by_secondary_key("alice@example.com".as_user_email_key())
//!     .unwrap();
//!
//! // Multiple users can have the same secondary key value
//! for user in users {
//!     println!("Found user: {}", user.username);
//! }
//! ```
//!
//! ## Iteration
//!
//! Iterate over all records in a tree:
//!
//! ```rust
//! use netabase_store::{netabase_definition_module, NetabaseModel, netabase};
//! use netabase_store::databases::sled_store::SledStore;
//!
//! #[netabase_definition_module(BlogDefinition, BlogKeys)]
//! mod blog {
//!     use super::*;
//!     use netabase_store::{NetabaseModel, netabase};
//!     #[derive(NetabaseModel, Clone, Debug, PartialEq,
//!              bincode::Encode, bincode::Decode,
//!              serde::Serialize, serde::Deserialize)]
//!     #[netabase(BlogDefinition)]
//!     pub struct User {
//!         #[primary_key]
//!         pub id: u64,
//!         pub username: String,
//!         #[secondary_key]
//!         pub email: String,
//!     }
//! }
//! use blog::*;
//!
//! let store = SledStore::<BlogDefinition>::temp().unwrap();
//! let user_tree = store.open_tree::<User>();
//!
//! // Iterate over all users
//! for result in user_tree.iter() {
//!     let (_key, user) = result.unwrap();
//!     println!("User: {} ({})", user.username, user.email);
//! }
//! ```
//!
//! ## Testing
//!
//! Use temporary databases for testing:
//!
//! ```rust
//! use netabase_store::{netabase_definition_module, NetabaseModel, netabase};
//! use netabase_store::databases::sled_store::SledStore;
//! use netabase_store::traits::tree::NetabaseTreeSync;
//!
//! #[netabase_definition_module(BlogDefinition, BlogKeys)]
//! mod blog {
//!     use super::*;
//!     use netabase_store::{NetabaseModel, netabase};
//!     #[derive(NetabaseModel, Clone, Debug, PartialEq,
//!              bincode::Encode, bincode::Decode,
//!              serde::Serialize, serde::Deserialize)]
//!     #[netabase(BlogDefinition)]
//!     pub struct User {
//!         #[primary_key]
//!         pub id: u64,
//!         pub username: String,
//!     }
//! }
//! use blog::*;
//!
//! // Create an database in tmp database for testing
//! let store = SledStore::<BlogDefinition>::temp().unwrap();
//! let user_tree = store.open_tree::<User>();
//!
//! // Perform test operations
//! let test_user = User { id: 1, username: "test".into() };
//! user_tree.put(test_user).unwrap();
//! // ... assertions
//! ```
//!
//! ## libp2p Integration
//!
//! When using the `libp2p` feature, stores can be used as record stores for Kademlia DHT. This was designed to to be implementned with the
//! [netabase](https://github.com/newsnet-africa/netabase.git) crate, which (eventually) should be a dht networking layer abstraction.
//! The stores in the [databases module](crate::databases) implement [RecordStore](https://docs.rs/libp2p/latest/libp2p/kad/store/trait.RecordStore.html), which allow for the [libp2p's kademlia implementation](https://docs.rs/libp2p/latest/libp2p/kad/index.html).
//!
//!
//! ## Feature Flags
//!
//! - `sled` - Enable Sled backend (default, native only)
//! - `redb` - Enable Redb backend (native only)
//! - `wasm` - Enable IndexedDB backend (WASM only)
//! - `libp2p` - Enable libp2p RecordStore integration
//!
//! ## Error Handling
//!
//! All operations return `Result<T, NetabaseError>`:
//!
//! ```rust
//! use netabase_store::{netabase_definition_module, NetabaseModel, netabase};
//! use netabase_store::databases::sled_store::SledStore;
//! use netabase_store::traits::tree::NetabaseTreeSync;
//! use netabase_store::traits::model::NetabaseModelTrait;
//!
//! #[netabase_definition_module(BlogDefinition, BlogKeys)]
//! mod blog {
//!     use super::*;
//!     use netabase_store::{NetabaseModel, netabase};
//!     #[derive(NetabaseModel, Clone, Debug, PartialEq,
//!              bincode::Encode, bincode::Decode,
//!              serde::Serialize, serde::Deserialize)]
//!     #[netabase(BlogDefinition)]
//!     pub struct User {
//!         #[primary_key]
//!         pub id: u64,
//!         pub username: String,
//!     }
//! }
//! use blog::*;
//!
//! let store = SledStore::<BlogDefinition>::temp().unwrap();
//! let user_tree = store.open_tree::<User>();
//!
//! let user = User { id: 1, username: "alice".into() };
//! match user_tree.get(user.primary_key()) {
//!     Ok(Some(user)) => println!("Found: {}", user.username),
//!     Ok(None) => println!("User not found"),
//!     Err(e) => eprintln!("Database error: {}", e),
//! }
//! ```

pub mod config;
pub mod databases;
pub mod error;
// NOTE: Phase 4 - guards module re-enabled with proper architecture
// Guard-based API now works with proper lifetime management
#[cfg(all(feature = "redb", feature = "redb-zerocopy"))]
pub mod guards;
pub mod store;
pub mod subscription;
pub mod traits;
pub mod transaction;
pub mod utils;

// Re-export netabase_deps for users of the macros
pub use netabase_deps;
pub use netabase_deps::*;

// Re-export macros for convenience
pub use netabase_macros::*;
pub use store::NetabaseStore;
pub use subscription::subscription_tree::DefaultSubscriptionManager;
pub use traits::subscription::subscription_tree::ModelHash;
pub use traits::*;
pub use transaction::{ReadOnly, ReadWrite, TreeView, TxnGuard};
pub use utils::{NetabaseDateTime, chrono};

// Re-export zero-copy redb types for convenience
#[cfg(all(feature = "redb", feature = "redb-zerocopy"))]
pub use databases::redb_zerocopy::{
    RedbReadTransactionZC, RedbStoreZeroCopy, RedbTree, RedbTreeMut, RedbWriteTransactionZC,
};
#[cfg(all(feature = "redb", feature = "redb-zerocopy"))]
pub use guards::{BorrowedGuard, BorrowedIter};

// Conditional Send + Sync bounds for WASM compatibility
#[cfg(not(target_arch = "wasm32"))]
pub trait MaybeSend: Send {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Send> MaybeSend for T {}

#[cfg(target_arch = "wasm32")]
pub trait MaybeSend {}
#[cfg(target_arch = "wasm32")]
impl<T> MaybeSend for T {}

#[cfg(not(target_arch = "wasm32"))]
pub trait MaybeSync: Sync {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Sync> MaybeSync for T {}

#[cfg(target_arch = "wasm32")]
pub trait MaybeSync {}
#[cfg(target_arch = "wasm32")]
impl<T> MaybeSync for T {}

// Helper trait to bundle all discriminant requirements
pub trait DiscriminantBounds:
    AsRef<str>
    + Clone
    + Copy
    + std::fmt::Debug
    + std::fmt::Display
    + PartialEq
    + Eq
    + std::hash::Hash
    + strum::IntoEnumIterator
    + MaybeSend
    + MaybeSync
    + 'static
    + std::str::FromStr
{
}

// Blanket implementation
impl<T> DiscriminantBounds for T where
    T: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + MaybeSend
        + MaybeSync
        + 'static
        + std::str::FromStr
{
}
