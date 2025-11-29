//! Transaction types for redb zero-copy backend
//!
//! This module provides transaction abstractions that manage the lifetime
//! and borrowing relationships between the store, transactions, and trees.

use crate::error::NetabaseError;
use crate::traits::definition::NetabaseDefinitionTrait;
use crate::traits::model::{NetabaseModelTrait, NetabaseModelTraitKey};
use redb::{ReadTransaction, WriteTransaction};
use std::marker::PhantomData;

use super::tree::{RedbTree, RedbTreeMut};
use super::utils::{get_secondary_table_name, get_table_name};

/// Write transaction for zero-copy redb backend
///
/// Write transactions are exclusive and must be explicitly committed or aborted.
/// They provide methods to open mutable trees for different model types.
///
/// # Lifetime Management
///
/// The transaction borrows from the database store for its lifetime `'db`.
/// Trees opened from this transaction will further borrow from the transaction.
///
/// # Examples
///
/// ```no_run
/// use netabase_store::databases::redb_zerocopy::RedbStoreZeroCopy;
/// use netabase_store::{netabase_definition_module, NetabaseModel, netabase, error::NetabaseError};
///
/// #[netabase_definition_module(MyDefinition, MyKeys)]
/// mod my_models {
///     use netabase_store::{NetabaseModel, netabase};
///
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
/// tree.put(User { id: 1, name: "Alice".to_string() })?;
/// drop(tree);
/// txn.commit()?; // Must be committed to persist changes
/// # Ok(())
/// # }
/// ```
pub struct RedbWriteTransactionZC<'db, D>
where
    D: NetabaseDefinitionTrait,
{
    pub(crate) inner: WriteTransaction,
    pub(crate) _phantom: PhantomData<&'db D>,
}

impl<'db, D> RedbWriteTransactionZC<'db, D>
where
    D: NetabaseDefinitionTrait,
{
    /// Create a new write transaction from a redb WriteTransaction
    pub(crate) fn new(inner: WriteTransaction) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }

    /// Open a mutable tree for a specific model type
    ///
    /// The tree borrows from this transaction and can be used for read/write operations.
    /// The tree must be dropped before the transaction can be committed.
    ///
    /// # Type Parameters
    ///
    /// * `M` - The model type to open a tree for
    ///
    /// # Returns
    ///
    /// A mutable tree that can perform CRUD operations on the model type
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use netabase_store::databases::redb_zerocopy::RedbStoreZeroCopy;
    /// use netabase_store::{netabase_definition_module, NetabaseModel, netabase, error::NetabaseError};
    ///
    /// #[netabase_definition_module(MyDefinition, MyKeys)]
    /// mod my_models {
    ///     use netabase_store::{NetabaseModel, netabase};
    ///
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
    /// let user = User { id: 1, name: "Alice".to_string() };
    /// tree.put(user)?;
    ///
    /// drop(tree); // Must drop tree before commit
    /// txn.commit()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn open_tree<M>(&mut self) -> Result<RedbTreeMut<'_, 'db, D, M>, NetabaseError>
    where
        M: NetabaseModelTrait<D>,
        M::Keys: NetabaseModelTraitKey<D>,
    {
        let discriminant = M::DISCRIMINANT;

        // Get table names from discriminant
        let table_name = get_table_name::<D>(discriminant);
        let secondary_table_name = get_secondary_table_name::<D>(discriminant);

        Ok(RedbTreeMut {
            txn: &mut self.inner,
            discriminant,
            table_name,
            secondary_table_name,
            _phantom: PhantomData,
        })
    }

    /// Commit the transaction, making all changes permanent
    ///
    /// This consumes the transaction, ensuring it cannot be used after commit.
    /// All trees opened from this transaction must be dropped before calling commit.
    ///
    /// # Errors
    ///
    /// Returns an error if the commit operation fails due to I/O issues
    /// or if there are active borrows from trees.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use netabase_store::databases::redb_zerocopy::RedbStoreZeroCopy;
    /// use netabase_store::{netabase_definition_module, NetabaseModel, netabase, error::NetabaseError};
    ///
    /// #[netabase_definition_module(MyDefinition, MyKeys)]
    /// mod my_models {
    ///     use netabase_store::{NetabaseModel, netabase};
    ///
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
    /// tree.put(User { id: 1, name: "Alice".to_string() })?;
    /// drop(tree);
    /// txn.commit()?; // All changes are now persistent
    /// # Ok(())
    /// # }
    /// ```
    pub fn commit(self) -> Result<(), NetabaseError> {
        self.inner.commit()?;
        Ok(())
    }

    /// Abort the transaction, discarding all changes
    ///
    /// This consumes the transaction and discards any changes made during
    /// the transaction. This is automatically called if the transaction
    /// is dropped without being committed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use netabase_store::databases::redb_zerocopy::RedbStoreZeroCopy;
    /// use netabase_store::{netabase_definition_module, NetabaseModel, netabase, error::NetabaseError};
    ///
    /// #[netabase_definition_module(MyDefinition, MyKeys)]
    /// mod my_models {
    ///     use netabase_store::{NetabaseModel, netabase};
    ///
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
    /// tree.put(User { id: 1, name: "Alice".to_string() })?;
    /// drop(tree);
    ///
    /// let should_abort = false;
    /// if should_abort {
    ///     txn.abort()?; // All changes are discarded
    /// } else {
    ///     txn.commit()?; // Changes are persisted
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn abort(self) -> Result<(), NetabaseError> {
        self.inner.abort()?;
        Ok(())
    }
}

