//! Type-safe wrapper around the [redb](https://docs.rs/redb) embedded database.
//!
//! This module provides a high-performance, type-safe interface to the underlying redb database,
//! using model discriminants as table names and ensuring all operations are compile-time checked.
//!
//! # Module Organization
//!
//! The implementation is organized into submodules for better maintainability:
//!
//! ## Module Structure
//!
//! - `types.rs` - Shared types (BincodeWrapper, CompositeKey)
//! - `store.rs` - Main store implementation (RedbStore)
//! - `tree.rs` - Tree implementation (RedbStoreTree)
//! - `iterator.rs` - Iterator implementation (RedbIter)
//! - `batch.rs` - Batch operations (RedbBatchBuilder)
//! - `trait_impls.rs` - Trait implementations (NetabaseTreeSync, StoreOps, etc.)
//!
//! # Examples
//!
//! ```rust
//! use netabase_store::{netabase_definition_module, NetabaseModel, netabase};
//! use netabase_store::databases::redb_store::RedbStore;
//! use netabase_store::traits::tree::NetabaseTreeSync;
//! use netabase_store::traits::model::NetabaseModelTrait;
//!
//! // Define your schema
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
//!         pub name: String,
//!         #[secondary_key]
//!         pub email: String,
//!     }
//! }
//!
//! use blog::*;
//!
//! // Create a temporary database for testing
//! let store = RedbStore::<BlogDefinition>::new("./test.redb").unwrap();
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
//!
//! # Performance Characteristics
//!
//! - **Excellent for read-heavy workloads**: Zero-copy reads where possible
//! - **ACID transactions**: Full ACID compliance with proper isolation
//! - **Type safety**: Compile-time checked operations with redb's type system
//! - **Efficient storage**: Optimized binary format with compression
//! - **Bulk operations**: Efficient batch inserts and queries
//!
//! # Comparison with Sled
//!
//! | Feature | Redb | Sled |
//! |---------|------|------|
//! | ACID | ✅ Full | ✅ Eventual |
//! | Type Safety | ✅ Native | ⚠️ Wrapper |
//! | Zero-Copy | ✅ Yes | ❌ No |
//! | Write Performance | ✅ Excellent | ✅ Good |
//! | Read Performance | ✅ Excellent | ✅ Very Good |
//! | Memory Usage | ✅ Lower | ⚠️ Higher |
//! | Maturity | ⚠️ Newer | ✅ Battle-tested |

pub mod batch;
pub mod iterator;
pub mod store;
pub mod trait_impls;
pub mod tree;
pub mod types;

// Re-export the main types for external use
pub use batch::RedbBatchBuilder;
pub use iterator::{RedbIter, RedbSubscriptionTreeIter};
pub use store::RedbStore;
pub use tree::RedbStoreTree;
pub use types::{BincodeWrapper, CompositeKey};
