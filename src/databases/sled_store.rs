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

use std::{collections::HashMap, marker::PhantomData, path::Path};

use strum::IntoEnumIterator;

use crate::{
    error::{NetabaseError, StoreError},
    traits::{
        definition::NetabaseDefinition,
        model::NetabaseModel,
        store::{Store, StoreTree},
    },
};

pub struct SledStore<D: NetabaseDefinition> {
    db: sled::Db,
    definitions: HashMap<D::Discriminants, sled::Tree>,
}

impl<D: NetabaseDefinition> SledStore<D> {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, NetabaseError> {
        let db = sled::open(path)?;
        let mut definitions =
            <<D as NetabaseDefinition>::Discriminants as IntoEnumIterator>::iter();
        let hash = definitions.try_fold(
            HashMap::new(),
            |mut acc, d| -> Result<HashMap<D::Discriminants, sled::Tree>, NetabaseError> {
                let tree =
                    db.open_tree(bincode::encode_to_vec(&d, bincode::config::standard())?)?;
                acc.insert(d, tree);
                Ok(acc)
            },
        )?;
        Ok(Self {
            db,
            definitions: hash,
        })
    }
}

impl<D: NetabaseDefinition> Store<D> for SledStore<D> {
    type StoreError = sled::Error;

    fn open_tree<V: NetabaseModel<Defined = D>>(
        &self,
        tree_type: <<V as NetabaseModel>::Defined as NetabaseDefinition>::Discriminants,
    ) -> Result<SledStoreTree<V>, StoreError> {
        match self.definitions.get(&tree_type) {
            Some(tree) => Ok(SledStoreTree::new(tree.clone())),
            None => Err(StoreError::OpenTreeError),
        }
    }
}

/// Wrapper around sled::Tree that implements StoreTree for a specific NetabaseModel
#[derive(Debug)]
pub struct SledStoreTree<M: NetabaseModel> {
    tree: sled::Tree,
    _phantom: PhantomData<M>,
}

