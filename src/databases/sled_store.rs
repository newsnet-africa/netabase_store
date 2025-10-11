//! Sled database implementation with NetabaseModel wrapper types
//!
//! This module provides:
//! - `SledStore` - Main store implementation using sled database
//! - `SledStoreTree` - Wrapper around `sled::Tree` that implements `StoreTree<M>`
//! - `SledStoreIter` - Iterator wrapper that converts IVec to NetabaseModel types
//!
//! ## Key Features:
//! - Trees are identified by encoded/decoded Discriminant values
//! - Automatic conversion between IVec and NetabaseModel/NetabaseModelKey types
//! - Type-safe iteration over model data
//! - Full sled::Tree API coverage with type safety
//! - Robust key deserialization flow: RecordKey → NetabaseDefinitionKey → NetabaseModelKey
//! - On-demand tree opening using discriminants

use std::{borrow::Cow, marker::PhantomData, path::Path};

use strum::IntoEnumIterator;

use crate::{
    error::{NetabaseError, StoreError},
    traits::{
        definition::{NetabaseDefinition, NetabaseDefinitionKeys},
        model::NetabaseModel,
        store::{Store, StoreTree},
    },
};

#[cfg(feature = "libp2p")]
use libp2p::{
    PeerId,
    kad::{
        ProviderRecord, Record, RecordKey,
        store::{Error as RecordStoreError, RecordStore},
    },
};

#[cfg(feature = "libp2p")]
use crate::traits::dht::{KademliaRecord, KademliaRecordKey, provider_record_helpers};

pub struct SledStore<D: NetabaseDefinition> {
    db: sled::Db,
    #[cfg(feature = "libp2p")]
    provider_tree: sled::Tree,
    _phantom: PhantomData<D>,
}

impl<D: NetabaseDefinition> SledStore<D> {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, NetabaseError> {
        let db = sled::open(path)?;

        #[cfg(feature = "libp2p")]
        let provider_tree = db.open_tree("__provider_records__")?;

        Ok(Self {
            db,
            #[cfg(feature = "libp2p")]
            provider_tree,
            _phantom: PhantomData,
        })
    }

    /// Get a typed SledStoreTree for a specific model type
    pub fn get_typed_tree<M: NetabaseModel<Defined = D>>(
        &self,
    ) -> Result<SledStoreTree<M>, StoreError> {
        let discriminant = M::DISCRIMINANT;
        let discriminant_bytes = bincode::encode_to_vec(&discriminant, bincode::config::standard())
            .map_err(|_| StoreError::OpenTreeError)?;

        let tree = self
            .db
            .open_tree(discriminant_bytes)
            .map_err(|_| StoreError::OpenTreeError)?;

        Ok(SledStoreTree::new(tree, discriminant))
    }

    /// Open a tree for a specific discriminant
    fn open_tree_for_discriminant(
        &self,
        discriminant: D::Discriminants,
    ) -> Result<sled::Tree, StoreError> {
        let discriminant_bytes = bincode::encode_to_vec(&discriminant, bincode::config::standard())
            .map_err(|_| StoreError::OpenTreeError)?;

        self.db
            .open_tree(discriminant_bytes)
            .map_err(|_| StoreError::OpenTreeError)
    }
}

impl<D: NetabaseDefinition> Store<D> for SledStore<D> {
    type StoreError = sled::Error;

    fn open_tree<V: NetabaseModel<Defined = D>>(
        &self,
        tree_type: <<V as NetabaseModel>::Defined as NetabaseDefinition>::Discriminants,
    ) -> Result<SledStoreTree<V>, StoreError> {
        let discriminant_bytes = bincode::encode_to_vec(&tree_type, bincode::config::standard())
            .map_err(|_| StoreError::OpenTreeError)?;

        let tree = self
            .db
            .open_tree(discriminant_bytes)
            .map_err(|_| StoreError::OpenTreeError)?;

        Ok(SledStoreTree::new(tree, tree_type))
    }
}

