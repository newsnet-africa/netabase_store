//! SledStore implementation of libp2p Kademlia RecordStore

use super::{PROVIDED_TREE_NAME, PROVIDER_TREE_NAME, RecordStoreConfig, utils};
use crate::databases::sled_store::SledStore;
use crate::traits::definition::NetabaseDefinitionTrait;
use libp2p::PeerId;
use libp2p::kad::store::{Error, Result};
use libp2p::kad::{ProviderRecord, RecordKey as Key};
use std::str::FromStr;
use strum::IntoDiscriminant;

impl<D> SledStore<D>
where
    D: NetabaseDefinitionTrait,
    <D as IntoDiscriminant>::Discriminant: AsRef<str>
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
{
    /// Get the configuration for the record store
    pub fn record_store_config(&self) -> RecordStoreConfig {
        RecordStoreConfig::default()
    }

    /// Get the libp2p providers tree
    fn providers_tree(&self) -> sled::Tree {
        self.db()
            .open_tree(PROVIDER_TREE_NAME)
            .expect("Failed to open providers tree")
    }

    /// Get the provided records tree (records provided by this peer)
    fn provided_tree(&self) -> sled::Tree {
        self.db()
            .open_tree(PROVIDED_TREE_NAME)
            .expect("Failed to open provided tree")
    }

    /// Create a composite key for provider records (key + provider_id)
    fn provider_composite_key(key: &Key, provider: &PeerId) -> Vec<u8> {
        let mut composite = utils::encode_key(key);
        composite.extend_from_slice(&provider.to_bytes());
        composite
    }

    /// Get the count of records in the store (across all trees)
    fn record_count(&self) -> usize {
        use strum::{IntoDiscriminant, IntoEnumIterator};

        <<D as IntoDiscriminant>::Discriminant as IntoEnumIterator>::iter()
            .filter_map(|disc| self.db().open_tree(disc.to_string()).ok())
            .map(|tree| tree.len())
            .sum()
    }

    /// Get providers for a key (internal helper)
    fn get_providers_for_key(&self, key: &Key) -> Result<Vec<ProviderRecord>> {
        let tree = self.providers_tree();
        let key_bytes = utils::encode_key(key);
        let mut providers = Vec::new();

        for result in tree.scan_prefix(&key_bytes) {
            let (_, value) = result.map_err(|_| Error::MaxRecords)?;
            providers.push(utils::decode_provider(&value)?);
        }

        Ok(providers)
    }

    /// Add a provider record (internal helper for generated RecordStore impl)
    pub(crate) fn add_provider_internal(&mut self, record: ProviderRecord) -> Result<()> {
        let tree = self.providers_tree();
        let composite_key = Self::provider_composite_key(&record.key, &record.provider);
        let value_bytes = utils::encode_provider(&record)?;

        tree.insert(composite_key, value_bytes.clone())
            .map_err(|_| Error::MaxRecords)?;

        // Also track in provided tree
        let provided_tree = self.provided_tree();
        let key_bytes = utils::encode_key(&record.key);
        provided_tree.insert(key_bytes, value_bytes)
            .map_err(|_| Error::MaxRecords)?;

        Ok(())
    }

    /// Get providers for a key (internal helper for generated RecordStore impl)
    pub(crate) fn providers_internal(&self, key: &Key) -> Result<Vec<ProviderRecord>> {
        self.get_providers_for_key(key)
    }

    /// Remove a provider record (internal helper for generated RecordStore impl)
    pub(crate) fn remove_provider_internal(&mut self, key: &Key, provider: &PeerId) {
        let tree = self.providers_tree();
        let composite_key = Self::provider_composite_key(key, provider);
        let _ = tree.remove(composite_key);

        // Note: We don't remove from provided_tree here because other peers may still provide this key
    }
}