//! Prelude module for convenient imports.
//!
//! This module re-exports the most commonly used types and traits,
//! allowing users to get started quickly with a single import.
//!
//! # Usage
//!
//! ```rust,ignore
//! use netabase_store::prelude::*;
//! ```
//!
//! # What's Included
//!
//! ## Core Traits
//!
//! - [`NetabaseDefinition`]: Trait for definition enums grouping models
//! - [`NetabaseModel`]: Trait for individual model structs
//! - [`NetabaseRepository`]: Trait for repository contexts
//!
//! ## Database Types
//!
//! - [`RedbStore`]: Main database store using redb backend
//! - [`RedbTransaction`]: Transaction wrapper for CRUD operations
//! - [`RedbReadTransaction`]: Read-only transaction
//! - [`RedbWriteTransaction`]: Read-write transaction
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
//! - [`RelationalLink`]: Type-safe reference to another model
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
//! - [`MigrateFrom`]: Trait for upgrading from older versions
//! - [`MigrateTo`]: Trait for downgrading to older versions (P2P)
//! - [`VersionContext`]: Context for version-aware deserialization
//!
//! # Common Patterns
//!
//! ## Basic CRUD Operations
//!
//! ```rust,ignore
//! use netabase_store::prelude::*;
//!
//! let store = RedbStore::<MyDef>::create("data.db")?;
//!
//! // Create
//! {
//!     let txn = store.begin_write()?;
//!     txn.create(&model)?;
//!     txn.commit()?;
//! }
//!
//! // Read
//! {
//!     let txn = store.begin_read()?;
//!     let result: Option<MyModel> = txn.read(&key)?;
//! }
//!
//! // Update
//! {
//!     let txn = store.begin_write()?;
//!     txn.update(&modified_model)?;
//!     txn.commit()?;
//! }
//!
//! // Delete
//! {
//!     let txn = store.begin_write()?;
//!     txn.delete(&key)?;
//!     txn.commit()?;
//! }
//! ```
//!
//! ## Querying with Configuration
//!
//! ```rust,ignore
//! use netabase_store::prelude::*;
//!
//! let config = QueryConfig::builder()
//!     .limit(10)
//!     .skip(20)
//!     .build();
//!
//! let txn = store.begin_read()?;
//! let results: QueryResult<MyModel> = txn.query(&config)?;
//!
//! for model in results {
//!     println!("Found: {:?}", model);
//! }
//! ```
//!
//! ## Working with Relational Links
//!
//! ```rust,ignore
//! use netabase_store::prelude::*;
//!
//! // Create a dehydrated link (just the key)
//! let link = RelationalLink::new_dehydrated(user_id);
//!
//! // Hydrate the link (load full data)
//! let txn = store.begin_read()?;
//! let hydrated = link.hydrate(&txn)?;
//!
//! match hydrated {
//!     Some(user) => println!("User: {}", user.name),
//!     None => println!("User not found"),
//! }
//! ```
//!
//! ## Migration Between Versions
//!
//! ```rust,ignore
//! use netabase_store::prelude::*;
//!
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
pub use crate::traits::registery::definition::NetabaseDefinition;
pub use crate::traits::registery::models::model::NetabaseModel;

// Database and transactions
pub use crate::databases::redb::RedbStore;
pub use crate::databases::redb::transaction::RedbTransaction;

// Query configuration
pub use crate::query::{FetchOptions, Pagination, QueryConfig, QueryMode, QueryResult};

// Error handling
pub use crate::errors::{NetabaseError, NetabaseResult};

// Re-export commonly used derive macros from netabase_macros
// Users will still need to import the macros crate, but this documents the pattern
