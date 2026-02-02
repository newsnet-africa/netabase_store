//! Libp2p RecordStore implementation for RedbStore.
//!
//! This module provides a bridge between the Netabase storage layer and libp2p's
//! Kademlia DHT record storage. It allows a Netabase database to serve as the
//! backing store for a libp2p node's DHT records and provider advertisements.
//!
//! # Overview
//!
//! The `Libp2pRedbStore` implements libp2p's `RecordStore` trait, enabling:
//! - Persistent storage of DHT records
//! - Provider record management
//! - Efficient lookups using Netabase indexes
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │         Libp2p Kademlia DHT         │
//! │                                      │
//! │    RecordStore trait interface       │
//! └──────────────┬──────────────────────┘
//!                │
//!                ▼
//! ┌─────────────────────────────────────┐
//! │      Libp2pRedbStore<D>             │
//! │                                      │
//! │  - get/put/remove records           │
//! │  - add_provider/get_providers       │
//! └──────────────┬──────────────────────┘
//!                │
//!                ▼
//! ┌─────────────────────────────────────┐
//! │        RedbStore<D>                 │
//! │                                      │
//! │  Netabase database backend          │
//! └─────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use netabase_store::databases::redb::libp2p::Libp2pRedbStore;
//! use libp2p::kad::store::RecordStore;
//! use libp2p::PeerId;
//!
//! let store = RedbStore::<MyDef>::new("./db.redb")?;
//! let peer_id = PeerId::random();
//! let mut record_store = Libp2pRedbStore::new(store, peer_id);
//!
//! // Now use with libp2p Kademlia
//! let kad_config = KademliaConfig::default();
//! let mut kad = Kademlia::with_config(peer_id, record_store, kad_config);
//! ```
//!
//! # Feature Flag
//!
//! This module is only available when the `libp2p` feature is enabled.

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

/// Libp2p RecordStore implementation backed by RedbStore.
///
/// This struct wraps a `RedbStore` and implements libp2p's `RecordStore` trait,
/// allowing the database to be used as persistent storage for DHT records.
///
/// # Type Parameters
///
/// - `D`: The definition type for the database
///
/// # Example
///
/// ```rust,ignore
/// let store = RedbStore::<MyDef>::new("./db.redb")?;
/// let peer_id = PeerId::random();
/// let libp2p_store = Libp2pRedbStore::new(store, peer_id);
/// ```
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
    /// Create a new Libp2p record store backed by the given RedbStore.
    ///
    /// # Arguments
    ///
    /// - `store`: The Netabase store to use as backing storage
    /// - `local_peer_id`: The peer ID of the local node
    ///
    /// # Returns
    ///
    /// A new `Libp2pRedbStore` instance ready to be used with libp2p Kademlia.
    pub fn new(store: RedbStore<D>, local_peer_id: PeerId) -> Self {
        Self {
            _store: store,
            _local_peer_id: local_peer_id,
        }
    }
}

/// Iterator over DHT records stored in the database.
///
/// This iterator returns owned `Record` instances wrapped in `Cow` for
/// compatibility with the libp2p `RecordStore` trait.
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

    /// Returns the next record in the iteration.
    fn next(&mut self) -> Option<Self::Item> {
        self.items.next()
    }
}

/// Iterator over provider records stored in the database.
///
/// This iterator returns owned `ProviderRecord` instances wrapped in `Cow`
/// for compatibility with the libp2p `RecordStore` trait.
pub struct RedbProvidedIter<'a> {
    items: std::vec::IntoIter<Cow<'a, ProviderRecord>>,
}

impl<'a> Iterator for RedbProvidedIter<'a> {
    type Item = Cow<'a, ProviderRecord>;

    /// Returns the next provider record in the iteration.
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

    /// Get a DHT record by its key.
    ///
    /// # Arguments
    ///
    /// - `k`: The record key to look up
    ///
    /// # Returns
    ///
    /// The record if found, or `None` if not present in the store.
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

    /// Store a DHT record.
    ///
    /// # Arguments
    ///
    /// - `r`: The record to store
    ///
    /// # Errors
    ///
    /// Returns `StoreError::MaxRecords` if the database can't accept more records
    /// or `StoreError::ValueTooLarge` if the record exceeds size limits.
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

    /// Remove a DHT record by its key.
    ///
    /// # Arguments
    ///
    /// - `k`: The key of the record to remove
    fn remove(&mut self, k: &Key) {
        if let Ok(txn) = self._store.begin_write() {
            let _ = txn.with_write_transaction(|wt| D::remove_record(wt, k));
            let _ = txn.commit();
        }
    }

    /// Iterate over all stored DHT records.
    ///
    /// # Returns
    ///
    /// An iterator over all records in the store.
    ///
    /// # Note
    ///
    /// Records are collected into memory to avoid transaction lifetime issues.
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

    /// Add a provider advertisement for a key.
    ///
    /// # Arguments
    ///
    /// - `record`: The provider record to add
    ///
    /// # Errors
    ///
    /// Returns an error if the provider can't be added or stored.
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

    /// Get all providers for a given key.
    ///
    /// # Arguments
    ///
    /// - `key`: The key to look up providers for
    ///
    /// # Returns
    ///
    /// A vector of provider records for this key.
    fn providers(&self, key: &Key) -> Vec<ProviderRecord> {
        if let Ok(txn) = self._store.begin_read() {
            txn.with_read_transaction(|rt| D::get_providers(rt, key))
                .unwrap_or_default()
        } else {
            Vec::new()
        }
    }

    /// Iterate over all keys this node provides.
    ///
    /// # Returns
    ///
    /// An iterator over provider records.
    ///
    /// # Note
    ///
    /// Currently returns an empty iterator. Full provider iteration is not
    /// yet implemented efficiently.
    fn provided(&self) -> Self::ProvidedIter<'_> {
        // Iterating provided keys is not yet supported efficiently
        RedbProvidedIter {
            items: vec![].into_iter(),
        }
    }

    /// Remove a provider advertisement.
    ///
    /// # Arguments
    ///
    /// - `k`: The key to remove the provider for
    /// - `p`: The peer ID of the provider to remove
    fn remove_provider(&mut self, k: &Key, p: &PeerId) {
        if let Ok(txn) = self._store.begin_write() {
            let _ = txn.with_write_transaction(|wt| D::remove_provider(wt, k, p));
            let _ = txn.commit();
        }
    }
}
