/// Backend trait definitions for Netabase Store
///
/// These traits define the interface that any key-value store backend must implement
/// to be compatible with Netabase. They abstract over storage implementations like
/// redb, sled, IndexedDB, etc.

use std::fmt::Debug;
use super::error::BackendError;

/// Trait for types that can be used as keys in the backend storage
///
/// Keys must be serializable, comparable, and clonable. The backend
/// determines the specific serialization format.
pub trait BackendKey: Debug + Clone + Send + Sync + 'static {
    /// Serialize the key to bytes for storage
    fn to_bytes(&self) -> Vec<u8>;

    /// Deserialize the key from bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn BackendError>>
    where
        Self: Sized;
}

/// Trait for types that can be used as values in the backend storage
///
/// Values must be serializable and clonable. The backend determines
/// the specific serialization format.
pub trait BackendValue: Debug + Clone + Send + Sync + 'static {
    /// Serialize the value to bytes for storage
    fn to_bytes(&self) -> Vec<u8>;

    /// Deserialize the value from bytes
    fn from_bytes(bytes: &[u8]) -> Result<Self, Box<dyn BackendError>>
    where
        Self: Sized;
}

/// Type alias for backend iterators to enable trait object usage
pub type BoxedIterator<K, V> = Box<dyn Iterator<Item = Result<(K, V), Box<dyn BackendError>>> + Send>;

/// Read-only table/tree interface for the backend
///
/// This represents a single table/tree in the storage backend that can be queried.
pub trait BackendReadableTable<K: BackendKey, V: BackendValue> {
    /// Get a value by key, returns None if not found
    fn get(&self, key: &K) -> Result<Option<V>, Box<dyn BackendError>>;

    /// Returns an iterator over all key-value pairs in the table
    fn iter(&self) -> Result<BoxedIterator<K, V>, Box<dyn BackendError>>;

    /// Returns the number of entries in the table (if efficiently computable)
    fn len(&self) -> Result<Option<u64>, Box<dyn BackendError>> {
        Ok(None) // Default: not all backends can efficiently compute this
    }

    /// Returns true if the table is empty
    fn is_empty(&self) -> Result<bool, Box<dyn BackendError>> {
        Ok(self.len()?.map(|l| l == 0).unwrap_or(false))
    }
}

/// Writable table/tree interface for the backend
///
/// This extends the readable interface with mutation operations.
pub trait BackendWritableTable<K: BackendKey, V: BackendValue>: BackendReadableTable<K, V> {
    /// Insert or update a key-value pair
    fn insert(&mut self, key: &K, value: &V) -> Result<(), Box<dyn BackendError>>;

    /// Remove a key-value pair, returns true if it existed
    fn remove(&mut self, key: &K) -> Result<bool, Box<dyn BackendError>>;
}

/// Read-only transaction interface
///
/// Provides isolated, consistent read access to the database.
pub trait BackendReadTransaction {
    /// Open a read-only table with the given name
    ///
    /// The table name identifies a specific tree/table in the storage backend.
    /// If the table doesn't exist, returns an error.
    fn open_table<K: BackendKey, V: BackendValue>(
        &self,
        table_name: &str,
    ) -> Result<Box<dyn BackendReadableTable<K, V>>, Box<dyn BackendError>>;

    /// Check if a table exists
    fn table_exists(&self, table_name: &str) -> Result<bool, Box<dyn BackendError>>;
}

/// Write transaction interface
///
/// Provides isolated, consistent read-write access to the database.
/// Changes are atomic and only visible after commit.
pub trait BackendWriteTransaction: BackendReadTransaction {
    /// Open a writable table with the given name
    ///
    /// If the table doesn't exist, it will be created.
    fn open_table_mut<K: BackendKey, V: BackendValue>(
        &mut self,
        table_name: &str,
    ) -> Result<Box<dyn BackendWritableTable<K, V>>, Box<dyn BackendError>>;

    /// Commit the transaction, making all changes durable
    fn commit(self) -> Result<(), Box<dyn BackendError>>
    where
        Self: Sized;

    /// Abort the transaction, discarding all changes
    fn abort(self) -> Result<(), Box<dyn BackendError>>
    where
        Self: Sized,
    {
        // Default: just drop the transaction
        Ok(())
    }
}

/// Main backend store interface
///
/// This is the top-level trait that storage backends must implement.
/// It provides transaction creation and database lifecycle management.
pub trait BackendStore: Send + Sync {
    /// The read transaction type for this backend
    type ReadTransaction<'a>: BackendReadTransaction
    where
        Self: 'a;

    /// The write transaction type for this backend
    type WriteTransaction: BackendWriteTransaction;

    /// Begin a read-only transaction
    fn begin_read(&self) -> Result<Self::ReadTransaction<'_>, Box<dyn BackendError>>;

    /// Begin a read-write transaction
    fn begin_write(&self) -> Result<Self::WriteTransaction, Box<dyn BackendError>>;

    /// Execute a closure within a read transaction
    fn read<F, R>(&self, f: F) -> Result<R, Box<dyn BackendError>>
    where
        F: FnOnce(&Self::ReadTransaction<'_>) -> Result<R, Box<dyn BackendError>>,
    {
        let txn = self.begin_read()?;
        f(&txn)
    }

    /// Execute a closure within a write transaction, automatically committing on success
    fn write<F, R>(&self, f: F) -> Result<R, Box<dyn BackendError>>
    where
        F: FnOnce(&mut Self::WriteTransaction) -> Result<R, Box<dyn BackendError>>,
    {
        let mut txn = self.begin_write()?;
        let result = f(&mut txn)?;
        txn.commit()?;
        Ok(result)
    }

    /// Flush any cached changes to disk
    fn flush(&self) -> Result<(), Box<dyn BackendError>> {
        Ok(()) // Default: no-op for backends that don't need explicit flushing
    }

    /// Close the database
    fn close(self) -> Result<(), Box<dyn BackendError>>
    where
        Self: Sized,
    {
        Ok(()) // Default: no-op for backends that don't need explicit closing
    }
}

/// Marker trait that combines backend requirements for table operations
///
/// This is a convenience trait that bundles the common requirements for
/// tables in Netabase operations.
pub trait BackendTable<K: BackendKey, V: BackendValue>:
    BackendReadableTable<K, V> + BackendWritableTable<K, V>
{
}

// Blanket implementation
impl<T, K: BackendKey, V: BackendValue> BackendTable<K, V> for T where
    T: BackendReadableTable<K, V> + BackendWritableTable<K, V>
{
}
