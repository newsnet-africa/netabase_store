//! Unified store interface providing a single entry point for all storage backends.
//!
//! The `NetabaseStore<Backend>` wrapper provides a consistent API across different
//! storage backends (Sled, Redb, IndexedDB, Memory) while allowing backend-specific
//! functionality through specialized implementations.
//!
//! # Examples
//!
//! ```ignore
//! use netabase_store::NetabaseStore;
//!
//! // Create a Sled-backed store
//! let store = NetabaseStore::sled("./data")?;
//! let tree = store.open_tree::<User>();
//! tree.put_raw(user)?;
//!
//! // Create a Redb-backed store
//! let store = NetabaseStore::redb("./data.redb")?;
//! let tree = store.open_tree::<User>();
//! ```

use crate::error::NetabaseError;
use crate::traits::definition::NetabaseDefinitionTrait;
use crate::traits::model::NetabaseModelTrait;
use crate::traits::store_ops::{OpenTree, StoreOps};
use std::marker::PhantomData;
use std::path::Path;

/// Marker trait for backends that store a specific Definition type.
///
/// This trait is automatically implemented for all backend stores and binds
/// the Definition type to the backend at compile time.
pub trait BackendFor<D: NetabaseDefinitionTrait> {}

// Blanket implementation for all backend types that match the pattern
#[cfg(feature = "sled")]
impl<D> BackendFor<D> for crate::databases::sled_store::SledStore<D> where
    D: NetabaseDefinitionTrait
{
}

#[cfg(feature = "redb")]
impl<D> BackendFor<D> for crate::databases::redb_store::RedbStore<D> where
    D: NetabaseDefinitionTrait
{
}

#[cfg(feature = "memory")]
impl<D> BackendFor<D> for crate::databases::memory_store::MemoryStore<D> where
    D: NetabaseDefinitionTrait
{
}

/// Unified store wrapper providing a consistent API across all storage backends.
///
/// This generic wrapper allows you to write backend-agnostic code while still
/// having access to backend-specific features when needed.
///
/// # Type Parameters
///
/// * `D` - The NetabaseDefinition type that defines all models for this store
/// * `Backend` - The underlying storage backend (SledStore, RedbStore, etc.)
pub struct NetabaseStore<D, Backend>
where
    D: NetabaseDefinitionTrait,
    Backend: BackendFor<D>,
{
    backend: Backend,
    _phantom: PhantomData<D>,
}

impl<D, Backend> NetabaseStore<D, Backend>
where
    D: NetabaseDefinitionTrait,
    Backend: BackendFor<D>,
{
    /// Create a new store from an existing backend instance.
    ///
    /// This is useful when you need to configure the backend with specific options
    /// before wrapping it in a NetabaseStore.
    pub fn from_backend(backend: Backend) -> Self {
        Self {
            backend,
            _phantom: PhantomData,
        }
    }

    /// Get a reference to the underlying backend.
    ///
    /// This allows access to backend-specific methods and configuration.
    pub fn backend(&self) -> &Backend {
        &self.backend
    }

    /// Get a mutable reference to the underlying backend.
    ///
    /// This allows access to backend-specific mutable methods.
    pub fn backend_mut(&mut self) -> &mut Backend {
        &mut self.backend
    }

    /// Consume the store and return the underlying backend.
    pub fn into_backend(self) -> Backend {
        self.backend
    }
}

// Implement OpenTree for any backend that implements it
impl<D, M, Backend> OpenTree<D, M> for NetabaseStore<D, Backend>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
    <D as strum::IntoDiscriminant>::Discriminant: crate::traits::definition::NetabaseDiscriminant,
    Backend: OpenTree<D, M>,
{
    type Tree<'a>
        = Backend::Tree<'a>
    where
        Self: 'a;

    fn open_tree(&self) -> Self::Tree<'_> {
        self.backend.open_tree()
    }
}