/// Implementation of libp2p RecordStore trait for SledStore with robust key deserialization
#[cfg(feature = "libp2p")]
impl<D> RecordStore for SledStore<D>
where
    D: NetabaseDefinition + KademliaRecord + Clone + 'static,
    D::Keys: KademliaRecordKey,
{
    type RecordsIter<'a>
        = RecordIter<'a, D>
    where
        D: 'a;
    type ProvidedIter<'a> = std::iter::Map<
        provider_record_helpers::ProviderRecordIter<sled::Iter>,
        fn(Cow<'static, ProviderRecord>) -> Cow<'a, ProviderRecord>,
    >;

    fn get(&self, k: &RecordKey) -> Option<Cow<'_, Record>> {
        // Step 1: Try to decode the RecordKey to NetabaseDefinitionKeys
        if let Ok(definition_keys) = D::Keys::try_from_record_key(k) {
            // Step 2: Extract discriminant from definition_keys to target specific tree
            let discriminant = definition_keys.discriminant();

            // Step 3: Open the specific tree for this discriminant
            if let Ok(tree) = self.open_tree_for_discriminant(discriminant) {
                // Step 4: Use the original key bytes to get the value
                // TODO: Convert definition_keys to specific NetabaseModelKey for more precise lookup
                let key_bytes = k.as_ref();
                if let Ok(Some(value_bytes)) = tree.get(key_bytes) {
                    // Step 5: Try to decode the value as a NetabaseDefinition
                    if let Ok((model_data, _)) = bincode::decode_from_slice::<D, _>(
                        &value_bytes,
                        bincode::config::standard(),
                    ) {
                        // Step 6: Convert to libp2p Record
                        if let Ok(record) = model_data.try_to_record() {
                            return Some(Cow::Owned(record));
                        }
                    }
                }
            }
        }
        None
    }

    fn put(&mut self, r: Record) -> Result<(), RecordStoreError> {
        // Step 1: Decode the Record back to NetabaseDefinition
        let definition = D::try_from_record(r).map_err(|_| RecordStoreError::ValueTooLarge)?;

        // Step 2: Get the definition keys to determine the correct tree and key
        let definition_keys = definition.keys();

        // Step 3: Convert to appropriate model key and value bytes
        let key_bytes = definition_keys
            .try_to_vec()
            .map_err(|_| RecordStoreError::ValueTooLarge)?;
        let value_bytes = definition
            .try_to_vec()
            .map_err(|_| RecordStoreError::ValueTooLarge)?;

        // Step 4: Extract discriminant from definition_keys to target specific tree
        let discriminant = definition_keys.discriminant();

        if let Ok(tree) = self.open_tree_for_discriminant(discriminant) {
            // Insert into the specific tree
            tree.insert(key_bytes, value_bytes)
                .map_err(|_| RecordStoreError::ValueTooLarge)?;
            return Ok(());
        }

        Err(RecordStoreError::ValueTooLarge)
    }

    fn remove(&mut self, k: &RecordKey) {
        // Step 1: Try to decode the key to determine the correct tree
        if let Ok(definition_keys) = D::Keys::try_from_record_key(k) {
            // Step 2: Extract discriminant from definition_keys to target specific tree
            let discriminant = definition_keys.discriminant();
            let key_bytes = k.as_ref();

            if let Ok(tree) = self.open_tree_for_discriminant(discriminant) {
                let _ = tree.remove(key_bytes);
            }
        }
    }

    fn records(&self) -> Self::RecordsIter<'_> {
        RecordIter::new(self)
    }

    fn add_provider(&mut self, record: ProviderRecord) -> Result<(), RecordStoreError> {
        provider_record_helpers::add_provider_to_key(&self.provider_tree, &record)
            .map_err(|_| RecordStoreError::ValueTooLarge)
    }

    fn providers(&self, key: &RecordKey) -> Vec<ProviderRecord> {
        provider_record_helpers::get_providers_for_key(&self.provider_tree, key).unwrap_or_default()
    }

    fn provided(&self) -> Self::ProvidedIter<'_> {
        provider_record_helpers::ProviderRecordIter::new(self.provider_tree.iter()).map(
            |cow: Cow<'static, ProviderRecord>| -> Cow<'_, ProviderRecord> {
                match cow {
                    Cow::Borrowed(record) => Cow::Borrowed(record),
                    Cow::Owned(record) => Cow::Owned(record),
                }
            },
        )
    }

    fn remove_provider(&mut self, k: &RecordKey, p: &PeerId) {
        let _ = provider_record_helpers::remove_provider_from_key(&self.provider_tree, k, p);
    }
}

