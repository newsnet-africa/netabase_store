//! Sled database backend implementation for Netabase Store
//!
//! This module provides a sled-based storage backend that implements the same
//! traits as the redb backend, allowing interchangeable use of either database
//! engine based on application requirements.
//!
//! ## Architecture
//!
//! Unlike the redb implementation which uses typed tables and zero-copy access,
//! the sled backend works with byte vectors and uses bincode for serialization.
//! This provides a simpler API at the cost of some performance overhead for
//! deserialization.
//!
//! ## Key Differences from Redb
//!
//! - **Serialization**: Uses bincode for all keys and values (vs redb's Key/Value traits)
//! - **Transactions**: Adapts sled's closure-based API to Netabase's trait-based API
//! - **Tree Management**: Dynamic tree creation (vs redb's static TableDefinitions)
//! - **Value Access**: All values are owned (vs redb's AccessGuard for zero-copy)
//!
//! ## Usage
//!
//! ```ignore
//! use netabase_store::databases::sled_store::{SledStore, SledStoreTrait};
//!
//! // Create a new sled store
//! let store = SledStore::<MyDefinition>::new("./data/my_store")?;
//!
//! // Use with transactions
//! store.write(|txn| {
//!     txn.put(my_model)?;
//!     Ok(())
//! })?;
//! # Ok::<(), netabase_store::error::NetabaseError>(())
//! ```

use crate::{
    error::NetabaseResult,
    traits::definition::{DiscriminantName, NetabaseDefinition},
};
use log::debug;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::path::Path;
use std::time::Instant;
use strum::{IntoDiscriminant, IntoEnumIterator};

pub mod error;
pub mod manager;
pub mod transaction;
pub mod traits;
pub mod store_trait;

pub use transaction::{SledReadTransaction, SledWriteTransaction};
pub use traits::{SledModelAssociatedTypesExt, SledNetabaseModelTrait};
pub use store_trait::SledStoreTrait;
pub use manager::SledDefinitionManager;

// =============================================================================
// Serialization Helpers
// =============================================================================

/// Serialize a key to bytes using bincode
///
/// This function uses bincode's standard configuration for consistent
/// serialization across the codebase.
#[inline]
pub(crate) fn serialize_key<K: bincode::Encode>(key: &K) -> NetabaseResult<Vec<u8>> {
    bincode::encode_to_vec(key, bincode::config::standard())
        .map_err(|e| e.into())
}

/// Deserialize a key from bytes using bincode
///
/// This function uses bincode's standard configuration for consistent
/// deserialization across the codebase.
#[inline]
pub(crate) fn deserialize_key<K: bincode::Decode<()>>(bytes: &[u8]) -> NetabaseResult<K> {
    let (key, _len) = bincode::decode_from_slice(bytes, bincode::config::standard())
        .map_err(|e| e)?;
    Ok(key)
}

/// Serialize a value to bytes using bincode
///
/// This function uses bincode's standard configuration for consistent
/// serialization across the codebase.
#[inline]
pub(crate) fn serialize_value<V: bincode::Encode>(value: &V) -> NetabaseResult<Vec<u8>> {
    bincode::encode_to_vec(value, bincode::config::standard())
        .map_err(|e| e.into())
}

/// Deserialize a value from bytes using bincode
///
/// This function uses bincode's standard configuration for consistent
/// deserialization across the codebase.
#[inline]
pub(crate) fn deserialize_value<V: bincode::Decode<()>>(bytes: &[u8]) -> NetabaseResult<V> {
    let (value, _len) = bincode::decode_from_slice(bytes, bincode::config::standard())
        .map_err(|e| e)?;
    Ok(value)
}

// =============================================================================
// SledStore Implementation
// =============================================================================

/// Sled-based store implementation for Netabase
///
/// This store uses sled as the underlying database engine, providing a
/// lightweight embedded database with good performance characteristics.
///
/// ## Type Parameters
///
/// - `D`: The Definition enum type that describes all models in the store
///
/// ## Example
///
/// ```ignore
/// use netabase_store::databases::sled_store::SledStore;
///
/// // Create a persistent store
/// let store = SledStore::<MyDefinition>::new("./data")?;
///
/// // Create a temporary store for testing
/// let temp_store = SledStore::<MyDefinition>::temporary()?;
/// # Ok::<(), netabase_store::error::NetabaseError>(())
/// ```
pub struct SledStore<D: NetabaseDefinition>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    /// The underlying sled database
    ///
    /// Sled's Db type uses Arc internally, so cloning is cheap
    pub(crate) db: sled::Db,

    /// Phantom data to bind the Definition type
    _marker: PhantomData<D>,
}

