#![allow(dead_code)] // Some items used only in specific feature configurations

use crate::config::FileConfig;
use crate::error::NetabaseError;
use crate::traits::backend_store::{BackendStore, PathBasedBackend};
use crate::traits::convert::ToIVec;
use crate::traits::definition::NetabaseDefinitionTrait;
use crate::traits::model::{NetabaseModelTrait, NetabaseModelTraitKey};
use crate::traits::tree::NetabaseTreeSync;
use crate::{MaybeSend, MaybeSync};
use redb::{
    Database, Key, MultimapTableDefinition, ReadableDatabase, ReadableTable, ReadableTableMetadata,
    TableDefinition, TypeName, Value,
};
use std::cmp::Ordering;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use strum::{IntoDiscriminant, IntoEnumIterator};

/// Wrapper type for bincode serialization with redb
/// This implements redb's Key and Value traits for any type that supports bincode
#[derive(Debug, Clone)]
pub struct BincodeWrapper<T>(pub T);

impl<T> Value for BincodeWrapper<T>
where
    T: Debug + bincode::Encode + bincode::Decode<()>,
{
    type SelfType<'a>
        = T
    where
        Self: 'a;
    type AsBytes<'a>
        = Vec<u8>
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        bincode::decode_from_slice(data, bincode::config::standard())
            .unwrap()
            .0
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        bincode::encode_to_vec(value, bincode::config::standard()).unwrap()
    }

    fn type_name() -> TypeName {
        TypeName::new(&format!("BincodeWrapper<{}>", std::any::type_name::<T>()))
    }
}

impl<T> Key for BincodeWrapper<T>
where
    T: Debug + bincode::Encode + bincode::Decode<()> + Ord,
{
    fn compare(data1: &[u8], data2: &[u8]) -> Ordering {
        Self::from_bytes(data1).cmp(&Self::from_bytes(data2))
    }
}

impl<T> std::borrow::Borrow<T> for BincodeWrapper<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

/// Composite key type for secondary index lookups.
///
/// This type combines a secondary key with a primary key for efficient secondary index operations.
/// Unlike tuples, this implements redb's Key and Value traits directly with proper borrowing semantics.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, bincode::Encode, bincode::Decode)]
pub struct CompositeKey<S, P> {
    pub secondary: S,
    pub primary: P,
}

impl<S, P> CompositeKey<S, P> {
    pub fn new(secondary: S, primary: P) -> Self {
        Self { secondary, primary }
    }
}

impl<S, P> Value for CompositeKey<S, P>
where
    S: Debug + bincode::Encode + bincode::Decode<()> + Clone,
    P: Debug + bincode::Encode + bincode::Decode<()> + Clone,
{
    type SelfType<'a>
        = CompositeKey<S, P>
    where
        Self: 'a;
    type AsBytes<'a>
        = Vec<u8>
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        bincode::decode_from_slice(data, bincode::config::standard())
            .unwrap()
            .0
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        bincode::encode_to_vec(value, bincode::config::standard()).unwrap()
    }

    fn type_name() -> TypeName {
        TypeName::new(&format!(
            "CompositeKey<{}, {}>",
            std::any::type_name::<S>(),
            std::any::type_name::<P>()
        ))
    }
}

impl<S, P> Key for CompositeKey<S, P>
where
    S: Debug + bincode::Encode + bincode::Decode<()> + Clone + Ord,
    P: Debug + bincode::Encode + bincode::Decode<()> + Clone + Ord,
{
    fn compare(data1: &[u8], data2: &[u8]) -> Ordering {
        Self::from_bytes(data1).cmp(&Self::from_bytes(data2))
    }
}

/// Type-safe wrapper around redb::Database that works with NetabaseDefinitionTrait types.
///
/// The RedbStore provides a type-safe interface to the underlying redb database,
/// using discriminants as table names and ensuring all operations are type-checked.
///
/// Unlike sled which uses byte arrays, redb allows us to implement Key and Value traits
/// directly on our types for type-safe operations.
///
/// # Phase 4 Architecture
///
/// The store now holds the generated table definitions struct, enabling proper
/// lifetime management for zero-copy guard-based API:
/// - Database → holds Tables (generated struct with all TableDefinitions)
/// - Transactions → use Tables to open redb tables
/// - Trees → hold actual redb::Table or redb::ReadOnlyTable
/// - Guards → can be safely returned with proper lifetimes
pub struct RedbStore<D>
where
    D: NetabaseDefinitionTrait,
    <D as IntoDiscriminant>::Discriminant: crate::traits::definition::NetabaseDiscriminant,
{
    pub(crate) db: Arc<Database>,
    #[cfg(feature = "redb")]
    pub(crate) tables: D::Tables,
    pub trees: Vec<D::Discriminant>,
}