/// Iterator for records in SledStore
#[cfg(feature = "libp2p")]
pub struct RecordIter<'a, D: NetabaseDefinition + KademliaRecord> {
    store: &'a SledStore<D>,
    discriminants: std::vec::IntoIter<D::Discriminants>,
    current_tree_iter: Option<sled::Iter>,
}

#[cfg(feature = "libp2p")]
impl<'a, D: NetabaseDefinition + KademliaRecord> RecordIter<'a, D> {
    fn new(store: &'a SledStore<D>) -> Self {
        let discriminants: Vec<D::Discriminants> = D::Discriminants::iter().collect();

        Self {
            store,
            discriminants: discriminants.into_iter(),
            current_tree_iter: None,
        }
    }
}

#[cfg(feature = "libp2p")]
impl<'a, D: NetabaseDefinition + KademliaRecord> Iterator for RecordIter<'a, D> {
    type Item = Cow<'a, Record>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If we have a current tree iterator, try to get the next item
            if let Some(ref mut iter) = self.current_tree_iter {
                if let Some(Ok((_, value))) = iter.next() {
                    // Try to decode the value as a NetabaseDefinition
                    if let Ok((model_data, _)) = bincode::decode_from_slice::<D, _>(
                        value.as_ref(),
                        bincode::config::standard(),
                    ) {
                        if let Ok(record) = model_data.try_to_record() {
                            return Some(Cow::Owned(record));
                        }
                    }
                    // If decoding failed, continue to the next item
                    continue;
                }
            }

            // Current iterator is exhausted or doesn't exist, move to next discriminant
            if let Some(discriminant) = self.discriminants.next() {
                if let Ok(tree) = self.store.open_tree_for_discriminant(discriminant) {
                    self.current_tree_iter = Some(tree.iter());
                    // Continue the loop to try the new iterator
                    continue;
                }
            } else {
                // No more discriminants, we're done
                return None;
            }
        }
    }
}

pub struct SledStoreTree<M: NetabaseModel> {
    tree: sled::Tree,
    discriminant: <<M as NetabaseModel>::Defined as NetabaseDefinition>::Discriminants,
}

impl<M: NetabaseModel> SledStoreTree<M> {
    pub fn new(
        tree: sled::Tree,
        discriminant: <<M as NetabaseModel>::Defined as NetabaseDefinition>::Discriminants,
    ) -> Self {
        Self { tree, discriminant }
    }
}

impl<M: NetabaseModel> StoreTree<M> for SledStoreTree<M> {
    type Iter = SledStoreIter<M>;

