#[cfg(all(feature = "libp2p", feature = "redb"))]
use crate::databases::redb_store::RedbStore;
#[cfg(all(feature = "libp2p", feature = "sled"))]
use crate::databases::sled_store::SledStore;
#[cfg(feature = "libp2p")]
use crate::traits::definition::NetabaseDefinitionTrait;
#[cfg(feature = "libp2p")]
use libp2p::PeerId;
#[cfg(feature = "libp2p")]
use libp2p::kad::store::{Error, RecordStore, Result};
#[cfg(feature = "libp2p")]
use libp2p::kad::{ProviderRecord, Record, RecordKey as Key};
#[cfg(all(feature = "libp2p", feature = "redb"))]
use redb::{ReadableDatabase, ReadableTable, ReadableTableMetadata};
#[cfg(feature = "libp2p")]
use std::borrow::Cow;
#[cfg(feature = "libp2p")]
use std::str::FromStr;
#[cfg(feature = "libp2p")]
use strum::IntoDiscriminant;

/// Serializable version of libp2p::kad::Record
/// Note: expires is always None since Instant is not serializable
#[cfg(feature = "libp2p")]
#[derive(Debug, Clone, bincode::Encode, bincode::Decode)]
struct SerializableRecord {
    key: Vec<u8>,
    value: Vec<u8>,
    publisher: Option<Vec<u8>>,
    // expires is always None - we don't persist expiration times
}

/// Serializable version of libp2p::kad::ProviderRecord
/// Note: expires is always None since Instant is not serializable
#[cfg(feature = "libp2p")]
#[derive(Debug, Clone, bincode::Encode, bincode::Decode)]
struct SerializableProviderRecord {
    key: Vec<u8>,
    provider: Vec<u8>,
    // expires is always None - we don't persist expiration times
    addresses: Vec<Vec<u8>>,
}

#[cfg(feature = "libp2p")]
const PROVIDER_TREE_NAME: &str = "__libp2p_providers";
#[cfg(feature = "libp2p")]
const MAX_RECORDS: usize = 1024;
#[cfg(feature = "libp2p")]
const MAX_VALUE_BYTES: usize = 65 * 1024; // 65 KB
#[cfg(feature = "libp2p")]
const MAX_PROVIDERS_PER_KEY: usize = 20;
#[cfg(feature = "libp2p")]
const MAX_PROVIDED_KEYS: usize = 1024;

/// Configuration for the RecordStore implementation
#[cfg(feature = "libp2p")]
pub struct RecordStoreConfig {
    pub max_records: usize,
    pub max_value_bytes: usize,
    pub max_providers_per_key: usize,
    pub max_provided_keys: usize,
}

#[cfg(feature = "libp2p")]
impl Default for RecordStoreConfig {
    fn default() -> Self {
        Self {
            max_records: MAX_RECORDS,
            max_value_bytes: MAX_VALUE_BYTES,
            max_providers_per_key: MAX_PROVIDERS_PER_KEY,
            max_provided_keys: MAX_PROVIDED_KEYS,
        }
    }
}