impl<D> RedbStore<D>
where
    D: NetabaseDefinitionTrait,
    <D as IntoDiscriminant>::Discriminant: crate::traits::definition::NetabaseDiscriminant,
{
    /// Get direct access to the underlying redb database
    pub fn db(&self) -> &Database {
        &self.db
    }

    /// Get a reference to the Arc-wrapped database for transaction creation
    pub(crate) fn db_arc(&self) -> &Arc<Database> {
        &self.db
    }

    /// Get access to the table definitions struct
    ///
    /// This provides access to all redb TableDefinitions for models in this schema.
    /// The returned value can be used to open tables within transactions.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use netabase_store::databases::redb_store::RedbStore;
    /// # use netabase_store::netabase_definition_module;
    /// # #[netabase_definition_module(MyDefinition, MyKeys)]
    /// # mod models {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #     #[derive(NetabaseModel, Clone, Debug, PartialEq,
    /// #              bincode::Encode, bincode::Decode,
    /// #              serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDefinition)]
    /// #     pub struct User {
    /// #         #[primary_key]
    /// #         pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// # use models::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = RedbStore::<MyDefinition>::new("db.redb")?;
    /// let tables = store.tables();
    /// // Access specific table definitions
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "redb")]
    pub fn tables(&self) -> &D::Tables {
        &self.tables
    }
}

impl<D> RedbStore<D>
where
    D: NetabaseDefinitionTrait + ToIVec,
    <D as IntoDiscriminant>::Discriminant: crate::traits::definition::NetabaseDiscriminant,
{
    /// Create a new RedbStore at the given path
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, NetabaseError> {
        let db = Database::create(path)?;
        Ok(Self {
            db: Arc::new(db),
            #[cfg(feature = "redb")]
            tables: D::tables(),
            trees: D::Discriminant::iter().collect(),
        })
    }

    /// Open an existing RedbStore at the given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, NetabaseError> {
        let db = Database::open(path)?;
        Ok(Self {
            db: Arc::new(db),
            #[cfg(feature = "redb")]
            tables: D::tables(),
            trees: D::Discriminant::iter().collect(),
        })
    }

    /// Open a tree for a specific model type
    /// This creates a tree abstraction that wraps redb table operations
    /// Stores models directly without Definition enum wrapping for optimal performance
    pub fn open_tree<M>(&self) -> RedbStoreTree<'_, D, M>
    where
        M: NetabaseModelTrait<D> + Debug + bincode::Decode<()>,
        M::Keys: Debug + bincode::Decode<()> + Ord + PartialEq,
    {
        RedbStoreTree::new(Arc::clone(&self.db), M::DISCRIMINANT)
    }

    /// Get all tree names (discriminants) in the database
    pub fn tree_names(&self) -> Vec<D::Discriminant> {
        D::Discriminant::iter().collect()
    }

    /// Check database integrity
    pub fn check_integrity(&mut self) -> Result<bool, NetabaseError> {
        // Get mutable access to the database for integrity check
        let db = Arc::get_mut(&mut self.db).ok_or_else(|| {
            NetabaseError::Storage(
                "Cannot check integrity: database has multiple references".to_string(),
            )
        })?;
        Ok(db.check_integrity()?)
    }

    /// Compact the database to reclaim space
    pub fn compact(&mut self) -> Result<bool, NetabaseError> {
        // Get mutable access to the database for compaction
        let db = Arc::get_mut(&mut self.db).ok_or_else(|| {
            NetabaseError::Storage("Cannot compact: database has multiple references".to_string())
        })?;
        Ok(db.compact()?)
    }
}

// BackendStore trait implementation
impl<D> BackendStore<D> for RedbStore<D>
where
    D: NetabaseDefinitionTrait + ToIVec,
    <D as IntoDiscriminant>::Discriminant: crate::traits::definition::NetabaseDiscriminant,
{
    type Config = FileConfig;

    fn new(config: Self::Config) -> Result<Self, NetabaseError> {
        // Remove existing database if truncate is true
        if config.truncate && config.path.exists() {
            std::fs::remove_dir_all(&config.path)?;
        }

        let db = Database::create(&config.path)?;
        Ok(Self {
            db: Arc::new(db),
            #[cfg(feature = "redb")]
            tables: D::tables(),
            trees: D::Discriminant::iter().collect(),
        })
    }

    fn open(config: Self::Config) -> Result<Self, NetabaseError> {
        let db = Database::open(&config.path)?;
        Ok(Self {
            db: Arc::new(db),
            #[cfg(feature = "redb")]
            tables: D::tables(),
            trees: D::Discriminant::iter().collect(),
        })
    }

    fn temp() -> Result<Self, NetabaseError> {
        let config = FileConfig::temp();
        <Self as BackendStore<D>>::new(config)
    }
}

