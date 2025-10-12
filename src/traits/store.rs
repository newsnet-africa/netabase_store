//! Store trait and tree trait definitions
//!
//! This module provides:
//! - `Store` trait for database operations
//! - `StoreTree` trait for tree-specific operations
//!
//! ## Implementation Details
//!
//! Concrete implementations of these traits are provided in the `databases` module:
//! - `SledStore` implements `Store<D>` using the sled embedded database
//! - `SledStoreTree<M>` implements `StoreTree<M>` wrapping `sled::Tree`
//! - `SledStoreIter<M>` provides type-safe iteration over NetabaseModel types
//!
//! The traits define the interface while keeping implementation details separate.

use strum::{IntoDiscriminant, IntoEnumIterator};

use crate::{
    databases::sled_store::SledStoreTree,
    error::StoreError,
    traits::{
        definition::{NetabaseDefinition, NetabaseDefinitionDiscriminant},
        model::NetabaseModel,
    },
};

pub trait Store<D: NetabaseDefinition>
where
    <D as IntoDiscriminant>::Discriminant: NetabaseDefinitionDiscriminant,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
{
    type StoreError: std::error::Error;

    fn get_definitions(
        &self,
    ) -> <<D as IntoDiscriminant>::Discriminant as IntoEnumIterator>::Iterator {
        D::Discriminant::iter()
    }

    fn open_tree<V: NetabaseModel<Defined = D>>(
        &self,
        tree_type: <<V as NetabaseModel>::Defined as IntoDiscriminant>::Discriminant,
    ) -> Result<SledStoreTree<V>, StoreError>;

    fn get<V: NetabaseModel<Defined = D>>(&self, key: V::Key) -> Result<Option<V>, StoreError> {
        let tree = self.open_tree::<V>(V::DISCRIMINANT)?;
        tree.get(key)
    }
    fn put<V: NetabaseModel<Defined = D>>(&self, value: V) -> Result<Option<V>, StoreError> {
        let tree = self.open_tree::<V>(V::DISCRIMINANT)?;
        tree.insert(value)
    }
}

pub trait StoreTree<M: NetabaseModel> {
    type Iter: Iterator<Item = Result<(M::Key, M), StoreError>>;

    fn get(&self, key: M::Key) -> Result<Option<M>, StoreError>;
    fn insert(&self, value: M) -> Result<Option<M>, StoreError>;
    fn remove(&self, key: M::Key) -> Result<Option<M>, StoreError>;
    fn iter(&self) -> Self::Iter;
    fn range<R>(&self, range: R) -> Self::Iter
    where
        R: std::ops::RangeBounds<M::Key>;
    fn scan_prefix(&self, prefix: &[u8]) -> Self::Iter;
    fn contains_key(&self, key: M::Key) -> Result<bool, StoreError>;
    fn get_lt(&self, key: M::Key) -> Result<Option<(M::Key, M)>, StoreError>;
    fn get_gt(&self, key: M::Key) -> Result<Option<(M::Key, M)>, StoreError>;
    fn first(&self) -> Result<Option<(M::Key, M)>, StoreError>;
    fn last(&self) -> Result<Option<(M::Key, M)>, StoreError>;
    fn pop_min(&self) -> Result<Option<(M::Key, M)>, StoreError>;
    fn pop_max(&self) -> Result<Option<(M::Key, M)>, StoreError>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn clear(&self) -> Result<(), StoreError>;
    fn flush(&self) -> Result<usize, StoreError>;
    fn update_and_fetch<F>(&self, key: M::Key, f: F) -> Result<Option<M>, StoreError>
    where
        F: FnMut(Option<&M>) -> Option<M>;
    fn fetch_and_update<F>(&self, key: M::Key, f: F) -> Result<Option<M>, StoreError>
    where
        F: FnMut(Option<&M>) -> Option<M>;
    fn compare_and_swap(
        &self,
        key: M::Key,
        old: Option<M>,
        new: Option<M>,
    ) -> Result<Result<(), (Option<M>, Option<M>)>, StoreError>;
    fn name(&self) -> sled::IVec;
    fn checksum(&self) -> Result<u32, StoreError>;
}
