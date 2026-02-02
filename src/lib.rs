//! # Netabase Store
//!
//! A type-safe, high-performance embedded database library for Rust with
//! automatic model migration and compile-time schema validation.
//!
//! ## Features
//!
//! - **Type-Safe**: Compile-time schema validation with Rust's type system
//! - **High Performance**: Zero-copy operations with postcard serialization
//! - **Auto Migration**: Automatic schema versioning and data migration
//! - **Transactions**: ACID-compliant read/write transactions
//! - **Secondary Indexes**: Fast lookups on non-primary fields
//! - **Relational Links**: Support for relationships between models
//!
//! ## Quick Start
//!
//! ```rust
//! use netabase_store::prelude::*;
//! use netabase_store::traits::database::store::NBStore;
//! use serde::{Serialize, Deserialize};
//!
//! // 1. Define your definition with models inside it
//! #[netabase_macros::netabase_definition(MyApp)]
//! mod my_models {
//!     use super::*;
//!
//!     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
//!     pub struct User {
//!         #[primary_key]
//!         pub id: String,
//!         pub name: String,
//!         #[secondary_key]
//!         pub email: String,
//!     }
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use my_models::*;
//!
//! // 2. Create an in-memory database for testing
//! let (store, _temp) = RedbStore::<MyApp>::new_temporary()?;
//!
//! // 3. Write data in a transaction
//! let txn = store.begin_write()?;
//! txn.create(&User {
//!     id: UserID("alice".into()),
//!     name: "Alice".into(),
//!     email: "alice@example.com".into(),
//! })?;
//! txn.commit()?;
//!
//! // 4. Read data back
//! let txn = store.begin_read()?;
//! let user: Option<User> = txn.read(&UserID("alice".into()))?;
//! assert_eq!(user.unwrap().name, "Alice");
//! # Ok(())
//! # }
//! ```
//!
//! ## Advanced Features
//!
//! ### Secondary Index Queries
//!
//! ```rust
//! use netabase_store::prelude::*;
//! use netabase_store::traits::database::store::NBStore;
//! use netabase_store::databases::redb::transaction::RedbModelCrud;
//! use serde::{Serialize, Deserialize};
//!
//! #[netabase_macros::netabase_definition(Shop)]
//! mod shop_models {
//!     use super::*;
//!
//!     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
//!     pub struct Product {
//!         #[primary_key]
//!         pub sku: String,
//!         pub name: String,
//!         #[secondary_key]
//!         pub category: String,
//!         pub price: u64,
//!     }
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use shop_models::*;
//!
//! let (store, _temp) = RedbStore::<Shop>::new_temporary()?;
//!
//! // Create some products
//! let txn = store.begin_write()?;
//! txn.create(&Product {
//!     sku: ProductID("001".into()),
//!     name: "Laptop".into(),
//!     category: "Electronics".into(),
//!     price: 999,
//! })?;
//! txn.commit()?;
//!
//! // Read back by primary key
//! let txn = store.begin_read()?;
//! let product: Option<Product> = txn.read(&ProductID("001".into()))?;
//! assert_eq!(product.unwrap().name, "Laptop");
//! # Ok(())
//! # }
//! ```
//!
//! ### Relational Links
//!
//! ```rust
//! use netabase_store::doc_example::*;
//! use netabase_store::databases::redb::RedbStore;
//! use netabase_store::databases::redb::transaction::RedbModelCrud;
//! use netabase_store::relational::RelationalLink;
//! use netabase_store::traits::registry::repository::Standalone;
//!
//! let store = RedbStore::<ExampleDef>::new_in_memory().unwrap();
//!
//! // Create related models
//! let txn = store.begin_write().unwrap();
//! txn.create(&Author {
//!     id: AuthorID("author1".into()),
//!     name: "Jane Doe".into(),
//!     genre: "Fiction".into(),
//! }).unwrap();
//! txn.create(&Book {
//!     isbn: BookID("978-3-16".into()),
//!     title: "Rust Guide".into(),
//!     genre: "Technology".into(),
//!     author: RelationalLink::new_dehydrated(AuthorID("author1".into())),
//! }).unwrap();
//! txn.commit().unwrap();
//!
//! // Read the book and access its author link
//! let txn = store.begin_read().unwrap();
//! let book: Book = txn.read(&BookID("978-3-16".into())).unwrap().unwrap();
//! // The author field is a RelationalLink that can be resolved
//! let author: Option<Author> = txn.read(&AuthorID("author1".into())).unwrap();
//! assert_eq!(author.unwrap().name, "Jane Doe");
//! ```
//!
//! ### Model Versioning and Migration
//!
//! For models that evolve over time, define version families and migration paths:
//!
//! ```rust,ignore
//! use netabase_store::prelude::*;
//! use netabase_store::traits::database::store::NBStore;
//! use serde::{Serialize, Deserialize};
//!
//! #[netabase_macros::netabase_definition(CRM)]
//! mod crm_models {
//!     use super::*;
//!
//!     // Old version of your model
//!     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
//!     #[netabase_version(family = "Customer", version = 1)]
//!     pub struct CustomerV1 {
//!         #[primary_key]
//!         pub id: String,
//!         pub name: String,
//!     }
//!
//!     // New version with additional field
//!     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
//!     #[netabase_version(family = "Customer", version = 2)]
//!     pub struct Customer {
//!         #[primary_key]
//!         pub id: String,
//!         pub name: String,
//!         pub email: String,  // New field!
//!     }
//!
//!     // Define how to migrate from V1 to V2
//!     impl MigrateFrom<CustomerV1> for Customer {
//!         fn migrate_from(old: CustomerV1) -> Self {
//!             Customer {
//!                 id: old.id,
//!                 name: old.name,
//!                 email: String::new(),  // Default for new field
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! See the [`traits::migration`](crate::traits::migration) module for details.

