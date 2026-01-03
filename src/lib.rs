//! # Netabase Store
//!
//! A type-safe, high-performance embedded database library for Rust with
//! automatic model migration and compile-time schema validation.
//!
//! ## Features
//!
//! - **Type-Safe**: Compile-time schema validation with Rust's type system
//! - **High Performance**: Zero-copy operations with bincode serialization
//! - **Auto Migration**: Automatic schema versioning and data migration
//! - **Transactions**: ACID-compliant read/write transactions
//! - **Secondary Indexes**: Fast lookups on non-primary fields
//! - **Relational Links**: Support for relationships between models
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use netabase_store::prelude::*;
//! use netabase_macros::{NetabaseModel, netabase_definition};
//!
//! #[netabase_definition]
//! mod mydb {
//!     #[derive(NetabaseModel, Clone, Encode, Decode)]
//!     pub struct User {
//!         #[primary]
//!         pub id: u64,
//!         pub name: String,
//!     }
//! }
//!
//! // Open a database
//! let store = RedbStore::<mydb::Mydb>::new("my.db")?;
//!
//! // Write data
//! let txn = store.begin_write()?;
//! txn.create(&mydb::User { id: 1, name: "Alice".into() })?;
//! txn.commit()?;
//!
//! // Read data
//! let txn = store.begin_read()?;
//! let user: Option<mydb::User> = txn.read::<mydb::User>(&1u64)?;
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