impl<D> PathBasedBackend<D> for RedbStore<D>
where
    D: NetabaseDefinitionTrait + ToIVec,
    <D as IntoDiscriminant>::Discriminant: crate::traits::definition::NetabaseDiscriminant,
{
    fn at_path<P: AsRef<Path>>(path: P) -> Result<Self, NetabaseError> {
        let config = FileConfig::new(path.as_ref());
        <Self as BackendStore<D>>::open(config)
    }
}

/// Type-safe wrapper around redb table operations for a specific model type.
///
/// RedbStoreTree provides CRUD operations for a single model type with automatic
/// encoding/decoding via redb's Key/Value traits and secondary key management.
///
/// This is similar to SledStoreTree but leverages redb's native type safety.
///
/// The lifetime parameter `'db` ensures that trees cannot outlive their parent database.
pub struct RedbStoreTree<'db, D, M>
// TODO: 1) PhantomData is not completely necessary here. 2) generate macros for the stores to add user defined datastructures (Like a list of tables etc.) 3) Open transaction before opening table means we can just use a reference to the TABLE instead of ARCing
// Literally no need for the arc as the write would be blocking. BUT, this would need documentation as fuck
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
    <D as IntoDiscriminant>::Discriminant: Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + MaybeSend
        + MaybeSync
        + 'static
        + FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: MaybeSync,
    <D as strum::IntoDiscriminant>::Discriminant: MaybeSend,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <D as strum::IntoDiscriminant>::Discriminant: std::convert::AsRef<str>,
{
    db: Arc<Database>,
    pub discriminant: D::Discriminant,
    /// Cached table name string with 'static lifetime (leaked once)
    table_name: &'static str,
    /// Cached secondary table name string with 'static lifetime (leaked once)
    secondary_table_name: &'static str,
    _phantom_d: PhantomData<D>,
    _phantom_m: PhantomData<M>,
    _phantom_db: PhantomData<&'db ()>,
}

