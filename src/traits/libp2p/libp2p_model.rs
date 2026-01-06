use serde::{Serialize, Deserialize};

/// Metadata for Libp2p integration injected into models.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default, Hash, PartialOrd, Ord)]
pub struct Libp2pMetadata {
    /// Placeholder for future metadata extensions
    pub extra: Option<Vec<u8>>,
}

/// Trait for models that support Libp2p functionality.
/// This is automatically implemented for all models by the macro.
pub trait Libp2pModel: Serialize + for<'a> Deserialize<'a> {
    fn get_libp2p_metadata(&self) -> Option<&Libp2pMetadata>;
}
