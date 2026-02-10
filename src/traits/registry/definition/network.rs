//! Network-capable definition trait for P2P systems.
//!
//! This module defines the `NetworkDefinition` trait, which extends `NetabaseDefinition`
//! with capabilities required for peer-to-peer networking and distributed systems.
//!
//! # Purpose
//!
//! While `NetabaseDefinition` handles local database operations, `NetworkDefinition`
//! adds the traits and types needed for:
//! - Advertising capabilities to peers
//! - Negotiating feature support
//! - Coordinating schema versions across nodes
//! - Managing network-specific metadata
//!
//! # Example
//!
//! ```rust,no_run
//! // A definition that supports networking
//! #[derive(NetworkDefinition)]
//! pub enum MyNetworkDef {
//!     User(User),
//!     Post(Post),
//! }
//!
//! impl NetworkDefinition for MyNetworkDef {
//!     type DefinitionCapabilities = MyCapabilities;
//! }
//! ```
//!
//! # Feature Flag
//!
//! This trait is only available when the `libp2p` feature is enabled.

use super::NetabaseDefinition;

/// Extension of `NetabaseDefinition` for network-capable definitions.
///
/// This trait adds support for advertising definition capabilities in a
/// peer-to-peer network, allowing nodes to discover what features and
/// models each peer supports.
///
/// # Associated Types
///
/// - `DefinitionCapabilities`: Type representing what features this definition
///   supports (e.g., which optional features are enabled, version numbers, etc.)
///
/// # Requirements
///
/// Implementations must also implement `NetabaseDefinition`, ensuring the
/// definition works both locally and in networked contexts.
///
/// # Example
///
/// ```rust,no_run
/// #[derive(Debug, Clone)]
/// pub struct MyCapabilities {
///     supports_blobs: bool,
///     supports_subscriptions: bool,
///     version: u32,
/// }
///
/// impl NetworkDefinition for MyDef {
///     type DefinitionCapabilities = MyCapabilities;
/// }
/// ```
pub trait NetworkDefinition: NetabaseDefinition
where
    <Self as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <Self as strum::IntoDiscriminant>::Discriminant: 'static,
{
    /// Type representing the capabilities of this definition.
    ///
    /// This type should describe what features and versions this definition
    /// supports, allowing peers to determine compatibility.
    type DefinitionCapabilities;
}
