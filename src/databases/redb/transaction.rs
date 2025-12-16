use redb::{ReadableDatabase, ReadableTable, TableError, CommitError, TransactionError};
use strum::IntoDiscriminant;
use std::borrow::Borrow;

use crate::{
    databases::redb::{RedbPermissions, RedbStorePermissions},
    relational::{CrossDefinitionPermissions, GlobalDefinitionEnum},
    traits::{
        database::transaction::NBTransaction,
        permissions::{AccessType, NetabasePermissionTicket},
        registery::{
            definition::{NetabaseDefinition, redb_definition::RedbDefinition},
            models::{
                keys::NetabaseModelKeys,
                model::{NetabaseModel, redb_model::{RedbModelTableDefinitions, RedbNetbaseModel}},
            },
        },
    },
    errors::{NetabaseResult, NetabaseError},
};

/// Wrapper around redb::ReadTransaction that enforces permissions
pub struct NetabaseRedbReadTransaction<D: RedbDefinition>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    pub inner: redb::ReadTransaction,
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

pub struct RedbTransactionInner<D: RedbDefinition>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    transaction: RedbTransactionType<D>,
    permissions: RedbPermissions<D>,
}

pub enum RedbTransactionType<D: RedbDefinition>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    Read(NetabaseRedbReadTransaction<D>),
    Write(NetabaseRedbWriteTransaction<D>),
}

pub type RedbTransaction<'db, D> = RedbTransactionInner<D>;

pub struct ModelOpenTables<'txn, 'db, D: RedbDefinition, M: RedbNetbaseModel<'db, D> + redb::Key> 
where
    D::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: redb::Key + 'static,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: redb::Key + 'static,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: redb::Key + 'static,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    // Add missing static bound for subscription keys
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'db>: 'static,
    M: 'static
{
    pub main: TablePermission<'txn, <M::Keys as NetabaseModelKeys<D, M>>::Primary<'db>, M>,

    pub secondary: Vec<(
        TablePermission<'txn, <M::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>, <M::Keys as NetabaseModelKeys<D, M>>::Primary<'db>>,
        &'db str,
    )>,

    pub relational: Vec<(
        TablePermission<'txn, <M::Keys as NetabaseModelKeys<D, M>>::Relational<'db>, <M::Keys as NetabaseModelKeys<D, M>>::Primary<'db>>,
        &'db str,
    )>,

    pub subscription: Vec<(
        TablePermission<'txn, <M::Keys as NetabaseModelKeys<D, M>>::Subscription<'db>, <M::Keys as NetabaseModelKeys<D, M>>::Primary<'db>>,
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
}

/// Trait to handle automatic insertion/update of models into their respective tables
pub trait RedbModelCrud<'db, 'data, D>: RedbNetbaseModel<'data, D>
where
    D: RedbDefinition + Clone,
    <D as IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Primary<'data>: redb::Key + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Secondary<'data>: redb::Key + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Relational<'data>: redb::Key + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription<'data>: redb::Key + 'static,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Secondary<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Relational<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    // Add missing static bound
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription<'data>: 'static,
    Self: 'static
{
    fn create_entry<'txn>(
        &'data self,
        tables: &mut ModelOpenTables<'txn, 'data, D, Self>
    ) -> NetabaseResult<()>;

    fn read_entry<'txn>(
        key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'data>,
        tables: &ModelOpenTables<'txn, 'data, D, Self>
    ) -> NetabaseResult<Option<Self>>;

    fn update_entry<'txn>(
        &'data self,
        tables: &mut ModelOpenTables<'txn, 'data, D, Self>
    ) -> NetabaseResult<()>;

    fn delete_entry<'txn>(
        key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'data>,
        tables: &mut ModelOpenTables<'txn, 'data, D, Self>
    ) -> NetabaseResult<()>;
}