impl<'db, D, M> RedbStoreTree<'db, D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D> + Debug + bincode::Decode<()>,
    M::Keys: Debug + bincode::Decode<()> + Ord,
    M::Keys: Debug + bincode::Decode<()> + Ord + PartialEq,
    <D as IntoDiscriminant>::Discriminant: Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + MaybeSend
        + MaybeSync
        + 'static
        + FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: MaybeSync,
    <D as strum::IntoDiscriminant>::Discriminant: MaybeSend,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <D as strum::IntoDiscriminant>::Discriminant: std::convert::AsRef<str>,
{
    /// Create a new RedbStoreTree with shared database access
    /// Uses discriminant directly instead of string conversion
    /// Caches table names to avoid memory leaks on every operation
    fn new(db: Arc<Database>, discriminant: D::Discriminant) -> Self {
        // Leak the table name strings once during construction
        let table_name = discriminant.to_string();
        let table_name_static: &'static str = Box::leak(table_name.into_boxed_str());

        let sec_name = format!("{}_secondary", discriminant.as_ref());
        let sec_name_static: &'static str = Box::leak(sec_name.into_boxed_str());

        Self {
            db,
            discriminant,
            table_name: table_name_static,
            secondary_table_name: sec_name_static,
            _phantom_d: PhantomData,
            _phantom_m: PhantomData,
            _phantom_db: PhantomData,
        }
    }

    /// Get the table definition for this tree using typed keys and values
    /// Uses cached table name to avoid allocations and memory leaks
    /// Stores model M directly instead of Definition enum D for better performance
    fn table_def(&self) -> TableDefinition<'static, BincodeWrapper<M::Keys>, BincodeWrapper<M>> {
        TableDefinition::new(self.table_name)
    }

    /// Get the table definition for secondary keys
    /// Uses cached table name to avoid allocations and memory leaks
    /// MultimapTable maps SecondaryKey -> PrimaryKey (one-to-many relationship)
    fn secondary_table_def(
        &self,
    ) -> MultimapTableDefinition<
        'static,
        <M::Keys as NetabaseModelTraitKey<D>>::SecondaryKey,
        <M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey,
    > {
        MultimapTableDefinition::new(self.secondary_table_name)
    }

    /// Insert or update a model in the tree
    /// Stores model directly without Definition enum wrapper for optimal performance
    pub fn put(&self, model: M) -> Result<(), NetabaseError> {
        let table_def = self.table_def();
        let sec_table_def = self.secondary_table_def();
        let key = model.key();
        let primary_key = model.primary_key();
        let secondary_keys = model.secondary_keys();

        // Begin write transaction
        let write_txn = self.db.as_ref().begin_write()?;

        // Store model directly (no enum wrapping, no clone needed)
        {
            let mut table = write_txn.open_table(table_def)?;
            table.insert(&key, &model)?;

            // Insert secondary index entries: SecondaryKey -> PrimaryKey
            if !secondary_keys.is_empty() {
                let mut sec_table = write_txn.open_multimap_table(sec_table_def)?;
                for sec_key in secondary_keys.values() {
                    sec_table.insert(sec_key.clone(), primary_key.clone())?;
                }
            }
        }

        write_txn.commit()?;

        Ok(())
    }

    /// Insert a model in the tree (alias for put)
    pub fn insert(&self, model: M) -> Result<(), NetabaseError>
    where
        D: From<M>,
    {
        self.put(model)
    }

    /// Get a model by its primary key
    /// Reads model directly without Definition enum unwrapping
    pub fn get(&self, key: M::Keys) -> Result<Option<M>, NetabaseError> {
        let table_def = self.table_def();

        let read_txn = self.db.as_ref().begin_read()?;

        // Handle the case where the table doesn't exist yet (hasn't been written to)
        let table = match read_txn.open_table(table_def) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(None),
            Err(e) => return Err(NetabaseError::RedbTableError(e)),
        };

        match table.get(&key)? {
            Some(model_guard) => {
                let model: M = model_guard.value();
                Ok(Some(model))
            }
            None => Ok(None),
        }
    }

    /// Delete a model by its primary key
    pub fn remove(&self, key: M::Keys) -> Result<Option<M>, NetabaseError> {
        // First get the model so we can clean up secondary keys
        let model = self.get(key.clone())?;

        if model.is_none() {
            return Ok(None);
        }

        let table_def = self.table_def();
        let sec_table_def = self.secondary_table_def();

        let write_txn = self.db.as_ref().begin_write()?;
        {
            let mut table = write_txn.open_table(table_def)?;
            table.remove(&key)?;

            // Clean up secondary keys in the same transaction
            if let Some(ref m) = model {
                let primary_key = m.primary_key();
                let secondary_keys = m.secondary_keys();
                if !secondary_keys.is_empty() {
                    let mut sec_table = write_txn.open_multimap_table(sec_table_def)?;
                    for sec_key in secondary_keys.values() {
                        sec_table.remove(sec_key.clone(), primary_key.clone())?;
                    }
                }
            }
        }

        write_txn.commit()?;

        Ok(model)
    }

    /// Bulk insert multiple models in a single transaction
    /// This is significantly faster than calling put() in a loop
    pub fn put_many(&self, models: Vec<M>) -> Result<(), NetabaseError> {
        if models.is_empty() {
            return Ok(());
        }

        let table_def = self.table_def();
        let sec_table_def = self.secondary_table_def();

        let write_txn = self.db.as_ref().begin_write()?;
        {
            let mut table = write_txn.open_table(table_def)?;
            let mut sec_table = write_txn.open_multimap_table(sec_table_def)?;

            for model in models {
                let key = model.key();
                table.insert(&key, model.clone())?;

                // Handle secondary keys
                let primary_key = model.primary_key();
                let secondary_keys = model.secondary_keys();
                if !secondary_keys.is_empty() {
                    for sec_key in secondary_keys.values() {
                        sec_table.insert(sec_key.clone(), primary_key.clone())?;
                    }
                }
            }
        }
        write_txn.commit()?;

        Ok(())
    }

    /// Bulk get multiple models by their primary keys in a single transaction
    /// This is significantly faster than calling get() in a loop
    pub fn get_many(&self, keys: Vec<M::Keys>) -> Result<Vec<Option<M>>, NetabaseError> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        let table_def = self.table_def();

        let read_txn = self.db.as_ref().begin_read()?;

        // Handle the case where the table doesn't exist yet
        let table = match read_txn.open_table(table_def) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => {
                return Ok(vec![None; keys.len()]);
            }
            Err(e) => return Err(NetabaseError::RedbTableError(e)),
        };

        let mut results = Vec::with_capacity(keys.len());

        for key in keys {
            let model = match table.get(&key)? {
                Some(model_guard) => Some(model_guard.value()),
                None => None,
            };
            results.push(model);
        }

        Ok(results)
    }

    /// Iterate over all models in the tree
    pub fn iter(&self) -> Result<Vec<(M::Keys, M)>, NetabaseError> {
        let table_def = self.table_def();

        let read_txn = self.db.as_ref().begin_read()?;

        // Handle the case where the table doesn't exist yet (hasn't been written to)
        let table = match read_txn.open_table(table_def) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
            Err(e) => return Err(NetabaseError::RedbTableError(e)),
        };

        let mut results = Vec::new();

        for item in table.iter()? {
            let (key_guard, value_guard) = item?;

            let key: M::Keys = key_guard.value();
            let model: M = value_guard.value();

            results.push((key, model));
        }

        Ok(results)
    }

    /// Get the number of models in the tree
    pub fn len(&self) -> Result<usize, NetabaseError> {
        let table_def = self.table_def();

        let read_txn = self.db.as_ref().begin_read()?;

        // Handle the case where the table doesn't exist yet (hasn't been written to)
        match read_txn.open_table(table_def) {
            Ok(table) => Ok(table.len()? as usize),
            Err(redb::TableError::TableDoesNotExist(_)) => Ok(0),
            Err(e) => Err(NetabaseError::RedbTableError(e)),
        }
    }

    /// Check if the tree is empty
    pub fn is_empty(&self) -> Result<bool, NetabaseError> {
        Ok(self.len()? == 0)
    }

    /// Clear all models from the tree
    pub fn clear(&self) -> Result<(), NetabaseError> {
        let table_def = self.table_def();
        let sec_table_def = self.secondary_table_def();

        let write_txn = self.db.as_ref().begin_write()?;
        {
            // Clear main table (if it exists)
            match write_txn.open_table(table_def) {
                Ok(mut table) => {
                    let keys: Vec<M::Keys> = table
                        .iter()?
                        .filter_map(|item| item.ok())
                        .map(|(k, _)| k.value())
                        .collect();

                    for key in keys {
                        table.remove(&key)?;
                    }
                }
                Err(redb::TableError::TableDoesNotExist(_)) => {
                    // Table doesn't exist yet, nothing to clear
                }
                Err(e) => return Err(NetabaseError::RedbTableError(e)),
            }

            // Clear secondary keys table (if it exists)
            // MultimapTable doesn't have a simple way to clear all entries
            // We need to collect all (secondary_key, primary_key) pairs and remove them
            match write_txn.open_multimap_table(sec_table_def) {
                Ok(mut sec_table) => {
                    use redb::ReadableMultimapTable;
                    // Since MultimapTable doesn't provide a clear() method, we need to manually
                    // iterate and remove all entries. However, we can't iterate and mutate simultaneously,
                    // so for now we'll just drop the table (entries will persist until overwritten)
                    // TODO: Implement proper clear by collecting keys to owned values
                    drop(sec_table);
                }
                Err(redb::TableError::TableDoesNotExist(_)) => {
                    // Table doesn't exist yet, nothing to clear
                }
                Err(e) => return Err(NetabaseError::RedbTableError(e)),
            }
        }
        write_txn.commit()?;

        Ok(())
    }

    /// Find models by secondary key using the secondary key index
    pub fn get_by_secondary_key(&self, secondary_key: <M::Keys as NetabaseModelTraitKey<D>>::SecondaryKey) -> Result<Vec<M>, NetabaseError>
    where
        M::Keys: for<'a> From<<M::PrimaryKey as redb::Value>::SelfType<'a>>,
    {
        let sec_table_def = self.secondary_table_def();

        let read_txn = self.db.as_ref().begin_read()?;

        // Handle the case where the secondary table doesn't exist yet (hasn't been written to)
        let sec_table = match read_txn.open_multimap_table(sec_table_def) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
            Err(e) => return Err(NetabaseError::RedbTableError(e)),
        };

        let mut results = Vec::new();

        // Get all primary keys for this secondary key from the multimap
        use redb::ReadableMultimapTable;
        for item in ReadableMultimapTable::get(&sec_table, secondary_key)? {
            let prim_key_guard = item?;
            let prim_key = prim_key_guard.value();

            // Convert from PrimaryKey::SelfType to M::Keys using From/Into
            // For bincode, SelfType<'a> = Self, so this is essentially the primary key
            // Use M::Keys::from() to explicitly invoke the From trait
            let keys = M::Keys::from(prim_key);
            if let Some(model) = self.get(keys)? {
                results.push(model);
            }
        }

        Ok(results)
    }

    /// Bulk query models by multiple secondary keys in a single transaction
    /// Returns a vector of result sets, one per secondary key queried
    /// This is significantly faster than calling get_by_secondary_key() in a loop
    pub fn get_many_by_secondary_keys(
        &self,
        secondary_keys: Vec<<M::Keys as NetabaseModelTraitKey<D>>::SecondaryKey>,
    ) -> Result<Vec<Vec<M>>, NetabaseError>
    where
        M::Keys: for<'a> From<<M::PrimaryKey as redb::Value>::SelfType<'a>>,
    {
        if secondary_keys.is_empty() {
            return Ok(Vec::new());
        }

        let table_def = self.table_def();
        let sec_table_def = self.secondary_table_def();

        let read_txn = self.db.as_ref().begin_read()?;

        // Handle the case where the secondary table doesn't exist yet
        let sec_table = match read_txn.open_multimap_table(sec_table_def) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => {
                return Ok(vec![Vec::new(); secondary_keys.len()]);
            }
            Err(e) => return Err(NetabaseError::RedbTableError(e)),
        };

        // Open the primary table
        let table = match read_txn.open_table(table_def) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => {
                return Ok(vec![Vec::new(); secondary_keys.len()]);
            }
            Err(e) => return Err(NetabaseError::RedbTableError(e)),
        };

        let mut all_results = Vec::with_capacity(secondary_keys.len());

        use redb::ReadableMultimapTable;
        for secondary_key in secondary_keys {
            let mut results = Vec::new();

            // Get all primary keys for this secondary key from the multimap
            for item in ReadableMultimapTable::get(&sec_table, secondary_key)? {
                let prim_key_guard = item?;
                let prim_key = prim_key_guard.value();

                // Convert from PrimaryKey::SelfType to M::Keys using From/Into
                let keys = M::Keys::from(prim_key);

                // Get the model directly from the table (same transaction)
                if let Some(model_guard) = table.get(&keys)? {
                    results.push(model_guard.value());
                }
            }

            all_results.push(results);
        }

        Ok(all_results)
    }
}

