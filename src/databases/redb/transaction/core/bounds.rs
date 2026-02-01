//! Trait bound helpers for redb transaction operations.
//!
//! This module defines marker traits that help document the requirements
//! for types used in the transaction system.
//!
//! # Design Philosophy
//!
//! The redb integration requires many complex trait bounds. This module
//! provides simple marker traits for documentation purposes, but the actual
//! bounds are still specified inline due to Rust's current limitations
//! with trait aliases.

use redb::Key;

/// Marker trait for types that can be used as redb keys with proper bounds.
///
/// This combines the common requirements for key types:
/// - `redb::Key` for database storage
/// - `'static` lifetime for storage
/// - `Clone` for value semantics
pub trait RedbKeyBounds: Key + Clone + 'static {}
impl<T: Key + Clone + 'static> RedbKeyBounds for T {}

/// Marker trait for discriminant types used in enum iteration.
///
/// Discriminants are used by strum to iterate over enum variants.
pub trait DiscriminantBounds: std::fmt::Debug + 'static {}
impl<T: std::fmt::Debug + 'static> DiscriminantBounds for T {}

