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
//! use netabase_store::doc_example::*;
//! use netabase_store::databases::redb::RedbStore;
//! use netabase_store::traits::database::store::NBStore;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create an in-memory database for testing
//! let (store, _temp) = RedbStore::<ExampleDef>::new_temporary()?;
//!
//! // Write data in a transaction
//! let txn = store.begin_write()?;
//! txn.create(&User {
//!     id: UserID("alice".into()),
//!     name: "Alice".into(),
//!     email: "alice@example.com".into(),
//! })?;
//! txn.commit()?;
//!
//! // Read data back
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
//! use netabase_store::doc_example::*;
//! use netabase_store::databases::redb::RedbStore;
//! use netabase_store::traits::database::store::NBStore;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let (store, _temp) = RedbStore::<ExampleDef>::new_temporary()?;
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
//! use netabase_store::relational::RelationalLink;
//! use netabase_store::traits::database::store::NBStore;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let (store, _temp) = RedbStore::<ExampleDef>::new_temporary()?;
//!
//! // Create related models
//! let txn = store.begin_write()?;
//! txn.create(&Author {
//!     id: AuthorID("author1".into()),
//!     name: "Jane Doe".into(),
//!     genre: "Fiction".into(),
//! })?;
//! txn.create(&Book {
//!     isbn: BookID("978-3-16".into()),
//!     title: "Rust Guide".into(),
//!     genre: "Technology".into(),
//!     author: RelationalLink::new_dehydrated(AuthorID("author1".into())),
//! })?;
//! txn.commit()?;
//!
//! // Read the book and access its author link
//! let txn = store.begin_read()?;
//! let book: Book = txn.read(&BookID("978-3-16".into()))?.unwrap();
//! // The author field is a RelationalLink that can be resolved
//! let author: Option<Author> = txn.read(&AuthorID("author1".into()))?;
//! assert_eq!(author.unwrap().name, "Jane Doe");
//! # Ok(())
//! # }
//! ```
//!
//! ### Model Versioning and Migration
//!
//! For models that evolve over time, define version families and migration paths:
//!
//! ```rust
//! use netabase_store::prelude::*;
//! use netabase_store::traits::database::store::NBStore;
//! use netabase_store::databases::redb::RedbStore;
//! use netabase_store::traits::migration::MigrateFrom;
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
//!     #[netabase_version(family = "Customer", version = 2, current)]
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
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use crm_models::*;
//!
//! let (store, _temp) = RedbStore::<CRM>::new_temporary()?;
//!
//! // Old data is automatically migrated when read
//! let txn = store.begin_write()?;
//! let customer = Customer {
//!     id: CustomerID("cust123".into()),
//!     name: "John Doe".into(),
//!     email: "john@example.com".into(),
//! };
//! txn.create(&customer)?;
//! txn.commit()?;
//!
//! let txn = store.begin_read()?;
//! let retrieved: Option<Customer> = txn.read(&CustomerID("cust123".into()))?;
//! assert_eq!(retrieved.unwrap().email, "john@example.com");
//! # Ok(())
//! # }
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
//! - [`doc_example`] - Pre-built models for documentation
extern crate self as netabase_store;
/// Re-export libp2p for use in networking features.
#[cfg(feature = "libp2p")]
pub use libp2p;
pub use strum;
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
/// ```rust
/// use netabase_macros::{NetabaseModel, netabase_definition};
/// ```
pub use netabase_macros as macros;
/// Re-export core macros at the crate root for ergonomics.
///
/// This enables:
///
/// ```rust
/// use netabase_store::{NetabaseModel, netabase_definition, netabase_repository, NetabaseBlobItem};
/// ```
pub use netabase_macros::{
    infer_netabase_definition, netabase_definition, netabase_repository,
    NetabaseBlobItem, NetabaseModel,
};
/// Core types and primitives.
///
/// Schema-related types for advanced features.
///
/// Contains blob storage, relational links, and subscription hashing.
pub mod schema;
/// Unified configuration system for queries and CRUD operations.
pub mod config;
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
/// CLI client generation functionality.
pub mod cli_generation;
/// Re-export blob types for backwards compatibility.
#[cfg(feature = "blobs")]
pub use schema::blob;
/// Re-export relational types for backwards compatibility.
#[cfg(feature = "relational_keys")]
pub use schema::relational;
/// Re-export subscription types for backwards compatibility.
#[cfg(feature = "subscriptions")]
pub use schema::subscription_hash;
/// Pre-built example models for documentation and testing.
///
/// This module provides `ExampleDef` with `User`, `Product`, `Author`, and `Book` models
/// that are used throughout the documentation examples.
///
/// These models demonstrate common patterns like primary/secondary keys,
/// content-addressed storage, relational links, and versioning.
pub mod doc_example;
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
/// Convenient re-exports for common usage patterns.
///
/// Import with `use netabase_store::prelude::*;` to get started quickly.
pub mod prelude;