#[cfg(all(feature = "libp2p", feature = "sled"))]
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

    /// Get the tree for a given Record by decoding the value's discriminant
    fn tree_for_record(&self, record: &Record) -> Result<sled::Tree> {
        // Decode the Record value to get the NetabaseDefinitionTrait
        let (definition, _): (D, _) =
            bincode::decode_from_slice(&record.value, bincode::config::standard())
                .map_err(|_| Error::MaxRecords)?;

        // Get the discriminant value to use as the tree name
        let tree_name = definition.discriminant_name();

        // Open the appropriate tree
        self.db()
            .open_tree(tree_name)
            .map_err(|_| Error::MaxRecords)
    }

    /// Get the tree for a given RecordKey by trying all Definition trees
    /// This is less efficient but necessary for get/remove operations where we only have the key
    fn tree_for_key(&self, key: &Key) -> Result<sled::Tree> {
        use strum::IntoEnumIterator;

        let key_bytes = Self::encode_key(key);

        // Try each tree to find which one contains this key
        for disc in D::Discriminant::iter() {
            if let Ok(tree) = self.db().open_tree(disc.to_string())
                && tree.contains_key(&key_bytes).unwrap_or(false)
            {
                return Ok(tree);
            }
        }

        // If not found in any tree, return the first tree (for new inserts)
        let first_disc = D::Discriminant::iter().next().ok_or(Error::MaxRecords)?;
        self.db()
            .open_tree(first_disc.to_string())
            .map_err(|_| Error::MaxRecords)
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
            .open_tree("__libp2p_provided")
            .expect("Failed to open provided tree")
    }

    /// Encode a Key to bytes
    fn encode_key(key: &Key) -> Vec<u8> {
        key.to_vec()
    }

    /// Encode a Record to bytes using SerializableRecord
    fn encode_record(record: &Record) -> Result<Vec<u8>> {
        let serializable = SerializableRecord {
            key: record.key.to_vec(),
            value: record.value.clone(),
            publisher: record.publisher.as_ref().map(|p| p.to_bytes()),
        };
        bincode::encode_to_vec(&serializable, bincode::config::standard())
            .map_err(|_| Error::ValueTooLarge)
    }

    /// Decode a Record from bytes using SerializableRecord
    fn decode_record(bytes: &[u8]) -> Result<Record> {
        let (serializable, _): (SerializableRecord, _) =
            bincode::decode_from_slice(bytes, bincode::config::standard())
                .map_err(|_| Error::MaxRecords)?;

        let publisher = match serializable.publisher {
            Some(bytes) => Some(PeerId::from_bytes(&bytes).map_err(|_| Error::MaxRecords)?),
            None => None,
        };

        Ok(Record {
            key: Key::from(serializable.key),
            value: serializable.value,
            publisher,
            expires: None, // Always None - we don't persist expiration times
        })
    }

    /// Encode a ProviderRecord to bytes using SerializableProviderRecord
    fn encode_provider(provider: &ProviderRecord) -> Result<Vec<u8>> {
        let serializable = SerializableProviderRecord {
            key: provider.key.to_vec(),
            provider: provider.provider.to_bytes(),
            addresses: provider.addresses.iter().map(|a| a.to_vec()).collect(),
        };
        bincode::encode_to_vec(&serializable, bincode::config::standard())
            .map_err(|_| Error::ValueTooLarge)
    }

    /// Decode a ProviderRecord from bytes using SerializableProviderRecord
    fn decode_provider(bytes: &[u8]) -> Result<ProviderRecord> {
        let (serializable, _): (SerializableProviderRecord, _) =
            bincode::decode_from_slice(bytes, bincode::config::standard())
                .map_err(|_| Error::MaxRecords)?;

        let provider = PeerId::from_bytes(&serializable.provider).map_err(|_| Error::MaxRecords)?;

        let addresses = serializable
            .addresses
            .iter()
            .filter_map(|bytes| libp2p::Multiaddr::try_from(bytes.clone()).ok())
            .collect();

        Ok(ProviderRecord {
            key: Key::from(serializable.key),
            provider,
            expires: None, // Always None - we don't persist expiration times
            addresses,
        })
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
        let key_bytes = Self::encode_key(key);
        let mut providers = Vec::new();

        for result in tree.scan_prefix(&key_bytes) {
            let (_, value) = result.map_err(|_| Error::MaxRecords)?;
            providers.push(Self::decode_provider(&value)?);
        }

        Ok(providers)
    }

    /// Create a composite key for provider records (key + provider_id)
    fn provider_composite_key(key: &Key, provider: &PeerId) -> Vec<u8> {
        let mut composite = Self::encode_key(key);
        composite.extend_from_slice(&provider.to_bytes());
        composite
    }
}

