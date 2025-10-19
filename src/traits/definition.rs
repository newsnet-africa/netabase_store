use std::str::FromStr;

use strum::IntoDiscriminant;

/// Trait for the module-level definition enum that wraps all models.
///
/// This trait is automatically implemented by the `#[netabase_definition_module]` macro.
/// The definition enum is used as the primary type for encoding, decoding, and moving
/// model data around.
pub trait NetabaseDefinitionTrait:
    bincode::Encode
    + bincode::Decode<()>
    + Clone
    + std::fmt::Debug
    + Sized
    + Send
    + Sync
    + 'static
    + IntoDiscriminant
where
    <Self as IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + FromStr,
    <Self as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <Self as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <Self as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <Self as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <Self as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <Self as strum::IntoDiscriminant>::Discriminant: FromStr,
    <Self as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <Self as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
    <Self as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <<Self as NetabaseDefinitionTrait>::Keys as IntoDiscriminant>::Discriminant:
        Clone + Copy + std::fmt::Debug + PartialEq + Eq + std::hash::Hash + Send + Sync + 'static,
{
    type Keys: NetabaseDefinitionTraitKey;
    /// Get the discriminant name as a string (for tree names)
    fn discriminant_name(&self) -> String {
        self.discriminant().to_string()
    }

    /// Convert this definition to a libp2p kad::Record
    #[cfg(feature = "libp2p")]
    fn to_record(&self) -> Result<libp2p::kad::Record, bincode::error::EncodeError> {
        let key_bytes = bincode::encode_to_vec(self, bincode::config::standard())?;
        let record_key = libp2p::kad::RecordKey::new(&key_bytes);
        let value_bytes = bincode::encode_to_vec(self, bincode::config::standard())?;

        Ok(libp2p::kad::Record {
            key: record_key,
            value: value_bytes,
            publisher: None,
            expires: None,
        })
    }
}

/// Trait for the module-level keys enum that wraps all model keys.
///
/// This trait is automatically implemented by the `#[netabase_definition_module]` macro.
pub trait NetabaseDefinitionTraitKey:
    bincode::Encode
    + bincode::Decode<()>
    + Clone
    + std::fmt::Debug
    + Sized
    + Send
    + Sync
    + 'static
    + strum::IntoDiscriminant
where
    <Self as IntoDiscriminant>::Discriminant:
        Clone + Copy + std::fmt::Debug + PartialEq + Eq + std::hash::Hash + Send + Sync + 'static,
{
    /// Convert this key to a libp2p kad::RecordKey
    #[cfg(feature = "libp2p")]
    fn to_record_key(&self) -> Result<libp2p::kad::RecordKey, bincode::error::EncodeError> {
        let key_bytes = bincode::encode_to_vec(self, bincode::config::standard())?;
        Ok(libp2p::kad::RecordKey::new(&key_bytes))
    }
}
