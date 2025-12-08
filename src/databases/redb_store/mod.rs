use crate::{
    databases::redb_store::{
        transaction::{RedbReadTransaction, RedbWriteTransaction},
    },
    error::NetabaseResult,
    traits::{
        definition::{NetabaseDefinition, NetabaseDefinitionTrait, DiscriminantName},
        store::{
            store::StoreTrait,
            transaction::WriteTransaction,
        },
    },
};
use redb::ReadableDatabase;
use strum::{IntoDiscriminant, IntoEnumIterator};
use std::fmt::Debug;

pub mod transaction;

pub struct RedbStore<D: NetabaseDefinition>
where
    <D as IntoDiscriminant>::Discriminant: IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    db: redb::Database,
    _marker: std::marker::PhantomData<D>,
}

impl<D: NetabaseDefinition> RedbStore<D>
where
    <D as IntoDiscriminant>::Discriminant: IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> NetabaseResult<Self> {
        let db = redb::Database::create(path)?;
        Ok(RedbStore {
            db,
            _marker: std::marker::PhantomData,
        })
    }
}

impl<D: NetabaseDefinition> StoreTrait<D> for RedbStore<D>

where

    <D as IntoDiscriminant>::Discriminant: IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,

{

    type ReadTxn<'a> = RedbReadTransaction<'a, D> where Self: 'a;

    type WriteTxn = RedbWriteTransaction<D>;



    fn read<'a, F, R>(&'a self, f: F) -> NetabaseResult<R>

    where

        F: FnOnce(&Self::ReadTxn<'a>) -> NetabaseResult<R>,

    {

        let txn = self.db.begin_read()?;

        let wrapper = RedbReadTransaction { txn, redb_store: self };

        f(&wrapper)

    }



    fn write<F, R>(&self, f: F) -> NetabaseResult<R>

    where

        F: FnOnce(&mut Self::WriteTxn) -> NetabaseResult<R>,

    {

        let txn = self.db.begin_write()?;

        let mut wrapper = RedbWriteTransaction::new(txn, self);

        let result = f(&mut wrapper)?;

        <RedbWriteTransaction<D> as WriteTransaction<D>>::commit(wrapper)?;

        Ok(result)

    }

}
