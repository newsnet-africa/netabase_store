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
//! // 1. Define your models with the netabase_model macro
//! #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq)]
//! struct User {
//!     #[primary_key]
//!     id: String,
//!     name: String,
//!     #[secondary_key]
//!     email: String,
//! }
//!
//! // 2. Group models into a definition
//! #[netabase_macros::netabase_definition(MyApp)]
//! mod my_models {
//!     use super::*;
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // 3. Create an in-memory database for testing
//! let (store, _temp) = RedbStore::<MyApp>::new_temporary()?;
//!
//! // 4. Write data in a transaction
//! let txn = store.begin_write()?;
//! txn.create(&User {
//!     id: "alice".into(),
//!     name: "Alice".into(),
//!     email: "alice@example.com".into(),
//! })?;
//! txn.commit()?;
//!
//! // 5. Read data back
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
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq)]
//! struct Product {
//!     #[primary_key]
//!     sku: String,
//!     name: String,
//!     #[secondary_key]
//!     category: String,
//!     price: f64,
//! }
//!
//! #[netabase_macros::netabase_definition(Shop)]
//! mod shop_models {
//!     use super::*;
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let (store, _temp) = RedbStore::<Shop>::new_temporary()?;
//!
//! // Query by secondary index
//! let txn = store.begin_read()?;
//! let electronics: QueryResult<Product> = txn.query_by_index(
//!     &ProductKeys::Category,
//!     &QueryConfig::new().with_limit(10)
//! )?;
//!
//! for product in electronics {
//!     println!(\"Found: {} - ${}\", product.name, product.price);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Relational Links
//!
//! ```rust
//! use netabase_store::prelude::*;
//! use netabase_store::traits::database::store::NBStore;
//! use netabase_store::relational::RelationalLink;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq)]
//! struct Author {
//!     #[primary_key]
//!     id: String,
//!     name: String,
//! }
//!
//! #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq)]
//! struct Book {
//!     #[primary_key]
//!     isbn: String,
//!     title: String,
//!     #[link(BlogApp, Author)]
//!     author: String,
//! }
//!
//! #[netabase_macros::netabase_definition(BlogApp)]
//! mod blog_models {
//!     use super::*;
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let (store, _temp) = RedbStore::<BlogApp>::new_temporary()?;
//!
//! // Create related models
//! let txn = store.begin_write()?;
//! txn.create(&Author { id: \"author1\".into(), name: \"Jane Doe\".into() })?;
//! txn.create(&Book {
//!     isbn: \"123\".into(),
//!     title: \"Rust Guide\".into(),
//!     author: RelationalLink::new_dehydrated(AuthorID(\"author1\".into())),
//! })?;
//! txn.commit()?;
//!
//! // Hydrate the relationship
//! let txn = store.begin_read()?;
//! let book: Book = txn.read(&BookISBN(\"123\".into()))?.unwrap();
//! let author: Option<Author> = book.author.hydrate(&txn)?;
//! assert_eq!(author.unwrap().name, \"Jane Doe\");
//! # Ok(())
//! # }
//! ```
//!
//! ### Model Versioning and Migration
//!
//! ```rust
//! use netabase_store::prelude::*;
//! use netabase_store::traits::database::store::NBStore;
//! use serde::{Serialize, Deserialize};
//!
//! // Old version of your model
//! #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq)]
//! #[netabase_version(family = \"Customer\", version = 1)]
//! struct CustomerV1 {
//!     #[primary_key]
//!     id: String,
//!     name: String,
//! }
//!
//! // New version with additional field
//! #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq)]
//! #[netabase_version(family = \"Customer\", version = 2)]
//! struct Customer {
//!     #[primary_key]
//!     id: String,
//!     name: String,
//!     email: String,  // New field!
//! }
//!
//! // Define how to migrate from V1 to V2
//! impl MigrateFrom<CustomerV1> for Customer {
//!     fn migrate_from(old: CustomerV1) -> Self {
//!         Customer {
//!             id: old.id,
//!             name: old.name,
//!             email: String::new(),  // Default for new field
//!         }
//!     }
//! }
//!
//! #[netabase_macros::netabase_definition(CRM)]
//! mod crm_models {
//!     use super::*;
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let (store, _temp) = RedbStore::<CRM>::new_temporary()?;
//!
//! // Migration runs automatically when needed
//! if store.needs_migration() {
//!     let result = store.migrate()?;
//!     println!(\"Migrated {} records\", result.total_migrated());
//! }
//! # Ok(())
//! # }
//! ```

#![feature(generic_const_items)]
#![allow(incomplete_features)]

pub mod blob;
pub mod databases;
pub mod errors;
pub mod prelude;
pub mod query;
pub mod relational;
pub mod traits;
