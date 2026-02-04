//! Database backend implementations.
//!
//! This module provides concrete implementations of database backends.
//!
//! # Available Backends
//!
//! - **`redb`**: Production-ready embedded database backend (fully implemented)
//! - **`memory`**: In-memory backend for testing and development
//! - **`indexedb`**: Browser-based storage (placeholder for future implementation)
//!
//! # Usage
//!
//! Most users will use the redb backend for production:
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
//!
//! For testing, use the memory backend:
//!
//! ```rust
//! use netabase_store::databases::memory::MemoryStore;
//! use netabase_store::doc_examples::ExampleDef;
//!
//! # fn main() {
//! // Fast, no disk I/O, ephemeral data
//! let store = MemoryStore::<ExampleDef>::new();
//! let mut txn = store.begin_write().unwrap();
//! txn.insert("users", b"alice".to_vec(), b"Alice".to_vec());
//! txn.commit().unwrap();
//! # }
//! ```

pub mod indexedb;
pub mod memory;
pub mod redb;
