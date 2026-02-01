use crate::traits::database::hash::HashAlgorithm;
use serde::{Serialize, Deserialize};

/// Trait for content-addressed models.
///
/// This trait is implemented by models that are identified by the hash of their content.
/// It provides the mechanism to compute the hash using a specific algorithm.
pub trait ContentAddressedModel {
    /// The hash algorithm used to compute the content address.
    type Hasher: HashAlgorithm;
    
    /// The key type produced by the hash (e.g., [u8; 32] or u64)
    type Key: Clone + Ord + std::hash::Hash + std::fmt::Debug + Send + Sync + 'static;

    /// Compute the hash of the model.
    ///
    /// This method serializes the model (typically excluding any transient fields)
    /// and computes the hash using the specified `Hasher`.
    fn compute_hash(&self) -> Self::Key;
}

/// Envelope for content-addressed models.
///
/// Stores the model data along with its pre-computed hash.
/// This structure is used for serialization to ensure the hash is preserved
/// and available for verification without re-computation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentAddressed<T, H> {
    pub hash: H,
    pub inner: T,
}

impl<T, H> ContentAddressed<T, H> {
    pub fn new(inner: T, hash: H) -> Self {
        Self { hash, inner }
    }
}
