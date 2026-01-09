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
    <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
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

    /// Create entry with pre-calculated hash (for immutable models)
    fn create_entry_with_hash<'txn>(
        &'db self,
        hash: &crate::subscription_hash::ModelHash,
        tables: &mut ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<()> {
        // Delegate to create_entry_with_subscriptions_and_hash with None (default behavior)
        self.create_entry_with_subscriptions_and_hash(tables, None, Some(hash))
    }

    /// Create entry with selective subscription topics
    /// 
    /// If `subscription_topics` is None, subscribes to all model topics (default behavior).
    /// If Some(vec), subscribes only to the specified topics.
    fn create_entry_with_subscriptions<'txn>(
        &'db self,
        tables: &mut ModelOpenTables<'txn, 'db, D, Self>,
        subscription_topics: Option<Vec<D::SubscriptionKeys>>,
    ) -> NetabaseResult<()> {
        self.create_entry_with_subscriptions_and_hash(tables, subscription_topics, None)
    }

    /// Internal helper for creation with optional hash
    ///
    /// This method is exposed for advanced usage where you might want to manually
    /// specify subscription topics AND provide a pre-calculated hash.
    fn create_entry_with_subscriptions_and_hash<'txn>(
        &'db self,
        tables: &mut ModelOpenTables<'txn, 'db, D, Self>,
        subscription_topics: Option<Vec<D::SubscriptionKeys>>,
        pre_calculated_hash: Option<&crate::subscription_hash::ModelHash>,
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

    /// Update entry with pre-calculated hash (for immutable models)
    fn update_entry_with_hash<'txn>(
        &'db self,
        hash: &crate::subscription_hash::ModelHash,
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
    /// Returns a list of model hashes for all models subscribed to the given topic.
    /// This enables efficient sync and change detection without loading full models.
    fn query_by_subscription<'a, 'txn, S>(
        subscription_key: &S,
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Vec<crate::subscription_hash::ModelHash>>
    where
        S: Into<D::SubscriptionKeys> + Clone,
        D::SubscriptionKeys: redb::Key + 'static;

    /// Query primary keys by secondary key.
    /// 
    /// Returns a list of primary keys for all models with the given secondary key value.
    /// Use the secondary key enum variant (e.g., `UserSecondaryKeys::Email("alice@example.com".into())`).
    /// 
    /// # Example
    /// 
    /// ```rust,ignore
    /// // Find all users with a specific email
    /// let txn = store.begin_read()?;
    /// let tables = txn.prepare_model::<User>()?;
    /// let primary_keys = User::query_by_secondary_key(
    ///     &UserSecondaryKeys::Email("alice@example.com".into()),
    ///     &tables
    /// )?;
    /// 
    /// // Load the full models
    /// for key in primary_keys {
    ///     let user = User::read_default(&key, &tables)?;
    /// }
    /// ```
    fn query_by_secondary_key<'a, 'txn>(
        secondary_key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Secondary,
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Vec<<Self::Keys as NetabaseModelKeys<D, Self>>::Primary>>
    where
        <Self::Keys as NetabaseModelKeys<D, Self>>::Primary: Clone,
        for<'v> <Self::Keys as NetabaseModelKeys<D, Self>>::Primary: redb::Value<SelfType<'v> = <Self::Keys as NetabaseModelKeys<D, Self>>::Primary>;

    /// Query primary keys by relational key.
    /// 
    /// Returns a list of primary keys for all models that have a relational link
    /// with the given key value.
    /// 
    /// # Example
    /// 
    /// ```rust,ignore
    /// // Find all posts by a specific author
    /// let txn = store.begin_read()?;
    /// let tables = txn.prepare_model::<Post>()?;
    /// let post_ids = Post::query_by_relational_key(
    ///     &PostRelationalKeys::Author(UserID("alice".into())),
    ///     &tables
    /// )?;
    /// ```
    fn query_by_relational_key<'a, 'txn>(
        relational_key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Relational,
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Vec<<Self::Keys as NetabaseModelKeys<D, Self>>::Primary>>
    where
        <Self::Keys as NetabaseModelKeys<D, Self>>::Primary: Clone,
        for<'v> <Self::Keys as NetabaseModelKeys<D, Self>>::Primary: redb::Value<SelfType<'v> = <Self::Keys as NetabaseModelKeys<D, Self>>::Primary>,
        for<'v> <<Self::Keys as NetabaseModelKeys<D, Self>>::Relational as redb::Value>::SelfType<'v>: PartialEq<<Self::Keys as NetabaseModelKeys<D, Self>>::Relational>;

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
    ///
    /// # Example
    /// 
    /// See [BLOB_QUERY_METHODS.md](../../../BLOB_QUERY_METHODS.md) for complete examples.
    /// 
    /// ```rust
    /// # // Blob query methods are low-level internal APIs
    /// # // See tests/blob_query_methods.rs for high-level usage
    /// # use netabase_store_examples::boilerplate_lib::definition::LargeUserFile;
    /// // Example: Create a large file that would be stored as blob
    /// let large_file = LargeUserFile {
    ///     data: vec![42u8; 100_000],  // 100KB will be chunked
    ///     metadata: "Large data".into(),
    /// };
    /// assert_eq!(large_file.data.len(), 100_000);
    /// ```
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
    ///
    /// # Example
    ///
    /// See [BLOB_QUERY_METHODS.md](../../../BLOB_QUERY_METHODS.md) for complete examples.
    ///
    /// ```rust
    /// # // Blob query methods are low-level internal APIs
    /// # // See tests/blob_query_methods.rs for high-level usage
    /// # use netabase_store_examples::boilerplate_lib::definition::LargeUserFile;
    /// // Example: Blob fields in models are automatically managed
    /// let files: Vec<LargeUserFile> = vec![
    ///     LargeUserFile { data: vec![1u8; 50_000], metadata: "File 1".into() },
    ///     LargeUserFile { data: vec![2u8; 50_000], metadata: "File 2".into() },
    /// ];
    /// assert_eq!(files.len(), 2);
    /// ```
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
    ///
    /// # Example
    ///
    /// See [BLOB_QUERY_METHODS.md](../../../BLOB_QUERY_METHODS.md) for complete examples.
    ///
    /// ```rust,ignore
    /// let total_blobs = User::count_blob_entries(&tables)?;
    /// println!("Total blob storage entries: {}", total_blobs);
    /// 
    /// // Check if rebalancing is needed
    /// if total_blobs > THRESHOLD {
    ///     trigger_rebalancing();
    /// }
    /// ```
    fn count_blob_entries<'txn>(
        tables: &ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<u64>;

    /// Get blob table metadata (table name and entry count) for each blob field.
    /// 
    /// Returns a vector of (table_name, entry_count) tuples.
    /// Useful for monitoring and debugging blob storage distribution.
    ///
    /// # Example
    ///
    /// See [BLOB_QUERY_METHODS.md](../../../BLOB_QUERY_METHODS.md) for complete examples.
    ///
    /// ```rust
    /// # // Blob query methods are low-level internal APIs
    /// # // See tests/blob_query_methods.rs for high-level usage
    /// # use netabase_store_examples::boilerplate_lib::definition::{LargeUserFile, AnotherLargeUserFile};
    /// // Example: Models can have multiple blob fields
    /// let bio = LargeUserFile { data: vec![1u8; 70_000], metadata: "Bio".into() };
    /// let another = AnotherLargeUserFile(vec![2u8; 80_000]);
    /// // Each blob field gets its own table for storage
    /// assert!(bio.data.len() > 0 && another.0.len() > 0);
    /// ```
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
{
    fn create_entry<'txn>(
        &'db self,
        tables: &mut ModelOpenTables<'txn, 'db, D, Self>
    ) -> NetabaseResult<()> 
    {
        // Delegate to create_entry_with_subscriptions with None (default behavior)
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
        match &mut tables.main {
            TablePermission::ReadWrite(ReadWriteTableType::Table(table)) => {
                table.insert(self.get_primary_key_ref().borrow(), self)
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
                     table.insert(k.borrow(), self.get_primary_key_ref().borrow())
                         .map_err(|e| NetabaseError::RedbError(e.into()))?;
                 }
                 _ => return Err(NetabaseError::Other),
             }
        }

        // 3. Insert into Relational Tables
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
        let subscription_keys_to_insert: Vec<D::SubscriptionKeys> = match subscription_topics {
            None => {
                let all_keys = self.get_subscription_keys();
                all_keys.into_iter()
                    .map(|key| key.try_into().map_err(|_| NetabaseError::Other))
                    .collect::<NetabaseResult<Vec<_>>>()?
            }
            Some(topics) => topics,
        };

        // Calculate hash if needed
        let hash_storage; // Lift lifetime
        let hash_ref = if let Some(h) = pre_calculated_hash {
            h
        } else {
            // Compute hash
            hash_storage = crate::subscription_hash::ModelHash::from_data(self).map_err(|_| NetabaseError::Other)?;
            &hash_storage
        };

        for ((table_perm, _name), key) in tables.subscription.iter_mut().zip(subscription_keys_to_insert.into_iter()) {
             match table_perm {
                 TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                     // Delegate to model implementation of insertion
                     self.insert_subscription_entry(key.clone(), table, Some(hash_ref))?;
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
        let hash = crate::subscription_hash::ModelHash::from_data(self).map_err(|_| NetabaseError::Other)?;
        self.update_entry_with_hash(&hash, tables)
    }

    fn update_entry_with_hash<'txn>(
        &'db self,
        hash: &crate::subscription_hash::ModelHash,
        tables: &mut ModelOpenTables<'txn, 'db, D, Self>
    ) -> NetabaseResult<()> {
        // 1. Update Main Table and get old model in one operation
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
            // Model existed, update secondary/relational/subscription tables

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
                            old_model.delete_subscription_entry(old_def_k, table, Some(&old_hash))?;
                            self.insert_subscription_entry(new_def_k, table, Some(new_hash))?;
                        } else {
                            self.update_subscription_entry(new_def_k, table, &old_model, Some(new_hash), Some(&old_hash))?;
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
            // Model didn't exist, insert into everything
            
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
            let hash = crate::subscription_hash::ModelHash::from_data(&model).map_err(|_| NetabaseError::Other)?;

            for ((table_perm, _name), subscription_key) in tables.subscription.iter_mut().zip(subscription_keys.into_iter()) {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        // Convert model-specific subscription key to definition-level subscription key
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

    fn query_by_relational_key<'a, 'txn>(
        relational_key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Relational,
        tables: &'a ModelOpenTables<'txn, 'db, D, Self>,
    ) -> NetabaseResult<Vec<<Self::Keys as NetabaseModelKeys<D, Self>>::Primary>>
    where
        <Self::Keys as NetabaseModelKeys<D, Self>>::Primary: Clone,
        <Self::Keys as NetabaseModelKeys<D, Self>>::Relational: PartialEq,
        for<'v> <Self::Keys as NetabaseModelKeys<D, Self>>::Primary: redb::Value<SelfType<'v> = <Self::Keys as NetabaseModelKeys<D, Self>>::Primary>,
        for<'v> <<Self::Keys as NetabaseModelKeys<D, Self>>::Relational as redb::Value>::SelfType<'v>: PartialEq<<Self::Keys as NetabaseModelKeys<D, Self>>::Relational>,
    {
        use redb::ReadableMultimapTable;
        
        // Relational tables are multimap: Primary Key -> Relational Keys
        // We need to scan to find all primary keys that have this relational key value
        let mut results = Vec::new();
        
        for (table_perm, _table_name) in &tables.relational {
            match table_perm {
                TablePermission::ReadOnly(TableType::MultimapTable(table)) => {
                    // Iterate through all primary keys
                    let iter = table.iter().map_err(|e| NetabaseError::RedbError(e.into()))?;
                    
                    for pk_result in iter {
                        let (pk, _) = pk_result.map_err(|e| NetabaseError::RedbError(e.into()))?;
                        let pk_value = pk.value();
                        
                        // Get all relational keys for this primary key
                        match table.get(pk_value.borrow()) {
                            Ok(rel_keys) => {
                                for rel_key_result in rel_keys {
                                    let rel_key = rel_key_result.map_err(|e| NetabaseError::RedbError(e.into()))?;
                                    
                                    // Check if this relational key matches what we're looking for
                                    if rel_key.value() == *relational_key {
                                        results.push(pk_value.clone());
                                        break; // Found a match for this primary key, move to next
                                    }
                                }
                            }
                            Err(_) => continue,
                        }
                    }
                }
                TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                    let iter = table.iter().map_err(|e| NetabaseError::RedbError(e.into()))?;
                    
                    for pk_result in iter {
                        let (pk, _) = pk_result.map_err(|e| NetabaseError::RedbError(e.into()))?;
                        let pk_value = pk.value();
                        
                        match table.get(pk_value.borrow()) {
                            Ok(rel_keys) => {
                                for rel_key_result in rel_keys {
                                    let rel_key = rel_key_result.map_err(|e| NetabaseError::RedbError(e.into()))?;
                                    
                                    if rel_key.value() == *relational_key {
                                        results.push(pk_value.clone());
                                        break;
                                    }
                                }
                            }
                            Err(_) => continue,
                        }
                    }
                }
                TablePermission::ReadOnlyWrite(ReadWriteTableType::MultimapTable(table)) => {
                    let iter = table.iter().map_err(|e| NetabaseError::RedbError(e.into()))?;
                    
                    for pk_result in iter {
                        let (pk, _) = pk_result.map_err(|e| NetabaseError::RedbError(e.into()))?;
                        let pk_value = pk.value();
                        
                        match table.get(pk_value.borrow()) {
                            Ok(rel_keys) => {
                                for rel_key_result in rel_keys {
                                    let rel_key = rel_key_result.map_err(|e| NetabaseError::RedbError(e.into()))?;
                                    
                                    if rel_key.value() == *relational_key {
                                        results.push(pk_value.clone());
                                        break;
                                    }
                                }
                            }
                            Err(_) => continue,
                        }
                    }
                }
                _ => continue,
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
