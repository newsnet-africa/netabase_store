use crate::{
    databases::redb::transaction::{RedbTransaction, RedbTransactionType},
    errors::NetabaseResult,
    relational::RelationalLink,
    traits::registery::{
        definition::{NetabaseDefinition, redb_definition::RedbDefinition},
        models::{
            keys::{NetabaseModelKeys, blob::NetabaseModelBlobKey},
            model::{ModelHydrator, NetabaseModel, redb_model::RedbNetabaseModel},
        },
        repository::{InRepository, NetabaseRepository},
    },
};
use redb::ReadableTable;
use strum::IntoDiscriminant;

pub struct RedbModelHydrator<'txn, D: RedbDefinition>
where
    <D as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    transaction: &'txn RedbTransaction<'txn, D>,
}

impl<'txn, D: RedbDefinition> RedbModelHydrator<'txn, D>
where
    <D as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    pub fn new(transaction: &'txn RedbTransaction<'txn, D>) -> Self {
        Self { transaction }
    }
}

impl<'txn, D: RedbDefinition> ModelHydrator for RedbModelHydrator<'txn, D>
where
    <D as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    fn hydrate_link<'a, R, S, T, M>(&mut self, link: &mut RelationalLink<'a, R, S, T, M>) -> NetabaseResult<()>
    where
        R: NetabaseRepository,
        S: NetabaseDefinition + InRepository<R> + 'static,
        S::Discriminant: std::fmt::Debug,
        T: RedbDefinition + InRepository<R> + 'static,
        T::Discriminant: std::fmt::Debug,
        M: for<'db> RedbNetabaseModel<'db, T> + NetabaseModel<T> + redb::Value + redb::Key + 'static,
        for<'b> M: redb::Value<SelfType<'b> = M>,
        <M::Keys as NetabaseModelKeys<T, M>>::Primary: redb::Key + 'static,
        for<'b> &'b <M::Keys as NetabaseModelKeys<T, M>>::Primary: std::borrow::Borrow<<<M::Keys as NetabaseModelKeys<T, M>>::Primary as redb::Value>::SelfType<'b>>,
        <<M as NetabaseModel<T>>::Keys as NetabaseModelKeys<T, M>>::Secondary: IntoDiscriminant,
        <<M as NetabaseModel<T>>::Keys as NetabaseModelKeys<T, M>>::Relational: IntoDiscriminant,
        <<M as NetabaseModel<T>>::Keys as NetabaseModelKeys<T, M>>::Blob: IntoDiscriminant,
        <<M as NetabaseModel<T>>::Keys as NetabaseModelKeys<T, M>>::Subscription: IntoDiscriminant,
        <<<M as NetabaseModel<T>>::Keys as NetabaseModelKeys<T, M>>::Secondary as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<T>>::Keys as NetabaseModelKeys<T, M>>::Relational as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<T>>::Keys as NetabaseModelKeys<T, M>>::Blob as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<T>>::Keys as NetabaseModelKeys<T, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    {
        // 1. Check if we need to hydrate
        if !link.is_dehydrated() {
            return Ok(());
        }

        let primary_key = link.get_primary_key();

        // 2. Fetch model using raw transaction
        let model_opt: Option<M> = match &self.transaction.transaction {
            RedbTransactionType::Read(read_txn) => {
                M::fetch_by_primary_key_read(&read_txn.inner, primary_key)
            }
            RedbTransactionType::Write(write_txn) => {
                M::fetch_by_primary_key_write(&write_txn.inner, primary_key)
            }
        }?;

        if let Some(model) = model_opt {
            *link = RelationalLink::new_owned(primary_key.clone(), model);
        }

        Ok(())
    }
}
