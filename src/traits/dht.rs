use std::borrow::Cow;

#[cfg(feature = "libp2p")]
use libp2p::{
    Multiaddr, PeerId,
    kad::{ProviderRecord, Record, RecordKey},
};

use crate::{
    error::EncodingDecodingError,
    traits::{
        definition::{NetabaseDefinition, NetabaseDefinitionKeys},
        store::Store,
    },
};

/// Trait for NetabaseDefinition types that can be converted to/from libp2p Records
#[cfg(feature = "libp2p")]
pub trait KademliaRecord: NetabaseDefinition + bincode::Encode + bincode::Decode<()> {
    type NetabaseRecordKey: KademliaRecordKey;

    fn record_keys(&self) -> Self::NetabaseRecordKey;

    fn try_to_vec(&self) -> Result<Vec<u8>, bincode::error::EncodeError> {
        bincode::encode_to_vec(self, bincode::config::standard())
    }

    fn try_from_vec<V: AsRef<[u8]>>(vec: V) -> Result<Self, bincode::error::DecodeError> {
        Ok(bincode::decode_from_slice(vec.as_ref(), bincode::config::standard())?.0)
    }

    fn try_to_record(&self) -> Result<Record, EncodingDecodingError> {
        Ok(Record {
            key: self.record_keys().try_to_record_key()?,
            value: self.try_to_vec()?,
            publisher: None,
            expires: None,
        })
    }

    fn try_from_record(record: Record) -> Result<Self, EncodingDecodingError> {
        Ok(Self::try_from_vec(record.value)?)
    }
}

/// Trait for NetabaseDefinitionKeys that can be converted to/from libp2p RecordKeys
#[cfg(feature = "libp2p")]
pub trait KademliaRecordKey:
    NetabaseDefinitionKeys + bincode::Encode + bincode::Decode<()>
{
    fn try_to_vec(&self) -> Result<Vec<u8>, bincode::error::EncodeError> {
        bincode::encode_to_vec(self, bincode::config::standard())
    }

    fn try_from_vec<V: AsRef<[u8]>>(vec: V) -> Result<Self, bincode::error::DecodeError> {
        Ok(bincode::decode_from_slice(vec.as_ref(), bincode::config::standard())?.0)
    }

    fn try_to_record_key(&self) -> Result<RecordKey, EncodingDecodingError> {
        Ok(RecordKey::new(&self.try_to_vec()?))
    }

    fn try_from_record_key(key: &RecordKey) -> Result<Self, EncodingDecodingError> {
        Ok(Self::try_from_vec(key.as_ref())?)
    }
}

/// Helper functions for ProviderRecord management in sled database
#[cfg(feature = "libp2p")]
pub mod provider_record_helpers {
    use super::*;
    use sled::{IVec, Tree};

    /// Helper struct for storing provider information in the value
    #[derive(Debug, Clone, bincode::Encode, bincode::Decode)]
    pub struct ProviderInfo {
        pub provider: Vec<u8>,       // PeerId bytes
        pub addresses: Vec<Vec<u8>>, // Multiaddr bytes
    }

    /// Creates a ProviderRecord from raw sled key-value data
    ///
    /// Expected key format: RecordKey bytes
    /// Expected value format: serialized ProviderInfo
    pub fn ivec_to_provider_record(
        key: &IVec,
        value: &IVec,
    ) -> Result<ProviderRecord, EncodingDecodingError> {
        let record_key = RecordKey::new(key);

        // Decode ProviderInfo from value
        let provider_info: ProviderInfo =
            bincode::decode_from_slice(value.as_ref(), bincode::config::standard())?.0;

        let provider = PeerId::from_bytes(&provider_info.provider)
            .map_err(|_| EncodingDecodingError::InvalidPeerId)?;

        let addresses: Vec<Multiaddr> = provider_info
            .addresses
            .into_iter()
            .filter_map(|bytes| Multiaddr::try_from(bytes).ok())
            .collect();

        Ok(ProviderRecord {
            key: record_key,
            provider,
            expires: None,
            addresses,
        })
    }

    /// Converts a ProviderRecord to sled key-value format
    ///
    /// Returns (key, value) where:
    /// - key: RecordKey bytes
    /// - value: serialized ProviderInfo
    pub fn provider_record_to_ivec(
        record: &ProviderRecord,
    ) -> Result<(IVec, IVec), EncodingDecodingError> {
        let key_bytes = record.key.to_vec();

        // Create ProviderInfo with PeerId and addresses
        let provider_info = ProviderInfo {
            provider: record.provider.to_bytes(),
            addresses: record.addresses.iter().map(|addr| addr.to_vec()).collect(),
        };

        let value_bytes = bincode::encode_to_vec(&provider_info, bincode::config::standard())?;

        Ok((key_bytes.into(), value_bytes.into()))
    }

    /// Iterator adapter for converting sled Tree iterator to ProviderRecord iterator
    pub struct ProviderRecordIter<I> {
        inner: I,
    }

