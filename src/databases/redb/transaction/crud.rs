use redb::{self, AccessGuard, ReadableTable, ReadableTableMetadata};
use strum::IntoDiscriminant;
use std::borrow::Borrow;

use crate::{
    traits::registery::{
        definition::redb_definition::RedbDefinition,
        models::{
            keys::{NetabaseModelKeys, blob::NetabaseModelBlobKey},
            model::{NetabaseModel, redb_model::RedbNetbaseModel},
        },
    },
    errors::{NetabaseResult, NetabaseError},
};
use super::tables::{ModelOpenTables, TablePermission, ReadWriteTableType, TableType};
use super::options::CrudOptions;

/// Trait to handle automatic insertion/update of models into their respective tables
pub trait RedbModelCrud<'db,  D>: RedbNetbaseModel<'db, D>
where
    D: RedbDefinition + Clone,
    <D as IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Primary<'db>: redb::Key + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Secondary<'db>: redb::Key + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Relational<'db>: redb::Key + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription<'db>: redb::Key + 'static,
    D::SubscriptionKeys: redb::Key + 'static,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Secondary<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Relational<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    // Add missing static bound
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription<'db>: 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob<'db>: redb::Key + 'static,
    for<'a> <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob<'db>: std::borrow::Borrow<<<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob<'db> as redb::Value>::SelfType<'a>>,
    for<'a> <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob<'a>: Into<<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob<'db>>,
    <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob<'db> as NetabaseModelBlobKey<'db, D, Self, <Self as NetabaseModel<D>>::Keys>>::BlobItem: redb::Key + 'static,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob<'db> as NetabaseModelBlobKey<'db, D, Self, <Self as NetabaseModel<D>>::Keys>>::BlobItem: std::borrow::Borrow<<<<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob<'db> as NetabaseModelBlobKey<'db, D, Self, <Self as NetabaseModel<D>>::Keys>>::BlobItem as redb::Value>::SelfType<'a>>,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob<'a> as NetabaseModelBlobKey<'a, D, Self, <Self as NetabaseModel<D>>::Keys>>::BlobItem: Into<<<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob<'db> as NetabaseModelBlobKey<'db, D, Self, <Self as NetabaseModel<D>>::Keys>>::BlobItem>,
    for<'a> <Self as RedbNetbaseModel<'db, D>>::TableV: redb::Value<SelfType<'a> = Self>,
    Self: 'db
{
    fn create_entry<'txn>(
        &'db self,
        tables: &mut ModelOpenTables<'txn, 'db, D, Self>
    ) -> NetabaseResult<()>;

    fn read_entry<'txn>(
        key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'db>,
        tables: &'txn ModelOpenTables<'txn, 'db, D, Self>,
        config: CrudOptions,
    ) -> NetabaseResult<Option<AccessGuard<'txn, <Self as RedbNetbaseModel<'db, D>>::TableV>>>
    where
    'db: 'txn;

    fn read_default<'txn>(
        key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'db>,
        tables: &'txn ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Option<Self>>
    where
    'db: 'txn,
    {
        Self::read_entry(key, tables, CrudOptions::default())
            .map(|opt| opt.map(|g| g.value()))
    }

    fn update_entry<'txn>(
        &'db self,
        tables: &mut ModelOpenTables<'txn, 'db, D, Self>
    ) -> NetabaseResult<()>;

    fn delete_entry<'txn>(
        key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'db>,
        tables: &mut ModelOpenTables<'txn, 'db, D, Self>
    ) -> NetabaseResult<()>;

    fn list_entries<'a, 'txn>(
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
        config: CrudOptions,
    ) -> NetabaseResult<Vec<AccessGuard<'a, <Self as RedbNetbaseModel<'db, D>>::TableV>>>;

    fn list_default<'a, 'txn>(
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Vec<Self>> {
        Self::list_entries(tables, CrudOptions::default())
            .map(|vec| vec.into_iter().map(|g| g.value()).collect())
    }

    fn list_range<'a, 'txn, R>(
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
        range: R,
        config: CrudOptions,
    ) -> NetabaseResult<Vec<AccessGuard<'a, <Self as RedbNetbaseModel<'db, D>>::TableV>>>
    where R: std::ops::RangeBounds<<Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'db>> + Clone;

    fn count_entries<'txn>(
        tables: &ModelOpenTables<'txn, 'db, D, Self>
    ) -> NetabaseResult<u64>;

    /// Query primary keys by subscription topic.
    /// 
    /// Returns a list of primary keys for all models subscribed to the given topic.
    /// Use the subscription enum variant (e.g., `DefinitionSubscriptions::Topic1`) as the key.
    fn query_by_subscription<'a, 'txn, S>(
        subscription_key: &S,
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Vec<<Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'db>>>
    where
        S: Into<D::SubscriptionKeys> + Clone,
        D::SubscriptionKeys: redb::Key + 'static,
        <Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'db>: Clone,
        for<'v> <Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'db>: redb::Value<SelfType<'v> = <Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'db>>;
}

