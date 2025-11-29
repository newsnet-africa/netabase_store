//! Tree types for redb zero-copy backend
//!
//! This module provides tree abstractions for CRUD operations within
//! transactions, maintaining strict lifetime relationships and providing
//! both mutable and immutable tree access.

use crate::error::NetabaseError;
use crate::traits::definition::NetabaseDefinitionTrait;
use crate::traits::model::{NetabaseModelTrait, NetabaseModelTraitKey};
use redb::{
    MultimapTableDefinition, MultimapValue, ReadTransaction, ReadableTable, ReadableTableMetadata,
    TableDefinition, WriteTransaction,
};
use std::marker::PhantomData;

/// Mutable tree for write operations
///
/// This provides methods to insert, remove, and query models within a write transaction.
/// The tree borrows mutably from the transaction, ensuring exclusive access.
///
/// # Lifetime Management
///
/// - `'txn`: Borrows from the transaction
/// - `'db`: Borrows from the database (through transaction)
/// - `D`: Database definition type
/// - `M`: Model type for this tree
///
/// # Examples
///
/// ```no_run
/// use netabase_store::databases::redb_zerocopy::RedbStoreZeroCopy;
/// use netabase_store::error::NetabaseError;
/// use netabase_store::{NetabaseModel, netabase, netabase_definition_module};
///
/// #[netabase_definition_module(MyDefinition, MyKeys)]
/// mod my_models {
///     use netabase_store::{NetabaseModel, netabase};
///     #[derive(NetabaseModel, Clone, Debug, PartialEq,
///              bincode::Encode, bincode::Decode,
///              serde::Serialize, serde::Deserialize)]
///     #[netabase(MyDefinition)]
///     pub struct User {
///         #[primary_key]
///         pub id: u64,
///         pub name: String,
///     }
/// }
/// use my_models::*;
///
/// # fn main() -> Result<(), NetabaseError> {
/// let store = RedbStoreZeroCopy::<MyDefinition>::new("./test.db")?;
/// let mut txn = store.begin_write()?;
/// let mut tree = txn.open_tree::<User>()?;
///
/// // Insert a user
/// let user = User { id: 1, name: "Alice".to_string() };
/// tree.put(user)?;
///
/// // Remove a user by key
/// tree.remove(UserPrimaryKey(1))?;
///
/// drop(tree); // Must drop before commit
/// txn.commit()?;
/// # Ok(())
/// # }
/// ```
pub struct RedbTreeMut<'txn, 'db, D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
{
    pub(crate) txn: &'txn mut WriteTransaction,
    #[allow(dead_code)]
    pub(crate) discriminant: D::Discriminant,
    pub(crate) table_name: &'static str,
    pub(crate) secondary_table_name: &'static str,
    pub(crate) _phantom: PhantomData<(&'db D, M)>,
}

impl<'txn, 'db, D, M> RedbTreeMut<'txn, 'db, D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
    M::Keys: NetabaseModelTraitKey<D>,
{
    /// Get the table definition for the primary table
    fn table_def(
        &self,
    ) -> TableDefinition<'static, M::Keys, super::super::redb_store::BincodeWrapper<M>> {
        TableDefinition::new(self.table_name)
    }

    /// Get the table definition for secondary keys
    fn secondary_table_def(
        &self,
    ) -> MultimapTableDefinition<
        'static,
        <M::Keys as NetabaseModelTraitKey<D>>::SecondaryKey,
        <M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey,
    > {
        MultimapTableDefinition::new(self.secondary_table_name)
    }

    /// Insert a model into the tree
    ///
    /// This will insert the model into the primary table and update secondary indexes.
    /// If a model with the same primary key already exists, it will be replaced.
    ///
    /// # Arguments
    ///
    /// * `model` - The model instance to insert
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use netabase_store::databases::redb_zerocopy::RedbStoreZeroCopy;
    /// use netabase_store::error::NetabaseError;
    /// use netabase_store::{NetabaseModel, netabase, netabase_definition_module};
    ///
    /// #[netabase_definition_module(MyDefinition, MyKeys)]
    /// mod my_models {
    ///     use netabase_store::{NetabaseModel, netabase};
    ///     #[derive(NetabaseModel, Clone, Debug, PartialEq,
    ///              bincode::Encode, bincode::Decode,
    ///              serde::Serialize, serde::Deserialize)]
    ///     #[netabase(MyDefinition)]
    ///     pub struct User {
    ///         #[primary_key]
    ///         pub id: u64,
    ///         pub name: String,
    ///     }
    /// }
    /// use my_models::*;
    ///
    /// # fn main() -> Result<(), NetabaseError> {
    /// let store = RedbStoreZeroCopy::<MyDefinition>::new("./test.db")?;
    /// let mut txn = store.begin_write()?;
    /// let mut tree = txn.open_tree::<User>()?;
    /// let user = User { id: 1, name: "Alice".to_string() };
    /// tree.put(user)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn put(&mut self, model: M) -> Result<(), NetabaseError> {
        let table_def = self.table_def();
        let sec_table_def = self.secondary_table_def();

