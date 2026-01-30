use libp2p::PeerId;

/// Trait for Node Metadata
pub trait NodeMetadataTrait {
    fn new(node_id: PeerId) -> Self;
    fn node_id(&self) -> PeerId;
}
