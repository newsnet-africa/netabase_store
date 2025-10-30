#[cfg(feature = "paxos")]
use paxakos::LogEntry;
use std::str::FromStr;

use crate::{
    MaybeSend, MaybeSync,
    model::DynModel,
};
use strum::IntoDiscriminant;

/// Trait for the module-level definition enum that wraps all models.
///
/// This trait is automatically implemented by the `#[netabase_definition_module]` macro.
/// The definition enum is used as the primary type for encoding, decoding, and moving
/// model data around.
#[cfg(feature = "paxos")]
pub trait NetabaseDefinitionTrait:
    bincode::Encode
    + bincode::Decode<()>
    + Clone
    + std::fmt::Debug
    + MaybeSend
    + MaybeSync
    + 'static
    + IntoDiscriminant
    + DynDefinition
    + LogEntry
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
        + MaybeSend
        + MaybeSync
        + 'static
        + FromStr,
    <Self as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <Self as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <Self as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <Self as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <Self as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <Self as strum::IntoDiscriminant>::Discriminant: FromStr,
    <Self as strum::IntoDiscriminant>::Discriminant: MaybeSync,
    <Self as strum::IntoDiscriminant>::Discriminant: MaybeSend,
    <Self as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <<Self as NetabaseDefinitionTrait>::Keys as IntoDiscriminant>::Discriminant: Clone
        + Copy
        + std::fmt::Debug
        + PartialEq
        + Eq
        + std::hash::Hash
        + MaybeSend
        + MaybeSync
        + 'static,
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

    /// Apply this definition entry to a store (for Paxos consensus)
    ///
    /// This method is implemented by the netabase_definition_module macro and routes
    /// each Definition variant to the appropriate store operation.
    ///
    /// # Phase 3 Integration
    /// This trait method declaration makes apply_to_store available on the generic
    /// NetabaseDefinitionTrait bound, while the macro provides the actual implementation.
    #[cfg(all(feature = "paxos", feature = "libp2p"))]
    fn apply_to_store<S>(&self, store: &mut S) -> Result<(), String>
    where
        S: libp2p::kad::store::RecordStore;
}

#[cfg(not(feature = "paxos"))]
pub trait NetabaseDefinitionTrait:
    bincode::Encode
    + bincode::Decode<()>
    + Clone
    + std::fmt::Debug
    + MaybeSend
    + MaybeSync
    + 'static
    + IntoDiscriminant
    + DynDefinition
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
        + MaybeSend
        + MaybeSync
        + 'static
        + FromStr,
    <Self as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <Self as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <Self as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <Self as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <Self as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <Self as strum::IntoDiscriminant>::Discriminant: FromStr,
    <Self as strum::IntoDiscriminant>::Discriminant: MaybeSync,
    <Self as strum::IntoDiscriminant>::Discriminant: MaybeSend,
    <Self as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <<Self as NetabaseDefinitionTrait>::Keys as IntoDiscriminant>::Discriminant: Clone
        + Copy
        + std::fmt::Debug
        + PartialEq
        + Eq
        + std::hash::Hash
        + MaybeSend
        + MaybeSync
        + 'static,
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

pub trait DynDefinition {
    fn inner(&self) -> &dyn DynModel;
}

/// Trait for the module-level keys enum that wraps all model keys.
///
/// This trait is automatically implemented by the `#[netabase_definition_module]` macro.
pub trait NetabaseDefinitionTraitKey:
    bincode::Encode
    + bincode::Decode<()>
    + DynDefinition
    + Clone
    + std::fmt::Debug
    + PartialEq
    + Eq
    + std::hash::Hash
    + PartialOrd
    + Ord
    + MaybeSend
    + MaybeSync
    + 'static
    + strum::IntoDiscriminant
where
    <Self as IntoDiscriminant>::Discriminant: Clone
        + Copy
        + std::fmt::Debug
        + PartialEq
        + Eq
        + std::hash::Hash
        + MaybeSend
        + MaybeSync
        + 'static,
{
    /// Convert this key to a libp2p kad::RecordKey
    #[cfg(feature = "libp2p")]
    fn to_record_key(&self) -> Result<libp2p::kad::RecordKey, bincode::error::EncodeError> {
        let key_bytes = bincode::encode_to_vec(self, bincode::config::standard())?;
        Ok(libp2p::kad::RecordKey::new(&key_bytes))
    }
}