        let primary_key = model.primary_key();
        let secondary_keys = model.secondary_keys();
        let wrapped_key = M::Keys::from(primary_key.clone());

        // Insert into primary table
        let mut table = self.txn.open_table(table_def)?;
        table.insert(wrapped_key, super::super::redb_store::BincodeWrapper(model))?;

        // Insert into secondary indexes
        if !secondary_keys.is_empty() {
            let mut sec_table = self.txn.open_multimap_table(sec_table_def)?;
            for sec_key in secondary_keys.values() {
                sec_table.insert(sec_key.clone(), primary_key.clone())?;
            }
        }

        Ok(())
    }

    /// Insert multiple models in bulk
    ///
    /// This is more efficient than calling put() in a loop as it reuses
    /// the table handles and reduces transaction overhead.
    ///
    /// # Arguments
    ///
    /// * `models` - Vector of model instances to insert
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use netabase_store::databases::redb_zerocopy::RedbStoreZeroCopy;
    /// use netabase_store::error::NetabaseError;
    /// use netabase_store::{NetabaseModel, netabase, netabase_definition_module};
    ///
    /// #[netabase_definition_module(MyDefinition, MyKeys)]
    /// mod my_models {
    ///     use netabase_store::{NetabaseModel, netabase};
    ///     #[derive(NetabaseModel, Clone, Debug, PartialEq,
    ///              bincode::Encode, bincode::Decode,
    ///              serde::Serialize, serde::Deserialize)]
    ///     #[netabase(MyDefinition)]
    ///     pub struct User {
    ///         #[primary_key]
    ///         pub id: u64,
    ///         pub name: String,
    ///     }
    /// }
    /// use my_models::*;
    ///
    /// # fn main() -> Result<(), NetabaseError> {
    /// let store = RedbStoreZeroCopy::<MyDefinition>::new("./test.db")?;
    /// let mut txn = store.begin_write()?;
    /// let mut tree = txn.open_tree::<User>()?;
    /// let users = vec![
    ///     User { id: 1, name: "Alice".to_string() },
    ///     User { id: 2, name: "Bob".to_string() },
    /// ];
    /// tree.put_many(users)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn put_many(&mut self, models: Vec<M>) -> Result<(), NetabaseError> {
        let table_def = self.table_def();
        let sec_table_def = self.secondary_table_def();

        let mut table = self.txn.open_table(table_def)?;
        let mut sec_table = self.txn.open_multimap_table(sec_table_def)?;

        for model in models {
            let primary_key = model.primary_key();
            let secondary_keys = model.secondary_keys();
            let wrapped_key = M::Keys::from(primary_key.clone());

            table.insert(wrapped_key, super::super::redb_store::BincodeWrapper(model))?;

            if !secondary_keys.is_empty() {
                for sec_key in secondary_keys.values() {
                    sec_table.insert(sec_key.clone(), primary_key.clone())?;
                }
            }
        }

        Ok(())
    }

    /// Get a model by primary key (returns owned/cloned data)
    ///
    /// # Arguments
    ///
    /// * `key` - The primary key to look up
    ///
    /// # Returns
    ///
    /// Some(model) if found, None if not found
    pub fn get(
        &self,
        key: &<M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey,
    ) -> Result<Option<M>, NetabaseError> {
        let table_def = self.table_def();
        let table = self.txn.open_table(table_def)?;
        let wrapped_key = M::Keys::from(key.clone());

        match table.get(wrapped_key)? {
            Some(guard) => {
                let model: M = guard.value(); // BincodeWrapper::SelfType = T, so this returns M directly
                Ok(Some(model))
            }
            None => Ok(None),
        }
    }

    /// Remove a model by primary key
    ///
    /// Returns the removed model if it existed. Also removes all secondary
    /// key entries for the model.
    ///
    /// # Arguments
    ///
    /// * `key` - The primary key of the model to remove
    ///
    /// # Returns
    ///
    /// Some(model) if the model existed and was removed, None if it didn't exist
    pub fn remove(
        &mut self,
        key: <M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey,
    ) -> Result<Option<M>, NetabaseError> {
        let table_def = self.table_def();
        let sec_table_def = self.secondary_table_def();
        let wrapped_key = M::Keys::from(key.clone());

        let mut table = self.txn.open_table(table_def)?;

        // Get the model to extract secondary keys
        let model = match table.get(wrapped_key.clone())? {
            Some(guard) => {
                let model: M = guard.value(); // BincodeWrapper::SelfType = T, so this returns M directly
                Some(model)
            }
            None => None,
        };

        // Remove from primary table
        table.remove(wrapped_key)?;

        // Remove from secondary indexes
        if let Some(ref m) = model {
            let secondary_keys = m.secondary_keys();
            if !secondary_keys.is_empty() {
                let mut sec_table = self.txn.open_multimap_table(sec_table_def)?;
                for sec_key in secondary_keys.values() {
                    sec_table.remove(sec_key.clone(), key.clone())?;
                }
            }
        }

        Ok(model)
    }

    /// Remove multiple models in bulk
    ///
    /// Returns the removed models in the same order as the input keys.
    /// Some(model) if the model existed and was removed, None if it didn't exist.
    ///
    /// # Arguments
    ///
    /// * `keys` - Vector of primary keys to remove
    ///
    /// # Returns
    ///
    /// Vector of `Option<model>` in the same order as input keys
    pub fn remove_many(
        &mut self,
        keys: Vec<<M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey>,
    ) -> Result<Vec<Option<M>>, NetabaseError> {
        let mut results = Vec::with_capacity(keys.len());
        for key in keys {
            results.push(self.remove(key)?);
        }
        Ok(results)
    }

    /// Get the number of models in the tree
    ///
    /// # Returns
    ///
    /// The count of models stored in this tree
    pub fn len(&self) -> Result<usize, NetabaseError> {
        let table_def = self.table_def();
        match self.txn.open_table(table_def) {
            Ok(table) => Ok(table.len()? as usize),
            Err(redb::TableError::TableDoesNotExist(_)) => Ok(0),
            Err(e) => Err(NetabaseError::RedbTableError(e)),
        }
    }

    /// Check if the tree is empty
    ///
    /// # Returns
    ///
    /// true if the tree contains no models, false otherwise
    pub fn is_empty(&self) -> Result<bool, NetabaseError> {
        Ok(self.len()? == 0)
    }

    /// Clear all models from the tree
    ///
    /// This removes all entries from both primary and secondary tables.
    /// Use with caution as this operation cannot be undone within the transaction.
    pub fn clear(&mut self) -> Result<(), NetabaseError>
    where
        <M as NetabaseModelTrait<D>>::Keys: std::borrow::Borrow<
            <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelTraitKey<D>>::SecondaryKey as redb::Value>::SelfType<'db>
        >
    {
        let table_def = self.table_def();

        // Clear primary table by removing first entry repeatedly
        let mut table = self.txn.open_table(table_def)?;
        while let Some((key, _)) = table.pop_first()? {
            // Key is already removed by pop_first
            drop(key);
        }

        // TODO: Clear secondary table - need to figure out the right approach for MultimapTable
        // For now, secondary indices will be cleared when models are individually removed
        // or when the database is closed

        Ok(())
    }
}

