use crate::databases::redb_store::{RedbNetabaseModelTrait, RedbStore};
use crate::error::{NetabaseError, NetabaseResult};
use crate::traits::definition::{
    DiscriminantName, NetabaseDefinition,
};
use crate::traits::model::{NetabaseModelTrait, ModelTypeContainer, key::NetabaseModelKeyTrait};
use crate::traits::store::transaction::{ReadTransaction, WriteTransaction};
use log::debug;
use redb::{Key, ReadableTable, TableDefinition, Value};
use std::borrow::Borrow;
use std::fmt::Debug;
use strum::{IntoDiscriminant, IntoEnumIterator};

pub struct RedbReadTransaction<'db, D: NetabaseDefinition>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    pub(crate) txn: redb::ReadTransaction,
    pub(crate) redb_store: &'db RedbStore<D>,
}

impl<'db, D: NetabaseDefinition> ReadTransaction<D> for RedbReadTransaction<'db, D>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    fn get<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        key: M::PrimaryKey,
    ) -> NetabaseResult<Option<M>>
    where
        M::PrimaryKey: Key + 'static,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        M: for<'a> Value<SelfType<'a> = M>,
        Vec<u8>: TryFrom<
            <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::PrimaryKey,
        >,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
    {
        let definition: TableDefinition<M::PrimaryKey, M> = M::definition(&self.redb_store);
        let table = self.txn.open_table(definition)?;
        let result = table.get(key)?;
        Ok(result.map(|v| v.value()))
    }

    fn get_pk_by_secondary_key<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        secondary_key: <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum,
    ) -> NetabaseResult<Option<M::PrimaryKey>>
    where
        M::PrimaryKey: Key + 'static + for<'a> Value<SelfType<'a> = M::PrimaryKey>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: Key + 'static,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: for<'a> Borrow<
            <<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as Value>::SelfType<'a>,
        >,
        M: for<'a> Value<SelfType<'a> = M>,
        <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum:
            strum::IntoEnumIterator,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
    {
        use strum::IntoDiscriminant;

        // Get the discriminant to determine which secondary key table to use
        let discriminant = secondary_key.discriminant();
        let table_name = M::secondary_key_table_name(discriminant);

        // Open the secondary key table
        let table_def: TableDefinition<
            <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum,
            M::PrimaryKey,
        > = TableDefinition::new(&table_name);
        let table = self.txn.open_table(table_def)?;

        // Look up the primary key
        let result = table.get(secondary_key)?;
        Ok(result.map(|v| v.value()))
    }

    fn get_by_secondary_key<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        secondary_key: <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum,
    ) -> NetabaseResult<Option<M>>
    where
        M::PrimaryKey: Key + 'static + for<'a> Value<SelfType<'a> = M::PrimaryKey>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: Key + 'static,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: for<'a> Borrow<
            <<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as Value>::SelfType<'a>,
        >,
        M: for<'a> Value<SelfType<'a> = M>,
        Vec<u8>: TryFrom<
            <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::PrimaryKey,
        >,
        <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum:
            strum::IntoEnumIterator,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
    {
        if let Some(pk) = self.get_pk_by_secondary_key::<M>(secondary_key)? {
            self.get::<M>(pk)
        } else {
            Ok(None)
        }
    }

    fn get_pk_by_relational_key<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        relational_key: <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum,
    ) -> NetabaseResult<Option<M::PrimaryKey>>
    where
        M::PrimaryKey: Key + 'static + for<'a> Value<SelfType<'a> = M::PrimaryKey>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum: Key + 'static,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum: for<'a> Borrow<
            <<M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as Value>::SelfType<'a>,
        >,
        M: for<'a> Value<SelfType<'a> = M>,
        <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum:
            strum::IntoEnumIterator,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
    {
        use strum::IntoDiscriminant;

        // Get the discriminant to determine which relational key table to use
        let discriminant = relational_key.discriminant();
        let table_name = M::relational_key_table_name(discriminant);

        // Open the relational key table
        let table_def: TableDefinition<
            <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum,
            M::PrimaryKey,
        > = TableDefinition::new(&table_name);
        let table = self.txn.open_table(table_def)?;

        // Look up the primary key
        let result = table.get(relational_key)?;
        Ok(result.map(|v| v.value()))
    }

    fn get_pk_by_hash<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        hash: M::Hash,
    ) -> NetabaseResult<Option<M::PrimaryKey>>
    where
        M::PrimaryKey: Key + 'static + for<'a> Value<SelfType<'a> = M::PrimaryKey>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        M::Hash: Key + 'static,
        M::Hash: for<'a> Borrow<<M::Hash as Value>::SelfType<'a>>,
        M: for<'a> Value<SelfType<'a> = M>,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
    {
        let table_name = M::hash_tree_table_name();
        let table_def: TableDefinition<M::Hash, M::PrimaryKey> =
            TableDefinition::new(&table_name);
        let table = self.txn.open_table(table_def)?;
        
        let result = table.get(hash)?;
        Ok(result.map(|v| v.value()))
    }

    fn get_subscription_accumulator<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        subscription_discriminant: <M as ModelTypeContainer>::Subscriptions,
    ) -> NetabaseResult<([u8; 32], u64)>
    where
        M::PrimaryKey: Key + 'static,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
    {
        let table_name = M::subscription_key_table_name(subscription_discriminant);

        // Open the subscription tree: PrimaryKey -> Hash
        let table_def: TableDefinition<M::PrimaryKey, [u8; 32]> = TableDefinition::new(&table_name);
        let table = self.txn.open_table(table_def)?;

        // XOR accumulator - order-independent hash accumulation
        let mut accumulator = [0u8; 32];
        let mut count = 0u64;

        // Iterate through all entries and XOR their hashes
        for entry in table.iter()? {
            let (_key, hash) = entry?;
            let hash_value = hash.value();

            // XOR each byte of the hash into the accumulator
            for i in 0..32 {
                accumulator[i] ^= hash_value[i];
            }

            count += 1;
        }

        Ok((accumulator, count))
    }

    fn get_subscription_keys<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        subscription_discriminant: <M as ModelTypeContainer>::Subscriptions,
    ) -> NetabaseResult<Vec<M::PrimaryKey>>
    where
        M::PrimaryKey: Key + 'static + for<'a> Value<SelfType<'a> = M::PrimaryKey>,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
    {
        let table_name = M::subscription_key_table_name(subscription_discriminant);

        // Open the subscription tree: PrimaryKey -> Hash
        let table_def: TableDefinition<M::PrimaryKey, [u8; 32]> = TableDefinition::new(&table_name);
        let table = self.txn.open_table(table_def)?;

        let mut keys = Vec::new();

        // Collect all primary keys
        for entry in table.iter()? {
            let (key, _hash) = entry?;
            keys.push(key.value());
        }

        Ok(keys)
    }
}

