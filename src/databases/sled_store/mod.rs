//! Type-safe wrapper around the [sled](https://docs.rs/sled) embedded database.
//!
//! This module provides a high-performance, type-safe interface to the underlying sled database,
//! using model discriminants as tree names and ensuring all operations are compile-time checked.
//!
//! # Module Organization
//!
//! The implementation is organized into submodules for better maintainability:
//!
//! ## Module Structure
//!
//! - `types.rs` - Shared types and enums (SecondaryKeyOp, etc.)
//! - `transaction.rs` - Transaction support (SledTransactionalTree)
//! - `tree.rs` - Tree implementation (SledStoreTree)
//! - `iterator.rs` - Iterator implementation (SledIter)
//! - `batch.rs` - Batch operations (SledBatchBuilder)
//! - `store.rs` - Main store implementation (SledStore)
//! - `trait_impls.rs` - Trait implementations (NetabaseTreeSync, StoreOps, etc.)
//!
//! # Examples
//!
//! ```rust
//! use netabase_store::{netabase_definition_module, NetabaseModel, netabase};
//! use netabase_store::databases::sled_store::SledStore;
//! use netabase_store::traits::tree::NetabaseTreeSync;
//! use netabase_store::traits::model::NetabaseModelTrait;
//!
//! // Define your schema
//! #[netabase_definition_module(BlogDefinition, BlogKeys)]
//! mod blog {
//!     use super::*;
//!     use netabase_store::{netabase_definition_module, NetabaseModel, netabase};
//!     #[derive(NetabaseModel, Clone, Debug, PartialEq,
//!              bincode::Encode, bincode::Decode,
//!              serde::Serialize, serde::Deserialize)]
//!     #[netabase(BlogDefinition)]
//!     pub struct User {
//!         #[primary_key]
//!         pub id: u64,
//!         pub name: String,
//!         #[secondary_key]
//!         pub email: String,
//!     }
//! }
//!
//! use blog::*;
//!
//! // Create a temporary database for testing
//! let store = SledStore::<BlogDefinition>::temp().unwrap();
//!
//! // Open a type-safe tree for the User model
//! let user_tree = store.open_tree::<User>();
//!
//! // Insert a user
//! let alice = User {
//!     id: 1,
//!     name: "Alice".to_string(),
//!     email: "alice@example.com".to_string(),
//! };
//! user_tree.put(alice.clone()).unwrap();
//!
//! // Retrieve by primary key
//! let retrieved = user_tree.get(alice.primary_key()).unwrap();
//! assert_eq!(retrieved, Some(alice));
//! ```

pub mod batch;
pub mod iterator;
pub mod store;
pub mod trait_impls;
pub mod transaction;
pub mod tree;
pub mod types;

// Re-export the main types for external use
pub use batch::SledBatchBuilder;
pub use iterator::SledIter;
pub use store::SledStore;
pub use transaction::SledTransactionalTree;
pub use tree::SledStoreTree;
pub use types::SecondaryKeyOp;
