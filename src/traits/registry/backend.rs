//! Backend abstraction for storage engines.
//!
//! This module introduces a type-level backend parameter that can be threaded
//! through core traits (like [`NetabaseModelKeys`]) to specialize behavior for
//! different storage backends without using dynamic dispatch.
//!
//! # Design
//!
//! - [`Backend`] is a marker trait implemented by backend types
//! - [`RedbBackend`] is the default backend used by the current implementation
//! - Core traits can add a generic parameter `B: Backend = RedbBackend` to opt-in
//!   to backend-specific behavior while remaining backwards compatible
//!
//! At the moment this is scaffolding: the existing redb-specific bounds still
//! live in redb-specific traits. Core traits remain backend-agnostic.

/// Marker trait for storage backends.
///
/// Backend types are zero-sized types that carry compile-time information about
/// which storage engine is being targeted (e.g., redb, in-memory, etc.).
pub trait Backend: 'static {}

/// Default backend used by netabase_store today.
///
/// This corresponds to the `redb` embedded
/// key-value store. All existing code effectively assumes `RedbBackend`.
#[derive(Debug, Clone, Copy, Default)]
pub struct RedbBackend;

impl Backend for RedbBackend {}
/// In-memory backend used for testing and development.
///
/// This corresponds to the in-process `MemoryStore` backend defined in
/// `databases::memory`. It is useful for validating that core traits are
/// backend-agnostic.
#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryBackend;

impl Backend for MemoryBackend {}