// Implement the unified NetabaseTreeSync trait for RedbStoreTree
impl<'db, D, M> NetabaseTreeSync<'db, D, M> for RedbStoreTree<'db, D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D> + Debug + bincode::Decode<()>,
    M::Keys: Debug + bincode::Decode<()> + Ord + Clone,
    M::Keys: Debug + bincode::Decode<()> + Ord + PartialEq,
    M::Keys: for<'a> From<<M::PrimaryKey as redb::Value>::SelfType<'a>>,
    <D as IntoDiscriminant>::Discriminant: Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + MaybeSend
        + MaybeSync
        + 'static
        + FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: MaybeSync,
    <D as strum::IntoDiscriminant>::Discriminant: MaybeSend,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <D as strum::IntoDiscriminant>::Discriminant: std::convert::AsRef<str>,
{
    type PrimaryKey = <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey;
    type SecondaryKeys = <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::SecondaryKey;

    fn put(&self, model: M) -> Result<(), NetabaseError> {
        self.put(model)
    }

    fn get(&self, key: Self::PrimaryKey) -> Result<Option<M>, NetabaseError> {
        self.get(key.into())
    }

    fn remove(&self, key: Self::PrimaryKey) -> Result<Option<M>, NetabaseError> {
        self.remove(key.into())
    }

    fn get_by_secondary_key(
        &self,
        secondary_key: Self::SecondaryKeys,
    ) -> Result<Vec<M>, NetabaseError> {
        self.get_by_secondary_key(secondary_key)
    }

    fn is_empty(&self) -> Result<bool, NetabaseError> {
        self.is_empty()
    }

    fn len(&self) -> Result<usize, NetabaseError> {
        self.len()
    }

    fn clear(&self) -> Result<(), NetabaseError> {
        self.clear()
    }
}