    impl<I> ProviderRecordIter<I>
    where
        I: Iterator<Item = Result<(IVec, IVec), sled::Error>>,
    {
        pub fn new(inner: I) -> Self {
            Self { inner }
        }
    }

    impl<I> Iterator for ProviderRecordIter<I>
    where
        I: Iterator<Item = Result<(IVec, IVec), sled::Error>>,
    {
        type Item = Cow<'static, ProviderRecord>;

        fn next(&mut self) -> Option<Self::Item> {
            self.inner.next().and_then(|res| {
                match res {
                    Ok((key, value)) => {
                        match ivec_to_provider_record(&key, &value) {
                            Ok(provider_record) => Some(Cow::Owned(provider_record)),
                            Err(_) => None, // Skip invalid records
                        }
                    }
                    Err(_) => None, // Skip errors
                }
            })
        }
    }

    /// Creates a specialized ProviderRecord tree for efficient lookups
    pub fn create_provider_record_tree(
        store: &sled::Db,
        tree_name: &str,
    ) -> Result<Tree, sled::Error> {
        store.open_tree(tree_name)
    }

    /// Helper to extract all ProviderRecords for a given RecordKey from the provider tree
    pub fn get_providers_for_key(
        provider_tree: &Tree,
        record_key: &RecordKey,
    ) -> Result<Vec<ProviderRecord>, EncodingDecodingError> {
        let key_bytes = record_key.to_vec();
        let mut providers = Vec::new();

        // Get all values for this exact key
        if let Ok(Some(value)) = provider_tree.get(&key_bytes) {
            match ivec_to_provider_record(&key_bytes.into(), &value) {
                Ok(provider_record) => providers.push(provider_record),
                Err(_) => {} // Skip invalid records
            }
        }

        Ok(providers)
    }

    /// Helper to add a provider to an existing key or create a new entry
    pub fn add_provider_to_key(
        provider_tree: &Tree,
        provider_record: &ProviderRecord,
    ) -> Result<(), EncodingDecodingError> {
        let key_bytes = provider_record.key.to_vec();

        // For now, we'll store one provider per key
        // In a full implementation, you might want to store multiple providers per key
        let (_, value) = provider_record_to_ivec(provider_record)?;
        provider_tree.insert(key_bytes, value).map_err(|_| {
            EncodingDecodingError::Encoding(bincode::error::EncodeError::UnexpectedEnd)
        })?;

        Ok(())
    }

    /// Helper to remove a specific provider from a key
    pub fn remove_provider_from_key(
        provider_tree: &Tree,
        record_key: &RecordKey,
        peer_id: &PeerId,
    ) -> Result<bool, EncodingDecodingError> {
        let key_bytes = record_key.to_vec();

        if let Ok(Some(value)) = provider_tree.get(&key_bytes) {
            if let Ok(existing_record) = ivec_to_provider_record(&key_bytes.clone().into(), &value)
            {
                // If this is the provider we want to remove, delete the entry
                if existing_record.provider == *peer_id {
                    provider_tree.remove(&key_bytes).map_err(|_| {
                        EncodingDecodingError::Encoding(bincode::error::EncodeError::UnexpectedEnd)
                    })?;
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
}

/// Record iteration helpers for RecordStore implementation
#[cfg(feature = "libp2p")]
pub mod record_helpers {
    use super::*;

    /// Placeholder iterator for Records - converts from NetabaseStore iterator to libp2p Records
    /// This should be implemented based on your specific Store trait
    pub fn create_record_iter<'a, D, S>(_store: &'a S) -> impl Iterator<Item = Cow<'a, Record>> + 'a
    where
        D: NetabaseDefinition + KademliaRecord,
        S: Store<D>,
    {
        // TODO: Implement this iterator by:
        // 1. Iterating over all trees in the store
        // 2. Converting each NetabaseModel to NetabaseDefinition
        // 3. Converting NetabaseDefinition to Record via KademliaRecord trait
        // 4. Wrapping in Cow::Owned(record)

        std::iter::empty() // Placeholder - replace with actual implementation
    }

    /// Helper function to create the iterator - placeholder for now
    pub fn iter<'a, D, S>(_store: &'a S) -> RecordIter<'a, D, S>
    where
        D: NetabaseDefinition + KademliaRecord,
        S: Store<D>,
    {
        todo!("Implement record iteration over NetabaseStore")
    }

    /// Iterator wrapper for converting store data to Records
    pub struct RecordIter<'a, D, S>
    where
        D: NetabaseDefinition + KademliaRecord,
        S: Store<D>,
    {
        _store: &'a S,
        _phantom: std::marker::PhantomData<D>,
    }

    impl<'a, D, S> Iterator for RecordIter<'a, D, S>
    where
        D: NetabaseDefinition + KademliaRecord,
        S: Store<D>,
    {
        type Item = Cow<'a, Record>;

        fn next(&mut self) -> Option<Self::Item> {
            todo!("Implement record iteration over NetabaseStore")
        }
    }
}
