use crate::traits::registery::{
    definition::redb_definition::RedbDefinition,
    models::{
        StoreKey,
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
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: redb::Key + 'static,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: redb::Key + 'static,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'db>: redb::Key + 'static,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: redb::Key + 'static,
    D::SubscriptionKeys: redb::Key + 'static,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'db>: 'static,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'a>: IntoDiscriminant,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'a> as NetabaseModelBlobKey<'a, D, M, <M as NetabaseModel<D>>::Keys>>::BlobTypes: redb::Key + redb::Value + 'static,
    <M as NetabaseModel<D>>::Keys: 'static
{
    pub main: TablePermission<'txn, <M::Keys as NetabaseModelKeys<D, M>>::Primary<'db>, M::TableV>,

    pub secondary: Vec<(
        TablePermission<'txn, <M::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>, <M::Keys as NetabaseModelKeys<D, M>>::Primary<'db>>,
        &'db str,
    )>,

    pub relational: Vec<(
        TablePermission<'txn, <M::Keys as NetabaseModelKeys<D, M>>::Primary<'db>, <M::Keys as NetabaseModelKeys<D, M>>::Relational<'db>>,
        &'db str,
    )>,

    pub subscription: Vec<(
        TablePermission<'txn, D::SubscriptionKeys, <M::Keys as NetabaseModelKeys<D, M>>::Primary<'db>>,
        &'db str,
    )>,

    pub blob: Vec<(
        TablePermission<'txn, <M::Keys as NetabaseModelKeys<D, M>>::Blob<'db>, <<M::Keys as NetabaseModelKeys<D, M>>::Blob<'db> as NetabaseModelBlobKey<'db, D, M, <M as NetabaseModel<D>>::Keys>>::BlobTypes>,
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
