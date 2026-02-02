//! Node metadata trait for P2P identity management.
//!
//! This module defines the `NodeMetadataTrait` which provides a standard interface
//! for accessing node identity information in P2P networks.
//!
//! # Purpose
//!
//! In a peer-to-peer network, each node needs to:
//! - Identify itself with a unique peer ID
//! - Advertise capabilities and supported features
//! - Exchange metadata with other peers
//!
//! This trait provides the foundational interface for managing that identity.
//!
//! # Example
//!
//! ```rust,ignore
//! use libp2p::PeerId;
//! use netabase_store::traits::node_metadata::NodeMetadataTrait;
//!
//! struct MyNodeMetadata {
//!     peer_id: PeerId,
//!     capabilities: Vec<String>,
//! }
//!
//! impl NodeMetadataTrait for MyNodeMetadata {
//!     fn new(node_id: PeerId) -> Self {
//!         MyNodeMetadata {
//!             peer_id: node_id,
//!             capabilities: vec!["sync".into(), "relay".into()],
//!         }
//!     }
//!     
//!     fn node_id(&self) -> PeerId {
//!         self.peer_id
//!     }
//! }
//! ```
//!
//! # Feature Flag
//!
//! This module requires the `libp2p` feature to be enabled.

use libp2p::PeerId;

/// Trait for managing node identity and metadata in P2P networks.
///
/// This trait provides the basic interface for creating and accessing node
/// metadata, primarily centered around the node's peer ID.
///
/// # Example
///
/// ```rust,ignore
/// let peer_id = PeerId::random();
/// let metadata = MyNodeMetadata::new(peer_id);
/// assert_eq!(metadata.node_id(), peer_id);
/// ```
pub trait NodeMetadataTrait {
    /// Create new node metadata with the given peer ID.
    ///
    /// # Arguments
    ///
    /// - `node_id`: The libp2p peer ID for this node
    ///
    /// # Returns
    ///
    /// A new instance of node metadata initialized with the given peer ID.
    fn new(node_id: PeerId) -> Self;
    
    /// Get the peer ID of this node.
    ///
    /// # Returns
    ///
    /// The libp2p peer ID identifying this node in the network.
    fn node_id(&self) -> PeerId;
}