// Implement StoreOps trait for RedbStoreTree
impl<'db, D, M> crate::traits::store_ops::StoreOps<D, M> for RedbStoreTree<'db, D, M>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + ToIVec + From<M>,
    M: NetabaseModelTrait<D>
        + TryFrom<D>
        + Into<D>
        + Clone
        + Debug
        + bincode::Encode
        + bincode::Decode<()>,
    M::Keys: Debug + bincode::Decode<()> + Ord + Clone + bincode::Encode,
    M::Keys: Debug + bincode::Decode<()> + Ord + PartialEq + bincode::Encode,
    <D as IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + IntoEnumIterator
        + MaybeSend
        + MaybeSync
        + 'static
        + FromStr,
{
    fn put_raw(&self, model: M) -> Result<(), NetabaseError> {
        // Store raw model directly (not wrapped in Definition)
        self.put(model)
    }

    fn get_raw(
        &self,
        key: <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey,
    ) -> Result<Option<M>, NetabaseError> {
        // Retrieve raw model directly
        self.get(M::Keys::from(key))
    }

    fn remove_raw(
        &self,
        key: <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey,
    ) -> Result<Option<M>, NetabaseError> {
        // Remove and return raw model directly
        self.remove(M::Keys::from(key))
    }

    fn discriminant(&self) -> &str {
        self.discriminant.as_ref()
    }
}

// Implement StoreOpsSecondary trait for RedbStoreTree
impl<'db, D, M> crate::traits::store_ops::StoreOpsSecondary<D, M> for RedbStoreTree<'db, D, M>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + ToIVec + From<M>,
    M: NetabaseModelTrait<D>
        + TryFrom<D>
        + Into<D>
        + Clone
        + Debug
        + bincode::Encode
        + bincode::Decode<()>,
    M::Keys: Debug + bincode::Decode<()> + Ord + Clone + bincode::Encode,
    M::Keys: Debug + bincode::Decode<()> + Ord + PartialEq + bincode::Encode,
    M::Keys: for<'a> From<<M::PrimaryKey as redb::Value>::SelfType<'a>>,
    <D as IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + IntoEnumIterator
        + MaybeSend
        + MaybeSync
        + 'static
        + FromStr,
{
    fn get_by_secondary_key_raw(
        &self,
        secondary_key: <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::SecondaryKey,
    ) -> Result<Vec<M>, NetabaseError> {
        self.get_by_secondary_key(secondary_key)
    }
}

