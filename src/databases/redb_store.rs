use crate::error::NetabaseError;
use crate::traits::convert::ToIVec;
use crate::traits::definition::NetabaseDefinitionTrait;
use crate::traits::model::NetabaseModelTrait;
use redb::{
    Database, Key, ReadableDatabase, ReadableTable, ReadableTableMetadata, TableDefinition,
    TypeName, Value,
};
use std::cmp::Ordering;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use strum::{IntoDiscriminant, IntoEnumIterator};

/// Wrapper type for bincode serialization with redb
/// This allows any bincode-compatible type to be used as a redb Key or Value
#[derive(Debug)]
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

/// Type-safe wrapper around redb::Database that works with NetabaseDefinitionTrait types.
///
/// The RedbStore provides a type-safe interface to the underlying redb database,
/// using discriminants as table names and ensuring all operations are type-checked.
pub struct RedbStore<D>
where
    D: NetabaseDefinitionTrait,
    <D as IntoDiscriminant>::Discriminant: Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <D as strum::IntoDiscriminant>::Discriminant: std::convert::AsRef<str>,
{
    pub(crate) db: Arc<Database>,
    pub trees: Vec<D::Discriminant>,
}

impl<D> RedbStore<D>
where
    D: NetabaseDefinitionTrait,
    <D as IntoDiscriminant>::Discriminant: Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <D as strum::IntoDiscriminant>::Discriminant: std::convert::AsRef<str>,
{
    /// Get direct access to the underlying redb database
    pub fn db(&self) -> &Database {
        &self.db
    }
}

impl<D> RedbStore<D>
where
    D: NetabaseDefinitionTrait + ToIVec,
    <D as IntoDiscriminant>::Discriminant: Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <D as strum::IntoDiscriminant>::Discriminant: std::convert::AsRef<str>,
{
    /// Create a new RedbStore at the given path
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, NetabaseError> {
        let db = Database::create(path)?;
        Ok(Self {
            db: Arc::new(db),
            trees: D::Discriminant::iter().collect(),
        })
    }

    /// Open an existing RedbStore at the given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, NetabaseError> {
        let db = Database::open(path)?;
        Ok(Self {
            db: Arc::new(db),
            trees: D::Discriminant::iter().collect(),
        })
    }

    /// Open a tree for a specific model type
    /// This creates a tree abstraction that can handle dynamic table creation
    pub fn open_tree<M>(&self) -> RedbStoreTree<D, M>
    where
        M: NetabaseModelTrait<D> + TryFrom<D> + Into<D>,
        M::PrimaryKey: Debug + bincode::Decode<()> + Ord,
        M::SecondaryKeys: Debug + bincode::Decode<()> + Ord + PartialEq,
        D: TryFrom<M> + ToIVec + Debug,
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

/// Type-safe wrapper around redb table operations for a specific model type.
///
/// RedbStoreTree provides CRUD operations for a single model type with automatic
/// encoding/decoding and secondary key management. It handles dynamic table creation
/// similar to sled's Tree abstraction.
pub struct RedbStoreTree<D, M>
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
        + Send
        + Sync
        + 'static
        + FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <D as strum::IntoDiscriminant>::Discriminant: std::convert::AsRef<str>,
{
    db: Arc<Database>,
    discriminant: D::Discriminant,
    _phantom_d: PhantomData<D>,
    _phantom_m: PhantomData<M>,
}