pub struct RedbWriteTransaction<D: NetabaseDefinition>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    pub(crate) txn: redb::WriteTransaction,
    pub(crate) redb_store: *const RedbStore<D>, // Store reference for model definitions
}

impl<D: NetabaseDefinition> RedbWriteTransaction<D>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    pub fn new(txn: redb::WriteTransaction, redb_store: &RedbStore<D>) -> Self {
        Self {
            txn,
            redb_store: redb_store as *const RedbStore<D>,
        }
    }
}

impl<D: NetabaseDefinition> ReadTransaction<D> for RedbWriteTransaction<D>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    fn get<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        key: M::PrimaryKey,
    ) -> NetabaseResult<Option<M>>
    where
        M::PrimaryKey: Key + 'static,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        M: for<'a> Value<SelfType<'a> = M>,
        Vec<u8>: TryFrom<
            <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::PrimaryKey,
        >,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
    {
        let definition: TableDefinition<M::PrimaryKey, M> =
            M::definition(unsafe { &*self.redb_store });
        let table = self.txn.open_table(definition)?;
        let result = table.get(key)?;
        Ok(result.map(|v| v.value()))
    }

    fn get_pk_by_secondary_key<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        secondary_key: <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum,
    ) -> NetabaseResult<Option<M::PrimaryKey>>
    where
        M::PrimaryKey: Key + 'static + for<'a> Value<SelfType<'a> = M::PrimaryKey>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: Key + 'static,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: for<'a> Borrow<
            <<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as Value>::SelfType<'a>,
        >,
        M: for<'a> Value<SelfType<'a> = M>,
        <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum:
            strum::IntoEnumIterator,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
    {
        use strum::IntoDiscriminant;

        // Get the discriminant to determine which secondary key table to use
        let discriminant = secondary_key.discriminant();
        let table_name = M::secondary_key_table_name(discriminant);

        // Open the secondary key table
        let table_def: TableDefinition<
            <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum,
            M::PrimaryKey,
        > = TableDefinition::new(&table_name);
        let table = self.txn.open_table(table_def)?;

        // Look up the primary key
        let result = table.get(secondary_key)?;
        Ok(result.map(|v| v.value()))
    }

    fn get_pk_by_relational_key<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        relational_key: <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum,
    ) -> NetabaseResult<Option<M::PrimaryKey>>
    where
        M::PrimaryKey: Key + 'static + for<'a> Value<SelfType<'a> = M::PrimaryKey>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum: Key + 'static,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum: for<'a> Borrow<
            <<M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as Value>::SelfType<'a>,
        >,
        M: for<'a> Value<SelfType<'a> = M>,
        <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum:
            strum::IntoEnumIterator,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
    {
        use strum::IntoDiscriminant;

        // Get the discriminant to determine which relational key table to use
        let discriminant = relational_key.discriminant();
        let table_name = M::relational_key_table_name(discriminant);

        // Open the relational key table
        let table_def: TableDefinition<
            <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum,
            M::PrimaryKey,
        > = TableDefinition::new(&table_name);
        let table = self.txn.open_table(table_def)?;

        // Look up the primary key
        let result = table.get(relational_key)?;
        Ok(result.map(|v| v.value()))
    }

    fn get_pk_by_hash<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        hash: M::Hash,
    ) -> NetabaseResult<Option<M::PrimaryKey>>
    where
        M::PrimaryKey: Key + 'static + for<'a> Value<SelfType<'a> = M::PrimaryKey>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        M::Hash: Key + 'static,
        M::Hash: for<'a> Borrow<<M::Hash as Value>::SelfType<'a>>,
        M: for<'a> Value<SelfType<'a> = M>,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
    {
        let table_name = M::hash_tree_table_name();
        let table_def: TableDefinition<M::Hash, M::PrimaryKey> =
            TableDefinition::new(&table_name);
        let table = self.txn.open_table(table_def)?;
        
        let result = table.get(hash)?;
        Ok(result.map(|v| v.value()))
    }

    fn get_subscription_accumulator<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        subscription_discriminant: <M as ModelTypeContainer>::Subscriptions,
    ) -> NetabaseResult<([u8; 32], u64)>
    where
        M::PrimaryKey: Key + 'static,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
    {
        let table_name = M::subscription_key_table_name(subscription_discriminant);

        // Open the subscription tree: PrimaryKey -> Hash
        let table_def: TableDefinition<M::PrimaryKey, [u8; 32]> = TableDefinition::new(&table_name);
        let table = self.txn.open_table(table_def)?;

        // XOR accumulator - order-independent hash accumulation
        let mut accumulator = [0u8; 32];
        let mut count = 0u64;

        // Iterate through all entries and XOR their hashes
        for entry in table.iter()? {
            let (_key, hash) = entry?;
            let hash_value = hash.value();

            // XOR each byte of the hash into the accumulator
            for i in 0..32 {
                accumulator[i] ^= hash_value[i];
            }

            count += 1;
        }

        Ok((accumulator, count))
    }

    fn get_subscription_keys<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        subscription_discriminant: <M as ModelTypeContainer>::Subscriptions,
    ) -> NetabaseResult<Vec<M::PrimaryKey>>
    where
        M::PrimaryKey: Key + 'static + for<'a> Value<SelfType<'a> = M::PrimaryKey>,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
    {
        let table_name = M::subscription_key_table_name(subscription_discriminant);

        // Open the subscription tree: PrimaryKey -> Hash
        let table_def: TableDefinition<M::PrimaryKey, [u8; 32]> = TableDefinition::new(&table_name);
        let table = self.txn.open_table(table_def)?;

        let mut keys = Vec::new();

        // Collect all primary keys
        for entry in table.iter()? {
            let (key, _hash) = entry?;
            keys.push(key.value());
        }

        Ok(keys)
    }
}

