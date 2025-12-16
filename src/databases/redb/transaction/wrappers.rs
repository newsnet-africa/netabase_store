use redb::{TableError, CommitError};
use crate::{
    databases::redb::{RedbPermissions},
    traits::registery::definition::redb_definition::RedbDefinition,
    errors::{NetabaseResult, NetabaseError},
};

/// Wrapper around redb::ReadTransaction that enforces permissions
pub struct NetabaseRedbReadTransaction<D: RedbDefinition>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    pub inner: redb::ReadTransaction,
    #[allow(dead_code)] // permissions might be used in future or other methods
    permissions: RedbPermissions<D>,
}

impl<D: RedbDefinition> NetabaseRedbReadTransaction<D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    pub fn new(inner: redb::ReadTransaction, permissions: RedbPermissions<D>) -> Self {
        Self { inner, permissions }
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

/// Wrapper around redb::WriteTransaction that enforces permissions
pub struct NetabaseRedbWriteTransaction<D: RedbDefinition>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    pub inner: redb::WriteTransaction,
    #[allow(dead_code)]
    permissions: RedbPermissions<D>,
}

impl<D: RedbDefinition> NetabaseRedbWriteTransaction<D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    pub fn new(inner: redb::WriteTransaction, permissions: RedbPermissions<D>) -> Self {
        Self { inner, permissions }
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
