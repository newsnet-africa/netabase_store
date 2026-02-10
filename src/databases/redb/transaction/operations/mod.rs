//! High-level CRUD operations for transactions.
//!
//! This module provides clean, ergonomic wrappers around the low-level CRUD operations.
//! The operations are split into logical submodules for better organization.
//!
//! # Modules
//!
//! - [`create`] - Create/insert operations
//! - [`read`] - Read/query operations  
//! - [`update`] - Update/modify operations
//! - [`delete`] - Delete/remove operations
//! - [`list`] - List/iteration operations
//!
//! # Usage
//!
//! These operations are available directly on [`RedbTransaction`](super::RedbTransaction):
//!
//! ```rust,no_run
//! // Create
//! txn.create(&model)?;
//!
//! // Read
//! let model: Option<Model> = txn.read(&key)?;
//!
//! // Update
//! txn.update(&model)?;
//!
//! // Delete
//! txn.delete::<Model>(&key)?;
//!
//! // List
//! let all = txn.list::<Model>()?;
//! ```

// Re-export trait bounds helper
pub use super::crud::RedbModelCrud;

// Note: The actual CRUD operations are implemented directly on RedbTransaction
// in mod.rs due to the complex trait bounds involved. This module serves as
// documentation and future expansion point for cleaner operation abstractions.
