use crate::{
    errors::{NetabaseError, NetabaseResult},
    traits::registery::definition::redb_definition::RedbDefinition,
};
use redb::{CommitError, TableError};
use std::marker::PhantomData;

/// Wrapper around redb::ReadTransaction
pub struct NetabaseRedbReadTransaction<'txn, D: RedbDefinition>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    pub inner: redb::ReadTransaction,
    _marker: PhantomData<(&'txn (), D)>,
}

impl<'txn, D: RedbDefinition> NetabaseRedbReadTransaction<'txn, D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    pub fn new(inner: redb::ReadTransaction) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }

    pub fn open_table<K: redb::Key, V: redb::Value>(
        &self,
        definition: redb::TableDefinition<'static, K, V>,
    ) -> NetabaseResult<redb::ReadOnlyTable<K, V>> {
        self.inner
            .open_table(definition)
            .map_err(move |e: TableError| NetabaseError::RedbError(e.into()))
    }

    pub fn open_multimap_table<K: redb::Key, V: redb::Key>(
        &self,
        definition: redb::MultimapTableDefinition<'static, K, V>,
    ) -> NetabaseResult<redb::ReadOnlyMultimapTable<K, V>> {
        self.inner
            .open_multimap_table(definition)
            .map_err(move |e: TableError| NetabaseError::RedbError(e.into()))
    }
}

/// Wrapper around redb::WriteTransaction
pub struct NetabaseRedbWriteTransaction<'db, D: RedbDefinition>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    pub inner: redb::WriteTransaction,
    _marker: PhantomData<(&'db (), D)>,
}

impl<'db, D: RedbDefinition> NetabaseRedbWriteTransaction<'db, D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    pub fn new(inner: redb::WriteTransaction) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }

    pub fn open_table<'a, K: redb::Key, V: redb::Value>(
        &'a self,
        definition: redb::TableDefinition<'static, K, V>,
    ) -> NetabaseResult<redb::Table<'a, K, V>> {
        self.inner
            .open_table(definition)
            .map_err(move |e: TableError| NetabaseError::RedbError(e.into()))
    }

    pub fn open_multimap_table<'a, K: redb::Key, V: redb::Key>(
        &'a self,
        definition: redb::MultimapTableDefinition<'static, K, V>,
    ) -> NetabaseResult<redb::MultimapTable<'a, K, V>> {
        self.inner
            .open_multimap_table(definition)
            .map_err(move |e: TableError| NetabaseError::RedbError(e.into()))
    }

    pub fn commit(self) -> NetabaseResult<()> {
        self.inner
            .commit()
            .map_err(|e: CommitError| NetabaseError::RedbError(e.into()))
    }
}
