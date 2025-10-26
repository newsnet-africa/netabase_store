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
//! ### Using Sled Backend (Native)
//!
//! ```rust
//! use netabase_store::databases::sled_store::SledStore;
//! use netabase_store::traits::tree::NetabaseTreeSync;
//!
//! // Open a database
//! let store = SledStore::<BlogDefinition>::new("./my_database").unwrap();
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
//! let retrieved = user_tree.get(UserPrimaryKey(1)).unwrap().unwrap();
//! assert_eq!(retrieved, alice);
//!
//! // Query by secondary key (email)
//! let users_by_email = user_tree
//!     .get_by_secondary_key(UserSecondaryKeys::EmailKey(
//!         "alice@example.com".to_string()
//!     ))
//!     .unwrap();
//! assert_eq!(users_by_email.len(), 1);
//!
//! // Update the user
//! let mut alice_updated = alice.clone();
//! alice_updated.age = 31;
//! user_tree.put(alice_updated).unwrap();
//!
//! // Remove the user
//! user_tree.remove(UserPrimaryKey(1)).unwrap();
//! ```
//!
//! ### Using Redb Backend (Native)
//!
//! ```rust
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
//!
//! // Open a database with Redb backend
//! let store = RedbStore::<BlogDefinition>::new("./my_redb_database").unwrap();
//!
//! // API is identical to SledStore
//! let user_tree = store.open_tree::<User>();
//! // ... same operations as Sled
//! ```
//!
//! ### Using IndexedDB Backend (WASM)
//!
//! ```rust
//! use netabase_store::databases::indexeddb_store::IndexedDBStore;
//! use netabase_store::traits::tree::NetabaseTreeAsync;
//!
//! // Open a database in the browser
//! let store = IndexedDBStore::<BlogDefinition>::new("my_app_db").await.unwrap();
//!
//! // Get an async tree (WASM uses async API)
//! let user_tree = store.open_tree::<User>().await.unwrap();
//!
//! // Async operations
//! user_tree.put(alice).await.unwrap();
//! let retrieved = user_tree.get(UserPrimaryKey(1)).await.unwrap().unwrap();
//! ```
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
//! // Query by email (secondary key)
//! let users = user_tree
//!     .get_by_secondary_key(UserSecondaryKeys::EmailKey(
//!         "alice@example.com".to_string()
//!     ))
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
//! // Iterate over all users
//! for result in user_tree.iter() {
//!     let user = result.unwrap();
//!     println!("User: {} ({})", user.username, user.email);
//! }
//! ```
//!
//! ## Testing
//!
//! Use temporary databases for testing:
//!
//! ```rust
//! // Create an in-memory database for testing
//! let store = SledStore::<BlogDefinition>::temp().unwrap();
//! let user_tree = store.open_tree::<User>();
//!
//! // Perform test operations
//! user_tree.put(test_user).unwrap();
//! // ... assertions
//! ```
//!
//! ## libp2p Integration
//!
//! When using the `libp2p` feature, stores can be used as record stores for Kademlia DHT:
//!
//! ```rust
//! use libp2p::kad::{Behaviour, RecordStore};
//!
//! let store = SledStore::<MyDefinition>::new("./dht_store").unwrap();
//! let kad_behaviour = Behaviour::new(peer_id, store);
//! ```
//!
//! ## Performance Considerations
//!
//! - **Primary Key Access**: O(log n) - Very fast
//! - **Secondary Key Queries**: O(m) where m is matching records
//! - **Iteration**: O(n) - Scans all records
//! - **Batch Operations**: Use transactions for multiple writes (when available)
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
//! match user_tree.get(UserPrimaryKey(1)) {
//!     Ok(Some(user)) => println!("Found: {}", user.username),
//!     Ok(None) => println!("User not found"),
//!     Err(e) => eprintln!("Database error: {}", e),
//! }
//! ```

pub mod databases;
pub mod error;
pub mod traits;

// Re-export netabase_deps for users of the macros
pub use netabase_deps;
pub use netabase_deps::*;

// Re-export macros for convenience
pub use netabase_macros::*;
pub use traits::*;

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
