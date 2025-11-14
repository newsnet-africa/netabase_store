//! RedbStore implementation of libp2p Kademlia RecordStore

use super::{PROVIDED_TREE_NAME, PROVIDER_TREE_NAME, RecordStoreConfig, utils};
use crate::databases::redb_store::RedbStore;
use crate::traits::definition::NetabaseDefinitionTrait;
use libp2p::PeerId;
use libp2p::kad::store::{Error, Result};
use libp2p::kad::{ProviderRecord, RecordKey as Key};
use redb::{ReadableDatabase, ReadableTable, ReadableTableMetadata, TableDefinition};
use std::str::FromStr;
use strum::IntoDiscriminant;

impl<D> RedbStore<D>
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

    /// Get table definition for a discriminant
    fn table_def_for_disc(
        disc: &D::Discriminant,
    ) -> TableDefinition<'static, &'static [u8], &'static [u8]> {
        let table_name = disc.to_string();
        let static_name: &'static str = Box::leak(table_name.into_boxed_str());
        TableDefinition::new(static_name)
    }

    /// Get the libp2p providers table definition
    fn providers_table_def() -> TableDefinition<'static, &'static [u8], &'static [u8]> {
        TableDefinition::new(PROVIDER_TREE_NAME)
    }

    /// Get the provided records table definition (records provided by this peer)
    fn provided_table_def() -> TableDefinition<'static, &'static [u8], &'static [u8]> {
        TableDefinition::new(PROVIDED_TREE_NAME)
    }

    /// Get the count of records in the store (across all tables)
    fn record_count(&self) -> Result<usize> {
        use strum::IntoEnumIterator;

        let read_txn = self.db.begin_read().map_err(|_| Error::MaxRecords)?;
        let mut count = 0;

        for disc in D::Discriminant::iter() {
            let table_def = Self::table_def_for_disc(&disc);
            if let Ok(table) = read_txn.open_table(table_def) {
                count += table.len().map_err(|_| Error::MaxRecords)? as usize;
            }
        }

        Ok(count)
    }

    /// Get providers for a key (internal helper)
    fn get_providers_for_key(&self, key: &Key) -> Result<Vec<ProviderRecord>> {
        let key_bytes = utils::encode_key(key);
        let mut providers = Vec::new();

        let read_txn = self.db.begin_read().map_err(|_| Error::MaxRecords)?;
        let table_def = Self::providers_table_def();

        let table = match read_txn.open_table(table_def) {
            Ok(t) => t,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(providers),
            Err(_) => return Err(Error::MaxRecords),
        };

        // Iterate through all entries and filter by prefix
        for item in table.iter().map_err(|_| Error::MaxRecords)? {
            let (k, v) = item.map_err(|_| Error::MaxRecords)?;
            let k_bytes = k.value();

            // Check if this key starts with our search key
            if k_bytes.starts_with(&key_bytes) {
                providers.push(utils::decode_provider(v.value())?);
            }
        }

        Ok(providers)
    }

    /// Create a composite key for provider records (key + provider_id)
    fn provider_composite_key(key: &Key, provider: &PeerId) -> Vec<u8> {
        let mut composite = utils::encode_key(key);
        composite.extend_from_slice(&provider.to_bytes());
        composite
    }

    /// Add a provider record (internal helper for generated RecordStore impl)
    pub fn add_provider_internal(&mut self, record: ProviderRecord) -> Result<()> {
        let write_txn = self.db.begin_write().map_err(|_| Error::MaxRecords)?;
        let composite_key = Self::provider_composite_key(&record.key, &record.provider);
        let value_bytes = utils::encode_provider(&record)?;

        {
            let mut table = write_txn
                .open_table(Self::providers_table_def())
                .map_err(|_| Error::MaxRecords)?;
            table
                .insert(composite_key.as_slice(), value_bytes.as_slice())
                .map_err(|_| Error::MaxRecords)?;
        }

        // Also track in provided tree
        {
            let mut provided_table = write_txn
                .open_table(Self::provided_table_def())
                .map_err(|_| Error::MaxRecords)?;
            let key_bytes = utils::encode_key(&record.key);
            provided_table
                .insert(key_bytes.as_slice(), value_bytes.as_slice())
                .map_err(|_| Error::MaxRecords)?;
        }

        write_txn.commit().map_err(|_| Error::MaxRecords)?;
        Ok(())
    }

    /// Get providers for a key (internal helper for generated RecordStore impl)
    pub fn providers_internal(&self, key: &Key) -> Result<Vec<ProviderRecord>> {
        self.get_providers_for_key(key)
    }

    /// Remove a provider record (internal helper for generated RecordStore impl)
    pub fn remove_provider_internal(&mut self, key: &Key, provider: &PeerId) {
        if let Ok(write_txn) = self.db.begin_write() {
            let composite_key = Self::provider_composite_key(key, provider);
            if let Ok(mut table) = write_txn.open_table(Self::providers_table_def()) {
                let _ = table.remove(composite_key.as_slice());
            }
            let _ = write_txn.commit();
        }
        // Note: We don't remove from provided_table here because other peers may still provide this key
    }
}
