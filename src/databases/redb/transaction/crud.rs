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
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Primary: redb::Key + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Secondary: redb::Key + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Relational: redb::Key + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription: redb::Key + 'static,
    D::SubscriptionKeys: redb::Key + 'static,
    <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Secondary as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Relational as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    // Add missing static bound
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription: 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob: redb::Key + 'static,
    for<'a> <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob: std::borrow::Borrow<<<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob as redb::Value>::SelfType<'a>>,
    <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob as NetabaseModelBlobKey<D, Self>>::BlobItem: redb::Key + 'static,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob as NetabaseModelBlobKey<D, Self>>::BlobItem: std::borrow::Borrow<<<<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob as NetabaseModelBlobKey<D, Self>>::BlobItem as redb::Value>::SelfType<'a>>,
    for<'a> <Self as RedbNetbaseModel<'db, D>>::TableV: redb::Value<SelfType<'a> = Self>,
    Self: 'db
{
    fn create_entry<'txn>(
        &'db self,
        tables: &mut ModelOpenTables<'txn, 'db, D, Self>
    ) -> NetabaseResult<()>;

    fn read_entry<'txn>(
        key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary,
        tables: &'txn ModelOpenTables<'txn, 'db, D, Self>,
        config: CrudOptions,
    ) -> NetabaseResult<Option<AccessGuard<'txn, <Self as RedbNetbaseModel<'db, D>>::TableV>>>
    where
    'db: 'txn;

    fn read_default<'txn>(
        key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary,
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
        key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary,
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
    where R: std::ops::RangeBounds<<Self::Keys as NetabaseModelKeys<D, Self>>::Primary> + Clone;

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
    ) -> NetabaseResult<Vec<<Self::Keys as NetabaseModelKeys<D, Self>>::Primary>>
    where
        S: Into<D::SubscriptionKeys> + Clone,
        D::SubscriptionKeys: redb::Key + 'static,
        <Self::Keys as NetabaseModelKeys<D, Self>>::Primary: Clone,
        for<'v> <Self::Keys as NetabaseModelKeys<D, Self>>::Primary: redb::Value<SelfType<'v> = <Self::Keys as NetabaseModelKeys<D, Self>>::Primary>;

    // =========================================================================
    // Blob Query Methods (Read-Only)
    // =========================================================================
    // These methods enable parallel fetching and sharded storage patterns
    // for decentralized networks.

    /// Read all blob items for a specific blob key.
    /// 
    /// This is useful for fetching blob data independently of the main model,
    /// enabling parallel fetching in decentralized networks.
    /// 
    /// # Arguments
    /// * `blob_key` - The blob key to query
    /// * `tables` - The opened model tables (read-only access is sufficient)
    /// 
    /// # Returns
    /// A vector of blob items associated with the given key
    fn read_blob_items<'a, 'txn>(
        blob_key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Blob,
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Vec<<<Self::Keys as NetabaseModelKeys<D, Self>>::Blob as NetabaseModelBlobKey<D, Self>>::BlobItem>>
    where
        'db: 'txn,
        <<Self::Keys as NetabaseModelKeys<D, Self>>::Blob as NetabaseModelBlobKey<D, Self>>::BlobItem: Clone;

    /// List all blob keys in a specific blob table.
    /// 
    /// Useful for discovering what blobs exist, enabling sharded storage
    /// where different nodes may store different blob keys.
    /// 
    /// # Arguments
    /// * `table_index` - Index of the blob table (corresponds to blob field order)
    /// * `tables` - The opened model tables
    /// 
    /// # Returns
    /// A vector of all blob keys in that table
    fn list_blob_keys<'a, 'txn>(
        table_index: usize,
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Vec<<Self::Keys as NetabaseModelKeys<D, Self>>::Blob>>
    where
        'db: 'txn,
        <Self::Keys as NetabaseModelKeys<D, Self>>::Blob: Clone;

    /// Count total blob entries across all blob tables.
    /// 
    /// Useful for storage metrics and load balancing in sharded systems.
    fn count_blob_entries<'txn>(
        tables: &ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<u64>;

    /// Get blob table metadata (table name and entry count) for each blob field.
    /// 
    /// Returns a vector of (table_name, entry_count) tuples.
    /// Useful for monitoring and debugging blob storage distribution.
    fn blob_table_stats<'txn>(
        tables: &ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Vec<(String, u64)>>;
}

impl<'db, D, M> RedbModelCrud<'db, D> for M
where
    D: RedbDefinition + Clone,
    M: RedbNetbaseModel<'db, D> + Clone,
    D::Discriminant: 'static + std::fmt::Debug,
    <D as IntoDiscriminant>::Discriminant: std::fmt::Debug,
    for<'a> M::TableV: redb::Value<SelfType<'a> = M>,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key + 'static,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
    
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key + 'static,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary as redb::Value>::SelfType<'a>>,
    
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key + 'static,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational as redb::Value>::SelfType<'a>>,
    
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: redb::Key + 'static,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as redb::Value>::SelfType<'a>>,

    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: 'static,
    D::SubscriptionKeys: redb::Key + 'static + PartialEq,
    for<'a> D::SubscriptionKeys: std::borrow::Borrow<<D::SubscriptionKeys as redb::Value>::SelfType<'a>>,
    M: 'db,
    for<'a> &'a <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as redb::Value>::SelfType<'a>>,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Value<SelfType<'a> = <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob>,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: std::borrow::Borrow<<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem as redb::Value>::SelfType<'a>>,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Value<SelfType<'a> = <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem>,
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
        let secondary_keys: Vec<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary> = self.get_secondary_keys();
        for ((table_perm, _name), key) in tables.secondary.iter_mut().zip::<std::vec::IntoIter<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary>>(<Vec<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary> as IntoIterator>::into_iter(secondary_keys)) {
             match table_perm {
                 TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                     let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary = key;
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
                     let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational = key;
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
                     let model_key: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription = key;
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
        key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary,
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
            let old_secondary: Vec<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary> = old_model.get_secondary_keys();
            let new_secondary: Vec<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary> = self.get_secondary_keys();

            for (((table_perm, _name), old_key), new_key) in tables.secondary.iter_mut()
                .zip::<std::vec::IntoIter<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary>>(<Vec<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary> as IntoIterator>::into_iter(old_secondary))
                .zip::<std::vec::IntoIter<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary>>(<Vec<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary> as IntoIterator>::into_iter(new_secondary))
            {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        let old_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary = old_key;
                        let new_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary = new_key;

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
                        let old_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational = old_key;
                        let new_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational = new_key;

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
                        let old_model_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription = old_key;
                        let new_model_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription = new_key;

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
                            let old_key: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob = old_key;
                            let old_item: <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem = old_item;
                            
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
                        let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary = key;
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
                        let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational = key;
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
                        let model_key: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription = key;
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
        key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary,
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
            let secondary_keys: Vec<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary> = model.get_secondary_keys();
            for ((table_perm, _name), secondary_key) in tables.secondary.iter_mut().zip::<std::vec::IntoIter<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary>>(<Vec<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary> as IntoIterator>::into_iter(secondary_keys)) {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary = secondary_key;
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
                        let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational = relational_key;
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
                        let model_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription = subscription_key;
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
                            let key: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob = key;
                            let item: <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem = item;
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
    where R: std::ops::RangeBounds<<Self::Keys as NetabaseModelKeys<D, Self>>::Primary> + Clone
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
    ) -> NetabaseResult<Vec<<Self::Keys as NetabaseModelKeys<D, Self>>::Primary>>
    where
        S: Into<D::SubscriptionKeys> + Clone,
        D::SubscriptionKeys: redb::Key + 'static,
        <Self::Keys as NetabaseModelKeys<D, Self>>::Primary: Clone,
        for<'v> <Self::Keys as NetabaseModelKeys<D, Self>>::Primary: redb::Value<SelfType<'v> = <Self::Keys as NetabaseModelKeys<D, Self>>::Primary>,
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

    // =========================================================================
    // Blob Query Methods Implementation
    // =========================================================================

    fn read_blob_items<'a, 'txn>(
        blob_key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Blob,
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Vec<<<Self::Keys as NetabaseModelKeys<D, Self>>::Blob as NetabaseModelBlobKey<D, Self>>::BlobItem>>
    where
        'db: 'txn,
        <<Self::Keys as NetabaseModelKeys<D, Self>>::Blob as NetabaseModelBlobKey<D, Self>>::BlobItem: Clone,
    {
        use redb::ReadableMultimapTable;
        
        let mut result = Vec::new();
        
        // Search all blob tables for matching key
        for (table_perm, _table_name) in &tables.blob {
            match table_perm {
                TablePermission::ReadOnly(TableType::MultimapTable(table)) => {
                    if let Ok(values) = table.get(blob_key.borrow()) {
                        for item in values {
                            if let Ok(guard) = item {
                                result.push(guard.value());
                            }
                        }
                    }
                }
                TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                    if let Ok(values) = table.get(blob_key.borrow()) {
                        for item in values {
                            if let Ok(guard) = item {
                                result.push(guard.value());
                            }
                        }
                    }
                }
                TablePermission::ReadOnlyWrite(ReadWriteTableType::MultimapTable(table)) => {
                    if let Ok(values) = table.get(blob_key.borrow()) {
                        for item in values {
                            if let Ok(guard) = item {
                                result.push(guard.value());
                            }
                        }
                    }
                }
                _ => continue,
            }
        }
        
        Ok(result)
    }

    fn list_blob_keys<'a, 'txn>(
        table_index: usize,
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Vec<<Self::Keys as NetabaseModelKeys<D, Self>>::Blob>>
    where
        'db: 'txn,
        <Self::Keys as NetabaseModelKeys<D, Self>>::Blob: Clone,
    {
        use redb::ReadableMultimapTable;
        
        if table_index >= tables.blob.len() {
            return Err(NetabaseError::Other);
        }
        
        let (table_perm, _table_name) = &tables.blob[table_index];
        let mut result = Vec::new();
        
        // Note: This may return duplicate keys since it's a multimap.
        // For unique keys, caller should deduplicate.
        match table_perm {
            TablePermission::ReadOnly(TableType::MultimapTable(table)) => {
                let iter = table.iter().map_err(|e| NetabaseError::RedbError(e.into()))?;
                for item in iter {
                    let (key_guard, _value_guard) = item.map_err(|e| NetabaseError::RedbError(e.into()))?;
                    result.push(key_guard.value());
                }
            }
            TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                let iter = table.iter().map_err(|e| NetabaseError::RedbError(e.into()))?;
                for item in iter {
                    let (key_guard, _value_guard) = item.map_err(|e| NetabaseError::RedbError(e.into()))?;
                    result.push(key_guard.value());
                }
            }
            TablePermission::ReadOnlyWrite(ReadWriteTableType::MultimapTable(table)) => {
                let iter = table.iter().map_err(|e| NetabaseError::RedbError(e.into()))?;
                for item in iter {
                    let (key_guard, _value_guard) = item.map_err(|e| NetabaseError::RedbError(e.into()))?;
                    result.push(key_guard.value());
                }
            }
            _ => return Err(NetabaseError::Other),
        }
        
        Ok(result)
    }

    fn count_blob_entries<'txn>(
        tables: &ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<u64> {
        let mut total = 0u64;
        
        for (table_perm, _table_name) in &tables.blob {
            let count = match table_perm {
                TablePermission::ReadOnly(TableType::MultimapTable(table)) => {
                    table.len().map_err(|e| NetabaseError::RedbError(e.into()))?
                }
                TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                    table.len().map_err(|e| NetabaseError::RedbError(e.into()))?
                }
                TablePermission::ReadOnlyWrite(ReadWriteTableType::MultimapTable(table)) => {
                    table.len().map_err(|e| NetabaseError::RedbError(e.into()))?
                }
                _ => continue,
            };
            total += count;
        }
        
        Ok(total)
    }

    fn blob_table_stats<'txn>(
        tables: &ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Vec<(String, u64)>> {
        let mut stats = Vec::with_capacity(tables.blob.len());
        
        for (table_perm, table_name) in &tables.blob {
            let count = match table_perm {
                TablePermission::ReadOnly(TableType::MultimapTable(table)) => {
                    table.len().map_err(|e| NetabaseError::RedbError(e.into()))?
                }
                TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                    table.len().map_err(|e| NetabaseError::RedbError(e.into()))?
                }
                TablePermission::ReadOnlyWrite(ReadWriteTableType::MultimapTable(table)) => {
                    table.len().map_err(|e| NetabaseError::RedbError(e.into()))?
                }
                _ => continue,
            };
            stats.push((table_name.to_string(), count));
        }
        
        Ok(stats)
    }
}
