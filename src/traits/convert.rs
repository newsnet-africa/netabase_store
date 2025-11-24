use crate::error::{EncodingDecodingError, NetabaseError};
use bincode::{config::standard, decode_from_slice, encode_to_vec};

/// Trait for types that can be converted to/from byte vectors.
///
/// Only NetabaseDefinitionTrait and NetabaseDefinitionTraitKey should implement this trait.
/// This ensures type safety by preventing direct byte manipulation at lower levels.
pub trait ToIVec: bincode::Encode + bincode::Decode<()> + Sized {
    /// Convert this type to a sled::IVec (available when sled or native feature is enabled)
    #[cfg(all(feature = "sled", not(feature = "wasm")))]
    fn to_ivec(&self) -> Result<sled::IVec, NetabaseError> {
        let bytes = encode_to_vec(self, standard()).map_err(EncodingDecodingError::from)?;
        Ok(sled::IVec::from(bytes))
    }

    /// Convert from a sled::IVec to this type (available when sled or native feature is enabled)
    #[cfg(all(feature = "sled", not(feature = "wasm")))]
    fn from_ivec(ivec: &sled::IVec) -> Result<Self, NetabaseError> {
        let (decoded, _) =
            decode_from_slice(&ivec[..], standard()).map_err(EncodingDecodingError::from)?;
        Ok(decoded)
    }

    /// Convert this type to bytes (WASM version)
    #[cfg(all(feature = "wasm", not(feature = "sled")))]
    fn to_vec(&self) -> Result<Vec<u8>, NetabaseError> {
        let bytes = encode_to_vec(self, standard()).map_err(EncodingDecodingError::from)?;
        Ok(bytes)
    }

    /// Convert from bytes to this type (WASM version)
    #[cfg(all(feature = "wasm", not(feature = "sled")))]
    fn from_vec(bytes: &[u8]) -> Result<Self, NetabaseError> {
        let (decoded, _) =
            decode_from_slice(bytes, standard()).map_err(EncodingDecodingError::from)?;
        Ok(decoded)
    }

    /// Convert this type to a sled::IVec (when both sled and wasm features are enabled, prefer sled)
    #[cfg(all(feature = "sled", feature = "wasm"))]
    fn to_ivec(&self) -> Result<sled::IVec, NetabaseError> {
        let bytes = encode_to_vec(self, standard()).map_err(EncodingDecodingError::from)?;
        Ok(sled::IVec::from(bytes))
    }

    /// Convert from a sled::IVec to this type (when both sled and wasm features are enabled, prefer sled)
    #[cfg(all(feature = "sled", feature = "wasm"))]
    fn from_ivec(ivec: &sled::IVec) -> Result<Self, NetabaseError> {
        let (decoded, _) =
            decode_from_slice(&ivec[..], standard()).map_err(EncodingDecodingError::from)?;
        Ok(decoded)
    }
}