impl<'db, 'data, D, M> RedbModelCrud<'db, 'data, D> for M
where
    D: RedbDefinition + Clone,
    M: RedbNetbaseModel<'data, D> + Clone,
    D::Discriminant: 'static + std::fmt::Debug,
    <D as IntoDiscriminant>::Discriminant: std::fmt::Debug,
    M: std::borrow::Borrow<<M as redb::Value>::SelfType<'data>>,
    for<'a> M: redb::Value<SelfType<'a> = M>,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'data>: redb::Key + Clone + 'static,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'data>: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'data> as redb::Value>::SelfType<'a>>,
    
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'data>: redb::Key + Clone + 'static + PartialEq,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'data>: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'data> as redb::Value>::SelfType<'a>>,
    // Add Into bound to allow bridging lifetimes
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'a>: Into<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'data>>,
    
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'data>: redb::Key + Clone + 'static + PartialEq,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'data>: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'data> as redb::Value>::SelfType<'a>>,
    // Add Into bound to allow bridging lifetimes
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'a>: Into<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'data>>,
    
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'data>: redb::Key + Clone + 'static + PartialEq,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'data>: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'data> as redb::Value>::SelfType<'a>>,
    // Add Into bound to allow bridging lifetimes
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'a>: Into<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'data>>,

    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'data>: 'static,
    M: 'static,
    for<'a> &'a <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'data>: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'data> as redb::Value>::SelfType<'a>>
{
    fn create_entry<'txn>(
        &'data self,
        tables: &mut ModelOpenTables<'txn, 'data, D, Self>
    ) -> NetabaseResult<()> 
    {
        // 1. Insert into Main Table
        match &mut tables.main {
            TablePermission::ReadWrite(ReadWriteTableType::Table(table)) => {
                table.insert(self.get_primary_key().borrow(), self)
                    .map_err(|e| NetabaseError::RedbError(e.into()))?;
            }
            _ => return Err(NetabaseError::Permission),
        }

        // 2. Insert into Secondary Tables
        let secondary_keys = self.get_secondary_keys();
        for ((table_perm, _name), key) in tables.secondary.iter_mut().zip(secondary_keys.into_iter()) {
             match table_perm {
                 TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                     // key is Secondary<'local> (from self via get_secondary_keys)
                     // self is 'data, so key is Secondary<'data>
                     // so key.into() works trivially
                     let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'data> = key.into();
                     table.insert(k.borrow(), self.get_primary_key().borrow())
                         .map_err(|e| NetabaseError::RedbError(e.into()))?;
                 }
                 _ => return Err(NetabaseError::Permission),
             }
        }

        // 3. Insert into Relational Tables
        // TODO: Add permission checks before inserting relational keys
        // For each relational key, we should:
        // 1. Determine the target model discriminant from the relational key
        // 2. Check Self::PERMISSIONS.can_access_model(target_discriminant, AccessType::Create)
        // 3. Return PermissionDenied error if not allowed
        // This requires a mapping from relational key discriminants to target model discriminants
        let relational_keys = self.get_relational_keys();
        for ((table_perm, _name), key) in tables.relational.iter_mut().zip(relational_keys.into_iter()) {
             match table_perm {
                 TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                     let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'data> = key.into();
                     table.insert(k.borrow(), self.get_primary_key().borrow())
                         .map_err(|e| NetabaseError::RedbError(e.into()))?;
                 }
                 _ => return Err(NetabaseError::Permission),
             }
        }

        // 4. Insert into Subscription Tables
        let subscription_keys = self.get_subscription_keys();
        for ((table_perm, _name), key) in tables.subscription.iter_mut().zip(subscription_keys.into_iter()) {
             match table_perm {
                 TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                     let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'data> = key.into();
                     table.insert(k.borrow(), self.get_primary_key().borrow())
                         .map_err(|e| NetabaseError::RedbError(e.into()))?;
                 }
                 _ => return Err(NetabaseError::Permission),
             }
        }

        Ok(())
    }

    fn read_entry<'txn>(
        key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'data>,
        tables: &ModelOpenTables<'txn, 'data, D, Self>
    ) -> NetabaseResult<Option<Self>>
    {
        match &tables.main {
            TablePermission::ReadOnly(TableType::Table(table)) => {
                let result = table.get(key.borrow()).map_err(|e| NetabaseError::RedbError(e.into()))?;
                Ok(result.map(|access_guard| access_guard.value()))
            },
            TablePermission::ReadWrite(ReadWriteTableType::Table(table)) => {
                let result = table.get(key.borrow()).map_err(|e| NetabaseError::RedbError(e.into()))?;
                Ok(result.map(|access_guard| access_guard.value()))
            },
            _ => Err(NetabaseError::Permission),
        }
    }

    fn update_entry<'txn>(
        &'data self,
        tables: &mut ModelOpenTables<'txn, 'data, D, Self>
    ) -> NetabaseResult<()>
    {
        // 1. Get the old model
        let old_model = Self::read_entry(&self.get_primary_key(), tables)?;
        
        // 2. Update Main Table
        match &mut tables.main {
            TablePermission::ReadWrite(ReadWriteTableType::Table(table)) => {
                table.insert(self.get_primary_key().borrow(), self)
                    .map_err(|e| NetabaseError::RedbError(e.into()))?;
            }
            _ => return Err(NetabaseError::Permission),
        }

        if let Some(old_model) = old_model {
            let primary_key = self.get_primary_key();

            // 3. Update Secondary Tables
            let old_secondary = old_model.get_secondary_keys();
            let new_secondary = self.get_secondary_keys();
            
            for (((table_perm, _name), old_key), new_key) in tables.secondary.iter_mut()
                .zip(old_secondary.into_iter())
                .zip(new_secondary.into_iter()) 
            {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        // Convert keys to 'data lifetime to satisfy Borrow bound and PartialEq
                        let old_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'data> = old_key.into();
                        let new_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'data> = new_key.into();
                        
                        if old_k != new_k {
                            table.remove(old_k.borrow(), primary_key.borrow())
                                .map_err(|e| NetabaseError::RedbError(e.into()))?;
                            table.insert(new_k.borrow(), primary_key.borrow())
                                .map_err(|e| NetabaseError::RedbError(e.into()))?;
                        }
                    }
                    _ => return Err(NetabaseError::Permission),
                }
            }

            // 4. Update Relational Tables
            // TODO: Add permission checks before updating relational keys
            // Should check permissions for both removing old relations and creating new ones
            // Similar to create_entry, requires mapping relational key discriminants to target models
            let old_relational = old_model.get_relational_keys();
            let new_relational = self.get_relational_keys();

            for (((table_perm, _name), old_key), new_key) in tables.relational.iter_mut()
                .zip(old_relational.into_iter())
                .zip(new_relational.into_iter())
            {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        let old_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'data> = old_key.into();
                        let new_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'data> = new_key.into();

                        if old_k != new_k {
                            table.remove(old_k.borrow(), primary_key.borrow())
                                .map_err(|e| NetabaseError::RedbError(e.into()))?;
                            table.insert(new_k.borrow(), primary_key.borrow())
                                .map_err(|e| NetabaseError::RedbError(e.into()))?;
                        }
                    }
                    _ => return Err(NetabaseError::Permission),
                }
            }

            // 5. Update Subscription Tables
            let old_subscription = old_model.get_subscription_keys();
            let new_subscription = self.get_subscription_keys();

            for (((table_perm, _name), old_key), new_key) in tables.subscription.iter_mut()
                .zip(old_subscription.into_iter())
                .zip(new_subscription.into_iter())
            {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        let old_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'data> = old_key.into();
                        let new_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'data> = new_key.into();

                        if old_k != new_k {
                            table.remove(old_k.borrow(), primary_key.borrow())
                                .map_err(|e| NetabaseError::RedbError(e.into()))?;
                            table.insert(new_k.borrow(), primary_key.borrow())
                                .map_err(|e| NetabaseError::RedbError(e.into()))?;
                        }
                    }
                    _ => return Err(NetabaseError::Permission),
                }
            }
        } else {
            self.create_entry(tables)?;
        }

        Ok(())
    }

    fn delete_entry<'txn>(
        key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'data>,
        tables: &mut ModelOpenTables<'txn, 'data, D, Self>
    ) -> NetabaseResult<()>
    {
        let model = Self::read_entry(key, tables)?;

        if let Some(model) = model {
            // 2. Remove from Main Table
            match &mut tables.main {
                TablePermission::ReadWrite(ReadWriteTableType::Table(table)) => {
                    table.remove(key.borrow())
                        .map_err(|e| NetabaseError::RedbError(e.into()))?;
                }
                _ => return Err(NetabaseError::Permission),
            }

            // 3. Remove from Secondary Tables
            let secondary_keys = model.get_secondary_keys();
            for ((table_perm, _name), secondary_key) in tables.secondary.iter_mut().zip(secondary_keys.into_iter()) {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'data> = secondary_key.into();
                        table.remove(k.borrow(), key.borrow())
                            .map_err(|e| NetabaseError::RedbError(e.into()))?;
                    }
                    _ => return Err(NetabaseError::Permission),
                }
            }

            // 4. Remove from Relational Tables
            // TODO: Add permission checks before deleting relational keys
            // Should verify model has permission to modify relations with target models
            // Requires mapping relational key discriminants to target model discriminants
            let relational_keys = model.get_relational_keys();
            for ((table_perm, _name), relational_key) in tables.relational.iter_mut().zip(relational_keys.into_iter()) {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'data> = relational_key.into();
                        table.remove(k.borrow(), key.borrow())
                            .map_err(|e| NetabaseError::RedbError(e.into()))?;
                    }
                    _ => return Err(NetabaseError::Permission),
                }
            }

            // 5. Remove from Subscription Tables
            let subscription_keys = model.get_subscription_keys();
            for ((table_perm, _name), subscription_key) in tables.subscription.iter_mut().zip(subscription_keys.into_iter()) {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'data> = subscription_key.into();
                        table.remove(k.borrow(), key.borrow())
                            .map_err(|e| NetabaseError::RedbError(e.into()))?;
                    }
                    _ => return Err(NetabaseError::Permission),
                }
            }
        }

        Ok(())
    }
}