#[cfg(all(feature = "libp2p", feature = "sled"))]
impl<D> RecordStore for SledStore<D>
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
    type RecordsIter<'a>
        = RecordsIter<'a>
    where
        Self: 'a;
    type ProvidedIter<'a>
        = ProvidedIter<'a>
    where
        Self: 'a;

    fn get(&self, k: &Key) -> Option<Cow<'_, Record>> {
        let tree = self.tree_for_key(k).ok()?;
        let key_bytes = Self::encode_key(k);

        tree.get(key_bytes)
            .ok()
            .flatten()
            .and_then(|bytes| Self::decode_record(&bytes).ok())
            .map(Cow::Owned)
    }

    fn put(&mut self, r: Record) -> Result<()> {
        let config = self.record_store_config();

        if r.value.len() >= config.max_value_bytes {
            return Err(Error::ValueTooLarge);
        }

        let tree = self.tree_for_record(&r)?;
        let key_bytes = Self::encode_key(&r.key);
        let record_bytes = Self::encode_record(&r)?;

        // Check if we're at capacity and this is a new record
        if tree
            .get(&key_bytes)
            .map_err(|_| Error::MaxRecords)?
            .is_none()
            && self.record_count() >= config.max_records {
                return Err(Error::MaxRecords);
            }

        tree.insert(key_bytes, record_bytes)
            .map_err(|_| Error::MaxRecords)?;

        Ok(())
    }

    fn remove(&mut self, k: &Key) {
        if let Ok(tree) = self.tree_for_key(k) {
            let key_bytes = Self::encode_key(k);
            let _ = tree.remove(key_bytes);
        }
    }

    fn records(&self) -> Self::RecordsIter<'_> {
        use strum::IntoEnumIterator;

        // Collect all tree iterators
        let tree_iters: Vec<sled::Iter> = D::Discriminant::iter()
            .filter_map(|disc| self.db().open_tree(disc.to_string()).ok())
            .map(|tree| tree.iter())
            .collect();

        RecordsIter {
            tree_iters,
            current_index: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    fn add_provider(&mut self, record: ProviderRecord) -> Result<()> {
        let config = self.record_store_config();
        let providers_tree = self.providers_tree();
        let provided_tree = self.provided_tree();

        // Get current providers for this key
        let providers = self.get_providers_for_key(&record.key)?;

        // Check if we need to create a new key entry
        if providers.is_empty() {
            // Count unique keys
            let unique_keys: std::collections::HashSet<Vec<u8>> = providers_tree
                .iter()
                .filter_map(|r| r.ok())
                .map(|(k, _)| {
                    // Extract just the key part (without provider ID)
                    let key_len = k.len().saturating_sub(38); // PeerId is typically 38 bytes
                    k[..key_len].to_vec()
                })
                .collect();

            if unique_keys.len() >= config.max_provided_keys {
                return Err(Error::MaxProvidedKeys);
            }
        }

        // Check if this provider already exists for this key
        let composite_key = Self::provider_composite_key(&record.key, &record.provider);
        let provider_exists = providers_tree
            .get(&composite_key)
            .map_err(|_| Error::MaxRecords)?
            .is_some();

        if !provider_exists {
            // Check providers per key limit
            if providers.len() >= config.max_providers_per_key {
                // Silently ignore (mitigate Sybil attacks)
                return Ok(());
            }
        }

        // Store the provider record
        let provider_bytes = Self::encode_provider(&record)?;
        providers_tree
            .insert(&composite_key, provider_bytes.clone())
            .map_err(|_| Error::MaxRecords)?;

        // If this is a local provider, also add to provided tree
        // Note: We can't check local_key.preimage() here as we don't store the local key
        // This would need to be passed in or stored separately
        // For now, we'll store it in the provided tree with a special prefix
        provided_tree
            .insert(&composite_key, provider_bytes)
            .map_err(|_| Error::MaxRecords)?;

        Ok(())
    }

    fn providers(&self, key: &Key) -> Vec<ProviderRecord> {
        self.get_providers_for_key(key).unwrap_or_default()
    }

    fn provided(&self) -> Self::ProvidedIter<'_> {
        ProvidedIter {
            inner: self.provided_tree().iter(),
            _phantom: std::marker::PhantomData,
        }
    }

    fn remove_provider(&mut self, key: &Key, provider: &PeerId) {
        let providers_tree = self.providers_tree();
        let provided_tree = self.provided_tree();
        let composite_key = Self::provider_composite_key(key, provider);

        let _ = providers_tree.remove(&composite_key);
        let _ = provided_tree.remove(&composite_key);
    }
}