impl<D: NetabaseDefinition> WriteTransaction<D> for RedbWriteTransaction<D>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <D as NetabaseDefinition>::Keys: IntoDiscriminant,
{
    fn put<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D> + Send + Clone>(
        &mut self,
        model: M,
    ) -> NetabaseResult<()>
    where
        M::PrimaryKey: Key + 'static + Send + Clone + TryInto<Vec<u8>>,
        Vec<u8>: TryFrom<<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::PrimaryKey>,
        Vec<u8>: TryFrom<<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum>,
        Vec<u8>: TryFrom<<M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        M: for<'a> Borrow<<M as Value>::SelfType<'a>>,
        M::Hash: Clone + Into<[u8; 32]>,
        <<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: IntoEnumIterator + bincode::Encode,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: IntoDiscriminant + Clone + Debug + Send + Key + TryInto<Vec<u8>> + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: Debug,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum: IntoDiscriminant + Clone + Debug + Send + Key + TryInto<Vec<u8>> + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <<M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as IntoDiscriminant>::Discriminant: Debug + bincode::Encode,
        <D as NetabaseDefinition>::Keys: IntoDiscriminant,
        <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: strum::IntoEnumIterator,
        <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum: strum::IntoEnumIterator,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
    {
        // Get store reference (unsafe dereference of raw pointer to store)
        // This is necessary because RedbWriteTransaction holds a raw pointer to RedbStore
        // to avoid lifetime issues with the transaction and the store
        let _store = unsafe { &*self.redb_store };
        let pk = model.primary_key();
        let model_tree_name = M::MODEL_TREE_NAME;
        // let model_hash: [u8; 32] = model.compute_hash().into();

        // 1. Insert into main tree
        let main_table_name =
            D::get_tree_name(&model_tree_name).ok_or(NetabaseError::TreeNotFound)?;
        let main_table_def: TableDefinition<M::PrimaryKey, M> =
            TableDefinition::new(main_table_name.as_str());
        let mut main_table = self.txn.open_table(main_table_def)?;
        main_table.insert(pk.clone(), model.clone())?;

        // 2. Insert into secondary key trees
        for sk in <M::Keys as NetabaseModelKeyTrait<D, M>>::secondary_keys(&model) {
            let sk_discriminant = sk.discriminant();
            let secondary_table_name = M::secondary_key_table_name(sk_discriminant);
            let secondary_table_def: TableDefinition<M::SecondaryKeys, M::PrimaryKey> =
                TableDefinition::new(secondary_table_name.as_str());
            let mut secondary_table = self.txn.open_table(secondary_table_def)?;
            secondary_table.insert(sk, pk.clone())?;
        }

        // 3. Insert into relational key trees
        for rk in <M::Keys as NetabaseModelKeyTrait<D, M>>::relational_keys(&model) {
            let rk_discriminant = rk.discriminant();
            let relational_table_name = M::relational_key_table_name(rk_discriminant);
            let relational_table_def: TableDefinition<M::RelationalKeys, M::PrimaryKey> =
                TableDefinition::new(relational_table_name.as_str());
            let mut relational_table = self.txn.open_table(relational_table_def)?;
            relational_table.insert(rk, pk.clone())?;
        }

        // 4. Insert into hash tree
        let hash_table_name = M::hash_tree_table_name();
        let hash_table_def: TableDefinition<M::Hash, M::PrimaryKey> =
            TableDefinition::new(hash_table_name.as_str());
        let mut hash_table = self.txn.open_table(hash_table_def)?;
        // We use M::Hash directly as key. M::Hash is Key + 'static from RedbNetabaseModelTrait
        hash_table.insert(model.compute_hash(), pk.clone())?;

        // 5. Insert into subscription trees
        for sub in model.get_subscriptions() {
            let subscription_table_name = M::subscription_key_table_name(sub.clone());
            let subscription_table_def: TableDefinition<M::Subscriptions, M::PrimaryKey> =
                TableDefinition::new(subscription_table_name.as_str());
            let mut subscription_table = self.txn.open_table(subscription_table_def)?;
            subscription_table.insert(sub, pk.clone())?;
        }

        Ok(())
    }

