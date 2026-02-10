//! Prelude module for convenient imports.
//!
//! This module re-exports the most commonly used types and traits,
//! allowing users to get started quickly with a single import.
//!
//! # Usage
//!
//! ```rust,no_run
//! use netabase_store::prelude::*;
//! // Now you have access to all common types and traits
//! ```
//!
//! # What's Included
//!
//! ## Core Traits
//!
//! - [`NetabaseDefinition`]: Trait for definition enums grouping models
//! - [`NetabaseModel`]: Trait for individual model structs
//! - `NetabaseRepository`: Trait for repository contexts
//!
//! ## Database Types
//!
//! - [`RedbStore`]: Main database store using redb backend
//! - [`RedbTransaction`]: Transaction wrapper for CRUD operations
//! - `RedbReadTransaction`: Read-only transaction
//! - `RedbWriteTransaction`: Read-write transaction
//!
//! ## Query System
//!
//! - [`QueryConfig`]: Configuration for queries (pagination, filtering, sorting)
//! - [`QueryMode`]: Execution mode (stream results or collect all)
//! - [`QueryResult`]: Iterator over query results
//! - [`FetchOptions`]: Options for hydrating relational links
//! - [`Pagination`]: Cursor-based or offset-based pagination
//!
//! ## Relational System
//!
//! - `RelationalLink`: Type-safe reference to another model
//! - Supports hydration (loading the full referenced model)
//! - Enforces repository isolation at compile time
//!
//! ## Error Handling
//!
//! - [`NetabaseError`]: Comprehensive error type for all operations
//! - [`NetabaseResult`]: Result alias (`Result<T, NetabaseError>`)
//!
//! ## Migration System
//!
//! - `MigrateFrom`: Trait for upgrading from older versions
//! - `MigrateTo`: Trait for downgrading to older versions (P2P)
//! - `VersionContext`: Context for version-aware deserialization
//!
//! # Common Patterns
//!
//! ## Basic CRUD Operations
//!
//! ```rust,no_run
//! use netabase_store::doc_example::*;
//! use netabase_store::databases::redb::RedbStore;
//! use netabase_store::databases::redb::transaction::RedbModelCrud;
//!
//! let store = RedbStore::<ExampleDef>::new_in_memory().unwrap();
//!
//! // Create
//! {
//!     let txn = store.begin_write().unwrap();
//!     txn.create(&User { id: UserID("u1".into()), name: "Alice".into(), email: "a@b.com".into() }).unwrap();
//!     txn.commit().unwrap();
//! }
//!
//! // Read
//! {
//!     let txn = store.begin_read().unwrap();
//!     let result: Option<User> = txn.read(&UserID("u1".into())).unwrap();
//!     assert!(result.is_some());
//! }
//!
//! // Update
//! {
//!     let txn = store.begin_write().unwrap();
//!     txn.update(&User { id: UserID("u1".into()), name: "Alice Updated".into(), email: "a@b.com".into() }).unwrap();
//!     txn.commit().unwrap();
//! }
//!
//! // Delete
//! {
//!     let txn = store.begin_write().unwrap();
//!     txn.delete::<User>(&UserID("u1".into())).unwrap();
//!     txn.commit().unwrap();
//! }
//! ```
//!
//! ## Querying with Configuration
//!
//! Query operations use model-level methods on open tables:
//!
//! ```rust,no_run
//! use netabase_store::prelude::*;
//! use netabase_store::databases::redb::RedbStore;
//! use netabase_store::databases::redb::transaction::{CrudOptions, RedbModelCrud};
//! use netabase_store::doc_example::*;
//! use netabase_store::traits::database::store::NBStore;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let (store, _temp) = RedbStore::<ExampleDef>::new_temporary()?;
//!
//! // Seed some data
//! let txn = store.begin_write()?;
//! txn.create(&User { id: UserID("u1".into()), name: "Alice".into(), email: "a@b.com".into() })?;
//! txn.create(&User { id: UserID("u2".into()), name: "Bob".into(), email: "b@c.com".into() })?;
//! txn.commit()?;
//!
//! // Open tables for the model
//! let txn = store.begin_read()?;
//! use netabase_store::traits::registry::models::model::redb_model::RedbNetbaseModel;
//! let table_defs = User::table_definitions();
//! let tables = txn.open_model_tables(table_defs, None)?;
//!
//! // List with options
//! let config = CrudOptions::new()
//!     .with_limit(1)
//!     .with_offset(1);
//! let results = User::list_entries(&tables, config)?;
//!
//! assert_eq!(results.len(), 1);
//! # Ok(())
//! # }
//! ```
//!
//! See [tests/integration_list.rs](../tests/integration_list.rs) for complete examples.
//!
//! ## Working with Relational Links
//!
//! ```rust,no_run
//! use netabase_store::doc_example::*;
//! use netabase_store::databases::redb::RedbStore;
//! use netabase_store::databases::redb::transaction::RedbModelCrud;
//! use netabase_store::relational::RelationalLink;
//! use netabase_store::traits::registry::repository::Standalone;
//!
//! let store = RedbStore::<ExampleDef>::new_in_memory().unwrap();
//!
//! // Create an author first
//! let txn = store.begin_write().unwrap();
//! txn.create(&Author {
//!     id: AuthorID("author1".into()),
//!     name: "Jane Doe".into(),
//!     genre: "Fiction".into(),
//! }).unwrap();
//! txn.commit().unwrap();
//!
//! // Read the author back
//! let txn = store.begin_read().unwrap();
//! let author: Option<Author> = txn.read(&AuthorID("author1".into())).unwrap();
//!
//! match author {
//!     Some(a) => assert_eq!(a.name, "Jane Doe"),
//!     None => panic!("Author not found"),
//! }
//! ```
//!
//! ## Migration Between Versions
//!
//! Migration is defined using the `MigrateFrom` trait. Here's a conceptual example:
//!
//! ```rust,no_run
//! use netabase_store::prelude::*;
//!
//! // Given UserV1 and UserV2 in the same definition:
//! impl MigrateFrom<UserV1> for UserV2 {
//!     fn migrate_from(old: UserV1) -> Self {
//!         UserV2 {
//!             id: old.id,
//!             name: old.name,
//!             email: String::new(), // New field with default
//!         }
//!     }
//! }
//! ```
//!
//! See the [`traits::migration`](crate::traits::migration) module for full details.
//!
//! # Rules and Best Practices
//!
//! - Always commit write transactions explicitly
//! - Use read transactions for queries to allow concurrent access
//! - Configure pagination for large result sets
//! - Hydrate links only when needed (it's an additional query)
//! - Handle errors with `?` or proper error matching
//!
//! # Not Included
//!
//! The following are intentionally not in the prelude to avoid namespace pollution:
//!
//! - Macro attributes (`#[netabase_model]`, `#[primary_key]`, etc.)
//! - Backend-specific implementation details
//! - Internal trait helpers
//! - Advanced migration chain builders
//!
//! Import these explicitly when needed from their respective modules.

// Core traits
pub use crate::traits::registry::definition::NetabaseDefinition;
pub use crate::traits::registry::models::model::NetabaseModel;

// Database and transactions
pub use crate::databases::redb::RedbStore;
pub use crate::databases::redb::StoreConfig;
pub use crate::databases::redb::transaction::RedbTransaction;

// Query configuration
pub use crate::query::{FetchOptions, Pagination, QueryConfig, QueryMode, QueryResult};

// Error handling
pub use crate::errors::{NetabaseError, NetabaseResult};

// Re-export commonly used derive macros from netabase_macros
// Users will still need to import the macros crate, but this documents the pattern
