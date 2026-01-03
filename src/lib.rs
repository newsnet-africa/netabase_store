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
//! ```rust,no_run
//! # use netabase_store::prelude::*;
//! # use netabase_store::traits::database::store::NBStore;
//! # use netabase_store_examples::boilerplate_lib::Definition;
//! # use netabase_store_examples::boilerplate_lib::definition::{User, UserID, LargeUserFile, AnotherLargeUserFile};
//! # use netabase_store::relational::RelationalLink;
//! # use netabase_store_examples::boilerplate_lib::CategoryID;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let db_path = "my.db";
//!
//! // Open a database
//! let store = RedbStore::<Definition>::new(&db_path)?;
//!
//! // Write data
//! let txn = store.begin_write()?;
//! let user = User {
//!     id: UserID("alice".into()),
//!     first_name: "Alice".into(),
//!     last_name: "Smith".into(),
//!     age: 30,
//!     partner: RelationalLink::new_dehydrated(UserID("none".into())),
//!     category: RelationalLink::new_dehydrated(CategoryID("none".into())),
//!     bio: LargeUserFile::default(),
//!     another: AnotherLargeUserFile::default(),
//!     subscriptions: vec![],
//! };
//! txn.create(&user)?;
//! txn.commit()?;
//!
//! // Read data
//! let txn = store.begin_read()?;
//! let retrieved: Option<User> = txn.read(&UserID("alice".into()))?;
//! assert!(retrieved.is_some());
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
