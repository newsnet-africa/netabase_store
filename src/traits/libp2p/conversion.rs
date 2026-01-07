use libp2p::kad::Record;
use libp2p::kad::RecordKey as Key;

use crate::errors::NetabaseResult;

/// Trait for converting between Models and Libp2p Records.
///
/// This trait should be implemented by definitions or models to define how they
/// are mapped to the Kademlia DHT.
pub trait Libp2pRecordConversion {
    /// Convert the model/definition to a Libp2p Record.
    fn to_record(&self) -> NetabaseResult<Record>;

    /// Create a model/definition from a Libp2p Record.
    fn from_record(record: Record) -> NetabaseResult<Self>
    where
        Self: Sized;

    /// Derive the Libp2p Key for this entity.
    fn derive_key(&self) -> Key;
}