impl<'db, D, M> RedbModelCrud<'db, D> for M
where
    D: RedbDefinition + Clone,
    M: RedbNetbaseModel<'db, D> + Clone,
    D::Discriminant: 'static + std::fmt::Debug,
    <D as IntoDiscriminant>::Discriminant: std::fmt::Debug,
    for<'a> M::TableV: redb::Value<SelfType<'a> = M>,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: redb::Key + 'static,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db> as redb::Value>::SelfType<'a>>,
    
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: redb::Key + 'static,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db> as redb::Value>::SelfType<'a>>,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'a>: Into<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>>,
    
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: redb::Key + 'static,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db> as redb::Value>::SelfType<'a>>,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'a>: Into<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>>,
    
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'db>: redb::Key + 'static,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'db>: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'db> as redb::Value>::SelfType<'a>>,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'a>: Into<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'db>>,

    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'db>: 'static,
    D::SubscriptionKeys: redb::Key + 'static + PartialEq,
    for<'a> D::SubscriptionKeys: std::borrow::Borrow<<D::SubscriptionKeys as redb::Value>::SelfType<'a>>,
    M: 'db,
    for<'a> &'a <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db> as redb::Value>::SelfType<'a>>,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'db>: redb::Key + 'static,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'db>: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'db> as redb::Value>::SelfType<'a>>,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'a>: Into<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'db>>,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'db> as NetabaseModelBlobKey<'db, D, M, <M as NetabaseModel<D>>::Keys>>::BlobItem: redb::Key + 'static,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'db> as NetabaseModelBlobKey<'db, D, M, <M as NetabaseModel<D>>::Keys>>::BlobItem: std::borrow::Borrow<<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'db> as NetabaseModelBlobKey<'db, D, M, <M as NetabaseModel<D>>::Keys>>::BlobItem as redb::Value>::SelfType<'a>>,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'a> as NetabaseModelBlobKey<'a, D, M, <M as NetabaseModel<D>>::Keys>>::BlobItem: Into<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'db> as NetabaseModelBlobKey<'db, D, M, <M as NetabaseModel<D>>::Keys>>::BlobItem>
{
    fn create_entry<'txn>(
        &'db self,
        tables: &mut ModelOpenTables<'txn, 'db, D, Self>
    ) -> NetabaseResult<()> 
    {
        // 1. Insert into Main Table
        match &mut tables.main {
            TablePermission::ReadWrite(ReadWriteTableType::Table(table)) => {
                table.insert(self.get_primary_key().borrow(), self.borrow())
                    .map_err(|e| NetabaseError::RedbError(e.into()))?;
            }
            _ => return Err(NetabaseError::Other),
        }

        // 2. Insert into Secondary Tables
        let secondary_keys: Vec<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'_>> = self.get_secondary_keys();
        for ((table_perm, _name), key) in tables.secondary.iter_mut().zip::<std::vec::IntoIter<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'_>>>(<Vec<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'_>> as IntoIterator>::into_iter(secondary_keys)) {
             match table_perm {
                 TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                     // key is Secondary<'local> (from self via get_secondary_keys)
                     // self is 'data, so key is Secondary<'data>
                     // so key.into() works trivially
                     let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db> = key.into();
                     table.insert(k.borrow(), self.get_primary_key().borrow())
                         .map_err(|e| NetabaseError::RedbError(e.into()))?;
                 }
                 _ => return Err(NetabaseError::Other),
             }
        }

        // 3. Insert into Relational Tables
        // Store as: PrimaryKey -> RelationalKey (swapped from previous implementation)
        // This allows looking up related foreign keys from a model's primary key
        let relational_keys = self.get_relational_keys();
        let primary_key = self.get_primary_key();
        for ((table_perm, _name), key) in tables.relational.iter_mut().zip(relational_keys.into_iter()) {
             match table_perm {
                 TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                     let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db> = key.into();
                     // Swapped: primary_key is now the key, relational key is the value
                     table.insert(primary_key.borrow(), k.borrow())
                         .map_err(|e| NetabaseError::RedbError(e.into()))?;
                 }
                 _ => return Err(NetabaseError::Other),
             }
        }

        // 4. Insert into Subscription Tables
        let subscription_keys = self.get_subscription_keys();
        for ((table_perm, _name), key) in tables.subscription.iter_mut().zip(subscription_keys.into_iter()) {
             match table_perm {
                 TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                     // Convert model-specific subscription key to definition-level subscription key
                     let model_key: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'db> = key.into();
                     let def_key: D::SubscriptionKeys = model_key.try_into().map_err(|_| NetabaseError::Other)?;
                     table.insert(def_key.borrow(), self.get_primary_key().borrow())
                         .map_err(|e| NetabaseError::RedbError(e.into()))?;
                 }
                 _ => return Err(NetabaseError::Other),
             }
        }

        // 5. Insert into Blob Tables
        let blob_entries = self.get_blob_entries();
        for ((table_perm, _name), field_blobs) in tables.blob.iter_mut().zip(blob_entries.into_iter()) {
             match table_perm {
                 TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                     for (key, item) in field_blobs {
                         table.insert(key, item)
                             .map_err(|e| NetabaseError::RedbError(e.into()))?;
                     }
                 }
                 _ => return Err(NetabaseError::Other),
             }
        }

        Ok(())
    }

    fn read_entry<'a, 'txn>(
        key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'db>,
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
        _config: CrudOptions,
    ) -> NetabaseResult<Option<AccessGuard<'a, <Self as RedbNetbaseModel<'db, D>>::TableV>>>
    where
    'db: 'txn
{
        match &tables.main {
            TablePermission::ReadOnly(TableType::Table(table)) => {
                let result = table.get(key.borrow()).map_err(|e| NetabaseError::RedbError(e.into()))?;
                Ok(result)

            },
            TablePermission::ReadWrite(ReadWriteTableType::Table(table)) => {
                let result = table.get(key.borrow()).map_err(|e| NetabaseError::RedbError(e.into()))?;
                Ok(result)
            },
            _ => Err(NetabaseError::Other),
        }
    }

    fn update_entry<'txn>(
        &'db self,
        tables: &mut ModelOpenTables<'txn, 'db, D, Self>
    ) -> NetabaseResult<()>
    {
        // 1. Update Main Table and get old model in one operation
        // redb's insert() returns the old value if the key existed
        let old_model = match &mut tables.main {
            TablePermission::ReadWrite(ReadWriteTableType::Table(table)) => {
                table.insert(self.get_primary_key().borrow(), self)
                    .map_err(|e| NetabaseError::RedbError(e.into()))?
                    .map(|access_guard| access_guard.value())
            }
            _ => return Err(NetabaseError::Other),
        };

        let primary_key = self.get_primary_key();

        if let Some(old_model) = old_model {
            // Model existed, update secondary/relational/subscription tables by comparing old and new keys

            // 2. Update Secondary Tables
            let old_secondary: Vec<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'_>> = old_model.get_secondary_keys();
            let new_secondary: Vec<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'_>> = self.get_secondary_keys();

            for (((table_perm, _name), old_key), new_key) in tables.secondary.iter_mut()
                .zip::<std::vec::IntoIter<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'_>>>(<Vec<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'_>> as IntoIterator>::into_iter(old_secondary))
                .zip::<std::vec::IntoIter<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'_>>>(<Vec<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'_>> as IntoIterator>::into_iter(new_secondary))
            {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        let old_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db> = old_key.into();
                        let new_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db> = new_key.into();

                        if old_k != new_k {
                            table.remove(old_k.borrow(), primary_key.borrow())
                                .map_err(|e| NetabaseError::RedbError(e.into()))?;
                            table.insert(new_k.borrow(), primary_key.borrow())
                                .map_err(|e| NetabaseError::RedbError(e.into()))?;
                        }
                    }
                    _ => return Err(NetabaseError::Other),
                }
            }

            // 3. Update Relational Tables
            let old_relational = old_model.get_relational_keys();
            let new_relational = self.get_relational_keys();

            for (((table_perm, _name), old_key), new_key) in tables.relational.iter_mut()
                .zip(old_relational.into_iter())
                .zip(new_relational.into_iter())
            {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        let old_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db> = old_key.into();
                        let new_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db> = new_key.into();

                        if old_k != new_k {
                            table.remove(primary_key.borrow(), old_k.borrow())
                                .map_err(|e| NetabaseError::RedbError(e.into()))?;
                            table.insert(primary_key.borrow(), new_k.borrow())
                                .map_err(|e| NetabaseError::RedbError(e.into()))?;
                        }
                    }
                    _ => return Err(NetabaseError::Other),
                }
            }

            // 4. Update Subscription Tables
            let old_subscription = old_model.get_subscription_keys();
            let new_subscription = self.get_subscription_keys();

            for (((table_perm, _name), old_key), new_key) in tables.subscription.iter_mut()
                .zip(old_subscription.into_iter())
                .zip(new_subscription.into_iter())
            {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        let old_model_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'db> = old_key.into();
                        let new_model_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'db> = new_key.into();

                        let old_def_k: D::SubscriptionKeys = old_model_k.try_into().map_err(|_| NetabaseError::Other)?;
                        let new_def_k: D::SubscriptionKeys = new_model_k.try_into().map_err(|_| NetabaseError::Other)?;

                        if old_def_k != new_def_k {
                            table.remove(old_def_k.borrow(), primary_key.borrow())
                                .map_err(|e| NetabaseError::RedbError(e.into()))?;
                            table.insert(new_def_k.borrow(), primary_key.borrow())
                                .map_err(|e| NetabaseError::RedbError(e.into()))?;
                        }
                    }
                    _ => return Err(NetabaseError::Other),
                }
            }

            // 5. Update Blob Tables
            let old_blob_entries = old_model.get_blob_entries();
            let new_blob_entries = self.get_blob_entries();

            for (((table_perm, _name), old_blobs), new_blobs) in tables.blob.iter_mut()
                .zip(old_blob_entries.into_iter())
                .zip(new_blob_entries.into_iter())
            {
                 match table_perm {
                     TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        for (old_key, old_item) in old_blobs {
                            let old_key: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'db> = old_key.into();
                            let old_item: <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'db> as NetabaseModelBlobKey<'db, D, M, <M as NetabaseModel<D>>::Keys>>::BlobItem = old_item.into();
                            
                            table.remove(old_key, old_item)
                                .map_err(|e| NetabaseError::RedbError(e.into()))?;
                        }

                        for (new_key, new_item) in new_blobs {
                            table.insert(new_key, new_item)
                                .map_err(|e| NetabaseError::RedbError(e.into()))?;
                        }
                     }
                     _ => return Err(NetabaseError::Other),
                 }
            }
        } else {
            // Model didn't exist before, insert into secondary/relational/subscription tables
            // (main table already updated above)

            // Insert into Secondary Tables
            let secondary_keys = self.get_secondary_keys();
            for ((table_perm, _name), key) in tables.secondary.iter_mut().zip(secondary_keys.into_iter()) {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db> = key.into();
                        table.insert(k.borrow(), primary_key.borrow())
                            .map_err(|e| NetabaseError::RedbError(e.into()))?;
                    }
                    _ => return Err(NetabaseError::Other),
                }
            }

            // Insert into Relational Tables
            let relational_keys = self.get_relational_keys();
            for ((table_perm, _name), key) in tables.relational.iter_mut().zip(relational_keys.into_iter()) {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db> = key.into();
                        table.insert(primary_key.borrow(), k.borrow())
                            .map_err(|e| NetabaseError::RedbError(e.into()))?;
                    }
                    _ => return Err(NetabaseError::Other),
                }
            }

            // Insert into Subscription Tables
            let subscription_keys = self.get_subscription_keys();
            for ((table_perm, _name), key) in tables.subscription.iter_mut().zip(subscription_keys.into_iter()) {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        let model_key: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'db> = key.into();
                        let def_key: D::SubscriptionKeys = model_key.try_into().map_err(|_| NetabaseError::Other)?;
                        table.insert(def_key.borrow(), primary_key.borrow())
                            .map_err(|e| NetabaseError::RedbError(e.into()))?;
                    }
                    _ => return Err(NetabaseError::Other),
                }
            }

            // Insert into Blob Tables
            let blob_entries = self.get_blob_entries();
            for ((table_perm, _name), field_blobs) in tables.blob.iter_mut().zip(blob_entries.into_iter()) {
                 match table_perm {
                     TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                         for (key, item) in field_blobs {
                             table.insert(key, item)
                                 .map_err(|e| NetabaseError::RedbError(e.into()))?;
                         }
                     }
                     _ => return Err(NetabaseError::Other),
                 }
            }
        }

        Ok(())
    }

    fn delete_entry<'txn>(
        key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'db>,
        tables: &mut ModelOpenTables<'txn, 'db, D, Self>
    ) -> NetabaseResult<()>
    {
        // 2. Remove from Main Table first and get the old model
        let model_option = match &mut tables.main {
            TablePermission::ReadWrite(ReadWriteTableType::Table(table)) => {
                table.remove(key.borrow())
                    .map_err(|e| NetabaseError::RedbError(e.into()))?
                    .map(|g| g.value())
            }
            _ => return Err(NetabaseError::Other),
        };

        if let Some(model) = model_option {
            // 3. Remove from Secondary Tables
            let secondary_keys: Vec<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'_>> = model.get_secondary_keys();
            for ((table_perm, _name), secondary_key) in tables.secondary.iter_mut().zip::<std::vec::IntoIter<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'_>>>(<Vec<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'_>> as IntoIterator>::into_iter(secondary_keys)) {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db> = secondary_key.into();
                        table.remove(k.borrow(), key.borrow())
                            .map_err(|e| NetabaseError::RedbError(e.into()))?;
                    }
                    _ => return Err(NetabaseError::Other),
                }
            }

            // 4. Remove from Relational Tables
            // Store as: PrimaryKey -> RelationalKey (swapped from previous implementation)
            let relational_keys = model.get_relational_keys();
            for ((table_perm, _name), relational_key) in tables.relational.iter_mut().zip(relational_keys.into_iter()) {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db> = relational_key.into();
                        // Swapped: key (primary) is the table key, relational key is the value
                        table.remove(key.borrow(), k.borrow())
                            .map_err(|e| NetabaseError::RedbError(e.into()))?;
                    }
                    _ => return Err(NetabaseError::Other),
                }
            }

            // 5. Remove from Subscription Tables
            let subscription_keys = model.get_subscription_keys();
            for ((table_perm, _name), subscription_key) in tables.subscription.iter_mut().zip(subscription_keys.into_iter()) {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        // Convert model-specific subscription key to definition-level subscription key
                        let model_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'db> = subscription_key.into();
                        let def_k: D::SubscriptionKeys = model_k.try_into().map_err(|_| NetabaseError::Other)?;
                        table.remove(def_k.borrow(), key.borrow())
                            .map_err(|e| NetabaseError::RedbError(e.into()))?;
                    }
                    _ => return Err(NetabaseError::Other),
                }
            }

            // 6. Remove from Blob Tables
            let blob_entries = model.get_blob_entries();
            for ((table_perm, _name), field_blobs) in tables.blob.iter_mut().zip(blob_entries.into_iter()) {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        for (key, item) in field_blobs {
                            let key: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'db> = key.into();
                            let item: <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'db> as NetabaseModelBlobKey<'db, D, M, <M as NetabaseModel<D>>::Keys>>::BlobItem = item.into();
                            table.remove(key, item)
                                .map_err(|e| NetabaseError::RedbError(e.into()))?;
                        }
                    }
                    _ => return Err(NetabaseError::Other),
                }
            }
        }

        Ok(())
    }

    fn list_entries<'a, 'txn>(
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
        config: CrudOptions,
    ) -> NetabaseResult<Vec<AccessGuard<'a, <Self as RedbNetbaseModel<'db, D>>::TableV>>> {
        Self::list_range(tables, .., config)
    }

    fn list_range<'a, 'txn, R>(
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
        range: R,
        config: CrudOptions,
    ) -> NetabaseResult<Vec<AccessGuard<'a, <Self as RedbNetbaseModel<'db, D>>::TableV>>>
    where R: std::ops::RangeBounds<<Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'db>> + Clone
    {
        let limit = config.list.limit;
        let offset = config.list.offset;
        println!("RedbModelCrud::list_range: limit={:?}, offset={:?}", limit, offset);
        match &tables.main {
            TablePermission::ReadOnly(TableType::Table(table)) => {
                let iter = table.range(range).map_err(|e| NetabaseError::RedbError(e.into()))?;
                let iter = iter.skip(offset.unwrap_or(0));
                
                let mut result = Vec::new();
                if let Some(limit) = limit {
                    for item in iter.take(limit) {
                        let (_k, v) = item.map_err(|e| NetabaseError::RedbError(e.into()))?;
                        result.push(v);
                    }
                } else {
                     for item in iter {
                        let (_k, v) = item.map_err(|e| NetabaseError::RedbError(e.into()))?;
                        result.push(v);
                    }
                }
                println!("RedbModelCrud::list_range: found {} items", result.len());
                Ok(result)
            },
            TablePermission::ReadWrite(ReadWriteTableType::Table(table)) => {
                let iter = table.range(range).map_err(|e| NetabaseError::RedbError(e.into()))?;
                let iter = iter.skip(offset.unwrap_or(0));
                
                let mut result = Vec::new();
                if let Some(limit) = limit {
                    for item in iter.take(limit) {
                        let (_k, v) = item.map_err(|e| NetabaseError::RedbError(e.into()))?;
                        result.push(v);
                    }
                } else {
                     for item in iter {
                        let (_k, v) = item.map_err(|e| NetabaseError::RedbError(e.into()))?;
                        result.push(v);
                    }
                }
                println!("RedbModelCrud::list_range: found {} items", result.len());
                Ok(result)
            },
            _ => Err(NetabaseError::Other),
        }
    }

    fn count_entries<'txn>(
        tables: &ModelOpenTables<'txn, 'db, D, Self>
    ) -> NetabaseResult<u64> {
         match &tables.main {
            TablePermission::ReadOnly(TableType::Table(table)) => {
                let count = table.len().map_err(|e| NetabaseError::RedbError(e.into()))?;
                println!("RedbModelCrud::count_entries: {}", count);
                Ok(count)
            },
            TablePermission::ReadWrite(ReadWriteTableType::Table(table)) => {
                let count = table.len().map_err(|e| NetabaseError::RedbError(e.into()))?;
                println!("RedbModelCrud::count_entries: {}", count);
                Ok(count)
            },
            _ => Err(NetabaseError::Other),
        }
    }

    fn query_by_subscription<'a, 'txn, S>(
        subscription_key: &S,
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Vec<<Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'db>>>
    where
        S: Into<D::SubscriptionKeys> + Clone,
        D::SubscriptionKeys: redb::Key + 'static,
        <Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'db>: Clone,
        for<'v> <Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'db>: redb::Value<SelfType<'v> = <Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'db>>,
    {
        use redb::ReadableMultimapTable;
        
        let def_key: D::SubscriptionKeys = subscription_key.clone().into();
        
        // Find the subscription table that matches this key
        // Each subscription topic has its own table
        for (table_perm, _table_name) in &tables.subscription {
            match table_perm {
                TablePermission::ReadOnly(TableType::MultimapTable(table)) => {
                    // Try to get values for this key from this table
                    match table.get(def_key.borrow()) {
                        Ok(values) => {
                            let mut result = Vec::new();
                            for item in values {
                                let guard = item.map_err(|e| NetabaseError::RedbError(e.into()))?;
                                result.push(guard.value());
                            }
                            if !result.is_empty() {
                                return Ok(result);
                            }
                        }
                        Err(_) => continue, // Key not found in this table, try next
                    }
                }
                TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                    match table.get(def_key.borrow()) {
                        Ok(values) => {
                            let mut result = Vec::new();
                            for item in values {
                                let guard = item.map_err(|e| NetabaseError::RedbError(e.into()))?;
                                result.push(guard.value());
                            }
                            if !result.is_empty() {
                                return Ok(result);
                            }
                        }
                        Err(_) => continue,
                    }
                }
                TablePermission::ReadOnlyWrite(ReadWriteTableType::MultimapTable(table)) => {
                    match table.get(def_key.borrow()) {
                        Ok(values) => {
                            let mut result = Vec::new();
                            for item in values {
                                let guard = item.map_err(|e| NetabaseError::RedbError(e.into()))?;
                                result.push(guard.value());
                            }
                            if !result.is_empty() {
                                return Ok(result);
                            }
                        }
                        Err(_) => continue,
                    }
                }
                _ => continue,
            }
        }
        
        // No subscribers found for this topic
        Ok(Vec::new())
    }
}
