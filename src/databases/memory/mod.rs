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
//! The memory backend implements the same trait interface as the redb backend,
//! allowing you to swap backends without changing application code.
//!
//! ```rust
//! use netabase_store::databases::memory::MemoryStore;
//! use netabase_store::doc_example::ExampleDef;
//!
//! // Create an in-memory store
//! let store = MemoryStore::<ExampleDef>::new();
//!
//! // Begin a write transaction
//! let mut txn = store.begin_write().unwrap();
//!
//! // Insert raw bytes (low-level API)
//! txn.insert("users", b"alice".to_vec(), b"Alice Smith".to_vec());
//! txn.commit().unwrap();
//!
//! // Read back
//! let txn = store.begin_read().unwrap();
//! let value = txn.get("users", b"alice");
//! assert_eq!(value, Some(b"Alice Smith".to_vec()));
//! ```
//!
//! # Comparison with RedbStore
//!
//! | Feature | MemoryStore | RedbStore |
//! |---------|-------------|-----------|
//! | Persistence | ❌ | ✅ |
//! | ACID | ❌ | ✅ |
//! | Performance | 🚀 Very fast | ⚡ Fast |
//! | Disk I/O | None | Required |
//! | Use case | Testing | Production |
//!
//! # Limitations
//!
//! - **Not persistent** - Data is lost when the store is dropped
//! - **Single process** - No cross-process sharing
//! - **No ACID guarantees** - Simplified transaction model for testing
//! - **No crash recovery** - Unlike redb, data is not durable
//!
//! # Implementation Details
//!
//! The memory backend uses `BTreeMap` for ordered key-value storage,
//! mirroring redb's B-tree based storage model. This ensures that
//! range queries and iterations produce results in the same order
//! as the redb backend.
//!
//! ## Storage Structure
//!
//! - **Regular tables**: `BTreeMap<Vec<u8>, Vec<u8>>` - single value per key
//! - **Multimap tables**: `BTreeMap<Vec<u8>, Vec<Vec<u8>>>` - multiple values per key
//!
//! ## Transaction Semantics
//!
//! - Read transactions operate on snapshots (isolation from writes)
//! - Write transactions accumulate mutations and apply atomically on commit
//! - Dropping an uncommitted write transaction discards all changes

mod storage;
mod store;
mod transaction;

pub use storage::Storage;
pub use store::MemoryStore;
pub use transaction::{MemoryReadTransaction, MemoryWriteTransaction};