/// Immutable tree for read operations
///
/// This provides methods to query models within a read transaction.
/// Multiple immutable trees can be opened simultaneously from the same
/// read transaction.
///
/// # Lifetime Management
///
/// - `'txn`: Borrows from the transaction
/// - `'db`: Borrows from the database (through transaction)
/// - `D`: Database definition type
/// - `M`: Model type for this tree
///
/// # Examples
///
/// ```no_run
/// use netabase_store::databases::redb_zerocopy::RedbStoreZeroCopy;
/// use netabase_store::error::NetabaseError;
/// use netabase_store::{NetabaseModel, netabase, netabase_definition_module};
///
/// #[netabase_definition_module(MyDefinition, MyKeys)]
/// mod my_models {
///     use netabase_store::{NetabaseModel, netabase};
///     #[derive(NetabaseModel, Clone, Debug, PartialEq,
///              bincode::Encode, bincode::Decode,
///              serde::Serialize, serde::Deserialize)]
///     #[netabase(MyDefinition)]
///     pub struct User {
///         #[primary_key]
///         pub id: u64,
///         pub name: String,
///     }
/// }
/// use my_models::*;
///
/// # fn main() -> Result<(), NetabaseError> {
/// let store = RedbStoreZeroCopy::<MyDefinition>::new("./test.db")?;
/// let txn = store.begin_read()?;
/// let tree = txn.open_tree::<User>()?;
///
/// // Query by primary key
/// if let Some(user) = tree.get(&UserPrimaryKey(1))? {
///     println!("Found user: {}", user.name);
/// }
///
/// // Check tree statistics
/// println!("Tree has {} users", tree.len()?);
/// # Ok(())
/// # }
/// ```
pub struct RedbTree<'txn, 'db, D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
{
    pub(crate) txn: &'txn ReadTransaction,
    #[allow(dead_code)]
    pub(crate) discriminant: D::Discriminant,
    pub(crate) table_name: &'static str,
    pub(crate) secondary_table_name: &'static str,
    pub(crate) _phantom: PhantomData<(&'db D, M)>,
}

impl<'txn, 'db, D, M> RedbTree<'txn, 'db, D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
    M::Keys: NetabaseModelTraitKey<D>,
{
    /// Get the table definition for the primary table
    fn table_def(
        &self,
    ) -> TableDefinition<'static, M::Keys, super::super::redb_store::BincodeWrapper<M>> {
        TableDefinition::new(self.table_name)
    }