/// Iterator over records in the store
#[cfg(feature = "libp2p")]
pub struct RecordsIter<'a> {
    tree_iters: Vec<sled::Iter>,
    current_index: usize,
    _phantom: std::marker::PhantomData<&'a ()>,
}

#[cfg(feature = "libp2p")]
impl<'a> Iterator for RecordsIter<'a> {
    type Item = Cow<'a, Record>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Check if we've exhausted all trees
            if self.current_index >= self.tree_iters.len() {
                return None;
            }

            // Try to get next item from current iterator
            match self.tree_iters[self.current_index].next() {
                Some(result) => {
                    if let Ok((_, v)) = result
                        && let Ok(record) = dummy_util::decode_record(&v) {
                            return Some(Cow::Owned(record));
                        }
                    // Continue to next item on error
                    continue;
                }
                None => {
                    // Current iterator exhausted, move to next tree
                    self.current_index += 1;
                    continue;
                }
            }
        }
    }
}

/// Iterator over provided records in the store
#[cfg(feature = "libp2p")]
pub struct ProvidedIter<'a> {
    inner: sled::Iter,
    _phantom: std::marker::PhantomData<&'a ()>,
}

#[cfg(feature = "libp2p")]
impl<'a> Iterator for ProvidedIter<'a> {
    type Item = Cow<'a, ProviderRecord>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().and_then(|result| {
            result
                .ok()
                .and_then(|(_, v)| dummy_util::decode_provider(&v).ok().map(Cow::Owned))
        })
    }
}

pub mod dummy_util {
    use super::*;
    pub fn encode_key(key: &Key) -> Vec<u8> {
        key.to_vec()
    }

    /// Encode a Record to bytes using SerializableRecord
    pub fn encode_record(record: &Record) -> Result<Vec<u8>> {
        let serializable = SerializableRecord {
            key: record.key.to_vec(),
            value: record.value.clone(),
            publisher: record.publisher.as_ref().map(|p| p.to_bytes()),
        };
        bincode::encode_to_vec(&serializable, bincode::config::standard())
            .map_err(|_| Error::ValueTooLarge)
    }

    /// Decode a Record from bytes using SerializableRecord
    pub fn decode_record(bytes: &[u8]) -> Result<Record> {
        let (serializable, _): (SerializableRecord, _) =
            bincode::decode_from_slice(bytes, bincode::config::standard())
                .map_err(|_| Error::MaxRecords)?;

        let publisher = match serializable.publisher {
            Some(bytes) => Some(PeerId::from_bytes(&bytes).map_err(|_| Error::MaxRecords)?),
            None => None,
        };

        Ok(Record {
            key: Key::from(serializable.key),
            value: serializable.value,
            publisher,
            expires: None, // Always None - we don't persist expiration times
        })
    }

    /// Encode a ProviderRecord to bytes using SerializableProviderRecord
    pub fn encode_provider(provider: &ProviderRecord) -> Result<Vec<u8>> {
        let serializable = SerializableProviderRecord {
            key: provider.key.to_vec(),
            provider: provider.provider.to_bytes(),
            addresses: provider.addresses.iter().map(|a| a.to_vec()).collect(),
        };
        bincode::encode_to_vec(&serializable, bincode::config::standard())
            .map_err(|_| Error::ValueTooLarge)
    }

    /// Decode a ProviderRecord from bytes using SerializableProviderRecord
    pub fn decode_provider(bytes: &[u8]) -> Result<ProviderRecord> {
        let (serializable, _): (SerializableProviderRecord, _) =
            bincode::decode_from_slice(bytes, bincode::config::standard())
                .map_err(|_| Error::MaxRecords)?;

        let provider = PeerId::from_bytes(&serializable.provider).map_err(|_| Error::MaxRecords)?;

        let addresses = serializable
            .addresses
            .iter()
            .filter_map(|bytes| libp2p::Multiaddr::try_from(bytes.clone()).ok())
            .collect();

        Ok(ProviderRecord {
            key: Key::from(serializable.key),
            provider,
            expires: None, // Always None - we don't persist expiration times
            addresses,
        })
    }
}

// ============================================================================
// REDB STORE RECORDSTORE IMPLEMENTATION
// ============================================================================

