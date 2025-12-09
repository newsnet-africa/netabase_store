#![feature(associated_type_defaults)]
//!
//! # Netabase Store
//!
//! A type-safe, high-performance embedded database abstraction layer for Rust.
//! Netabase provides a strongly-typed interface for storing and querying structured data with support
//! for secondary indices, relational keys, subscription trees, and automatic tree management.
//!
//! Netabase is backend-agnostic and can work with any key-value store that implements the
//! [`BackendStore`](backend::BackendStore) trait. Currently supported backends:
//! - **redb**: High-performance embedded database (default)
//! - **sled**: Planned
//! - **IndexedDB**: Planned for WASM targets
//!
//! ## Core Concepts
//!
//! ### Models
//!
//! Models are the primary data structures you want to store. Each model must implement the
//! [`NetabaseModelTrait`](traits::model::NetabaseModelTrait) which defines:
//! - Primary key type and access
//! - Secondary indices for fast lookups by non-primary keys
//! - Relational keys for modeling foreign key relationships
//!
//! ### Definitions
//!
//! A Definition is an enum that wraps all your model types. It implements
//! [`NetabaseDefinition`](traits::definition::NetabaseDefinition) and provides:
//! - Type-safe routing to the correct storage tree
//! - Discriminant-based table identification
//! - Unified interface for multi-model operations
//!
//! ### Store
//!
//! The [`RedbStore`](databases::redb_store::RedbStore) is the main entry point for database operations:
//! - Type-safe CRUD operations
//! - Transaction support (read and write)
//! - Automatic secondary index management
//! - Tree lifecycle management
//!
//! ## Quick Start
//!
//! ```ignore
//! use netabase_store::{
//!     databases::redb_store::RedbStore,
//!     traits::{
//!         model::NetabaseModelTrait,
//!         store::store::StoreTrait,
//!     },
//! };
//!
//! // Define your models and implement NetabaseModelTrait
//! // See examples/boilerplate.rs for a complete example
//!
//! # // This is a minimal example stub
//! # use bincode::{Encode, Decode};
//! # #[derive(Clone, Debug, Encode, Decode)]
//! # struct User { id: u64, name: String }
//! # #[derive(Clone, Debug)]
//! # enum Definitions { User(User) }
//! #
//! // Create a store
//! let store: RedbStore<Definitions> = RedbStore::new("./my_database.db")
//!     .expect("Failed to create store");
//!
//! // Store operations are type-safe and use your model types
//! // store.put_one(my_user)?;
//! // let user = store.get_one(user_id)?;
//! ```
//!
//! ## Features
//!
//! - **Type Safety**: Compile-time guarantees for all database operations
//! - **Secondary Indices**: Fast lookups by any field, automatically maintained
//! - **Relational Keys**: Model foreign key relationships between entities
//! - **Transaction Support**: ACID-compliant read and write transactions
//! - **Tree Management**: Automatic creation and management of database trees
//! - **Zero-Copy Reads**: Efficient data access using Cow semantics
//! - **Discriminant-Based Routing**: Type-safe dispatch to correct storage trees
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │     Your Application Code               │
//! └─────────────────┬───────────────────────┘
//!                   │
//!                   ▼
//! ┌─────────────────────────────────────────┐
//! │         RedbStore<Definition>           │
//! │  (StoreTrait implementation)            │
//! └─────────────────┬───────────────────────┘
//!                   │
//!         ┌─────────┴─────────┐
//!         ▼                   ▼
//! ┌───────────────┐   ┌──────────────┐
//! │ Read Trans.   │   │ Write Trans. │
//! └───────┬───────┘   └──────┬───────┘
//!         │                  │
//!         └─────────┬────────┘
//!                   ▼
//! ┌─────────────────────────────────────────┐
//! │         redb Database                   │
//! │  - Primary key trees (main storage)     │
//! │  - Secondary index trees                │
//! │  - Relational key trees                 │
//! └─────────────────────────────────────────┘
//! ```
//!
//! ## Examples
//!
//! See [`examples/boilerplate.rs`](https://github.com/anthropics/netabase_store/blob/main/examples/boilerplate.rs)
//! for a comprehensive example with:
//! - 6 different model types (User, Product, Category, Review, Tag, ProductTag)
//! - Primary and secondary indices
//! - One-to-Many, Many-to-One, and Many-to-Many relationships
//! - Full CRUD operations
//! - Batch operations
//! - Query by secondary keys
//!
//! ## Performance Considerations
//!
//! - Use batch operations (`put_many`) when inserting multiple records
//! - Secondary indices have a write cost but make reads much faster
//! - Read transactions are cheap and can be held for extended periods
//! - Write transactions should be kept short to minimize lock contention
//!
//! ## Error Handling
//!
//! All database operations return `NetabaseResult<T>` which is an alias for
//! `Result<T, NetabaseError>`. See [`error`] module for error types.

pub mod backend;
pub mod error;
pub mod traits;
pub mod databases;

// Re-export commonly used types
pub use error::{NetabaseError, NetabaseResult};

// Backend abstraction re-exports
pub use backend::{
    BackendKey, BackendValue, BackendStore, BackendReadTransaction, BackendWriteTransaction,
    BackendError, BackendTable, BackendReadableTable, BackendWritableTable,
};
