//! Generic RecordStore implementations for SledStore and RedbStore
//!
//! These implementations work with any Definition type by using the macro-generated
//! helper methods on the Definition enum to dispatch to the appropriate model operations.

use crate::{
    databases::{sled_store::SledStore, redb_store::RedbStore},
    traits::definition::{NetabaseDefinitionTrait, RecordStoreExt},
};

#[cfg(feature = "libp2p")]
use libp2p::kad::{
    store::{RecordStore, Result as RecordStoreResult},
    Record, RecordKey, ProviderRecord,
};
use std::borrow::Cow;

/// Generic RecordStore implementation for `SledStore<D>`
///
/// This works with any Definition type D by:
/// 1. Deserializing Record values as the Definition enum
/// 2. Calling the instance method `handle_record_store_put` which dispatches based on the variant
/// 3. Storing the inner model type (not the enum wrapper)
#[cfg(all(feature = "libp2p", feature = "sled", not(target_arch = "wasm32")))]
impl<D> RecordStore for SledStore<D>
where
    D: NetabaseDefinitionTrait + RecordStoreExt,
    <D as strum::IntoDiscriminant>::Discriminant: crate::traits::definition::NetabaseDiscriminant,
    <<D as NetabaseDefinitionTrait>::Keys as strum::IntoDiscriminant>::Discriminant:
        crate::traits::definition::NetabaseKeyDiscriminant,
{
    type RecordsIter<'a> = Box<dyn Iterator<Item = Cow<'a, Record>> + 'a> where Self: 'a;
    type ProvidedIter<'a> = Box<dyn Iterator<Item = Cow<'a, ProviderRecord>> + 'a> where Self: 'a;

    fn get(&self, key: &RecordKey) -> Option<Cow<'_, Record>> {
        // Use the Definition's Sled-specific helper method which decodes the key and fetches the model
        let (_definition, record) = D::handle_sled_get(self, key)?;
        Some(Cow::Owned(record))
    }

    fn put(&mut self, record: Record) -> RecordStoreResult<()> {
        // Deserialize the record value as the Definition enum
        let (definition, _): (D, _) = bincode::decode_from_slice(
            &record.value,
            bincode::config::standard(),
        )
        .map_err(|_| libp2p::kad::store::Error::ValueTooLarge)?;

        // Use the Sled-specific instance method to dispatch based on the variant and store the inner model
        definition.handle_sled_put(self)
    }

    fn remove(&mut self, key: &RecordKey) {
        D::handle_sled_remove(self, key);
    }

    fn records(&self) -> Self::RecordsIter<'_> {
        eprintln!("[GENERIC_IMPL] SledStore::records() called, delegating to D::handle_sled_records");
        D::handle_sled_records(self)
    }

    fn add_provider(&mut self, record: ProviderRecord) -> RecordStoreResult<()> {
        self.add_provider_internal(record)
    }

    fn providers(&self, key: &RecordKey) -> Vec<ProviderRecord> {
        self.providers_internal(key).unwrap_or_default()
    }

    fn provided(&self) -> Self::ProvidedIter<'_> {
        use super::utils;

        // Get the provided iterator from the provided tree
        let tree = self.db().open_tree("__libp2p_provided")
            .expect("Failed to open provided tree");

        Box::new(tree.iter().filter_map(|result| {
            result.ok().and_then(|(_k, v)| {
                // Decode provider record from value
                utils::decode_provider(&v).ok().map(std::borrow::Cow::Owned)
            })
        }))
    }

    fn remove_provider(&mut self, key: &RecordKey, provider: &libp2p::PeerId) {
        self.remove_provider_internal(key, provider)
    }
}

/// Generic RecordStore implementation for `RedbStore<D>`
#[cfg(all(feature = "libp2p", feature = "redb", not(target_arch = "wasm32")))]
impl<D> RecordStore for RedbStore<D>
where
    D: NetabaseDefinitionTrait + RecordStoreExt,
    <D as strum::IntoDiscriminant>::Discriminant: crate::traits::definition::NetabaseDiscriminant,
    <<D as NetabaseDefinitionTrait>::Keys as strum::IntoDiscriminant>::Discriminant:
        crate::traits::definition::NetabaseKeyDiscriminant,
{
    type RecordsIter<'a> = Box<dyn Iterator<Item = Cow<'a, Record>> + 'a> where Self: 'a;
    type ProvidedIter<'a> = Box<dyn Iterator<Item = Cow<'a, ProviderRecord>> + 'a> where Self: 'a;

    fn get(&self, key: &RecordKey) -> Option<Cow<'_, Record>> {
        // Use the Definition's Redb-specific helper method which decodes the key and fetches the model
        let (_definition, record) = D::handle_redb_get(self, key)?;
        Some(Cow::Owned(record))
    }

    fn put(&mut self, record: Record) -> RecordStoreResult<()> {
        // Deserialize the record value as the Definition enum
        let (definition, _): (D, _) = bincode::decode_from_slice(
            &record.value,
            bincode::config::standard(),
        )
        .map_err(|_| libp2p::kad::store::Error::ValueTooLarge)?;

        // Use the Redb-specific instance method to dispatch based on the variant and store the inner model
        definition.handle_redb_put(self)
    }

    fn remove(&mut self, key: &RecordKey) {
        D::handle_redb_remove(self, key);
    }

    fn records(&self) -> Self::RecordsIter<'_> {
        D::handle_redb_records(self)
    }

    fn add_provider(&mut self, record: ProviderRecord) -> RecordStoreResult<()> {
        self.add_provider_internal(record)
    }

    fn providers(&self, key: &RecordKey) -> Vec<ProviderRecord> {
        self.providers_internal(key).unwrap_or_default()
    }

    fn provided(&self) -> Self::ProvidedIter<'_> {
        use redb::{ReadableDatabase, ReadableTable, TableDefinition};
        use super::utils;

        let table_def: TableDefinition<'static, &'static [u8], &'static [u8]> =
            TableDefinition::new("__libp2p_provided");

        // Read all provided records
        let records: Vec<ProviderRecord> = if let Ok(read_txn) = self.db.begin_read() {
            if let Ok(table) = read_txn.open_table(table_def) {
                table.iter()
                    .ok()
                    .into_iter()
                    .flatten()
                    .filter_map(|result| {
                        result.ok().and_then(|(_k, v)| {
                            utils::decode_provider(v.value()).ok()
                        })
                    })
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        Box::new(records.into_iter().map(std::borrow::Cow::Owned))
    }

    fn remove_provider(&mut self, key: &RecordKey, provider: &libp2p::PeerId) {
        self.remove_provider_internal(key, provider)
    }
}