#[cfg(all(feature = "libp2p", feature = "redb"))]
use redb::TableDefinition;

#[cfg(all(feature = "libp2p", feature = "redb"))]
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

    /// Get the table for a given Record by decoding the value's discriminant
    fn table_name_for_record(&self, record: &Record) -> Result<String> {
        // Decode the Record value to get the NetabaseDefinitionTrait
        let (definition, _): (D, _) =
            bincode::decode_from_slice(&record.value, bincode::config::standard())
                .map_err(|_| Error::MaxRecords)?;

        // Get the discriminant value to use as the table name
        Ok(definition.discriminant_name().to_string())
    }

    /// Get the table name for a given RecordKey by trying all Definition tables
    fn table_name_for_key(&self, key: &Key) -> Result<String> {
        use strum::IntoEnumIterator;

        let key_bytes = dummy_util::encode_key(key);
        let read_txn = self.db.begin_read().map_err(|_| Error::MaxRecords)?;

        // Try each table to find which one contains this key
        for disc in D::Discriminant::iter() {
            let table_def = Self::table_def_for_disc(&disc);
            if let Ok(table) = read_txn.open_table(table_def)
                && table.get(key_bytes.as_slice()).ok().flatten().is_some() {
                    return Ok(disc.to_string());
                }
        }

        // If not found in any table, return the first table (for new inserts)
        D::Discriminant::iter()
            .next()
            .map(|d| d.to_string())
            .ok_or(Error::MaxRecords)
    }

    /// Get the libp2p providers table definition
    fn providers_table_def() -> TableDefinition<'static, &'static [u8], &'static [u8]> {
        TableDefinition::new(PROVIDER_TREE_NAME)
    }

    /// Get the provided records table definition (records provided by this peer)
    fn provided_table_def() -> TableDefinition<'static, &'static [u8], &'static [u8]> {
        TableDefinition::new("__libp2p_provided")
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
        let key_bytes = dummy_util::encode_key(key);
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
                providers.push(dummy_util::decode_provider(v.value())?);
            }
        }

        Ok(providers)
    }

    /// Create a composite key for provider records (key + provider_id)
    fn provider_composite_key(key: &Key, provider: &PeerId) -> Vec<u8> {
        let mut composite = dummy_util::encode_key(key);
        composite.extend_from_slice(&provider.to_bytes());
        composite
    }
}

