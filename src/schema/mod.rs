//! Schema-related types for advanced features.
//!
//! This module contains types for optional schema features:
//!
//! - [`blob`] - Large binary data storage with chunking
//! - [`relational`] - Type-safe relational links between models
//! - [`subscription_hash`] - Merkle tree hashing for P2P synchronization
//!
//! Each submodule corresponds to a feature flag and provides types that extend
//! the base model functionality.

/// Blob storage for large binary data.
///
/// Provides automatic chunking and efficient storage for files, images,
/// and other large binary payloads.
#[cfg(feature = "blobs")]
pub mod blob;

/// Relational links between models.
///
/// Type-safe references between models with automatic hydration support.
#[cfg(feature = "relational_keys")]
pub mod relational;

/// Subscription-based hashing for P2P synchronization.
///
/// Merkle tree implementation for efficient synchronization of subscription
/// topics in distributed systems.
#[cfg(feature = "subscriptions")]
pub mod subscription_hash;
