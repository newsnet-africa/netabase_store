use libp2p::PeerId;
use libp2p::kad::{
    ProviderRecord, Record, RecordKey as Key,
    store::{Error as StoreError, RecordStore},
};
use std::borrow::Cow;
use std::marker::PhantomData;

use crate::{
    databases::redb::RedbStore,
    traits::registry::definition::redb_definition::RedbDefinition,
};

/// Implementation of libp2p RecordStore for RedbStore.
pub struct Libp2pRedbStore<D: RedbDefinition + Clone>
where
    D::Discriminant: 'static + std::fmt::Debug,
{
    _store: RedbStore<D>,
    _local_peer_id: PeerId,
}

impl<D: RedbDefinition + Clone> Libp2pRedbStore<D>
where
    D::Discriminant: 'static + std::fmt::Debug,
{
    pub fn new(store: RedbStore<D>, local_peer_id: PeerId) -> Self {
        Self {
            _store: store,
            _local_peer_id: local_peer_id,
        }
    }
}

pub struct RedbRecordsIter<'a, D: RedbDefinition + Clone>
where
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
{
    items: std::vec::IntoIter<Cow<'a, Record>>,
    _marker: PhantomData<D>,
}

impl<'a, D: RedbDefinition + Clone> Iterator for RedbRecordsIter<'a, D>
where
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
{
    type Item = Cow<'a, Record>;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.next()
    }
}

pub struct RedbProvidedIter<'a> {
    items: std::vec::IntoIter<Cow<'a, ProviderRecord>>,
}

impl<'a> Iterator for RedbProvidedIter<'a> {
    type Item = Cow<'a, ProviderRecord>;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.next()
    }
}

impl<D: RedbDefinition + Clone> RecordStore for Libp2pRedbStore<D>
where
    D::Discriminant: 'static + std::fmt::Debug,
{
    type RecordsIter<'a>
        = RedbRecordsIter<'a, D>
    where
        Self: 'a;
    type ProvidedIter<'a>
        = RedbProvidedIter<'a>
    where
        Self: 'a;

    fn get(&self, k: &Key) -> Option<Cow<'_, Record>> {
        if let Ok(txn) = self._store.begin_read() {
            return txn
                .with_read_transaction(|rt| D::find_record(rt, k))
                .ok()
                .flatten()
                .map(Cow::Owned);
        }
        None
    }

    fn put(&mut self, r: Record) -> Result<(), StoreError> {
        let txn = self
            ._store
            .begin_write()
            .map_err(|_| StoreError::MaxRecords)?;
        txn.with_write_transaction(|wt| D::put_record(wt, r))
            .map_err(|_| StoreError::ValueTooLarge)?;
        txn.commit().map_err(|_| StoreError::MaxRecords)?;
        Ok(())
    }

    fn remove(&mut self, k: &Key) {
        if let Ok(txn) = self._store.begin_write() {
            let _ = txn.with_write_transaction(|wt| D::remove_record(wt, k));
            let _ = txn.commit();
        }
    }

    fn records(&self) -> Self::RecordsIter<'_> {
        // Collect all records to avoid lifetime issues with transaction
        let mut records = Vec::new();
        if let Ok(txn) = self._store.begin_read() {
            let _ = txn.with_read_transaction(|rt| {
                if let Ok(tables) = D::open_read_only_tables(rt)
                    && let Ok(iter) = D::iter_records(&tables) {
                        for r in iter.flatten() {
                            records.push(Cow::Owned(r));
                        }
                    }
                Ok(())
            });
        }

        RedbRecordsIter {
            items: records.into_iter(),
            _marker: PhantomData,
        }
    }

    fn add_provider(&mut self, record: ProviderRecord) -> Result<(), StoreError> {
        let txn = self
            ._store
            .begin_write()
            .map_err(|_| StoreError::MaxRecords)?;
        txn.with_write_transaction(|wt| D::add_provider(wt, record))
            .map_err(|_| StoreError::MaxProvidedKeys)?;
        txn.commit().map_err(|_| StoreError::MaxRecords)?;
        Ok(())
    }

    fn providers(&self, key: &Key) -> Vec<ProviderRecord> {
        if let Ok(txn) = self._store.begin_read() {
            txn.with_read_transaction(|rt| D::get_providers(rt, key))
                .unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    fn provided(&self) -> Self::ProvidedIter<'_> {
        // Iterating provided keys is not yet supported efficiently
        RedbProvidedIter {
            items: vec![].into_iter(),
        }
    }

    fn remove_provider(&mut self, k: &Key, p: &PeerId) {
        if let Ok(txn) = self._store.begin_write() {
            let _ = txn.with_write_transaction(|wt| D::remove_provider(wt, k, p));
            let _ = txn.commit();
        }
    }
}
