use crate::traits::registery::{
    definition::redb_definition::RedbDefinition,
    models::{
        keys::{NetabaseModelKeys, blob::NetabaseModelBlobKey},
        model::{NetabaseModel, redb_model::RedbNetbaseModel},
    },
};
use strum::IntoDiscriminant;

pub struct ModelOpenTables<'txn, 'db, D: RedbDefinition, M: RedbNetbaseModel<'db, D> + redb::Key>
where
    'db: 'txn,
    D::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key + 'static,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key + 'static,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key + 'static,
    D::SubscriptionKeys: redb::Key + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: 'static,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
{
    pub main: TablePermission<'txn, <M::Keys as NetabaseModelKeys<D, M>>::Primary, M::TableV>,

    pub secondary: Vec<(
        TablePermission<'txn, <M::Keys as NetabaseModelKeys<D, M>>::Secondary, <M::Keys as NetabaseModelKeys<D, M>>::Primary>,
        &'db str,
    )>,

    pub blob: Vec<(
        TablePermission<'txn, <M::Keys as NetabaseModelKeys<D, M>>::Blob, <<M::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem>,
        &'db str,
    )>,

    pub relational: Vec<(
        TablePermission<'txn, <M::Keys as NetabaseModelKeys<D, M>>::Primary, <M::Keys as NetabaseModelKeys<D, M>>::Relational>,
        &'db str,
    )>,

    pub subscription: Vec<(
        TablePermission<'txn, D::SubscriptionKeys, <M::Keys as NetabaseModelKeys<D, M>>::Primary>,
        &'db str,
    )>,
}

pub enum TableType<K, V>
where
    K: redb::Key + 'static,
    V: redb::Value + redb::Key + 'static,
{
    Table(redb::ReadOnlyTable<K, V>),
    MultimapTable(redb::ReadOnlyMultimapTable<K, V>),
}

pub enum ReadWriteTableType<'a, K, V>
where
    K: redb::Key + 'static,
    V: redb::Value + redb::Key + 'static,
{
    Table(redb::Table<'a, K, V>),
    MultimapTable(redb::MultimapTable<'a, K, V>),
}

pub enum TablePermission<'a, K, V>
where
    K: redb::Key + 'static,
    V: redb::Value + redb::Key + 'static,
{
    ReadOnly(TableType<K, V>),
    ReadWrite(ReadWriteTableType<'a, K, V>),
    ReadOnlyWrite(ReadWriteTableType<'a, K, V>),
}
