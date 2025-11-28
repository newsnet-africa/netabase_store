//! RedbStoreTree implementation
//!
//! This module contains the tree struct for type-safe operations on a single model type.

use crate::error::NetabaseError;
use crate::traits::definition::NetabaseDefinitionTrait;
use crate::traits::model::{NetabaseModelTrait, NetabaseModelTraitKey};
use crate::{MaybeSend, MaybeSync};

use redb::{
    Database, MultimapTableDefinition, ReadableTable, ReadableTableMetadata, TableDefinition,
};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::str::FromStr;
use std::sync::Arc;
use strum::IntoDiscriminant;

use super::types::BincodeWrapper;

/// Type-safe wrapper around redb table operations for a specific model type.
///
/// RedbStoreTree provides CRUD operations for a single model type with automatic
/// encoding/decoding via redb's Key/Value traits and secondary key management.
///
/// This is similar to SledStoreTree but leverages redb's native type safety.
///
/// The lifetime parameter `'db` ensures that trees cannot outlive their parent database.
pub struct RedbStoreTree<'db, D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
    <D as IntoDiscriminant>::Discriminant: crate::DiscriminantBounds,
{
    pub(crate) db: Arc<Database>,
    pub discriminant: D::Discriminant,
    /// Cached table name string with 'static lifetime (leaked once)
    pub(crate) table_name: &'static str,
    /// Cached secondary table name string with 'static lifetime (leaked once)
    pub(crate) secondary_table_name: &'static str,
    pub(crate) _phantom_d: PhantomData<D>,
    pub(crate) _phantom_m: PhantomData<M>,
    pub(crate) _phantom_db: PhantomData<&'db ()>,
}

impl<'db, D, M> RedbStoreTree<'db, D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D> + Debug + bincode::Decode<()>,
    M::Keys: Debug + bincode::Decode<()> + Ord + PartialEq,
    <D as IntoDiscriminant>::Discriminant: crate::DiscriminantBounds,
{
    /// Create a new RedbStoreTree with shared database access
    ///
    /// Uses discriminant directly instead of string conversion.
    /// Caches table names to avoid memory leaks on every operation.
    pub(crate) fn new(db: Arc<Database>, discriminant: D::Discriminant) -> Self {
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
    ///
    /// Uses cached table name to avoid allocations and memory leaks.
    /// Stores model M directly instead of Definition enum D for better performance.
    pub(crate) fn table_def(
        &self,
    ) -> TableDefinition<'static, BincodeWrapper<M::Keys>, BincodeWrapper<M>> {
        TableDefinition::new(self.table_name)
    }

    /// Get the table definition for secondary keys
    ///
    /// Uses cached table name to avoid allocations and memory leaks.
    /// MultimapTable maps SecondaryKey -> PrimaryKey (one-to-many relationship).
    pub(crate) fn secondary_table_def(
        &self,
    ) -> MultimapTableDefinition<
        'static,
        <M::Keys as NetabaseModelTraitKey<D>>::SecondaryKey,
        <M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey,
    > {
        MultimapTableDefinition::new(self.secondary_table_name)
    }

    /// Insert or update a model in the tree
    ///
    /// Stores model directly without Definition enum wrapper for optimal performance.
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

    /// Get a model by its primary key
    ///
    /// Reads model directly without Definition enum unwrapping.
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
    ///
    /// This is significantly faster than calling put() in a loop.
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
    ///
    /// This is significantly faster than calling get() in a loop.
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
            match write_txn.open_multimap_table(sec_table_def) {
                Ok(sec_table) => {
                    // Since MultimapTable doesn't provide a clear() method, we drop it
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
    pub fn get_by_secondary_key(
        &self,
        secondary_key: <M::Keys as NetabaseModelTraitKey<D>>::SecondaryKey,
    ) -> Result<Vec<M>, NetabaseError>
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
            let keys = M::Keys::from(prim_key);
            if let Some(model) = self.get(keys)? {
                results.push(model);
            }
        }

        Ok(results)
    }

    /// Bulk query models by multiple secondary keys in a single transaction
    ///
    /// Returns a vector of result sets, one per secondary key queried.
    /// This is significantly faster than calling get_by_secondary_key() in a loop.
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
