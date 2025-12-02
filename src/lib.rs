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
//! - **Relational Links**: Type-safe relationships between models with automatic insertion and hydration
//! - **Subscription System**: Merkle tree-based change tracking for efficient synchronization
//! - **Database Introspection**: Query all internal trees, indexes, and database statistics
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
//! # use netabase_store::{NetabaseStore, netabase_definition_module, NetabaseModel, netabase};
//! # use netabase_store::traits::tree::NetabaseTreeSync;
//! # use netabase_store::traits::model::NetabaseModelTrait;
//!
//! # // Define your schema
//! # #[netabase_definition_module(BlogDefinition, BlogKeys)]
//! # mod blog {
//! #     use netabase_store::{NetabaseModel, netabase};
//!
//! #     #[derive(NetabaseModel, Clone, Debug, PartialEq,
//! #              bincode::Encode, bincode::Decode,
//! #              serde::Serialize, serde::Deserialize)]
//! #     #[netabase(BlogDefinition)]
//! #     pub struct User {
//! #         #[primary_key]
//! #         pub id: u64,
//! #         pub username: String,
//! #         #[secondary_key]
//! #         pub email: String,
//! #         pub age: u32,
//! #     }
//! # }
//! # use blog::*;
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
//! ### Backend-Specific Usage
//!
//! All backends share the same API through the `NetabaseStore` wrapper (recommended) or can be used directly.
//! Each backend has different performance characteristics:
//!
//! - **[Sled](databases::sled_store)**: Excellent for read-heavy workloads (~20% overhead)
//! - **[Redb](databases::redb_store)**: Best for write-heavy workloads with ACID guarantees
//! - **[Redb ZeroCopy](databases::redb_zerocopy)**: Maximum performance with explicit transaction control (10-54x faster)
//! - **[IndexedDB](databases::indexeddb_store)**: Browser-native storage for WASM applications
//!
//! **For detailed backend documentation, examples, and performance comparisons, see the [`databases`](databases) module.**
//!
//! ### IndexedDB for Web Applications
//!
//! IndexedDB backend provides persistent storage in web browsers with an async API.
//! **For complete WASM documentation and examples, see the [`databases::indexeddb_store`](databases::indexeddb_store) module.**
//!
//! ## Using the Configuration
//!
//! ```no_run
//! # use netabase_store::traits::backend_store::BackendStore;
//! # use netabase_store::config::FileConfig;
//! # use netabase_store::databases::sled_store::SledStore;
//! # use netabase_store::netabase_definition_module;
//! # #[netabase_definition_module(MyDef, MyKeys)]
//! # mod models {
//! #     use netabase_store::{NetabaseModel, netabase};
//! #     #[derive(NetabaseModel, Clone, Debug, PartialEq,
//! #              bincode::Encode, bincode::Decode,
//! #              serde::Serialize, serde::Deserialize)]
//! #     #[netabase(MyDef)]
//! #     pub struct User {
//! #         #[primary_key]
//! #         pub id: u64,
//! #         pub name: String,
//! #     }
//! # }
//! # use models::*;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use netabase_store::traits::backend_store::BackendStore;
//! // Unified API across all backends via BackendStore trait
//! let temp_dir = tempfile::tempdir()?;
//! let config = FileConfig::new(temp_dir.path().join("my_store.db"));
//! let store = <SledStore<MyDef> as BackendStore<MyDef>>::new(config.clone())?;
//!
//! // Or open existing
//! let store = <SledStore<MyDef> as BackendStore<MyDef>>::open(config)?;
//!
//! // Temporary store
//! let temp_store = <SledStore<MyDef> as BackendStore<MyDef>>::temp()?;
//! # Ok(())
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
//!     #[derive(NetabaseModel, Clone, Debug, PartialEq,
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
//! ## High-Performance Operations
//!
//! For maximum performance, netabase_store provides specialized APIs:
//!
//! ### Redb Zero-Copy API
//!
//! The zero-copy API provides explicit transaction control and can be **10-54x faster** than the regular API:
//! - **10x faster** bulk inserts (single transaction for 1000 items)
//! - **54x faster** secondary key queries
//! - Direct transaction management for fine-grained control
//!
//! Enable with features: `["redb", "redb-zerocopy"]`
//!
//! **For complete documentation, examples, and usage patterns, see the [`databases::redb_zerocopy`](databases::redb_zerocopy) module.**
//!
//! ## Advanced Features
//!
//! ### Relational Links
//!
//! Create type-safe relationships between models using `RelationalLink<D, M>` which supports both
//! eager loading (embedded entities) and lazy loading (references):
//!
//! - Define relations with `#[relation(name)]` attribute
//! - Automatically generated helper methods for each relation
//! - Support for Entity and Reference variants
//! - Automatic cascading insertion with `insert_with_relations()`
//! - Hydration methods to load referenced entities
//!
//! **For complete documentation, examples, and usage patterns, see:**
//! - The [`links`](links) module for API documentation
//! - `RELATIONAL_LINKS.md` for comprehensive guide with examples
//! - `examples/relational_links_showcase.rs` for working code
//!
//! ### Subscription System
//!
//! Track changes and synchronize data efficiently using Merkle tree-based subscriptions:
//!
//! - Define subscription topics with `#[streams(...)]` attribute
//! - Merkle root computation for efficient change detection
//! - Topic-based organization for selective synchronization
//! - Diff computation to find differences between managers
//!
//! **Ideal for:** P2P synchronization, change tracking, audit logs, distributed systems
//!
//! **For complete documentation and examples, see:**
//! - The [`subscription`](subscription) module for API documentation
//! - `examples/subscription_streams.rs` for working examples
//!
//! ### Database Introspection
//!
//! Inspect all internal database trees, indexes, and statistics:
//!
//! ```rust
//! # use netabase_store::{netabase_definition_module, NetabaseModel, netabase};
//! # use netabase_store::databases::sled_store::SledStore;
//! # use netabase_store::traits::introspection::DatabaseIntrospection;
//! # #[netabase_definition_module(TestDef, TestKeys)]
//! # mod models {
//! #     use netabase_store::{NetabaseModel, netabase};
//! #     #[derive(NetabaseModel, Clone, Debug, PartialEq,
//! #              bincode::Encode, bincode::Decode,
//! #              serde::Serialize, serde::Deserialize)]
//! #     #[netabase(TestDef)]
//! #     pub struct User {
//! #         #[primary_key]
//! #         pub id: u64,
//! #         pub name: String,
//! #         #[secondary_key]
//! #         pub email: String,
//! #     }
//! # }
//! # use models::*;
//! # fn main() -> Result<(), netabase_store::error::NetabaseError> {
//! let store = SledStore::<TestDef>::temp()?;
//!
//! // List all trees (models, indexes, system)
//! for tree in store.list_all_trees()? {
//!     println!("{}: {} entries ({:?})",
//!         tree.name,
//!         tree.entry_count.unwrap_or(0),
//!         tree.tree_type
//!     );
//! }
//!
//! // Get aggregate statistics
//! let stats = store.database_stats()?;
//! println!("Total trees: {}", stats.total_trees);
//! println!("Model trees: {}", stats.model_trees);
//! println!("Secondary indexes: {}", stats.secondary_trees);
//! println!("Total entries: {}", stats.total_entries);
//!
//! // Check specific tree
//! let user_count = store.tree_entry_count("User")?;
//! println!("User tree has {} entries", user_count);
//! # Ok(())
//! # }
//! ```
//!
//! **Available methods:**
//! - `list_all_trees()` - All trees in the database
//! - `list_model_trees()` - User-defined model trees only
//! - `list_secondary_trees()` - Secondary index trees only
//! - `tree_entry_count(name)` - Count entries in a specific tree
//! - `tree_keys_raw(name)` - Get all keys as raw bytes
//! - `database_stats()` - Aggregate statistics
//!
//! Introspection is supported on all backends (Sled, Redb, RedbZeroCopy).
//! See `INTROSPECTION_API.md` for complete documentation.
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
pub mod links;
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
pub use store::{NetabaseStore, TypedTree};
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