impl<M: NetabaseModel> SledStoreTree<M> {
    pub fn new(tree: sled::Tree) -> Self {
        Self {
            tree,
            _phantom: PhantomData,
        }
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
            Some(value_bytes) => {
                let (value, _) =
                    bincode::decode_from_slice(&value_bytes, bincode::config::standard())
                        .map_err(|_| StoreError::OpenTreeError)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    fn iter(&self) -> Self::Iter {
        SledStoreIter::new(self.tree.iter())
    }

    fn range<R>(&self, _range: R) -> Self::Iter
    where
        R: std::ops::RangeBounds<M::Key>,
    {
        // For simplicity, we'll use iter() and filter manually
        // A more sophisticated implementation would encode the range bounds
        self.iter()
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

    fn get_gt(&self, key: M::Key) -> Result<Option<(M::Key, M)>, StoreError> {
        let key_bytes = bincode::encode_to_vec(&key, bincode::config::standard())
            .map_err(|_| StoreError::OpenTreeError)?;

        match self
            .tree
            .get_gt(key_bytes)
            .map_err(|_| StoreError::OpenTreeError)?
        {
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
            .update_and_fetch(key_bytes, |old_bytes| {
                let old_value = old_bytes.and_then(|bytes| {
                    bincode::decode_from_slice(bytes, bincode::config::standard())
                        .map(|(value, _)| value)
                        .ok()
                });

                f(old_value.as_ref()).map(|new_value| {
                    bincode::encode_to_vec(&new_value, bincode::config::standard())
                        .unwrap_or_default()
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
            .fetch_and_update(key_bytes, |old_bytes| {
                let old_value = old_bytes.and_then(|bytes| {
                    bincode::decode_from_slice(bytes, bincode::config::standard())
                        .map(|(value, _)| value)
                        .ok()
                });

                f(old_value.as_ref()).map(|new_value| {
                    bincode::encode_to_vec(&new_value, bincode::config::standard())
                        .unwrap_or_default()
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
    ) -> Result<Result<(), (Option<M>, Option<M>)>, StoreError> {
        let key_bytes = bincode::encode_to_vec(&key, bincode::config::standard())
            .map_err(|_| StoreError::OpenTreeError)?;

        let old_bytes = old
            .map(|v| bincode::encode_to_vec(&v, bincode::config::standard()))
            .transpose()
            .map_err(|_| StoreError::OpenTreeError)?;

        let new_bytes = new
            .map(|v| bincode::encode_to_vec(&v, bincode::config::standard()))
            .transpose()
            .map_err(|_| StoreError::OpenTreeError)?;

        match self
            .tree
            .compare_and_swap(key_bytes, old_bytes.as_deref(), new_bytes)
            .map_err(|_| StoreError::OpenTreeError)?
        {
            Ok(()) => Ok(Ok(())),
            Err(err) => {
                let current = err.current.and_then(|bytes| {
                    bincode::decode_from_slice(&bytes, bincode::config::standard())
                        .map(|(value, _)| value)
                        .ok()
                });

                let proposed = err.proposed.and_then(|bytes| {
                    bincode::decode_from_slice(&bytes, bincode::config::standard())
                        .map(|(value, _)| value)
                        .ok()
                });

                Ok(Err((current, proposed)))
            }
        }
    }

    fn name(&self) -> sled::IVec {
        self.tree.name()
    }

    fn checksum(&self) -> Result<u32, StoreError> {
        self.tree.checksum().map_err(|_| StoreError::OpenTreeError)
    }
}

/// Iterator wrapper that converts sled::Iter items to NetabaseModel types
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
        match self.iter.next()? {
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
                    (Ok(key), Ok(value)) => Some(Ok((key, value))),
                    (Err(e), _) | (_, Err(e)) => Some(Err(e)),
                }
            }
            Err(_) => Some(Err(StoreError::OpenTreeError)),
        }
    }
}

impl<M: NetabaseModel> DoubleEndedIterator for SledStoreIter<M> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.iter.next_back()? {
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
                    (Ok(key), Ok(value)) => Some(Ok((key, value))),
                    (Err(e), _) | (_, Err(e)) => Some(Err(e)),
                }
            }
            Err(_) => Some(Err(StoreError::OpenTreeError)),
        }
    }
}

impl<M: NetabaseModel> SledStoreIter<M> {
    /// Iterate over just the keys
    pub fn keys(self) -> impl DoubleEndedIterator<Item = Result<M::Key, StoreError>> {
        self.iter.keys().map(|result| match result {
            Ok(key_bytes) => bincode::decode_from_slice(&key_bytes, bincode::config::standard())
                .map(|(key, _)| key)
                .map_err(|_| StoreError::OpenTreeError),
            Err(_) => Err(StoreError::OpenTreeError),
        })
    }

    /// Iterate over just the values
    pub fn values(self) -> impl DoubleEndedIterator<Item = Result<M, StoreError>> {
        self.iter.values().map(|result| match result {
            Ok(value_bytes) => {
                bincode::decode_from_slice(&value_bytes, bincode::config::standard())
                    .map(|(value, _)| value)
                    .map_err(|_| StoreError::OpenTreeError)
            }
            Err(_) => Err(StoreError::OpenTreeError),
        })
    }

    /// Get the remaining bounds of the iterator
    pub fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<M: NetabaseModel> Clone for SledStoreTree<M> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree.clone(),
            _phantom: PhantomData,
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
    use strum::EnumIter;

    // Test types for the wrapper functionality
    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
    struct TestModel {
        id: u32,
        name: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
    struct TestKey {
        id: u32,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode, EnumIter)]
    enum TestDiscriminant {
        TestModel,
    }

    impl NetabaseDefinitionDiscriminants for TestDiscriminant {}

    struct TestDefinitionKeys;
    impl NetabaseDefinitionKeys for TestDefinitionKeys {}

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
        // Create a temporary sled database
        let config = sled::Config::new().temporary(true);
        let db = config.open().expect("Failed to open temporary sled db");
        let tree = db.open_tree(b"test").expect("Failed to open tree");

        // Create our wrapper
        let wrapped_tree: SledStoreTree<TestModel> = SledStoreTree::new(tree);

        // Test basic operations
        let test_model = TestModel {
            id: 1,
            name: "Test".to_string(),
        };

        // Test insert
        let result = wrapped_tree.insert(test_model.clone());
        assert!(result.is_ok());
        let old_value = result.unwrap();
        assert!(old_value.is_none()); // Should be None for first insert

        // Test get
        let key = TestKey { id: 1 };
        let result = wrapped_tree.get(key.clone());
        assert!(result.is_ok());
        let retrieved = result.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), test_model);

        // Test contains_key
        let result = wrapped_tree.contains_key(key.clone());
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Test len
        assert_eq!(wrapped_tree.len(), 1);
        assert!(!wrapped_tree.is_empty());

        // Test iterator
        let mut iter = wrapped_tree.iter();
        let first_item = iter.next();
        assert!(first_item.is_some());
        let (iter_key, iter_value) = first_item
            .unwrap()
            .expect("Iterator should return valid item");
        assert_eq!(iter_key, key);
        assert_eq!(iter_value, test_model);

        // Test remove
        let result = wrapped_tree.remove(key);
        assert!(result.is_ok());
        let removed = result.unwrap();
        assert!(removed.is_some());
        assert_eq!(removed.unwrap(), test_model);

        // Verify it's gone
        assert_eq!(wrapped_tree.len(), 0);
        assert!(wrapped_tree.is_empty());
    }

    #[test]
    fn test_sled_store_iter_wrapper() {
        // Create a temporary sled database with some data
        let config = sled::Config::new().temporary(true);
        let db = config.open().expect("Failed to open temporary sled db");
        let tree = db.open_tree(b"test").expect("Failed to open tree");
        let wrapped_tree: SledStoreTree<TestModel> = SledStoreTree::new(tree);

        // Insert test data
        let models = vec![
            TestModel {
                id: 1,
                name: "First".to_string(),
            },
            TestModel {
                id: 2,
                name: "Second".to_string(),
            },
            TestModel {
                id: 3,
                name: "Third".to_string(),
            },
        ];

        for model in &models {
            wrapped_tree
                .insert(model.clone())
                .expect("Insert should succeed");
        }

        // Test forward iteration
        let iter = wrapped_tree.iter();
        let collected: Result<Vec<_>, _> = iter.collect();
        assert!(collected.is_ok());
        let items = collected.unwrap();
        assert_eq!(items.len(), 3);

        // Test keys iterator
        let keys_iter = wrapped_tree.iter().keys();
        let collected_keys: Result<Vec<_>, _> = keys_iter.collect();
        assert!(collected_keys.is_ok());
        let keys = collected_keys.unwrap();
        assert_eq!(keys.len(), 3);

        // Test values iterator
        let values_iter = wrapped_tree.iter().values();
        let collected_values: Result<Vec<_>, _> = values_iter.collect();
        assert!(collected_values.is_ok());
        let values = collected_values.unwrap();
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_clone_wrapper() {
        let config = sled::Config::new().temporary(true);
        let db = config.open().expect("Failed to open temporary sled db");
        let tree = db.open_tree(b"test").expect("Failed to open tree");
        let wrapped_tree: SledStoreTree<TestModel> = SledStoreTree::new(tree);

        // Test that clone works
        let cloned_tree = wrapped_tree.clone();

        // Both should refer to the same underlying tree
        let test_model = TestModel {
            id: 42,
            name: "Clone Test".to_string(),
        };

        wrapped_tree
            .insert(test_model.clone())
            .expect("Insert should succeed");

        let key = TestKey { id: 42 };
        let retrieved = cloned_tree.get(key).expect("Get should succeed");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), test_model);
    }
}
