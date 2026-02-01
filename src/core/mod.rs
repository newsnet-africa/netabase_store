//! Core types and primitives for Netabase.
//!
//! This module contains fundamental types used throughout the Netabase system:
//!
//! - [`key`] - Type-safe primary key wrappers
//! - [`primitives`] - Core constants and primitive types
//! - [`capabilities`] - Permission bitflags for access control
//!
//! These types form the foundation of the type system and are used by models,
//! definitions, and database implementations.

/// Type-safe key abstractions for primary keys.
///
/// Provides the [`Key`](key::Key) trait and wrapper types that ensure
/// compile-time type safety for model identifiers.
pub mod key;

/// Core primitive types and constants.
///
/// Defines fundamental constants like table name prefixes, maximum sizes,
/// and primitive type definitions.
pub mod primitives;

/// Capability bitflags for permission management.
///
/// Provides [`Capabilities`](capabilities::Capabilities) for fine-grained
/// access control in repository patterns.
pub mod capabilities;