#![feature(generic_const_items)]
#![allow(incomplete_features)]

//! # Netabase Store
//!
//! A type-safe, high-performance embedded database library for Rust with
//! automatic model migration and compile-time schema validation.
//!
//! ## Features
//!
//! - **Type-Safe**: Compile-time schema validation with Rust's type system
//! - **High Performance**: Zero-copy operations with postcard serialization
//! - **Auto Migration**: Automatic schema versioning and data migration
//! - **Transactions**: ACID-compliant read/write transactions
//! - **Secondary Indexes**: Fast lookups on non-primary fields
//! - **Relational Links**: Support for relationships between models
//!
//! ## Quick Start
//!
//! ```rust
//! use netabase_store::prelude::*;
//! use netabase_store::traits::database::store::NBStore;
//! use serde::{Serialize, Deserialize};
//!
//! // 1. Define your definition with models inside it
//! #[netabase_macros::netabase_definition(MyApp)]
//! mod my_models {
//!     use super::*;
//!
//!     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
//!     pub struct User {
//!         #[primary_key]
//!         pub id: String,
//!         pub name: String,
//!         #[secondary_key]
//!         pub email: String,
//!     }
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use my_models::*;
//!
//! // 2. Create an in-memory database for testing
//! let (store, _temp) = RedbStore::<MyApp>::new_temporary()?;
//!
//! // 3. Write data in a transaction
//! let txn = store.begin_write()?;
//! txn.create(&User {
//!     id: UserID("alice".into()),
//!     name: "Alice".into(),
//!     email: "alice@example.com".into(),
//! })?;
//! txn.commit()?;
//!
//! // 4. Read data back
//! let txn = store.begin_read()?;
//! let user: Option<User> = txn.read(&UserID("alice".into()))?;
//! assert_eq!(user.unwrap().name, "Alice");
//! # Ok(())
//! # }
//! ```
//!
//! ## Cargo Features
//!
//! All features are enabled by default. Disable default features and enable only
//! what you need for smaller binaries:
//!
//! - `secondary_keys` - Secondary key indexes for fast lookups
//! - `relational_keys` - Type-safe relational links between models
//! - `blobs` - Large binary data storage with automatic chunking
//! - `subscriptions` - Topic-based pub/sub with Merkle tree synchronization
//! - `migration` - Schema versioning and data migration
//! - `repository` - Repository pattern for access control
//! - `libp2p` - P2P networking integration
//!
//! ## Module Organization
//!
//! ### Core Modules
//! - [`core`] - Fundamental types (keys, primitives)
//! - [`schema`] - Schema types (blobs, relations, subscriptions)
//! - [`traits`] - Core traits for models, definitions, and repositories
//! - [`errors`] - Error types and result aliases
//!
//! ### Database Layer
//! - [`databases`] - Backend implementations (redb, indexeddb, memory)
//! - [`query`] - Query configuration and execution
//!
//! ### Convenience
//! - [`prelude`] - Convenient re-exports for quick imports
//! - [`tutorial`] - Comprehensive usage examples and guides
//! - [`doc_examples`] - Pre-built models for documentation