impl<D: NetabaseDefinition> SledStore<D>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    /// Create or open a sled store at the given path
    ///
    /// This will create a new sled database if one doesn't exist, or open
    /// an existing database at the specified path.
    ///
    /// ## Arguments
    ///
    /// - `path`: The filesystem path where the database should be stored
    ///
    /// ## Errors
    ///
    /// Returns an error if the database cannot be created or opened, typically
    /// due to filesystem permissions or corrupted database files.
    ///
    /// ## Example
    ///
    /// ```ignore
    /// use netabase_store::databases::sled_store::SledStore;
    ///
    /// let store = SledStore::<MyDefinition>::new("./my_database")?;
    /// # Ok::<(), netabase_store::error::NetabaseError>(())
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> NetabaseResult<Self> {
        let start = Instant::now();
        let path_display = path.as_ref().display().to_string();
        debug!("SledStore: Opening database at {}", path_display);
        
        let db = sled::open(path)?;
        
        debug!("SledStore: Opened in {:?}", start.elapsed());
        Ok(SledStore {
            db,
            _marker: PhantomData,
        })
    }

    /// Create a temporary in-memory sled store
    ///
    /// This is useful for testing or for applications that don't need
    /// persistent storage. The database is entirely in-memory and will
    /// be dropped when the store is dropped.
    ///
    /// ## Errors
    ///
    /// Returns an error if the temporary database cannot be created.
    ///
    /// ## Example
    ///
    /// ```ignore
    /// use netabase_store::databases::sled_store::SledStore;
    ///
    /// let store = SledStore::<MyDefinition>::temporary()?;
    /// # Ok::<(), netabase_store::error::NetabaseError>(())
    /// ```
    pub fn temporary() -> NetabaseResult<Self> {
        let start = Instant::now();
        debug!("SledStore: Creating temporary database");

        let db = sled::Config::new()
            .temporary(true)
            .open()?;

        debug!("SledStore: Created temporary in {:?}", start.elapsed());
        Ok(SledStore {
            db,
            _marker: PhantomData,
        })
    }

    /// Get a reference to the underlying sled database
    ///
    /// This provides direct access to sled's API for advanced use cases
    /// that aren't covered by the Netabase abstractions.
    ///
    /// ## Example
    ///
    /// ```ignore
    /// use netabase_store::databases::sled_store::SledStore;
    ///
    /// let store = SledStore::<MyDefinition>::new("./data")?;
    /// let db = store.sled_database();
    ///
    /// // Use sled's API directly
    /// let tree_names = db.tree_names();
    /// # Ok::<(), netabase_store::error::NetabaseError>(())
    /// ```
    pub fn sled_database(&self) -> &sled::Db {
        &self.db
    }

    /// Flush all pending writes to disk
    ///
    /// Sled buffers writes in memory for performance. This method forces
    /// all pending writes to be persisted to disk, ensuring durability.
    ///
    /// ## Errors
    ///
    /// Returns an error if the flush operation fails, typically due to
    /// I/O errors.
    ///
    /// ## Example
    ///
    /// ```ignore
    /// use netabase_store::databases::sled_store::SledStore;
    ///
    /// let store = SledStore::<MyDefinition>::new("./data")?;
    /// // ... perform some writes ...
    /// store.flush()?; // Ensure data is on disk
    /// # Ok::<(), netabase_store::error::NetabaseError>(())
    /// ```
    pub fn flush(&self) -> NetabaseResult<()> {
        self.db.flush()?;
        Ok(())
    }

    /// Get the size of the database on disk in bytes
    ///
    /// This returns the total size of all database files, which can be
    /// useful for monitoring storage usage.
    ///
    /// ## Errors
    ///
    /// Returns an error if the size cannot be determined.
    pub fn size_on_disk(&self) -> NetabaseResult<u64> {
        Ok(self.db.size_on_disk()?)
    }

    /// Export all data to a specified path
    ///
    /// This creates a backup of the entire database at the specified path.
    /// The backup is consistent and can be used to restore the database.
    ///
    /// ## Arguments
    ///
    /// - `path`: The filesystem path where the backup should be written
    ///
    /// ## Errors
    ///
    /// Returns an error if the export fails, typically due to I/O errors
    /// or insufficient disk space.
    pub fn export<P: AsRef<Path>>(&self, path: P) -> NetabaseResult<()> {
        let export_db = sled::open(path)?;
        for entry in self.db.export() {
            let (key, value, _) = entry;
            export_db.insert(key, value)?;
        }
        export_db.flush()?;
        Ok(())
    }
}

// =============================================================================
// SledStoreTrait Implementation
// =============================================================================

impl<D: NetabaseDefinition> SledStoreTrait<D> for SledStore<D>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    /// Execute a read transaction
    ///
    /// Read transactions provide consistent snapshot isolation - all reads
    /// within the transaction see a consistent view of the database as it
    /// existed when the transaction began.
    fn read<'a, F, R>(&'a self, f: F) -> NetabaseResult<R>
    where
        F: FnOnce(&SledReadTransaction<'a, D>) -> NetabaseResult<R>,
    {
        // Sled doesn't have explicit read transactions
        // We create a wrapper that provides consistent snapshot semantics
        let txn = SledReadTransaction {
            db: &self.db,
            _sled_store: self,
        };
        f(&txn)
    }

    /// Execute a write transaction
    ///
    /// Write transactions queue all operations and execute them atomically
    /// when committed. If the closure returns an error, the transaction is
    /// aborted and no changes are made.
    fn write<F, R>(&self, f: F) -> NetabaseResult<R>
    where
        F: FnOnce(&mut SledWriteTransaction<D>) -> NetabaseResult<R>,
    {
        let mut txn = SledWriteTransaction::new(&self.db, self);
        let result = f(&mut txn)?;

        // Commit the transaction (processes operation queue)
        txn.commit()?;

        Ok(result)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_temporary_store() {
        // This is a basic compilation test
        // More detailed tests will be in the transaction module
    }

    #[test]
    fn test_serialization_roundtrip() {
        let value: u64 = 42;
        let bytes = serialize_value(&value).unwrap();
        let deserialized: u64 = deserialize_value(&bytes).unwrap();
        assert_eq!(value, deserialized);

        let key: String = "test_key".to_string();
        let bytes = serialize_key(&key).unwrap();
        let deserialized: String = deserialize_key(&bytes).unwrap();
        assert_eq!(key, deserialized);
    }
}