    fn get(&self, key: M::Key) -> Result<Option<M>, StoreError> {
        let key_bytes = bincode::encode_to_vec(&key, bincode::config::standard())
            .map_err(|_| StoreError::OpenTreeError)?;

        match self
            .tree
            .get(key_bytes)
            .map_err(|_| StoreError::OpenTreeError)?
        {
            Some(value_bytes) => {
                let (value, _) =
                    bincode::decode_from_slice(&value_bytes, bincode::config::standard())
                        .map_err(|_| StoreError::OpenTreeError)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    fn insert(&self, value: M) -> Result<Option<M>, StoreError> {
        let key = value.key();
        let key_bytes = bincode::encode_to_vec(&key, bincode::config::standard())
            .map_err(|_| StoreError::OpenTreeError)?;
        let value_bytes = bincode::encode_to_vec(&value, bincode::config::standard())
            .map_err(|_| StoreError::OpenTreeError)?;

        match self
            .tree
            .insert(key_bytes, value_bytes)
            .map_err(|_| StoreError::OpenTreeError)?
        {
            Some(old_value_bytes) => {
                let (old_value, _) =
                    bincode::decode_from_slice(&old_value_bytes, bincode::config::standard())
                        .map_err(|_| StoreError::OpenTreeError)?;
                Ok(Some(old_value))
            }
            None => Ok(None),
        }
    }

    fn remove(&self, key: M::Key) -> Result<Option<M>, StoreError> {
        let key_bytes = bincode::encode_to_vec(&key, bincode::config::standard())
            .map_err(|_| StoreError::OpenTreeError)?;

        match self
            .tree
            .remove(key_bytes)
            .map_err(|_| StoreError::OpenTreeError)?
        {
            Some(old_value_bytes) => {
                let (old_value, _) =
                    bincode::decode_from_slice(&old_value_bytes, bincode::config::standard())
                        .map_err(|_| StoreError::OpenTreeError)?;
                Ok(Some(old_value))
            }
            None => Ok(None),
        }
    }

    fn iter(&self) -> Self::Iter {
        SledStoreIter::new(self.tree.iter())
    }

    fn range<R>(&self, range: R) -> Self::Iter
    where
        R: std::ops::RangeBounds<M::Key>,
    {
        let start_bound = match range.start_bound() {
            std::ops::Bound::Included(key) => {
                let key_bytes = bincode::encode_to_vec(key, bincode::config::standard()).unwrap();
                std::ops::Bound::Included(key_bytes)
            }
            std::ops::Bound::Excluded(key) => {
                let key_bytes = bincode::encode_to_vec(key, bincode::config::standard()).unwrap();
                std::ops::Bound::Excluded(key_bytes)
            }
            std::ops::Bound::Unbounded => std::ops::Bound::Unbounded,
        };

        let end_bound = match range.end_bound() {
            std::ops::Bound::Included(key) => {
                let key_bytes = bincode::encode_to_vec(key, bincode::config::standard()).unwrap();
                std::ops::Bound::Included(key_bytes)
            }
            std::ops::Bound::Excluded(key) => {
                let key_bytes = bincode::encode_to_vec(key, bincode::config::standard()).unwrap();
                std::ops::Bound::Excluded(key_bytes)
            }
            std::ops::Bound::Unbounded => std::ops::Bound::Unbounded,
        };

        SledStoreIter::new(self.tree.range((start_bound, end_bound)))
    }

    fn scan_prefix(&self, prefix: &[u8]) -> Self::Iter {
        SledStoreIter::new(self.tree.scan_prefix(prefix))
    }

    fn contains_key(&self, key: M::Key) -> Result<bool, StoreError> {
        let key_bytes = bincode::encode_to_vec(&key, bincode::config::standard())
            .map_err(|_| StoreError::OpenTreeError)?;

        self.tree
            .contains_key(key_bytes)
            .map_err(|_| StoreError::OpenTreeError)
    }

    fn get_lt(&self, key: M::Key) -> Result<Option<(M::Key, M)>, StoreError> {
        let key_bytes = bincode::encode_to_vec(&key, bincode::config::standard())
            .map_err(|_| StoreError::OpenTreeError)?;

        match self
            .tree
            .get_lt(key_bytes)
            .map_err(|_| StoreError::OpenTreeError)?
        {
            Some((found_key_bytes, value_bytes)) => {
                let (found_key, _) =
                    bincode::decode_from_slice(&found_key_bytes, bincode::config::standard())
                        .map_err(|_| StoreError::OpenTreeError)?;
                let (value, _) =
                    bincode::decode_from_slice(&value_bytes, bincode::config::standard())
                        .map_err(|_| StoreError::OpenTreeError)?;
                Ok(Some((found_key, value)))
            }
            None => Ok(None),
        }
    }

    fn get_gt(&self, key: M::Key) -> Result<Option<(M::Key, M)>, StoreError> {
        let key_bytes = bincode::encode_to_vec(&key, bincode::config::standard())
            .map_err(|_| StoreError::OpenTreeError)?;

        match self
            .tree
            .get_gt(key_bytes)
            .map_err(|_| StoreError::OpenTreeError)?
        {
            Some((found_key_bytes, value_bytes)) => {
                let (found_key, _) =
                    bincode::decode_from_slice(&found_key_bytes, bincode::config::standard())
                        .map_err(|_| StoreError::OpenTreeError)?;
                let (value, _) =
                    bincode::decode_from_slice(&value_bytes, bincode::config::standard())
                        .map_err(|_| StoreError::OpenTreeError)?;
                Ok(Some((found_key, value)))
            }
            None => Ok(None),
        }
    }

    fn first(&self) -> Result<Option<(M::Key, M)>, StoreError> {
        match self.tree.first().map_err(|_| StoreError::OpenTreeError)? {
            Some((key_bytes, value_bytes)) => {
                let (key, _) = bincode::decode_from_slice(&key_bytes, bincode::config::standard())
                    .map_err(|_| StoreError::OpenTreeError)?;
                let (value, _) =
                    bincode::decode_from_slice(&value_bytes, bincode::config::standard())
                        .map_err(|_| StoreError::OpenTreeError)?;
                Ok(Some((key, value)))
            }
            None => Ok(None),
        }
    }

    fn last(&self) -> Result<Option<(M::Key, M)>, StoreError> {
        match self.tree.last().map_err(|_| StoreError::OpenTreeError)? {
            Some((key_bytes, value_bytes)) => {
                let (key, _) = bincode::decode_from_slice(&key_bytes, bincode::config::standard())
                    .map_err(|_| StoreError::OpenTreeError)?;
                let (value, _) =
                    bincode::decode_from_slice(&value_bytes, bincode::config::standard())
                        .map_err(|_| StoreError::OpenTreeError)?;
                Ok(Some((key, value)))
            }
            None => Ok(None),
        }
    }

    fn pop_min(&self) -> Result<Option<(M::Key, M)>, StoreError> {
        match self.tree.pop_min().map_err(|_| StoreError::OpenTreeError)? {
            Some((key_bytes, value_bytes)) => {
                let (key, _) = bincode::decode_from_slice(&key_bytes, bincode::config::standard())
                    .map_err(|_| StoreError::OpenTreeError)?;
                let (value, _) =
                    bincode::decode_from_slice(&value_bytes, bincode::config::standard())
                        .map_err(|_| StoreError::OpenTreeError)?;
                Ok(Some((key, value)))
            }
            None => Ok(None),
        }
    }

    fn pop_max(&self) -> Result<Option<(M::Key, M)>, StoreError> {
        match self.tree.pop_max().map_err(|_| StoreError::OpenTreeError)? {
            Some((key_bytes, value_bytes)) => {
                let (key, _) = bincode::decode_from_slice(&key_bytes, bincode::config::standard())
                    .map_err(|_| StoreError::OpenTreeError)?;
                let (value, _) =
                    bincode::decode_from_slice(&value_bytes, bincode::config::standard())
                        .map_err(|_| StoreError::OpenTreeError)?;
                Ok(Some((key, value)))
            }
            None => Ok(None),
        }
    }

    fn len(&self) -> usize {
        self.tree.len()
    }

    fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }

    fn clear(&self) -> Result<(), StoreError> {
        self.tree.clear().map_err(|_| StoreError::OpenTreeError)
    }

    fn flush(&self) -> Result<usize, StoreError> {
        self.tree.flush().map_err(|_| StoreError::OpenTreeError)
    }

    fn update_and_fetch<F>(&self, key: M::Key, mut f: F) -> Result<Option<M>, StoreError>
    where
        F: FnMut(Option<&M>) -> Option<M>,
    {
        let key_bytes = bincode::encode_to_vec(&key, bincode::config::standard())
            .map_err(|_| StoreError::OpenTreeError)?;

        let result = self
            .tree
            .update_and_fetch(key_bytes, |old_value_bytes| {
                let old_value = old_value_bytes.and_then(|bytes| {
                    bincode::decode_from_slice(bytes, bincode::config::standard())
                        .map(|(value, _)| value)
                        .ok()
                });

                let new_value = f(old_value.as_ref());

                new_value.and_then(|value| {
                    bincode::encode_to_vec(&value, bincode::config::standard())
                        .map(|bytes| sled::IVec::from(bytes))
                        .ok()
                })
            })
            .map_err(|_| StoreError::OpenTreeError)?;

        match result {
            Some(value_bytes) => {
                let (value, _) =
                    bincode::decode_from_slice(&value_bytes, bincode::config::standard())
                        .map_err(|_| StoreError::OpenTreeError)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    fn fetch_and_update<F>(&self, key: M::Key, mut f: F) -> Result<Option<M>, StoreError>
    where
        F: FnMut(Option<&M>) -> Option<M>,
    {
        let key_bytes = bincode::encode_to_vec(&key, bincode::config::standard())
            .map_err(|_| StoreError::OpenTreeError)?;

        let result = self
            .tree
            .fetch_and_update(key_bytes, |old_value_bytes| {
                let old_value = old_value_bytes.and_then(|bytes| {
                    bincode::decode_from_slice(bytes, bincode::config::standard())
                        .map(|(value, _)| value)
                        .ok()
                });

                let new_value = f(old_value.as_ref());

                new_value.and_then(|value| {
                    bincode::encode_to_vec(&value, bincode::config::standard())
                        .map(|bytes| sled::IVec::from(bytes))
                        .ok()
                })
            })
            .map_err(|_| StoreError::OpenTreeError)?;

        match result {
            Some(value_bytes) => {
                let (value, _) =
                    bincode::decode_from_slice(&value_bytes, bincode::config::standard())
                        .map_err(|_| StoreError::OpenTreeError)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    fn compare_and_swap(
        &self,
        key: M::Key,
        old: Option<M>,
        new: Option<M>,
    ) -> Result<std::result::Result<(), (Option<M>, Option<M>)>, StoreError> {
        let key_bytes = bincode::encode_to_vec(&key, bincode::config::standard())
            .map_err(|_| StoreError::OpenTreeError)?;

        let old_bytes = old
            .map(|value| bincode::encode_to_vec(&value, bincode::config::standard()))
            .transpose()
            .map_err(|_| StoreError::OpenTreeError)?
            .map(|bytes| sled::IVec::from(bytes));

        let new_bytes = new
            .map(|value| bincode::encode_to_vec(&value, bincode::config::standard()))
            .transpose()
            .map_err(|_| StoreError::OpenTreeError)?
            .map(|bytes| sled::IVec::from(bytes));

        match self
            .tree
            .compare_and_swap(key_bytes, old_bytes, new_bytes)
            .map_err(|_| StoreError::OpenTreeError)?
        {
            Ok(()) => Ok(Ok(())),
            Err(sled::CompareAndSwapError { current, proposed }) => {
                let current_value = current
                    .map(|bytes| {
                        bincode::decode_from_slice(&bytes, bincode::config::standard())
                            .map(|(value, _)| value)
                            .ok()
                    })
                    .flatten();
                let proposed_value = proposed
                    .map(|bytes| {
                        bincode::decode_from_slice(&bytes, bincode::config::standard())
                            .map(|(value, _)| value)
                            .ok()
                    })
                    .flatten();
                Ok(Err((current_value, proposed_value)))
            }
        }
    }

    fn name(&self) -> sled::IVec {
        bincode::encode_to_vec(&self.discriminant, bincode::config::standard())
            .unwrap_or_default()
            .into()
    }

    fn checksum(&self) -> Result<u32, StoreError> {
        self.tree.checksum().map_err(|_| StoreError::OpenTreeError)
    }
}

pub struct SledStoreIter<M: NetabaseModel> {
    iter: sled::Iter,
    _phantom: PhantomData<M>,
}

impl<M: NetabaseModel> SledStoreIter<M> {
    pub fn new(iter: sled::Iter) -> Self {
        Self {
            iter,
            _phantom: PhantomData,
        }
    }
}

impl<M: NetabaseModel> Iterator for SledStoreIter<M> {
    type Item = Result<(M::Key, M), StoreError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|result| match result {
            Ok((key_bytes, value_bytes)) => {
                let key_result =
                    bincode::decode_from_slice(&key_bytes, bincode::config::standard())
                        .map(|(key, _)| key)
                        .map_err(|_| StoreError::OpenTreeError);

                let value_result =
                    bincode::decode_from_slice(&value_bytes, bincode::config::standard())
                        .map(|(value, _)| value)
                        .map_err(|_| StoreError::OpenTreeError);

                match (key_result, value_result) {
                    (Ok(key), Ok(value)) => Ok((key, value)),
                    (Err(e), _) | (_, Err(e)) => Err(e),
                }
            }
            Err(_) => Err(StoreError::OpenTreeError),
        })
    }
}

impl<M: NetabaseModel> DoubleEndedIterator for SledStoreIter<M> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|result| match result {
            Ok((key_bytes, value_bytes)) => {
                let key_result =
                    bincode::decode_from_slice(&key_bytes, bincode::config::standard())
                        .map(|(key, _)| key)
                        .map_err(|_| StoreError::OpenTreeError);

                let value_result =
                    bincode::decode_from_slice(&value_bytes, bincode::config::standard())
                        .map(|(value, _)| value)
                        .map_err(|_| StoreError::OpenTreeError);

                match (key_result, value_result) {
                    (Ok(key), Ok(value)) => Ok((key, value)),
                    (Err(e), _) | (_, Err(e)) => Err(e),
                }
            }
            Err(_) => Err(StoreError::OpenTreeError),
        })
    }
}

impl<M: NetabaseModel> SledStoreIter<M> {
    /// Iterator over just the keys
    pub fn keys(self) -> impl Iterator<Item = Result<M::Key, StoreError>> {
        self.map(|result| result.map(|(key, _)| key))
    }

    /// Iterator over just the values
    pub fn values(self) -> impl Iterator<Item = Result<M, StoreError>> + DoubleEndedIterator {
        self.map(|result| result.map(|(_, value)| value))
    }

    /// Size hint from underlying iterator
    pub fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<M: NetabaseModel> Clone for SledStoreTree<M> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree.clone(),
            discriminant: self.discriminant,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{
        definition::{NetabaseDefinition, NetabaseDefinitionDiscriminants, NetabaseDefinitionKeys},
        model::{NetabaseModel, NetabaseModelKey},
    };
    use bincode::{Decode, Encode};
    use strum::{EnumIter, IntoEnumIterator};
    use tempfile::TempDir;

    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
    struct TestModel {
        id: u32,
        name: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
    struct TestKey {
        id: u32,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode, EnumIter, Hash)]
    enum TestDiscriminant {
        TestModel,
    }

    impl NetabaseDefinitionDiscriminants for TestDiscriminant {}

    struct TestDefinitionKeys;
    impl NetabaseDefinitionKeys for TestDefinitionKeys {
        type Discriminants = TestDiscriminant;

        fn discriminant(&self) -> Self::Discriminants {
            TestDiscriminant::TestModel
        }
    }

    struct TestDefinition;
    impl NetabaseDefinition for TestDefinition {
        type Keys = TestDefinitionKeys;
        type Discriminants = TestDiscriminant;

        fn keys(&self) -> Self::Keys {
            TestDefinitionKeys
        }
    }

    impl NetabaseModelKey for TestKey {
        type Model = TestModel;
    }

    impl NetabaseModel for TestModel {
        type Key = TestKey;
        type Defined = TestDefinition;
        const DISCRIMINANT: TestDiscriminant = TestDiscriminant::TestModel;

        fn key(&self) -> Self::Key {
            TestKey { id: self.id }
        }
    }

    impl From<TestModel> for TestDefinition {
        fn from(_: TestModel) -> Self {
            TestDefinition
        }
    }

    #[test]
    fn test_sled_store_tree_wrapper() {
        let temp_dir = TempDir::new().unwrap();
        let store = SledStore::<TestDefinition>::new(temp_dir.path()).unwrap();
        let store_tree = store.get_typed_tree::<TestModel>().unwrap();

        let model = TestModel {
            id: 1,
            name: "Test".to_string(),
        };

        // Test insert
        let result = store_tree.insert(model.clone()).unwrap();
        assert!(result.is_none());

        // Test get
        let key = TestKey { id: 1 };
        let retrieved = store_tree.get(key.clone()).unwrap();
        assert_eq!(retrieved, Some(model.clone()));

        // Test contains_key
        assert!(store_tree.contains_key(key.clone()).unwrap());

        // Test remove
        let removed = store_tree.remove(key.clone()).unwrap();
        assert_eq!(removed, Some(model));

        // Test get after remove
        let retrieved = store_tree.get(key).unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_sled_store_iter_wrapper() {
        let temp_dir = TempDir::new().unwrap();
        let store = SledStore::<TestDefinition>::new(temp_dir.path()).unwrap();
        let store_tree = store.get_typed_tree::<TestModel>().unwrap();

        // Insert test data
        for i in 1..=3 {
            let model = TestModel {
                id: i,
                name: format!("Test{}", i),
            };
            store_tree.insert(model).unwrap();
        }

        // Test forward iteration
        let results: Result<Vec<_>, _> = store_tree.iter().collect();
        let results = results.unwrap();
        assert_eq!(results.len(), 3);

        // Test keys iterator
        let keys: Result<Vec<_>, _> = store_tree.iter().keys().collect();
        let keys = keys.unwrap();
        assert_eq!(keys.len(), 3);

        // Test values iterator
        let values: Result<Vec<_>, _> = store_tree.iter().values().collect();
        let values = values.unwrap();
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_clone_wrapper() {
        let temp_dir = TempDir::new().unwrap();
        let store = SledStore::<TestDefinition>::new(temp_dir.path()).unwrap();
        let store_tree = store.get_typed_tree::<TestModel>().unwrap();

        let cloned = store_tree.clone();

        let model = TestModel {
            id: 1,
            name: "Test".to_string(),
        };

        // Insert using original
        store_tree.insert(model.clone()).unwrap();

        // Retrieve using clone
        let key = TestKey { id: 1 };
        let retrieved = cloned.get(key).unwrap();
        assert_eq!(retrieved, Some(model));
    }

    #[test]
    fn test_tree_name_returns_discriminant() {
        let temp_dir = TempDir::new().unwrap();
        let store = SledStore::<TestDefinition>::new(temp_dir.path()).unwrap();
        let store_tree = store.get_typed_tree::<TestModel>().unwrap();

        let name = store_tree.name();
        let expected_discriminant_bytes =
            bincode::encode_to_vec(&TestDiscriminant::TestModel, bincode::config::standard())
                .unwrap();

        assert_eq!(name.as_ref(), expected_discriminant_bytes.as_slice());
    }

    #[test]
    fn test_on_demand_tree_opening() {
        let temp_dir = TempDir::new().unwrap();
        let store = SledStore::<TestDefinition>::new(temp_dir.path()).unwrap();

        // Test that we can get the same tree multiple times
        let tree1 = store.get_typed_tree::<TestModel>().unwrap();
        let tree2 = store.get_typed_tree::<TestModel>().unwrap();

        let model = TestModel {
            id: 1,
            name: "Test".to_string(),
        };

        // Insert using tree1
        tree1.insert(model.clone()).unwrap();

        // Retrieve using tree2 - should see the same data
        let key = TestKey { id: 1 };
        let retrieved = tree2.get(key).unwrap();
        assert_eq!(retrieved, Some(model));
    }
}