#[cfg(all(feature = "libp2p", feature = "redb"))]
impl<D> RecordStore for RedbStore<D>
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
    type RecordsIter<'a>
        = RedbRecordsIter<'a>
    where
        Self: 'a;
    type ProvidedIter<'a>
        = RedbProvidedIter<'a>
    where
        Self: 'a;

    fn get(&self, k: &Key) -> Option<Cow<'_, Record>> {
        let table_name = self.table_name_for_key(k).ok()?;
        let key_bytes = dummy_util::encode_key(k);

        let read_txn = self.db.begin_read().ok()?;
        let table_def: TableDefinition<&[u8], &[u8]> = {
            let static_name: &'static str = Box::leak(table_name.into_boxed_str());
            TableDefinition::new(static_name)
        };

        let table = read_txn.open_table(table_def).ok()?;
        let value = table.get(key_bytes.as_slice()).ok()??;

        dummy_util::decode_record(value.value())
            .ok()
            .map(Cow::Owned)
    }

    fn put(&mut self, r: Record) -> Result<()> {
        let config = self.record_store_config();

        if r.value.len() >= config.max_value_bytes {
            return Err(Error::ValueTooLarge);
        }

        let table_name = self.table_name_for_record(&r)?;
        let key_bytes = dummy_util::encode_key(&r.key);
        let record_bytes = dummy_util::encode_record(&r)?;

        let write_txn = self.db.begin_write().map_err(|_| Error::MaxRecords)?;

        {
            let table_def: TableDefinition<&[u8], &[u8]> = {
                let static_name: &'static str = Box::leak(table_name.into_boxed_str());
                TableDefinition::new(static_name)
            };
            let mut table = write_txn
                .open_table(table_def)
                .map_err(|_| Error::MaxRecords)?;

            // Check if we're at capacity and this is a new record
            let is_new = table
                .get(key_bytes.as_slice())
                .map_err(|_| Error::MaxRecords)?
                .is_none();

            if is_new && self.record_count()? >= config.max_records {
                return Err(Error::MaxRecords);
            }

            table
                .insert(key_bytes.as_slice(), record_bytes.as_slice())
                .map_err(|_| Error::MaxRecords)?;
        }

        write_txn.commit().map_err(|_| Error::MaxRecords)?;
        Ok(())
    }

    fn remove(&mut self, k: &Key) {
        if let Ok(table_name) = self.table_name_for_key(k) {
            let key_bytes = dummy_util::encode_key(k);

            if let Ok(write_txn) = self.db.begin_write() {
                let table_def: TableDefinition<&[u8], &[u8]> = {
                    let static_name: &'static str = Box::leak(table_name.into_boxed_str());
                    TableDefinition::new(static_name)
                };

                if let Ok(mut table) = write_txn.open_table(table_def) {
                    let _ = table.remove(key_bytes.as_slice());
                }
                let _ = write_txn.commit();
            }
        }
    }

    fn records(&self) -> Self::RecordsIter<'_> {
        use strum::IntoEnumIterator;

        let discriminants: Vec<String> = D::Discriminant::iter().map(|d| d.to_string()).collect();

        RedbRecordsIter {
            db: std::sync::Arc::clone(&self.db),
            discriminants,
            current_index: 0,
            current_read_txn: None,
            current_range: None,
            _phantom: std::marker::PhantomData,
        }
    }

    fn add_provider(&mut self, record: ProviderRecord) -> Result<()> {
        let config = self.record_store_config();

        // Get current providers for this key
        let providers = self.get_providers_for_key(&record.key)?;

        // Check if we need to create a new key entry
        if providers.is_empty() {
            let read_txn = self.db.begin_read().map_err(|_| Error::MaxRecords)?;
            let providers_table_def = Self::providers_table_def();

            if let Ok(table) = read_txn.open_table(providers_table_def) {
                let unique_keys: std::collections::HashSet<Vec<u8>> = table
                    .iter()
                    .map_err(|_| Error::MaxRecords)?
                    .filter_map(|r| r.ok())
                    .map(|(k, _)| {
                        let k_bytes = k.value();
                        // Extract just the key part (without provider ID)
                        let key_len = k_bytes.len().saturating_sub(38);
                        k_bytes[..key_len].to_vec()
                    })
                    .collect();

                if unique_keys.len() >= config.max_provided_keys {
                    return Err(Error::MaxProvidedKeys);
                }
            }
        }

        // Check if this provider already exists for this key
        let composite_key = Self::provider_composite_key(&record.key, &record.provider);

        let read_txn = self.db.begin_read().map_err(|_| Error::MaxRecords)?;
        let providers_table_def = Self::providers_table_def();
        let provider_exists = if let Ok(table) = read_txn.open_table(providers_table_def) {
            table.get(composite_key.as_slice()).ok().flatten().is_some()
        } else {
            false
        };

        if !provider_exists && providers.len() >= config.max_providers_per_key {
            // Silently ignore (mitigate Sybil attacks)
            return Ok(());
        }

        // Store the provider record
        let provider_bytes = dummy_util::encode_provider(&record)?;
        let write_txn = self.db.begin_write().map_err(|_| Error::MaxRecords)?;

        {
            let mut providers_table = write_txn
                .open_table(Self::providers_table_def())
                .map_err(|_| Error::MaxRecords)?;
            providers_table
                .insert(composite_key.as_slice(), provider_bytes.as_slice())
                .map_err(|_| Error::MaxRecords)?;

            let mut provided_table = write_txn
                .open_table(Self::provided_table_def())
                .map_err(|_| Error::MaxRecords)?;
            provided_table
                .insert(composite_key.as_slice(), provider_bytes.as_slice())
                .map_err(|_| Error::MaxRecords)?;
        }

        write_txn.commit().map_err(|_| Error::MaxRecords)?;
        Ok(())
    }

    fn providers(&self, key: &Key) -> Vec<ProviderRecord> {
        self.get_providers_for_key(key).unwrap_or_default()
    }

    fn provided(&self) -> Self::ProvidedIter<'_> {
        RedbProvidedIter {
            db: std::sync::Arc::clone(&self.db),
            read_txn: None,
            range: None,
            _phantom: std::marker::PhantomData,
        }
    }

    fn remove_provider(&mut self, key: &Key, provider: &PeerId) {
        let composite_key = Self::provider_composite_key(key, provider);

        if let Ok(write_txn) = self.db.begin_write() {
            if let Ok(mut providers_table) = write_txn.open_table(Self::providers_table_def()) {
                let _ = providers_table.remove(composite_key.as_slice());
            }
            if let Ok(mut provided_table) = write_txn.open_table(Self::provided_table_def()) {
                let _ = provided_table.remove(composite_key.as_slice());
            }
            let _ = write_txn.commit();
        }
    }
}