impl<D, M> RedbStoreTree<D, M>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + ToIVec + Debug,
    M: NetabaseModelTrait<D> + TryFrom<D> + Into<D>,
    M::PrimaryKey: Debug + bincode::Decode<()> + Ord,
    M::SecondaryKeys: Debug + bincode::Decode<()> + Ord + PartialEq,
    <D as IntoDiscriminant>::Discriminant: Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + Send
        + Sync
        + 'static
        + FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Copy,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Display,
    <D as strum::IntoDiscriminant>::Discriminant: FromStr,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Sync,
    <D as strum::IntoDiscriminant>::Discriminant: std::marker::Send,
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <D as strum::IntoDiscriminant>::Discriminant: std::convert::AsRef<str>,
{
    /// Create a new RedbStoreTree with shared database access
    /// Uses discriminant directly instead of string conversion
    fn new(db: Arc<Database>, discriminant: D::Discriminant) -> Self {
        Self {
            db,
            discriminant,
            _phantom_d: PhantomData,
            _phantom_m: PhantomData,
        }
    }

    /// Get the table definition for this tree using typed keys and values
    fn table_def(
        &self,
    ) -> TableDefinition<'static, BincodeWrapper<M::PrimaryKey>, BincodeWrapper<D>> {
        // Leak the discriminant string to get 'static lifetime - acceptable for table definitions
        let table_name = self.discriminant.to_string();
        let static_name: &'static str = Box::leak(table_name.into_boxed_str());
        TableDefinition::new(static_name)
    }

    /// Get the table definition for secondary keys
    fn secondary_table_def(
        &self,
    ) -> TableDefinition<
        'static,
        BincodeWrapper<(M::SecondaryKeys, M::PrimaryKey)>,
        BincodeWrapper<()>,
    > {
        // Use discriminant-based naming for secondary tables
        // Leak the string to get 'static lifetime - this is acceptable for table definitions
        let sec_name = format!("{}_secondary", self.discriminant.as_ref());
        let static_name: &'static str = Box::leak(sec_name.into_boxed_str());
        TableDefinition::new(static_name)
    }

    /// Insert or update a model in the tree
    pub fn put(&self, model: M) -> Result<(), NetabaseError>
    where
        D: From<M>,
    {
        let primary_key = model.primary_key();
        let definition: D = model.clone().into();

        let table_def = self.table_def();
        let sec_table_def = self.secondary_table_def();

        // Begin write transaction
        let write_txn = self.db.begin_write()?;

        // Handle secondary keys in the same transaction
        let secondary_keys = model.secondary_keys();
        {
            let mut table = write_txn.open_table(table_def)?;
            table.insert(&primary_key, &definition)?;

            if !secondary_keys.is_empty() {
                let mut sec_table = write_txn.open_table(sec_table_def)?;
                for sec_key in secondary_keys {
                    let composite_key = (sec_key, primary_key.clone());
                    sec_table.insert(&composite_key, &())?;
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
    pub fn get(&self, key: M::PrimaryKey) -> Result<Option<M>, NetabaseError> {
        let table_def = self.table_def();

        let read_txn = self.db.begin_read()?;

        // Handle the case where the table doesn't exist yet (hasn't been written to)
        let table = match read_txn.open_table(table_def) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(None),
            Err(e) => return Err(NetabaseError::RedbTableError(e)),
        };

        match table.get(&key)? {
            Some(definition_guard) => {
                let definition: D = definition_guard.value();
                match M::try_from(definition) {
                    Ok(model) => Ok(Some(model)),
                    Err(_) => Ok(None),
                }
            }
            None => Ok(None),
        }
    }

    /// Delete a model by its primary key
    pub fn remove(&self, key: M::PrimaryKey) -> Result<Option<M>, NetabaseError> {
        // First get the model so we can clean up secondary keys
        let model = self.get(key.clone())?;

        if model.is_none() {
            return Ok(None);
        }

        let table_def = self.table_def();
        let sec_table_def = self.secondary_table_def();

        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(table_def)?;
            table.remove(&key)?;

            // Clean up secondary keys in the same transaction
            if let Some(ref m) = model {
                let secondary_keys = m.secondary_keys();
                if !secondary_keys.is_empty() {
                    let mut sec_table = write_txn.open_table(sec_table_def)?;
                    for sec_key in secondary_keys {
                        let composite_key = (sec_key, key.clone());
                        sec_table.remove(&composite_key)?;
                    }
                }
            }
        }

        write_txn.commit()?;

        Ok(model)
    }

    /// Iterate over all models in the tree
    pub fn iter(&self) -> Result<Vec<(M::PrimaryKey, M)>, NetabaseError>
    where
        M::PrimaryKey: bincode::Decode<()>,
    {
        let table_def = self.table_def();

        let read_txn = self.db.begin_read()?;

        // Handle the case where the table doesn't exist yet (hasn't been written to)
        let table = match read_txn.open_table(table_def) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
            Err(e) => return Err(NetabaseError::RedbTableError(e)),
        };

        let mut results = Vec::new();

        for item in table.iter()? {
            let (key_guard, value_guard) = item?;

            let key: M::PrimaryKey = key_guard.value();
            let definition: D = value_guard.value();

            let model = M::try_from(definition).map_err(|_| {
                crate::error::NetabaseError::Conversion(
                    crate::error::EncodingDecodingError::Decoding(
                        bincode::error::DecodeError::Other("Type conversion failed"),
                    ),
                )
            })?;

            results.push((key, model));
        }

        Ok(results)
    }

    /// Get the number of models in the tree
    pub fn len(&self) -> Result<usize, NetabaseError> {
        let table_def = self.table_def();

        let read_txn = self.db.begin_read()?;

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

        let write_txn = self.db.begin_write()?;
        {
            // Clear main table (if it exists)
            match write_txn.open_table(table_def) {
                Ok(mut table) => {
                    let keys: Vec<M::PrimaryKey> = table
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
            match write_txn.open_table(sec_table_def) {
                Ok(mut sec_table) => {
                    let sec_keys: Vec<(M::SecondaryKeys, M::PrimaryKey)> = sec_table
                        .iter()?
                        .filter_map(|item| item.ok())
                        .map(|(k, _)| k.value())
                        .collect();

                    for key in sec_keys {
                        sec_table.remove(&key)?;
                    }
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
    pub fn get_by_secondary_key(
        &self,
        secondary_key: M::SecondaryKeys,
    ) -> Result<Vec<M>, NetabaseError>
    where
        M::PrimaryKey: bincode::Decode<()>,
    {
        let sec_table_def = self.secondary_table_def();

        let read_txn = self.db.begin_read()?;

        // Handle the case where the secondary table doesn't exist yet (hasn't been written to)
        let sec_table = match read_txn.open_table(sec_table_def) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
            Err(e) => return Err(NetabaseError::RedbTableError(e)),
        };

        let mut results = Vec::new();

        // Iterate through the secondary index to find matching entries
        for item in sec_table.iter()? {
            let (composite_key_guard, _) = item?;
            let (sec_key, prim_key): (M::SecondaryKeys, M::PrimaryKey) =
                composite_key_guard.value();

            // Check if the secondary key matches
            if sec_key == secondary_key {
                if let Some(model) = self.get(prim_key)? {
                    results.push(model);
                }
            }
        }

        Ok(results)
    }
}
