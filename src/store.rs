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
    Backend: BackendFor<D> + OpenTree<D, M>,
{
    type Tree<'a>
        = Backend::Tree<'a>
    where
        Self: 'a;

    fn open_tree(&self) -> Self::Tree<'_> {
        self.backend.open_tree()
    }
}

// Generic open_tree method for all backends
impl<D, Backend> NetabaseStore<D, Backend>
where
    D: NetabaseDefinitionTrait,
    Backend: BackendFor<D>,
{
    /// Open a tree for a specific model type.
    ///
    /// This is a generic method that works with any backend.
    ///
    /// # Type Parameters
    ///
    /// * `M` - The model type to open a tree for
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let store = NetabaseStore::<MyDefinition, _>::sled("./db")?;
    /// let user_tree = store.open_tree::<User>();
    /// ```
    #[inline]
    pub fn open_tree<M>(&self) -> Backend::Tree<'_>
    where
        M: crate::traits::model::NetabaseModelTrait<D>,
        Backend: crate::traits::store_ops::OpenTree<D, M>,
    {
        use crate::traits::store_ops::OpenTree;
        OpenTree::open_tree(self)
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

    /// Create a temporary Sled-backed store (Sled-specific).
    ///
    /// The database will be deleted when the store is dropped.
    /// This is useful for testing and temporary storage.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let store = NetabaseStore::<MyDefinition, _>::temp()?;
    /// ```
    pub fn temp() -> Result<Self, NetabaseError> {
        Ok(Self::from_backend(
            crate::databases::sled_store::SledStore::temp()?,
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

    /// Begin a read-only transaction.
    ///
    /// Read-only transactions allow multiple concurrent reads without blocking.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let txn = store.read();
    /// let tree = txn.open_tree::<User>();
    /// let user = tree.get(UserPrimaryKey(1))?;
    /// ```
    pub fn read(&self) -> crate::transaction::TxnGuard<'_, D, crate::transaction::ReadOnly> {
        crate::transaction::TxnGuard::read_sled(self.backend.db())
    }

    /// Begin a read-write transaction.
    ///
    /// Read-write transactions allow multiple operations to be batched together
    /// and committed atomically.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut txn = store.write();
    /// let mut tree = txn.open_tree::<User>();
    /// tree.put(user)?;
    /// txn.commit()?;
    /// ```
    pub fn write(&self) -> crate::transaction::TxnGuard<'_, D, crate::transaction::ReadWrite> {
        crate::transaction::TxnGuard::write_sled(self.backend.db())
    }

    /// Execute a transaction on a single model tree (Sled-specific).
    ///
    /// This is Sled's native transaction API that provides ACID guarantees for operations
    /// on a single model type. The transaction closure may be called multiple times if
    /// there are conflicts.
    ///
    /// # Type Parameters
    ///
    /// * `M` - The model type to operate on
    /// * `F` - The transaction closure
    /// * `R` - The return type
    ///
    /// # Arguments
    ///
    /// * `f` - Transaction closure that performs operations on the transactional tree
    ///
    /// # Returns
    ///
    /// * `Ok(R)` - Transaction succeeded, returns result from closure
    /// * `Err(NetabaseError)` - Transaction failed (conflict, I/O error, etc.)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Atomic transfer between accounts
    /// store.transaction::<Account, _, _>(|txn_tree| {
    ///     let mut from = txn_tree.get(AccountPrimaryKey(1))?.unwrap();
    ///     let mut to = txn_tree.get(AccountPrimaryKey(2))?.unwrap();
    ///
    ///     from.balance -= 100;
    ///     to.balance += 100;
    ///
    ///     txn_tree.put(from)?;
    ///     txn_tree.put(to)?;
    ///
    ///     Ok(())
    /// })?;
    /// ```
    pub fn transaction<M, F, R>(&self, f: F) -> Result<R, NetabaseError>
    where
        M: NetabaseModelTrait<D> + TryFrom<D> + Into<D>,
        D: TryFrom<M>,
        F: Fn(&crate::databases::sled_store::SledTransactionalTree<D, M>) -> Result<R, Box<dyn std::error::Error>>,
    {
        self.backend.transaction(f)
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

    // TODO: Transaction API for Redb is still being optimized
    // The Sled backend has a working transaction API - see sled implementation above
}
