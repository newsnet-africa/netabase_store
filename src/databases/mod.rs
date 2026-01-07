//! Database backend implementations.
//!
//! This module provides concrete implementations of database backends.
//!
//! # Available Backends
//!
//! - **`redb`**: Production-ready embedded database backend (fully implemented)
//! - **`indexedb`**: Browser-based storage (placeholder for future implementation)
//!
//! # Usage
//!
//! Most users will use the redb backend:
//!
//! ```rust
//! use netabase_store::databases::redb::RedbStore;
//! use netabase_store::traits::database::store::NBStore;
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
//! let (store, _temp) = RedbStore::<MyApp>::new_temporary()?;
//! # Ok(())
//! # }
//! ```

pub mod indexedb;
pub mod redb;
