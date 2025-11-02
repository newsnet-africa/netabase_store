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

/// Generic RecordStore implementation for SledStore<D>
///
/// This works with any Definition type D by:
/// 1. Deserializing Record values as the Definition enum
/// 2. Calling the instance method `handle_record_store_put` which dispatches based on the variant
/// 3. Storing the inner model type (not the enum wrapper)
#[cfg(all(feature = "libp2p", feature = "sled"))]
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
        // TODO: Implement records iterator using the generated helper
        Box::new(std::iter::empty())
    }

    fn add_provider(&mut self, _record: ProviderRecord) -> RecordStoreResult<()> {
        // TODO: Implement provider records
        Ok(())
    }

    fn providers(&self, _key: &RecordKey) -> Vec<ProviderRecord> {
        // TODO: Implement provider lookup
        Vec::new()
    }

    fn provided(&self) -> Self::ProvidedIter<'_> {
        // TODO: Implement provided iterator
        Box::new(std::iter::empty())
    }

    fn remove_provider(&mut self, _key: &RecordKey, _provider: &libp2p::PeerId) {
        // TODO: Implement provider removal
    }
}

/// Generic RecordStore implementation for RedbStore<D>
#[cfg(all(feature = "libp2p", feature = "redb"))]
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
        Box::new(std::iter::empty())
    }

    fn add_provider(&mut self, _record: ProviderRecord) -> RecordStoreResult<()> {
        Ok(())
    }

    fn providers(&self, _key: &RecordKey) -> Vec<ProviderRecord> {
        Vec::new()
    }

    fn provided(&self) -> Self::ProvidedIter<'_> {
        Box::new(std::iter::empty())
    }

    fn remove_provider(&mut self, _key: &RecordKey, _provider: &libp2p::PeerId) {
    }
}
