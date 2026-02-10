//! Core database trait abstractions.
//!
//! This module defines the fundamental interfaces that database backends must implement.
//! These traits provide a consistent API across different storage engines.
//!
//! # Traits
//!
//! - [`store::NBStore`] - Database creation and transaction lifecycle
//! - [`transaction`] - Read/write transaction operations
//! - [`hash`] - Content-addressable storage and hashing utilities
//!
//! # Example
//!
//! ```rust,no_run
//! use netabase_store::traits::database::store::NBStore;
//! use netabase_store::databases::redb::RedbStore;
//! # use serde::{Serialize, Deserialize};
//! # #[netabase_macros::netabase_definition(MyApp)]
//! # mod models {
//! #     use super::*;
//! #     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
//! #     pub struct User { #[primary_key] pub id: String }
//! # }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use models::*;
//!
//! // Any type implementing NBStore can be used
//! let (store, _temp) = RedbStore::<MyApp>::new_temporary()?;
//! let txn = store.begin_read()?;
//! # Ok(())
//! # }
//! ```

pub mod hash;
pub mod store;
pub mod transaction;
