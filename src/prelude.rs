//! Prelude module for convenient imports.
//!
//! This module re-exports the most commonly used types and traits,
//! allowing users to get started quickly with a single import.
//!
//! # Example
//!
//! ```rust,ignore
//! use netabase_store::prelude::*;
//! ```

// Core traits
pub use crate::traits::registery::definition::NetabaseDefinition;
pub use crate::traits::registery::models::model::NetabaseModel;

// Database and transactions
pub use crate::databases::redb::RedbStore;
pub use crate::databases::redb::transaction::{QueryConfig, RedbTransaction};

// Error handling
pub use crate::errors::{NetabaseError, NetabaseResult};

// Re-export commonly used derive macros from netabase_macros
// Users will still need to import the macros crate, but this documents the pattern
