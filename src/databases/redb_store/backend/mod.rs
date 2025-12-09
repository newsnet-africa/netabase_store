//! Redb backend implementation
//!
//! This module contains the redb-specific implementation of the backend traits.
//! All redb-specific types and functionality are isolated here.

pub mod error;
pub mod key;
pub mod value;
pub mod table;
pub mod transaction;
pub mod store;

pub use error::RedbBackendError;
pub use store::RedbBackendStore;