/// Read transaction for zero-copy redb backend
///
/// Read transactions provide a consistent snapshot of the database.
/// Multiple read transactions can be active concurrently, and they
/// don't block write transactions.
///
/// # Consistency Guarantee
///
/// The read transaction provides a point-in-time consistent view of
/// the database. All reads within the same transaction will see the
/// same state, even if concurrent writes occur.
///
/// # Examples
///
/// ```no_run
/// use netabase_store::databases::redb_zerocopy::RedbStoreZeroCopy;
/// use netabase_store::{netabase_definition_module, NetabaseModel, netabase, error::NetabaseError};
///
/// #[netabase_definition_module(MyDefinition, MyKeys)]
/// mod my_models {
///     use netabase_store::{NetabaseModel, netabase};
///
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
/// let user = tree.get(&UserPrimaryKey(1))?;
/// // Transaction automatically ends when dropped
/// # Ok(())
/// # }
/// ```
pub struct RedbReadTransactionZC<'db, D>
where
    D: NetabaseDefinitionTrait,
{
    pub(crate) inner: ReadTransaction,
    pub(crate) _phantom: PhantomData<&'db D>,
}

impl<'db, D> RedbReadTransactionZC<'db, D>
where
    D: NetabaseDefinitionTrait,
{
    /// Create a new read transaction from a redb ReadTransaction
    pub(crate) fn new(inner: ReadTransaction) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }

    /// Open an immutable tree for a specific model type
    ///
    /// The tree borrows from this transaction and can be used for read-only operations.
    /// Multiple trees can be open simultaneously from the same transaction.
    ///
    /// # Type Parameters
    ///
    /// * `M` - The model type to open a tree for
    ///
    /// # Returns
    ///
    /// An immutable tree that can perform read operations on the model type
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use netabase_store::databases::redb_zerocopy::RedbStoreZeroCopy;
    /// use netabase_store::{netabase_definition_module, NetabaseModel, netabase, error::NetabaseError};
    ///
    /// #[netabase_definition_module(MyDefinition, MyKeys)]
    /// mod my_models {
    ///     use netabase_store::{NetabaseModel, netabase};
    ///
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
    /// // Get a user by key
    /// if let Some(user) = tree.get(&UserPrimaryKey(1))? {
    ///     println!("User: {}", user.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn open_tree<M>(&self) -> Result<RedbTree<'_, 'db, D, M>, NetabaseError>
    where
        M: NetabaseModelTrait<D>,
        M::Keys: NetabaseModelTraitKey<D>,
    {
        let discriminant = M::DISCRIMINANT;

        // Get table names from discriminant
        let table_name = get_table_name::<D>(discriminant);
        let secondary_table_name = get_secondary_table_name::<D>(discriminant);

        Ok(RedbTree {
            txn: &self.inner,
            discriminant,
            table_name,
            secondary_table_name,
            _phantom: PhantomData,
        })
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
