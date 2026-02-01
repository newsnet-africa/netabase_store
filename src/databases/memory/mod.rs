//! In-memory database backend for testing and development.
//!
//! This module provides a pure in-memory implementation of the database backend,
//! useful for:
//! - **Unit testing** - Fast, deterministic tests without disk I/O
//! - **Development** - Quick iteration without persistence overhead
//! - **Architecture validation** - Proves the trait system is backend-agnostic
//!
//! # Usage
//!
//! ```rust,ignore
//! use netabase_store::databases::memory::MemoryStore;
//! use netabase_store::traits::database::store::NBStore;
//!
//! // Create an in-memory store
//! let store = MemoryStore::<MyDefinition>::new();
//!
//! // Use just like RedbStore
//! let txn = store.begin_write()?;
//! txn.create(&model)?;
//! txn.commit()?;
//! ```
//!
//! # Limitations
//!
//! - **Not persistent** - Data is lost when the store is dropped
//! - **Single process** - No cross-process sharing
//! - **No ACID guarantees** - Simplified transaction model
//!
//! # Implementation
//!
//! The memory backend uses `BTreeMap` for ordered key-value storage,
//! mirroring redb's B-tree based storage model.

mod storage;
mod store;
mod transaction;

pub use storage::Storage;
pub use store::MemoryStore;
pub use transaction::{MemoryReadTransaction, MemoryWriteTransaction};