// ... rest of file (same as before) ...
impl<'db, D: RedbDefinition> RedbTransaction<'db, D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    pub fn new(
        db: RedbStorePermissions,
        permissions: RedbPermissions<D>,
    ) -> NetabaseResult<Self> {
        let transaction = match db {
            RedbStorePermissions::ReadOnly(read_only_database) => {
                let read_txn = read_only_database
                    .begin_read()
                    .map_err(|e: TransactionError| NetabaseError::RedbTransactionError(e.into()))?;
                RedbTransactionType::Read(NetabaseRedbReadTransaction::new(read_txn, permissions.clone()))
            }
            RedbStorePermissions::ReadWrite(database) => {
                let write_txn = database
                    .begin_write()
                    .map_err(|e: TransactionError| NetabaseError::RedbTransactionError(e.into()))?;
                RedbTransactionType::Write(NetabaseRedbWriteTransaction::new(write_txn, permissions.clone()))
            }
        };

        Ok(RedbTransactionInner {
            transaction,
            permissions,
        })
    }

    /// Open tables for a specific model with proper permission checking (concrete implementation)
    ///
    /// TODO: Implement full relational permission filtering
    /// Currently opens all relational tables defined in M::TREE_NAMES.
    /// Future enhancement: Filter relational tables based on M::PERMISSIONS to only open
    /// tables for which the model has access permissions. This requires mapping from
    /// relational key discriminants to target model discriminants.
    pub fn open_model_tables<'txn, 'data, M>(
        &'txn self,
        definitions: RedbModelTableDefinitions<'data, M, D>,
    ) -> NetabaseResult<ModelOpenTables<'txn, 'data, D, M>>
    where
        M: RedbNetbaseModel<'data, D> + redb::Key + 'static,
        D::Discriminant: 'static + std::fmt::Debug,
        D: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'data>: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'data>: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'data>: redb::Key + 'static,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'data>: 'static,
    {

        // Access model permissions for future permission-based filtering
        let _table_definitions = definitions;  // Keep for future use

        // Use static table names from M::TREE_NAMES
        let main_def = redb::TableDefinition::new(M::TREE_NAMES.main.table_name);

        match &self.transaction {
            RedbTransactionType::Read(read_txn) => {
                // For read transactions, open read-only tables
                let main_table = {
                    read_txn.open_table(main_def).map(|table| TablePermission::ReadOnly(TableType::Table(table)))?
                };

                let secondary_tables: Result<Vec<_>, NetabaseError> = M::TREE_NAMES
                    .secondary
                    .iter()
                    .map(
                        |disc_table| -> Result<_, NetabaseError> {
                            let def = redb::MultimapTableDefinition::new(disc_table.table_name);
                            read_txn.open_multimap_table(def).map(|table| (TablePermission::ReadOnly(TableType::MultimapTable(table)), disc_table.table_name))
                        },
                    )
                    .collect();

                let relational_tables: Result<Vec<_>, NetabaseError> = M::TREE_NAMES
                    .relational
                    .iter()
                    .map(
                        |disc_table| -> Result<_, NetabaseError> {
                            let def = redb::MultimapTableDefinition::new(disc_table.table_name);
                            read_txn.open_multimap_table(def).map(|table| (TablePermission::ReadOnly(TableType::MultimapTable(table)), disc_table.table_name))
                        },
                    )
                    .collect();

                let subscription_tables: Result<Vec<_>, NetabaseError> = match M::TREE_NAMES.subscription {
                    Some(subs) => subs
                        .iter()
                        .map(
                            |disc_table| -> Result<_, NetabaseError> {
                                let def = redb::MultimapTableDefinition::new(disc_table.table_name);
                                read_txn.open_multimap_table(def).map(|table| (TablePermission::ReadOnly(TableType::MultimapTable(table)), disc_table.table_name))
                            },
                        )
                        .collect(),
                    None => Ok(Vec::new()),
                };

                Ok(ModelOpenTables {
                    main: main_table,
                    secondary: secondary_tables?,
                    relational: relational_tables?,
                    subscription: subscription_tables?,
                })
            }
            RedbTransactionType::Write(write_txn) => {
                // For write transactions, open read-write tables
                let main_table = {
                    write_txn.open_table(main_def).map(|table| TablePermission::ReadWrite(ReadWriteTableType::Table(table)))?
                };

                let secondary_tables: Result<Vec<_>, NetabaseError> = M::TREE_NAMES
                    .secondary
                    .iter()
                    .map(
                        |disc_table| -> Result<_, NetabaseError> {
                            let def = redb::MultimapTableDefinition::new(disc_table.table_name);
                            write_txn.open_multimap_table(def).map(|table| (TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)), disc_table.table_name))
                        },
                    )
                    .collect();

                let relational_tables: Result<Vec<_>, NetabaseError> = M::TREE_NAMES
                    .relational
                    .iter()
                    .map(
                        |disc_table| -> Result<_, NetabaseError> {
                            let def = redb::MultimapTableDefinition::new(disc_table.table_name);
                            write_txn.open_multimap_table(def).map(|table| (TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)), disc_table.table_name))
                        },
                    )
                    .collect();

                let subscription_tables: Result<Vec<_>, NetabaseError> = match M::TREE_NAMES.subscription {
                    Some(subs) => subs
                        .iter()
                        .map(
                            |disc_table| -> Result<_, NetabaseError> {
                                let def = redb::MultimapTableDefinition::new(disc_table.table_name);
                                write_txn.open_multimap_table(def).map(|table| (TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)), disc_table.table_name))
                            },
                        )
                        .collect(),
                    None => Ok(Vec::new()),
                };

                Ok(ModelOpenTables {
                    main: main_table,
                    secondary: secondary_tables?,
                    relational: relational_tables?,
                    subscription: subscription_tables?,
                })
            }
        }
    }

    /// Execute a function with the raw read transaction (limited scope)
    pub fn with_read_transaction<F, R>(&self, f: F) -> NetabaseResult<R>
    where
        F: FnOnce(&redb::ReadTransaction) -> NetabaseResult<R>,
    {
        match &self.transaction {
            RedbTransactionType::Read(read_txn) => f(&read_txn.inner),
            RedbTransactionType::Write(_) => return Err(NetabaseError::Permission),
        }
    }

    /// Execute a function with the raw write transaction (limited scope)
    pub fn with_write_transaction<F, R>(&self, f: F) -> NetabaseResult<R>
    where
        F: FnOnce(&redb::WriteTransaction) -> NetabaseResult<R>,
    {
        match &self.transaction {
            RedbTransactionType::Write(write_txn) => f(&write_txn.inner),
            RedbTransactionType::Read(_) => Err(NetabaseError::Permission),
        }
    }

    pub fn commit(self) -> NetabaseResult<()> {
        match self.transaction {
            RedbTransactionType::Write(write_txn) => write_txn.commit(),
            RedbTransactionType::Read(_) => {
                // Read transactions don't need to be committed
                Ok(())
            }
        }
    }

    // --- Inherent methods for Redb models ---

    pub fn create_redb<'data, M>(&'db self, model: &'data M) -> NetabaseResult<()>
    where
        M: RedbModelCrud<'db, 'data, D> + RedbNetbaseModel<'data, D> + Clone + std::borrow::Borrow<<M as redb::Value>::SelfType<'data>> + redb::Value<SelfType<'data> = M>,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'data>: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'data>: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'data>: Clone,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'data>: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'data>: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'data>: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'data>: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'data>: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'data>: 'static, <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'data>: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'data> as redb::Value>::SelfType<'data>>,
        // Add Subscription bounds
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'data>: 'static,
    {

        let definitions = M::table_definitions();
        let mut tables = self.open_model_tables(definitions)?;

        model.create_entry(&mut tables)
    }
}

