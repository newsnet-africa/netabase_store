use serde::{Deserialize, Serialize};

use libp2p::PeerId;

/// Metadata for Libp2p integration injected into models.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default, Hash, PartialOrd, Ord)]
pub struct Libp2pMetadata {
    /// The publisher of the record.
    pub publisher: Option<PeerId>,
    /// Expiration time of the record.
    /// Note: Instant is not serializable by default in serde, we might need a wrapper or SystemTime.
    /// libp2p uses Instant which is process-local. For storage, SystemTime is better.
    /// But libp2p::kad::Record uses Instant.
    /// We will use SystemTime for storage and convert.
    #[serde(with = "crate::utils::serde_system_time_option")]
    pub expires: Option<std::time::SystemTime>,
    /// Placeholder for future metadata extensions
    pub extra: Option<Vec<u8>>,
}

/// Trait for models that support Libp2p functionality.
/// This is automatically implemented for all models by the macro.
pub trait Libp2pModel: Serialize + for<'a> Deserialize<'a> {
    fn get_libp2p_metadata(&self) -> Option<&Libp2pMetadata>;
}

// impl<'a, D: NetabaseDefinition + Deserialize<'a>> TryFrom<Record> for D {
//     type Error = postcard::Error;

//     fn try_from(value: Record) -> Result<Self, Self::Error> {
//         let def: D = postcard::from_bytes::<D>(&value.value)?;
//         Ok(def)
//     }
// }