/// Iterator over records in the RedbStore
#[cfg(feature = "libp2p")]
pub struct RedbRecordsIter<'a> {
    db: std::sync::Arc<redb::Database>,
    discriminants: Vec<String>,
    current_index: usize,
    current_read_txn: Option<redb::ReadTransaction>,
    current_range: Option<redb::Range<'a, &'static [u8], &'static [u8]>>,
    _phantom: std::marker::PhantomData<&'a ()>,
}

#[cfg(feature = "libp2p")]
impl<'a> Iterator for RedbRecordsIter<'a> {
    type Item = Cow<'a, Record>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Try to get next item from current range
            if let Some(range) = &mut self.current_range
                && let Some(Ok((_, v))) = range.next() {
                    if let Ok(record) = dummy_util::decode_record(v.value()) {
                        return Some(Cow::Owned(record));
                    }
                    continue;
                }

            // Current range exhausted or doesn't exist, move to next table
            self.current_range = None;

            if self.current_index >= self.discriminants.len() {
                return None;
            }

            // Open next table
            let disc = &self.discriminants[self.current_index];
            self.current_index += 1;

            let read_txn = match self.db.begin_read() {
                Ok(txn) => txn,
                Err(_) => continue,
            };

            let table_def: TableDefinition<&[u8], &[u8]> = {
                let table_name = disc.to_string();
                let static_name: &'static str = Box::leak(table_name.into_boxed_str());
                TableDefinition::new(static_name)
            };

            if let Ok(table) = read_txn.open_table(table_def)
                && let Ok(iter) = table.iter() {
                    self.current_read_txn = Some(read_txn);
                    // SAFETY: We're storing the transaction and the range has the same lifetime
                    self.current_range = Some(unsafe { std::mem::transmute(iter) });
                    continue;
                }
        }
    }
}

/// Iterator over provided records in the RedbStore
#[cfg(feature = "libp2p")]
pub struct RedbProvidedIter<'a> {
    db: std::sync::Arc<redb::Database>,
    read_txn: Option<redb::ReadTransaction>,
    range: Option<redb::Range<'a, &'static [u8], &'static [u8]>>,
    _phantom: std::marker::PhantomData<&'a ()>,
}

#[cfg(feature = "libp2p")]
impl<'a> Iterator for RedbProvidedIter<'a> {
    type Item = Cow<'a, ProviderRecord>;

    fn next(&mut self) -> Option<Self::Item> {
        // Initialize on first call
        if self.range.is_none() {
            let read_txn = self.db.begin_read().ok()?;
            let table_def: TableDefinition<&[u8], &[u8]> = TableDefinition::new("__libp2p_provided");

            let table = match read_txn.open_table(table_def) {
                Ok(t) => t,
                Err(redb::TableError::TableDoesNotExist(_)) => return None,
                Err(_) => return None,
            };

            let iter = table.iter().ok()?;
            self.read_txn = Some(read_txn);
            // SAFETY: We're storing the transaction and the range has the same lifetime
            self.range = Some(unsafe { std::mem::transmute(iter) });
        }

        if let Some(range) = &mut self.range {
            range.next().and_then(|result| {
                result
                    .ok()
                    .and_then(|(_, v)| dummy_util::decode_provider(v.value()).ok().map(Cow::Owned))
            })
        } else {
            None
        }
    }
}