// Allow the crate to reference itself, needed for macros that generate
// code referencing `netabase_store::` paths
extern crate self as netabase_store;

// ============================================================================
// External Crate Re-exports
// ============================================================================

/// Re-export libp2p for use in networking features.
#[cfg(feature = "libp2p")]
pub use libp2p;

/// Re-export postcard for serialization.
pub use postcard;

/// Re-export the netabase_macros crate for convenience.
///
/// This allows users to import macros directly from `netabase_store`:
///
/// ```rust
/// use netabase_store::macros::{NetabaseModel, netabase_definition};
/// ```
///
/// Instead of requiring a separate import:
///
/// ```rust,ignore
/// use netabase_macros::{NetabaseModel, netabase_definition};
/// ```
pub use netabase_macros as macros;

// ============================================================================
// Core Modules
// ============================================================================

/// Core types and primitives.
///
/// Contains fundamental types: keys and primitives for storage operations.
pub mod core;

/// Schema-related types for advanced features.
///
/// Contains blob storage, relational links, and subscription hashing.
pub mod schema;

/// Error types and result aliases.
pub mod errors;

/// Query configuration and execution.
pub mod query;

/// Database backend implementations.
pub mod databases;

/// Core traits for models, definitions, and repositories.
pub mod traits;

/// Internal utility functions.
pub mod utils;

/// Node metadata for distributed systems.
pub mod node_metadata;

// ============================================================================
// Compatibility Re-exports (maintaining public API)
// ============================================================================

/// Re-export key types for backwards compatibility.
pub use core::key;

/// Re-export blob types for backwards compatibility.
#[cfg(feature = "blobs")]
pub use schema::blob;

/// Re-export relational types for backwards compatibility.
#[cfg(feature = "relational_keys")]
pub use schema::relational;

/// Re-export subscription types for backwards compatibility.
#[cfg(feature = "subscriptions")]
pub use schema::subscription_hash;

// ============================================================================
// Documentation Examples
// ============================================================================

/// Pre-built example models for documentation and testing.
/// Documentation examples module.
///
/// This module provides `ExampleDef` with `User`, `Product`, `Author`, and `Book` models
/// that are used throughout the documentation examples.
pub mod doc_examples;

/// Re-export for compatibility with docs using singular form.
pub use doc_examples as doc_example;

// ============================================================================
// Tutorial and Examples
// ============================================================================

/// Comprehensive tutorial and usage examples.
///
/// This module contains complete, runnable examples demonstrating all features
/// of Netabase from basic CRUD to advanced patterns.
///
/// Start here if you're new to Netabase!
pub mod tutorial;

/// Examples of code generated by the Netabase macros.
///
/// This module shows conceptual examples of what the `#[derive(NetabaseModel)]`
/// and `#[netabase_definition]` macros generate, helping you understand the
/// internals and debug issues.
pub mod macro_generated_examples;

// ============================================================================
// Prelude
// ============================================================================

/// Convenient re-exports for common usage patterns.
///
/// Import with `use netabase_store::prelude::*;` to get started quickly.
pub mod prelude;