    fn delete<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D> + Clone + Send>(
        &mut self,
        key: M::PrimaryKey,
    ) -> NetabaseResult<()>
    where
        M::PrimaryKey: Key + 'static + Send + Clone + TryInto<Vec<u8>>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        M: for<'a> Value<SelfType<'a> = M>,
        Vec<u8>: TryFrom<<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::PrimaryKey>,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum:
            IntoDiscriminant + Clone + Debug + Send + Key + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum:
            IntoDiscriminant + Clone + Debug + Send + Key + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <D as NetabaseDefinition>::Keys: IntoDiscriminant,
        <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum:
            strum::IntoEnumIterator,
        <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum:
            strum::IntoEnumIterator,
        <M as ModelTypeContainer>::SecondaryKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::SecondaryKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::RelationalKeys: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::RelationalKeys as Value>::SelfType<'a>>,
        <M as ModelTypeContainer>::Subscriptions: Key + 'static + for<'a> Borrow<<<M as ModelTypeContainer>::Subscriptions as Value>::SelfType<'a>>,
        <M as NetabaseModelTrait<D>>::Hash: Key + 'static + for<'a> Borrow<<<M as NetabaseModelTrait<D>>::Hash as Value>::SelfType<'a>>,
    {
        // Fetch the model first to know what to delete
        // Note: we can call get() because RedbWriteTransaction implements ReadTransaction
        // self.get() takes &self, so we can reborrow immutable
        let model_option = self.get::<M>(key.clone())?;

        if let Some(model) = model_option {
            let _store = unsafe { &*self.redb_store };
            
            // 2. Delete from secondary key trees
            for sk in <M::Keys as NetabaseModelKeyTrait<D, M>>::secondary_keys(&model) {
                let sk_discriminant = sk.discriminant();
                let secondary_table_name = M::secondary_key_table_name(sk_discriminant);
                let secondary_table_def: TableDefinition<M::SecondaryKeys, M::PrimaryKey> =
                    TableDefinition::new(secondary_table_name.as_str());
                let mut secondary_table = self.txn.open_table(secondary_table_def)?;
                secondary_table.remove(sk)?;
            }

            // 3. Delete from relational key trees
            for rk in <M::Keys as NetabaseModelKeyTrait<D, M>>::relational_keys(&model) {
                let rk_discriminant = rk.discriminant();
                let relational_table_name = M::relational_key_table_name(rk_discriminant);
                let relational_table_def: TableDefinition<M::RelationalKeys, M::PrimaryKey> =
                    TableDefinition::new(relational_table_name.as_str());
                let mut relational_table = self.txn.open_table(relational_table_def)?;
                relational_table.remove(rk)?;
            }

            // 4. Delete from hash tree
            let hash_table_name = M::hash_tree_table_name();
            let hash_table_def: TableDefinition<M::Hash, M::PrimaryKey> =
                TableDefinition::new(hash_table_name.as_str());
            let mut hash_table = self.txn.open_table(hash_table_def)?;
            hash_table.remove(model.compute_hash())?;

            // 5. Delete from subscription trees
            for sub in model.get_subscriptions() {
                let subscription_table_name = M::subscription_key_table_name(sub.clone());
                let subscription_table_def: TableDefinition<M::Subscriptions, M::PrimaryKey> =
                    TableDefinition::new(subscription_table_name.as_str());
                let mut subscription_table = self.txn.open_table(subscription_table_def)?;
                subscription_table.remove(sub)?;
            }

            // 6. Delete from main tree
            let model_tree_name = M::MODEL_TREE_NAME;
            let main_table_name =
                D::get_tree_name(&model_tree_name).ok_or(NetabaseError::TreeNotFound)?;
            let main_table_def: TableDefinition<M::PrimaryKey, M> =
                TableDefinition::new(main_table_name.as_str());
            let mut main_table = self.txn.open_table(main_table_def)?;
            main_table.remove(key)?;
        }

        Ok(())
    }

    fn commit(self) -> NetabaseResult<()> {
        let start = std::time::Instant::now();
        self.txn.commit()?;
        debug!("RedbWriteTransaction: Committed in {:?}", start.elapsed());
        Ok(())
    }
}