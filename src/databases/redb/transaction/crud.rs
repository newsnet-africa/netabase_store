use redb::{self, AccessGuard, ReadableTable, ReadableTableMetadata};
use std::borrow::Borrow;
use strum::IntoDiscriminant;

use super::options::CrudOptions;
use super::tables::{ModelOpenTables, ReadWriteTableType, TablePermission, TableType};
use crate::{
    errors::{NetabaseError, NetabaseResult},
    traits::registery::{
        definition::redb_definition::RedbDefinition,
        models::{
            keys::{NetabaseModelKeys, blob::NetabaseModelBlobKey},
            model::{NetabaseModel, redb_model::RedbNetbaseModel},
        },
    },
};

/// Trait to handle automatic insertion/update of models into their respective tables.
///
/// This trait abstracts the complexity of mapping a high-level `NetabaseModel` to the
/// underlying `redb` tables. It handles:
/// - Serialization/Deserialization
/// - Secondary Index maintenance
/// - Relational Link storage
/// - Subscription registration
/// - Blob chunking and storage
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
    <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription: 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob: redb::Key + 'static,
    for<'a> <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob: std::borrow::Borrow<<<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob as redb::Value>::SelfType<'a>>,
    <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob as NetabaseModelBlobKey<D, Self>>::BlobItem: redb::Key + 'static,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob as NetabaseModelBlobKey<D, Self>>::BlobItem: std::borrow::Borrow<<<<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob as NetabaseModelBlobKey<D, Self>>::BlobItem as redb::Value>::SelfType<'a>>,
    for<'a> <Self as RedbNetbaseModel<'db, D>>::TableV: redb::Value<SelfType<'a> = Self>,
    Self: 'db
{
    /// Creates a new entry for the model in the database.
    ///
    /// This method:
    /// 1. Serializes the model.
    /// 2. Stores it in the primary table.
    /// 3. Updates all secondary indexes.
    /// 4. Stores relational links.
    /// 5. Registers subscriptions (defaulting to all declared topics).
    /// 6. Chunks and stores any blob fields.
    ///
    /// # Arguments
    /// * `tables` - The opened model tables transaction context.
    fn create_entry<'txn>(
        &'db self,
        tables: &mut ModelOpenTables<'txn, 'db, D, Self>
    ) -> NetabaseResult<()>;

    /// Creates an entry with a pre-calculated content hash.
    ///
    /// This is an optimization for immutable/content-addressed models where the hash
    /// is already known, avoiding re-calculation.
    fn create_entry_with_hash<'txn>(
        &'db self,
        hash: &crate::subscription_hash::ModelHash,
        tables: &mut ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<()> {
        // Delegate to create_entry_with_subscriptions_and_hash with None (default behavior)
        self.create_entry_with_subscriptions_and_hash(tables, None, Some(hash))
    }

    /// Creates an entry with explicit subscription topics.
    /// 
    /// This allows controlling which topics the model subscribes to, overriding
    /// the default behavior (which is to subscribe to all topics defined on the model).
    /// 
    /// # Arguments
    /// * `subscription_topics` - If `Some`, only these topics are registered. If `None`, all defaults are used.
    fn create_entry_with_subscriptions<'txn>(
        &'db self,
        tables: &mut ModelOpenTables<'txn, 'db, D, Self>,
        subscription_topics: Option<Vec<D::SubscriptionKeys>>,
    ) -> NetabaseResult<()> {
        self.create_entry_with_subscriptions_and_hash(tables, subscription_topics, None)
    }

    /// Internal core method for creating an entry.
    ///
    /// Handles the low-level logic of inserting into all related tables (Main, Secondary,
    /// Relational, Subscription, Blob).
    fn create_entry_with_subscriptions_and_hash<'txn>(
        &'db self,
        tables: &mut ModelOpenTables<'txn, 'db, D, Self>,
        subscription_topics: Option<Vec<D::SubscriptionKeys>>,
        pre_calculated_hash: Option<&crate::subscription_hash::ModelHash>,
    ) -> NetabaseResult<()>;

    /// Reads a model entry by its primary key.
    ///
    /// Returns an `AccessGuard` to the data, allowing zero-copy access.
    fn read_entry<'txn>(
        key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary,
        tables: &'txn ModelOpenTables<'txn, 'db, D, Self>,
        config: CrudOptions,
    ) -> NetabaseResult<Option<AccessGuard<'txn, <Self as RedbNetbaseModel<'db, D>>::TableV>>>
    where
    'db: 'txn;

    /// Reads a model entry returning an owned value (deserialized).
    ///
    /// Uses default `CrudOptions`.
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

    /// Updates an existing entry in the database.
    ///
    /// This method handles index maintenance:
    /// 1. Reads the *old* value.
    /// 2. Compares secondary/relational/blob keys between old and new.
    /// 3. Removes stale index entries and inserts new ones.
    /// 4. Updates the main table.
    fn update_entry<'txn>(
        &'db self,
        tables: &mut ModelOpenTables<'txn, 'db, D, Self>
    ) -> NetabaseResult<()>;

    /// Updates an entry using a pre-calculated hash.
    ///
    /// See `update_entry` for logic. This variant avoids re-hashing the model if known.
    fn update_entry_with_hash<'txn>(
        &'db self,
        hash: &crate::subscription_hash::ModelHash,
        tables: &mut ModelOpenTables<'txn, 'db, D, Self>
    ) -> NetabaseResult<()>;

    /// Deletes an entry and cleans up all associated indexes.
    ///
    /// This operation is atomic and ensures no "dangling pointers" in secondary tables.
    /// It must first read the object to know which indexes to clean up.
    fn delete_entry<'txn>(
        key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary,
        tables: &mut ModelOpenTables<'txn, 'db, D, Self>
    ) -> NetabaseResult<()>;

    /// Lists entries for the model.
    ///
    /// Delegates to `list_range` with an unbounded range.
    fn list_entries<'a, 'txn>(
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
        config: CrudOptions,
    ) -> NetabaseResult<Vec<AccessGuard<'a, <Self as RedbNetbaseModel<'db, D>>::TableV>>>;

    /// Lists entries returning owned values.
    fn list_default<'a, 'txn>(
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Vec<Self>> {
        Self::list_entries(tables, CrudOptions::default())
            .map(|vec| vec.into_iter().map(|g| g.value()).collect())
    }

    /// Lists entries within a specific primary key range.
    ///
    /// Supports pagination via `CrudOptions` (limit/offset).
    fn list_range<'a, 'txn, R>(
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
        range: R,
        config: CrudOptions,
    ) -> NetabaseResult<Vec<AccessGuard<'a, <Self as RedbNetbaseModel<'db, D>>::TableV>>>
    where R: std::ops::RangeBounds<<Self::Keys as NetabaseModelKeys<D, Self>>::Primary> + Clone;

    /// Counts the total number of entries in the main table.
    fn count_entries<'txn>(
        tables: &ModelOpenTables<'txn, 'db, D, Self>
    ) -> NetabaseResult<u64>;

    /// Queries the subscription tables to find models in a specific topic.
    /// 
    /// Returns a list of `ModelHash`es. This is very efficient for sync protocols
    /// as it only scans the index, not the data.
    fn query_by_subscription<'a, 'txn, S>(
        subscription_key: &S,
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Vec<crate::subscription_hash::ModelHash>>
    where
        S: Into<D::SubscriptionKeys> + Clone,
        D::SubscriptionKeys: redb::Key + 'static;

    /// Queries a secondary index.
    /// 
    /// Returns the primary keys of models that match the given secondary key.
    fn query_by_secondary_key<'a, 'txn>(
        secondary_key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Secondary,
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Vec<<Self::Keys as NetabaseModelKeys<D, Self>>::Primary>>
    where
        <Self::Keys as NetabaseModelKeys<D, Self>>::Primary: Clone,
        for<'v> <Self::Keys as NetabaseModelKeys<D, Self>>::Primary: redb::Value<SelfType<'v> = <Self::Keys as NetabaseModelKeys<D, Self>>::Primary>;

    /// Queries all outgoing relational links from a model.
    ///
    /// Returns the relational keys stored for the given primary key.
    fn query_relations<'a, 'txn>(
        primary_key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary,
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Vec<AccessGuard<'a, <Self::Keys as NetabaseModelKeys<D, Self>>::Relational>>>
    where
        'db: 'txn;

    /// Queries outgoing relational links of a specific type/field.
    fn query_relations_by_type<'a, 'txn>(
        primary_key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary,
        relation_type: <<Self::Keys as NetabaseModelKeys<D, Self>>::Relational as IntoDiscriminant>::Discriminant,
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Vec<AccessGuard<'a, <Self::Keys as NetabaseModelKeys<D, Self>>::Relational>>>
    where
        'db: 'txn,
        <<Self::Keys as NetabaseModelKeys<D, Self>>::Relational as IntoDiscriminant>::Discriminant: PartialEq;

    // =========================================================================
    // Blob Query Methods (Read-Only)
    // =========================================================================

    /// Reads all blob items (chunks) for a specific blob key.
    ///
    /// Reconstructs the `Vec` of chunks that make up the blob field.
    fn read_blob_items<'a, 'txn>(
        blob_key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Blob,
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Vec<<<Self::Keys as NetabaseModelKeys<D, Self>>::Blob as NetabaseModelBlobKey<D, Self>>::BlobItem>>
    where
        'db: 'txn,
        <<Self::Keys as NetabaseModelKeys<D, Self>>::Blob as NetabaseModelBlobKey<D, Self>>::BlobItem: Clone;

    /// Reads specific blob chunks by their indices.
    ///
    /// This enables selective fetching, allowing a peer to request only the chunks
    /// they are missing (e.g., indices `[0, 5, 9]`).
    fn read_blob_chunks<'a, 'txn>(
        blob_key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Blob,
        indices: &[u8],
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Vec<<<Self::Keys as NetabaseModelKeys<D, Self>>::Blob as NetabaseModelBlobKey<D, Self>>::BlobItem>>
    where
        'db: 'txn,
        <<Self::Keys as NetabaseModelKeys<D, Self>>::Blob as NetabaseModelBlobKey<D, Self>>::BlobItem: Clone;

    /// Fetches the list of all available chunk indices for a blob key.
    ///
    /// This is used to determine the structure of a stored blob without downloading
    /// the actual data. Useful for reconciliation.
    fn fetch_blob_indices<'a, 'txn>(
        blob_key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Blob,
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Vec<u8>>
    where
        'db: 'txn;

    /// Lists all blob keys in a specific blob table.
    ///
    /// Useful for storage auditing or garbage collection.
    fn list_blob_keys<'a, 'txn>(
        table_index: usize,
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Vec<<Self::Keys as NetabaseModelKeys<D, Self>>::Blob>>
    where
        'db: 'txn,
        <Self::Keys as NetabaseModelKeys<D, Self>>::Blob: Clone;

    /// Counts the total number of blob chunks across all blob tables.
    fn count_blob_entries<'txn>(
        tables: &ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<u64>;

    /// Returns statistics (count) for each blob table.
    ///
    /// Returns a vector of `(Table Name, Entry Count)`.
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
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
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
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: crate::blob::NetabaseBlobItem,
{
    fn create_entry<'txn>(
        &'db self,
        tables: &mut ModelOpenTables<'txn, 'db, D, Self>
    ) -> NetabaseResult<()> 
    {
        self.create_entry_with_subscriptions(tables, None)
    }

    fn create_entry_with_subscriptions_and_hash<'txn>(
        &'db self,
        tables: &mut ModelOpenTables<'txn, 'db, D, Self>,
        subscription_topics: Option<Vec<D::SubscriptionKeys>>,
        pre_calculated_hash: Option<&crate::subscription_hash::ModelHash>,
    ) -> NetabaseResult<()> 
    {
        // 1. Insert into Main Table
        // This is the source of truth. We use the model's primary key.
        match &mut tables.main {
            TablePermission::ReadWrite(ReadWriteTableType::Table(table)) => {
                table.insert(self.get_primary_key_ref().borrow(), self)
                    .map_err(|e| NetabaseError::RedbError(e.into()))?;
            }
            _ => return Err(NetabaseError::Other),
        }

        // 2. Insert into Secondary Tables
        // Iterate through all secondary keys derived from the model and insert mapping:
        // Secondary Key -> Primary Key
        let secondary_keys: Vec<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary> = self.get_secondary_keys();
        for ((table_perm, _name), key) in tables.secondary.iter_mut().zip::<std::vec::IntoIter<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary>>(<Vec<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary> as IntoIterator>::into_iter(secondary_keys)) {
             match table_perm {
                 TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                     let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary = key;
                     table.insert(k.borrow(), self.get_primary_key_ref().borrow())
                         .map_err(|e| NetabaseError::RedbError(e.into()))?;
                 }
                 _ => return Err(NetabaseError::Other),
             }
        }

        // 3. Insert into Relational Tables
        // Store mappings: Primary Key -> Relational Key (outgoing links)
        let relational_keys = self.get_relational_keys();
        let primary_key = self.get_primary_key();
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

        // 4. Insert into Subscription Tables
        // If topics are provided, usage them. Otherwise, calculate defaults.
        // Stores: Topic Key -> Model Hash
        let subscription_keys_to_insert: Vec<D::SubscriptionKeys> = match subscription_topics {
            None => {
                let all_keys = self.get_subscription_keys();
                all_keys.into_iter()
                    .map(|key| key.try_into().map_err(|_| NetabaseError::Other))
                    .collect::<NetabaseResult<Vec<_>>>()?
            }
            Some(topics) => topics,
        };

        // Calculate hash if needed. This lifts the reference outside the conditional logic.
        let hash_storage; 
        let hash_ref = if let Some(h) = pre_calculated_hash {
            h
        } else {
            // Compute hash if not provided (standard creation path)
            hash_storage = crate::subscription_hash::ModelHash::from_data(self).map_err(|_| NetabaseError::Other)?;
            &hash_storage
        };

        for ((table_perm, _name), key) in tables.subscription.iter_mut().zip(subscription_keys_to_insert.into_iter()) {
             match table_perm {
                 TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                     // Delegate to model implementation of insertion (handles table selection)
                     self.insert_subscription_entry(key.clone(), table, Some(hash_ref))?;
                 }
                 _ => return Err(NetabaseError::Other),
             }
        }

        // 5. Insert into Blob Tables
        // Blob entries are vectors of chunks. We insert each chunk individually.
        // Blob Key -> Blob Chunk Item
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
        // Simple key-value lookup in the main table.
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
        let hash = crate::subscription_hash::ModelHash::from_data(self).map_err(|_| NetabaseError::Other)?;
        self.update_entry_with_hash(&hash, tables)
    }

    fn update_entry_with_hash<'txn>(
        &'db self,
        hash: &crate::subscription_hash::ModelHash,
        tables: &mut ModelOpenTables<'txn, 'db, D, Self>
    ) -> NetabaseResult<()> {
        // 1. Update Main Table and get old model in one operation
        // This is efficient: we replace the old record and get it back to diff indexes.
        let old_model = match &mut tables.main {
            TablePermission::ReadWrite(ReadWriteTableType::Table(table)) => {
                table.insert(self.get_primary_key_ref().borrow(), self)
                    .map_err(|e| NetabaseError::RedbError(e.into()))?
                    .map(|access_guard| access_guard.value())
            }
            _ => return Err(NetabaseError::Other),
        };

        let primary_key = self.get_primary_key();
        let new_hash = hash;

        if let Some(old_model) = old_model {
            // Model existed: We must perform "Diff & Patch" on indexes.

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

                        // Only update if key changed
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
                        let old_k = old_key;
                        let new_k = new_key;

                        if old_k != new_k {
                            // Remove old relation
                            table.remove(primary_key.borrow(), old_k.borrow())
                                .map_err(|e| NetabaseError::RedbError(e.into()))?;
                            
                            // Add new relation
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

            let old_hash = crate::subscription_hash::ModelHash::from_data(&old_model).map_err(|_| NetabaseError::Other)?;

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
                            // Topic changed: Unsubscribe from old, subscribe to new
                            old_model.delete_subscription_entry(old_def_k, table, Some(&old_hash))?;
                            self.insert_subscription_entry(new_def_k, table, Some(new_hash))?;
                        } else {
                            // Topic same: Update the hash in the topic
                            self.update_subscription_entry(new_def_k, table, &old_model, Some(new_hash), Some(&old_hash))?;
                        }
                    }
                    _ => return Err(NetabaseError::Other),
                }
            }

            // 5. Update Blob Tables
            // Blobs are large, so we remove all old chunks and insert all new ones.
            // Future optimization: Diff individual chunks to only update changed ones.
            let old_blob_entries = old_model.get_blob_entries();
            let new_blob_entries = self.get_blob_entries();

            for (((table_perm, _name), old_blobs), new_blobs) in tables.blob.iter_mut()
                .zip(old_blob_entries.into_iter())
                .zip(new_blob_entries.into_iter())
            {
                 match table_perm {
                     TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        // Remove all old blobs
                        for (old_key, old_item) in old_blobs {
                            let old_key: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob = old_key;
                            let old_item: <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem = old_item;
                            
                            table.remove(old_key, old_item)
                                .map_err(|e| NetabaseError::RedbError(e.into()))?;
                        }

                        // Insert all new blobs
                        for (new_key, new_item) in new_blobs {
                            table.insert(new_key, new_item)
                                .map_err(|e| NetabaseError::RedbError(e.into()))?;
                        }
                     }
                     _ => return Err(NetabaseError::Other),
                 }
            }
        } else {
            // Model didn't exist, this is actually an Insert.
            // Fall back to simple insertion logic for secondary tables.
            
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
                        
                        self.insert_subscription_entry(def_key, table, Some(new_hash))?;
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
            // Model existed, clean up all its indexes.

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
            let relational_keys = model.get_relational_keys();
            for ((table_perm, _name), relational_key) in tables.relational.iter_mut().zip(relational_keys.into_iter()) {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational = relational_key;
                        // Key is Primary, Value is Relation
                        table.remove(key.borrow(), k.borrow())
                            .map_err(|e| NetabaseError::RedbError(e.into()))?;
                    }
                    _ => return Err(NetabaseError::Other),
                }
            }

            // 5. Remove from Subscription Tables
            let subscription_keys = model.get_subscription_keys();
            let hash = crate::subscription_hash::ModelHash::from_data(&model).map_err(|_| NetabaseError::Other)?;

            for ((table_perm, _name), subscription_key) in tables.subscription.iter_mut().zip(subscription_keys.into_iter()) {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        let model_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription = subscription_key;
                        let def_k: D::SubscriptionKeys = model_k.try_into().map_err(|_| NetabaseError::Other)?;
                        
                        // Remove (Topic, Hash)
                        model.delete_subscription_entry(def_k, table, Some(&hash))?;
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
        
        // Helper to collect items from iterator with limit/offset
        let collect_iter = |iter: redb::Range<'a, <Self::Keys as NetabaseModelKeys<D, Self>>::Primary, <Self as RedbNetbaseModel<'db, D>>::TableV>| -> NetabaseResult<Vec<AccessGuard<'a, <Self as RedbNetbaseModel<'db, D>>::TableV>>> {
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
            Ok(result)
        };

        match &tables.main {
            TablePermission::ReadOnly(TableType::Table(table)) => {
                let iter = table.range(range).map_err(|e| NetabaseError::RedbError(e.into()))?;
                collect_iter(iter)
            },
            TablePermission::ReadWrite(ReadWriteTableType::Table(table)) => {
                let iter = table.range(range).map_err(|e| NetabaseError::RedbError(e.into()))?;
                collect_iter(iter)
            },
            _ => Err(NetabaseError::Other),
        }
    }

    fn count_entries<'txn>(
        tables: &ModelOpenTables<'txn, 'db, D, Self>
    ) -> NetabaseResult<u64> {
         match &tables.main {
            TablePermission::ReadOnly(TableType::Table(table)) => {
                table.len().map_err(|e| NetabaseError::RedbError(e.into()))
            },
            TablePermission::ReadWrite(ReadWriteTableType::Table(table)) => {
                table.len().map_err(|e| NetabaseError::RedbError(e.into()))
            },
            _ => Err(NetabaseError::Other),
        }
    }

    fn query_by_subscription<'a, 'txn, S>(
        subscription_key: &S,
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Vec<crate::subscription_hash::ModelHash>>
    where
        S: Into<D::SubscriptionKeys> + Clone,
        D::SubscriptionKeys: redb::Key + 'static,
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

    fn query_by_secondary_key<'a, 'txn>(
        secondary_key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Secondary,
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Vec<<Self::Keys as NetabaseModelKeys<D, Self>>::Primary>>
    where
        <Self::Keys as NetabaseModelKeys<D, Self>>::Primary: Clone,
        for<'v> <Self::Keys as NetabaseModelKeys<D, Self>>::Primary: redb::Value<SelfType<'v> = <Self::Keys as NetabaseModelKeys<D, Self>>::Primary>,
    {
        use redb::ReadableMultimapTable;
        
        // Find the secondary table that matches this key
        // Each secondary key field has its own table
        for (table_perm, _table_name) in &tables.secondary {
            match table_perm {
                TablePermission::ReadOnly(TableType::MultimapTable(table)) => {
                    // Try to get values for this key from this table
                    match table.get(secondary_key.borrow()) {
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
                    match table.get(secondary_key.borrow()) {
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
                    match table.get(secondary_key.borrow()) {
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
        
        // No results found for this secondary key
        Ok(Vec::new())
    }

    fn query_relations<'a, 'txn>(
        primary_key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary,
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Vec<AccessGuard<'a, <Self::Keys as NetabaseModelKeys<D, Self>>::Relational>>>
    where
        'db: 'txn
    {
        use redb::ReadableMultimapTable;
        
        let mut results = Vec::new();
        
        for (table_perm, _table_name) in &tables.relational {
            match table_perm {
                TablePermission::ReadOnly(TableType::MultimapTable(table)) => {
                    if let Ok(iter) = table.get(primary_key.borrow()) {
                        for item in iter {
                            if let Ok(guard) = item {
                                results.push(guard);
                            }
                        }
                    }
                }
                TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                    if let Ok(iter) = table.get(primary_key.borrow()) {
                        for item in iter {
                            if let Ok(guard) = item {
                                results.push(guard);
                            }
                        }
                    }
                }
                TablePermission::ReadOnlyWrite(ReadWriteTableType::MultimapTable(table)) => {
                    if let Ok(iter) = table.get(primary_key.borrow()) {
                        for item in iter {
                            if let Ok(guard) = item {
                                results.push(guard);
                            }
                        }
                    }
                }
                _ => continue,
            }
        }
        
        Ok(results)
    }

    fn query_relations_by_type<'a, 'txn>(
        primary_key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary,
        relation_type: <<Self::Keys as NetabaseModelKeys<D, Self>>::Relational as IntoDiscriminant>::Discriminant,
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Vec<AccessGuard<'a, <Self::Keys as NetabaseModelKeys<D, Self>>::Relational>>>
    where
        'db: 'txn,
        <<Self::Keys as NetabaseModelKeys<D, Self>>::Relational as IntoDiscriminant>::Discriminant: PartialEq
    {
        use redb::ReadableMultimapTable;
        
        let mut results = Vec::new();
        
        // Find the table index for this relation type
        let tree_names = Self::TREE_NAMES;
        let table_index = tree_names.relational.iter().position(|t| t.discriminant == relation_type);

        if let Some(index) = table_index {
            if let Some((table_perm, _)) = tables.relational.get(index) {
                match table_perm {
                    TablePermission::ReadOnly(TableType::MultimapTable(table)) => {
                        if let Ok(iter) = table.get(primary_key.borrow()) {
                            for item in iter {
                                if let Ok(guard) = item {
                                    results.push(guard);
                                }
                            }
                        }
                    }
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        if let Ok(iter) = table.get(primary_key.borrow()) {
                            for item in iter {
                                if let Ok(guard) = item {
                                    results.push(guard);
                                }
                            }
                        }
                    }
                    TablePermission::ReadOnlyWrite(ReadWriteTableType::MultimapTable(table)) => {
                        if let Ok(iter) = table.get(primary_key.borrow()) {
                            for item in iter {
                                if let Ok(guard) = item {
                                    results.push(guard);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        
        Ok(results)
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

    fn read_blob_chunks<'a, 'txn>(
        blob_key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Blob,
        indices: &[u8],
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Vec<<<Self::Keys as NetabaseModelKeys<D, Self>>::Blob as NetabaseModelBlobKey<D, Self>>::BlobItem>>
    where
        'db: 'txn,
        <<Self::Keys as NetabaseModelKeys<D, Self>>::Blob as NetabaseModelBlobKey<D, Self>>::BlobItem: Clone,
    {
        use redb::ReadableMultimapTable;
        use crate::blob::NetabaseBlobItem;
        
        let mut result = Vec::new();
        
        for (table_perm, _table_name) in &tables.blob {
            match table_perm {
                TablePermission::ReadOnly(TableType::MultimapTable(table)) => {
                    if let Ok(values) = table.get(blob_key.borrow()) {
                        for item in values {
                            if let Ok(guard) = item {
                                let value = guard.value();
                                if let Some(idx) = value.get_blob_index() {
                                    if indices.contains(&idx) {
                                        result.push(value);
                                    }
                                }
                            }
                        }
                    }
                }
                TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                    if let Ok(values) = table.get(blob_key.borrow()) {
                        for item in values {
                            if let Ok(guard) = item {
                                let value = guard.value();
                                if let Some(idx) = value.get_blob_index() {
                                    if indices.contains(&idx) {
                                        result.push(value);
                                    }
                                }
                            }
                        }
                    }
                }
                TablePermission::ReadOnlyWrite(ReadWriteTableType::MultimapTable(table)) => {
                    if let Ok(values) = table.get(blob_key.borrow()) {
                        for item in values {
                            if let Ok(guard) = item {
                                let value = guard.value();
                                if let Some(idx) = value.get_blob_index() {
                                    if indices.contains(&idx) {
                                        result.push(value);
                                    }
                                }
                            }
                        }
                    }
                }
                _ => continue,
            }
        }
        
        Ok(result)
    }

    fn fetch_blob_indices<'a, 'txn>(
        blob_key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Blob,
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Vec<u8>>
    where
        'db: 'txn,
    {
        use redb::ReadableMultimapTable;
        use crate::blob::NetabaseBlobItem;
        
        let mut result = Vec::new();
        
        for (table_perm, _table_name) in &tables.blob {
            match table_perm {
                TablePermission::ReadOnly(TableType::MultimapTable(table)) => {
                    if let Ok(values) = table.get(blob_key.borrow()) {
                        for item in values {
                            if let Ok(guard) = item {
                                if let Some(idx) = guard.value().get_blob_index() {
                                    result.push(idx);
                                }
                            }
                        }
                    }
                }
                TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                    if let Ok(values) = table.get(blob_key.borrow()) {
                        for item in values {
                            if let Ok(guard) = item {
                                if let Some(idx) = guard.value().get_blob_index() {
                                    result.push(idx);
                                }
                            }
                        }
                    }
                }
                TablePermission::ReadOnlyWrite(ReadWriteTableType::MultimapTable(table)) => {
                    if let Ok(values) = table.get(blob_key.borrow()) {
                        for item in values {
                            if let Ok(guard) = item {
                                if let Some(idx) = guard.value().get_blob_index() {
                                    result.push(idx);
                                }
                            }
                        }
                    }
                }
                _ => continue,
            }
        }
        
        result.sort();
        result.dedup();
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
