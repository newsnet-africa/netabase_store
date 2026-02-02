//! Core types for Netabase Store.
//!
//! This module contains fundamental types used throughout the Netabase Store system:
//!
//! - [`key`] - Key structures for entry addressing and path encoding
//!
//! These types form the foundation of the storage system and are used by models,
//! definitions, and database implementations.
//!
//! Note: Networking primitives, capabilities, and protocol types have been moved
//! to the `netabase` crate.

/// Key types for entry addressing.
///
/// Provides the [`NetabaseKey`](key::NetabaseKey) structure for uniquely
/// identifying and ordering entries in the storage system.
pub mod key;

// Re-export commonly used types
pub use key::{NetabaseKey, NetabasePath};
