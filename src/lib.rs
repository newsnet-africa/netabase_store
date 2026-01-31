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
//! use netabase_store::traits::registery::repository::Standalone;
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

// Allow the crate to reference itself, needed for macros that generate
// code referencing `netabase_store::` paths
extern crate self as netabase_store;

#[cfg(feature = "libp2p")]
pub use libp2p;
pub use postcard;

pub mod blob;
pub mod databases;
pub mod doc_examples;
// Re-export for compatibility with docs using singular form
pub use doc_examples as doc_example;
pub mod errors;
pub mod prelude;
pub mod query;
pub mod relational;
pub mod subscription_hash;
pub mod traits;
pub mod utils;

pub mod capabilities;
pub mod key;
pub mod node_metadata;
pub mod primitives;