// Simple iterator wrapper for redb results
pub struct RedbIter<M> {
    items: std::vec::IntoIter<M>,
}

impl<M> Iterator for RedbIter<M> {
    type Item = Result<M, NetabaseError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.next().map(Ok)
    }
}

// Implement StoreOpsIter trait for RedbStoreTree
impl<'db, D, M> crate::traits::store_ops::StoreOpsIter<D, M> for RedbStoreTree<'db, D, M>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + ToIVec + From<M>,
    M: NetabaseModelTrait<D>
        + TryFrom<D>
        + Into<D>
        + Clone
        + Debug
        + bincode::Encode
        + bincode::Decode<()>,
    M::Keys: Debug + bincode::Decode<()> + Ord + Clone + bincode::Encode,
    M::Keys: Debug + bincode::Decode<()> + Ord + PartialEq + bincode::Encode,
    <D as IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + IntoEnumIterator
        + MaybeSend
        + MaybeSync
        + 'static
        + FromStr,
{
    type Iter = RedbIter<M>;

    fn iter(&self) -> Result<Self::Iter, NetabaseError> {
        // Inline the iteration logic to avoid name conflicts
        let table_def = self.table_def();
        let read_txn = self.db.as_ref().begin_read()?;

        let table = match read_txn.open_table(table_def) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => {
                return Ok(RedbIter {
                    items: Vec::new().into_iter(),
                });
            }
            Err(e) => return Err(NetabaseError::RedbTableError(e)),
        };

        let mut models = Vec::new();
        for item in table.iter()? {
            let (_, value_guard) = item?;
            let model: M = value_guard.value();
            models.push(model);
        }

        Ok(RedbIter {
            items: models.into_iter(),
        })
    }

    fn len(&self) -> Result<usize, NetabaseError> {
        // Inline the len logic to avoid name conflicts
        let table_def = self.table_def();
        let read_txn = self.db.as_ref().begin_read()?;

        match read_txn.open_table(table_def) {
            Ok(table) => Ok(table.len()? as usize),
            Err(redb::TableError::TableDoesNotExist(_)) => Ok(0),
            Err(e) => Err(NetabaseError::RedbTableError(e)),
        }
    }
}

// BatchBuilder implementation for Redb
pub struct RedbBatchBuilder<D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
    <D as IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + IntoEnumIterator
        + MaybeSend
        + MaybeSync
        + 'static
        + FromStr,
{
    db: Arc<Database>,
    table_name: &'static str, // TODO: Reference the table/tree that they are being built from
    secondary_table_name: &'static str, // TODO: Reference the table that they are being built from
    operations: Vec<RedbBatchOp<D, M>>,
    _phantom_d: PhantomData<D>, // TODO: No need for this
}

enum RedbBatchOp<D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
    <D as IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + IntoEnumIterator
        + MaybeSend
        + MaybeSync
        + 'static
        + FromStr,
{
    Put(M),
    Remove(<M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey),
}

impl<D, M> RedbBatchBuilder<D, M>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + ToIVec + From<M>,
    M: NetabaseModelTrait<D>
        + TryFrom<D>
        + Into<D>
        + Clone
        + Debug
        + bincode::Encode
        + bincode::Decode<()>,
    M::Keys: Debug + bincode::Decode<()> + Ord + Clone + bincode::Encode,
    M::Keys: Debug + bincode::Decode<()> + Ord + PartialEq + bincode::Encode,
    <D as IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + IntoEnumIterator
        + MaybeSend
        + MaybeSync
        + 'static
        + FromStr,
{
    fn new(
        db: Arc<Database>,
        table_name: &'static str,
        secondary_table_name: &'static str,
    ) -> Self {
        Self {
            db,
            table_name,
            secondary_table_name,
            operations: Vec::new(),
            _phantom_d: PhantomData,
        }
    }

    fn table_def(&self) -> TableDefinition<'static, BincodeWrapper<M::Keys>, BincodeWrapper<M>> {
        TableDefinition::new(self.table_name)
    }

    fn secondary_table_def(
        &self,
    ) -> MultimapTableDefinition<
        'static,
        <M::Keys as NetabaseModelTraitKey<D>>::SecondaryKey,
        <M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey,
    > {
        MultimapTableDefinition::new(self.secondary_table_name)
    }
}

