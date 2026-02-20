//! Type registry system for models, definitions, and repositories.
//!
//! This module provides the compile-time type system that enforces schema correctness
//! and manages relationships between models, definitions, and repositories.
//!
//! # Architecture
//!
//! The registry system has three layers:
//!
//! 1. **Models** ([`models`]) - Individual data structures with primary keys, secondary keys, and links
//! 2. **Definitions** ([`definition`]) - Collections of related models forming a schema unit
//! 3. **Repositories** ([`repository`]) - Access boundaries that group definitions
//!
//! # Type Safety Guarantees
//!
//! - Models must belong to exactly one definition
//! - Links between models must respect repository boundaries
//! - Primary keys are unique and strongly typed
//! - Secondary keys create indexed lookups
//! - Blob fields are automatically chunked
//!
//! # Example
//!
//! ```rust
//! use netabase_store::doc_example::*;
//! use netabase_store::databases::redb::RedbStore;
//! use netabase_store::traits::database::store::NBStore;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a temporary database
//! let (store, _temp) = RedbStore::<ExampleDef>::new_temporary()?;
//!
//! // Create a user
//! let mut txn = store.begin_write()?;
//! txn.create(&User {
//!     id: UserID("alice".into()),
//!     name: "Alice".into(),
//!     email: "alice@example.com".into(),
//! })?;
//! txn.commit()?;
//!
//! // Read the user back
//! let txn = store.begin_read()?;
//! let user: Option<User> = txn.read(&UserID("alice".into()))?;
//! assert!(user.is_some());
//! # Ok(())
//! # }
//! ```

pub mod definition;
pub mod models;
pub mod repository;
pub mod backend;