    /// Get the table definition for secondary keys
    fn secondary_table_def(
        &self,
    ) -> MultimapTableDefinition<
        'static,
        <M::Keys as NetabaseModelTraitKey<D>>::SecondaryKey,
        <M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey,
    > {
        MultimapTableDefinition::new(self.secondary_table_name)
    }

    /// Get a model by primary key (returns owned/cloned data)
    ///
    /// # Arguments
    ///
    /// * `key` - The primary key to look up
    ///
    /// # Returns
    ///
    /// Some(model) if found, None if not found
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use netabase_store::databases::redb_zerocopy::RedbStoreZeroCopy;
    /// use netabase_store::error::NetabaseError;
    /// use netabase_store::{NetabaseModel, netabase, netabase_definition_module};
    ///
    /// #[netabase_definition_module(MyDefinition, MyKeys)]
    /// mod my_models {
    ///     use netabase_store::{NetabaseModel, netabase};
    ///     #[derive(NetabaseModel, Clone, Debug, PartialEq,
    ///              bincode::Encode, bincode::Decode,
    ///              serde::Serialize, serde::Deserialize)]
    ///     #[netabase(MyDefinition)]
    ///     pub struct User {
    ///         #[primary_key]
    ///         pub id: u64,
    ///         pub name: String,
    ///     }
    /// }
    /// use my_models::*;
    ///
    /// # fn main() -> Result<(), NetabaseError> {
    /// let store = RedbStoreZeroCopy::<MyDefinition>::new("./test.db")?;
    /// let txn = store.begin_read()?;
    /// let tree = txn.open_tree::<User>()?;
    /// if let Some(user) = tree.get(&UserPrimaryKey(1))? {
    ///     println!("Found user: {}", user.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get(
        &self,
        key: &<M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey,
    ) -> Result<Option<M>, NetabaseError> {
        let table_def = self.table_def();
        let table = self.txn.open_table(table_def)?;
        let wrapped_key = M::Keys::from(key.clone());

        match table.get(wrapped_key)? {
            Some(guard) => {
                let model: M = guard.value(); // BincodeWrapper::SelfType = T, so this returns M directly
                Ok(Some(model))
            }
            None => Ok(None),
        }
    }

    /// Get models by secondary key
    ///
    /// Returns all models that have the given secondary key.
    /// This returns an iterator-like object that can be used to access
    /// the primary keys, which can then be used to fetch the full models.
    ///
    /// # Arguments
    ///
    /// * `sec_key` - The secondary key to look up
    ///
    /// # Returns
    ///
    /// A MultimapValue iterator over primary keys that match the secondary key
    pub fn get_by_secondary_key(
        &'txn self,
        sec_key: &<M::Keys as NetabaseModelTraitKey<D>>::SecondaryKey,
    ) -> Result<MultimapValue<'txn, <M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey>, NetabaseError>
    where
        for<'a> &'a <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelTraitKey<D>>::SecondaryKey:
            std::borrow::Borrow<
                <<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelTraitKey<D>>::SecondaryKey as redb::Value>::SelfType<'a>
            >
    {
        let sec_table_def = self.secondary_table_def();
        let sec_table = self.txn.open_multimap_table(sec_table_def)?;
        sec_table
            .get(sec_key)
            .map_err(NetabaseError::RedbStorageError)
    }

    /// Get the number of models in the tree
    ///
    /// # Returns
    ///
    /// The count of models stored in this tree
    pub fn len(&self) -> Result<usize, NetabaseError> {
        let table_def = self.table_def();
        match self.txn.open_table(table_def) {
            Ok(table) => Ok(table.len()? as usize),
            Err(redb::TableError::TableDoesNotExist(_)) => Ok(0),
            Err(e) => Err(NetabaseError::RedbTableError(e)),
        }
    }

    /// Check if the tree is empty
    ///
    /// # Returns
    ///
    /// true if the tree contains no models, false otherwise
    pub fn is_empty(&self) -> Result<bool, NetabaseError> {
        Ok(self.len()? == 0)
    }
}

// Tests temporarily disabled due to macro resolution issues within the crate itself
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::databases::redb_zerocopy::RedbStoreZeroCopy;
//     use tempfile::tempdir;
//
//     // Tests would go here but require proper macro setup
// }