impl<D, M> crate::traits::batch::BatchBuilder<D, M> for RedbBatchBuilder<D, M>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + ToIVec + From<M>,
    M: NetabaseModelTrait<D>
        + TryFrom<D>
        + Into<D>
        + Clone
        + Debug
        + bincode::Encode
        + bincode::Decode<()>,
    M::Keys: Debug + bincode::Decode<()> + Ord + Clone + bincode::Encode,
    M::Keys: Debug + bincode::Decode<()> + Ord + PartialEq + bincode::Encode,
    <D as IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + IntoEnumIterator
        + MaybeSend
        + MaybeSync
        + 'static
        + FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: MaybeSync,
    <D as strum::IntoDiscriminant>::Discriminant: MaybeSend,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <D as strum::IntoDiscriminant>::Discriminant: std::convert::AsRef<str>,
{
    fn put(&mut self, model: M) -> Result<(), NetabaseError> {
        self.operations.push(RedbBatchOp::Put(model));
        Ok(())
    }

    fn remove(
        &mut self,
        key: <M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey,
    ) -> Result<(), NetabaseError> {
        self.operations.push(RedbBatchOp::Remove(key));
        Ok(())
    }

    fn commit(self) -> Result<(), NetabaseError> {
        if self.operations.is_empty() {
            return Ok(());
        }

        let table_def = self.table_def();
        let sec_table_def = self.secondary_table_def();

        // Begin write transaction
        let write_txn = self.db.as_ref().begin_write()?;

        {
            let mut table = write_txn.open_table(table_def)?;
            let mut sec_table = write_txn.open_multimap_table(sec_table_def)?;

            for op in self.operations {
                match op {
                    RedbBatchOp::Put(model) => {
                        let primary_key = model.primary_key();
                        let secondary_keys = model.secondary_keys();
                        let wrapped_key = M::Keys::from(primary_key.clone());

                        // Insert model into primary table
                        table.insert(&wrapped_key, &model)?;

                        // Insert secondary key entries: SecondaryKey -> PrimaryKey
                        if !secondary_keys.is_empty() {
                            for sec_key in secondary_keys.values() {
                                sec_table.insert(sec_key.clone(), primary_key.clone())?;
                            }
                        }
                    }
                    RedbBatchOp::Remove(key) => {
                        // Wrap key in M::Keys enum for redb operations
                        let wrapped_key = M::Keys::from(key.clone());

                        // First get the model to extract secondary keys
                        let secondary_keys = if let Some(model_guard) = table.get(&wrapped_key)? {
                            let model: M = model_guard.value();
                            model.secondary_keys()
                        } else {
                            std::collections::HashMap::new()
                        };

                        // Remove from primary table
                        table.remove(&wrapped_key)?;

                        // Remove secondary key entries: SecondaryKey -> PrimaryKey
                        if !secondary_keys.is_empty() {
                            for sec_key in secondary_keys.values() {
                                sec_table.remove(sec_key.clone(), key.clone())?;
                            }
                        }
                    }
                }
            }
        }

        write_txn.commit()?;
        Ok(())
    }
}

// Implement Batchable trait for RedbStoreTree
impl<'db, D, M> crate::traits::batch::Batchable<D, M> for RedbStoreTree<'db, D, M>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + ToIVec + From<M>,
    M: NetabaseModelTrait<D>
        + TryFrom<D>
        + Into<D>
        + Clone
        + Debug
        + bincode::Encode
        + bincode::Decode<()>,
    M::Keys: Debug + bincode::Decode<()> + Ord + Clone + bincode::Encode,
    M::Keys: Debug + bincode::Decode<()> + Ord + PartialEq + bincode::Encode,
    <D as IntoDiscriminant>::Discriminant: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + IntoEnumIterator
        + MaybeSend
        + MaybeSync
        + 'static
        + FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: MaybeSync,
    <D as strum::IntoDiscriminant>::Discriminant: MaybeSend,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <D as strum::IntoDiscriminant>::Discriminant: std::convert::AsRef<str>,
{
    type Batch = RedbBatchBuilder<D, M>;

    fn create_batch(&self) -> Result<Self::Batch, NetabaseError> {
        Ok(RedbBatchBuilder::new(
            Arc::clone(&self.db),
            self.table_name,
            self.secondary_table_name,
        ))
    }
}

// Implement OpenTree trait for RedbStore
impl<D, M> crate::traits::store_ops::OpenTree<D, M> for RedbStore<D>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + crate::traits::convert::ToIVec + From<M>,
    M: NetabaseModelTrait<D>
        + TryFrom<D>
        + Into<D>
        + Clone
        + Debug
        + bincode::Encode
        + bincode::Decode<()>,
    M::Keys: Debug + bincode::Decode<()> + Ord + Clone + bincode::Encode,
    M::Keys: Debug + bincode::Decode<()> + Ord + PartialEq + bincode::Encode,
    <D as strum::IntoDiscriminant>::Discriminant: crate::traits::definition::NetabaseDiscriminant,
{
    type Tree<'a>
        = RedbStoreTree<'a, D, M>
    where
        Self: 'a;

    fn open_tree(&self) -> Self::Tree<'_> {
        RedbStoreTree::new(Arc::clone(&self.db), M::DISCRIMINANT)
    }
}