impl<'db, D: RedbDefinition + GlobalDefinitionEnum> NBTransaction<'db, D> for RedbTransaction<'db, D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    type ReadTransaction = NetabaseRedbReadTransaction<D>;
    type WriteTransaction = NetabaseRedbWriteTransaction<D>;

    fn create<M>(&self, _model: M) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant:
            'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'static>: 'static
    {
         todo!("NBTransaction::create: Requires M to be RedbNetbaseModel. This trait bound mismatch is expected.")
    }

    fn read<M>(&self, _key: M::Keys) -> NetabaseResult<M>
    where
        M: NetabaseModel<D>,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant:
            'static,
    {
        todo!("NBTransaction::read")
    }

    fn update<M>(&self, _model: M) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant:
            'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'static>: 'static
    {
        todo!("NBTransaction::update")
    }

    fn delete<M>(&self, _key: M::Keys) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant:
            'static,
    {
        todo!("NBTransaction::delete")
    }

    fn create_many<M>(&self, _models: Vec<M>) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant:
            'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'static>: 'static, <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'static>: 'static
    {
        todo!("NBTransaction::create_many")
    }

    fn read_if<M, F>(&self, _predicate: F) -> NetabaseResult<Vec<M>>
    where
        M: NetabaseModel<D>,
        F: Fn(&M) -> bool,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant:
            'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'static>: 'static
    {
        todo!("NBTransaction::read_if")
    }

    fn read_range<M, K>(&self, _range: std::ops::Range<K>) -> NetabaseResult<Vec<M>>
    where
        M: NetabaseModel<D>,
        K: Into<M::Keys>,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant:
            'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'static>: 'static
    {
        todo!("NBTransaction::read_range")
    }

    fn update_range<M, K, F>(&self, _range: std::ops::Range<K>, _updater: F) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        K: Into<M::Keys>,
        F: Fn(&mut M),
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant:
            'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'static>: 'static
    {
        todo!("NBTransaction::update_range")
    }

    fn update_if<M, P, U>(&self, _predicate: P, _updater: U) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        P: Fn(&M) -> bool,
        U: Fn(&mut M),
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant:
            'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'static>: 'static
    {
        todo!("NBTransaction::update_if")
    }

    fn delete_many<M>(&self, _keys: Vec<M::Keys>) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant:
            'static,
    {
        todo!("NBTransaction::delete_many")
    }

    fn delete_if<M, F>(&self, _predicate: F) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        F: Fn(&M) -> bool,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant:
            'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'static>: 'static
    {
        todo!("NBTransaction::delete_if")
    }

    fn delete_range<M, K>(&self, _range: std::ops::Range<K>) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        K: Into<M::Keys>,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant:
            'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'static>: 'static
    {
        todo!("NBTransaction::delete_range")
    }

    fn write<F, R>(&self, f: F) -> NetabaseResult<R>
    where
        F: FnOnce(&Self::WriteTransaction) -> NetabaseResult<R>
    {
        match &self.transaction {
            RedbTransactionType::Write(write_txn) => f(write_txn),
            RedbTransactionType::Read(_) => Err(NetabaseError::Permission),
        }
    }

    fn read_fn<F, R>(&self, f: F) -> NetabaseResult<R>
    where
        F: FnOnce(&Self::ReadTransaction) -> NetabaseResult<R>
    {
        match &self.transaction {
            RedbTransactionType::Read(read_txn) => f(read_txn),
            RedbTransactionType::Write(_) => Err(NetabaseError::Permission), // This case should ideally not happen for a Read fn
        }
    }

    fn read_related<OD, M>(&self, _key: M::Keys) -> NetabaseResult<Option<M>>
    where
        OD: NetabaseDefinition,
        M: NetabaseModel<OD>,
        <OD as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        for<'a> <<M::Keys as NetabaseModelKeys<OD, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<OD, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<OD, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant:
            'static,
    {
        todo!("NBTransaction::read_related")
    }

    fn can_access_definition<OD>(&self) -> bool
    where
        OD: NetabaseDefinition,
        <OD as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug
    {
        // Simple implementation
        true 
    }

    /*
    fn get_cross_permissions<OD>(&self) -> Option<CrossDefinitionPermissions<D>>
    where
        OD: NetabaseDefinition + GlobalDefinitionEnum,
        <OD as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug
    {
        None
    }
    */
}