// Convenience constructors for each backend type
#[cfg(feature = "sled")]
impl<D> NetabaseStore<D, crate::databases::sled_store::SledStore<D>>
where
    D: NetabaseDefinitionTrait + crate::traits::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant: crate::traits::definition::NetabaseDiscriminant,
{
    /// Create a new Sled-backed store at the given path.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let store = NetabaseStore::<MyDefinition, _>::sled("./my_database")?;
    /// ```
    pub fn sled<P: AsRef<Path>>(path: P) -> Result<Self, NetabaseError> {
        Ok(Self::from_backend(
            crate::databases::sled_store::SledStore::new(path)?,
        ))
    }
}

#[cfg(feature = "redb")]
impl<D> NetabaseStore<D, crate::databases::redb_store::RedbStore<D>>
where
    D: NetabaseDefinitionTrait + crate::traits::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant: crate::traits::definition::NetabaseDiscriminant,
{
    /// Create a new Redb-backed store at the given path.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let store = NetabaseStore::redb("./my_database.redb")?;
    /// ```
    pub fn redb<P: AsRef<Path>>(path: P) -> Result<Self, NetabaseError> {
        Ok(Self::from_backend(
            crate::databases::redb_store::RedbStore::new(path)?,
        ))
    }

    /// Open an existing Redb-backed store at the given path.
    pub fn open_redb<P: AsRef<Path>>(path: P) -> Result<Self, NetabaseError> {
        Ok(Self::from_backend(
            crate::databases::redb_store::RedbStore::open(path)?,
        ))
    }
}

#[cfg(feature = "memory")]
impl<D> NetabaseStore<D, crate::databases::memory_store::MemoryStore<D>>
where
    D: NetabaseDefinitionTrait,
    <D as strum::IntoDiscriminant>::Discriminant: crate::traits::definition::NetabaseDiscriminant,
{
    /// Create a new in-memory store.
    ///
    /// This is useful for testing or temporary storage.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let store = NetabaseStore::memory();
    /// ```
    pub fn memory() -> Self {
        Self::from_backend(crate::databases::memory_store::MemoryStore::new())
    }
}

// Backend-specific implementations

#[cfg(feature = "sled")]
impl<D> NetabaseStore<D, crate::databases::sled_store::SledStore<D>>
where
    D: NetabaseDefinitionTrait + crate::traits::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant: crate::traits::definition::NetabaseDiscriminant,
{
    /// Get direct access to the underlying sled database (Sled-specific).
    pub fn db(&self) -> &sled::Db {
        self.backend.db()
    }

    /// Flush the database to disk (Sled-specific).
    ///
    /// This ensures all pending writes are persisted to disk.
    pub fn flush(&self) -> Result<usize, NetabaseError> {
        Ok(self.backend.db().flush()?)
    }

    /// Get the size of the database on disk in bytes (Sled-specific).
    pub fn size_on_disk(&self) -> Result<u64, NetabaseError> {
        Ok(self.backend.db().size_on_disk()?)
    }

    /// Check if the database was recovered from a previous run (Sled-specific).
    pub fn was_recovered(&self) -> bool {
        self.backend.db().was_recovered()
    }

    /// Generate a monotonic ID (Sled-specific).
    ///
    /// This is useful for generating unique IDs without coordination.
    pub fn generate_id(&self) -> Result<u64, NetabaseError> {
        Ok(self.backend.db().generate_id()?)
    }
}

#[cfg(feature = "redb")]
impl<D> NetabaseStore<D, crate::databases::redb_store::RedbStore<D>>
where
    D: NetabaseDefinitionTrait + crate::traits::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant: crate::traits::definition::NetabaseDiscriminant,
{
    /// Check database integrity (Redb-specific).
    ///
    /// Returns `true` if the database is consistent.
    pub fn check_integrity(&mut self) -> Result<bool, NetabaseError> {
        self.backend.check_integrity()
    }

    /// Compact the database to reclaim space (Redb-specific).
    ///
    /// Returns `true` if compaction was successful.
    pub fn compact(&mut self) -> Result<bool, NetabaseError> {
        self.backend.compact()
    }

    /// Get direct access to the underlying redb database (Redb-specific).
    pub fn db(&self) -> &redb::Database {
        self.backend.db()
    }

    /// Get all tree names (discriminants) in the database (Redb-specific).
    pub fn tree_names(&self) -> Vec<D::Discriminant> {
        self.backend.tree_names()
    }
}
