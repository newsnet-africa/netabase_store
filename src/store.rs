//! Unified store interface providing a single entry point for all storage backends.
//!
//! The `NetabaseStore<Backend>` wrapper provides a consistent API across different
//! storage backends (Sled, Redb, IndexedDB, Memory) while allowing backend-specific
//! functionality through specialized implementations.
//!
//! # Examples
//!
//! ```no_run
//! use netabase_store::{NetabaseStore, netabase_definition_module, NetabaseModel};
//! use netabase_store::traits::tree::NetabaseTreeSync;
//! use netabase_store::traits::model::NetabaseModelTrait;
//!
//! #[netabase_definition_module(MyDef, MyKeys)]
//! mod models {
//!     use netabase_store::{NetabaseModel, netabase};
//!     #[derive(NetabaseModel, Clone, Debug, PartialEq,
//!              bincode::Encode, bincode::Decode,
//!              serde::Serialize, serde::Deserialize)]
//!     #[netabase(MyDef)]
//!     pub struct User {
//!         #[primary_key]
//!         pub id: u64,
//!         pub name: String,
//!     }
//! }
//! use models::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a Sled-backed store
//! let temp_dir = tempfile::tempdir()?;
//! let store = NetabaseStore::sled(temp_dir.path().join("data"))?;
//! let tree = store.open_tree::<User>();
//! let user = User { id: 1, name: "Alice".to_string() };
//! tree.put(user)?;
//!
//! // Create a Redb-backed store
//! let store = NetabaseStore::redb(temp_dir.path().join("data.redb"))?;
//! let tree = store.open_tree::<User>();
//! # Ok(())
//! # }
//! ```

use crate::error::NetabaseError;
use crate::traits::definition::NetabaseDefinitionTrait;
use crate::traits::model::NetabaseModelTrait;
use crate::traits::store_ops::OpenTree;
use std::marker::PhantomData;
use std::path::Path;

/// Marker trait for backends that store a specific Definition type.
///
/// This trait is automatically implemented for all backend stores and binds
/// the Definition type to the backend at compile time.
pub trait BackendFor<D: NetabaseDefinitionTrait> {}

/// Trait for backends that can be constructed from a path.
///
/// This enables the generic `NetabaseStore::new()` constructor.
pub trait BackendConstructor<D: NetabaseDefinitionTrait>: BackendFor<D> + Sized {
    /// Create a new backend instance from a path.
    fn new_backend<P: AsRef<Path>>(path: P) -> Result<Self, NetabaseError>;
}

// Blanket implementation for all backend types that match the pattern
#[cfg(feature = "sled")]
impl<D> BackendFor<D> for crate::databases::sled_store::SledStore<D> where D: NetabaseDefinitionTrait
{}

#[cfg(feature = "sled")]
impl<D> BackendConstructor<D> for crate::databases::sled_store::SledStore<D>
where
    D: NetabaseDefinitionTrait + crate::traits::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant: crate::traits::definition::NetabaseDiscriminant,
{
    fn new_backend<P: AsRef<Path>>(path: P) -> Result<Self, NetabaseError> {
        crate::databases::sled_store::SledStore::new(path)
    }
}

#[cfg(feature = "redb")]
impl<D> BackendFor<D> for crate::databases::redb_store::RedbStore<D> where D: NetabaseDefinitionTrait
{}

#[cfg(feature = "redb")]
impl<D> BackendConstructor<D> for crate::databases::redb_store::RedbStore<D>
where
    D: NetabaseDefinitionTrait + crate::traits::convert::ToIVec,
    <D as strum::IntoDiscriminant>::Discriminant: crate::traits::definition::NetabaseDiscriminant,
{
    fn new_backend<P: AsRef<Path>>(path: P) -> Result<Self, NetabaseError> {
        crate::databases::redb_store::RedbStore::new(path)
    }
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
}

// Generic new() constructor for backends that support it
impl<D, Backend> NetabaseStore<D, Backend>
where
    D: NetabaseDefinitionTrait,
    Backend: BackendConstructor<D>,
{
    /// Create a new store with the specified backend at the given path.
    ///
    /// This generic constructor works with any backend that implements `BackendConstructor`.
    /// It's more flexible than backend-specific constructors like `sled()` or `redb()`.
    ///
    /// # Type Parameters
    ///
    /// You must specify both the Definition and Backend types using turbofish syntax:
    /// ```no_run
    /// # use netabase_store::{NetabaseStore, netabase_definition_module};
    /// # use netabase_store::databases::sled_store::SledStore;
    /// # use netabase_store::databases::redb_store::RedbStore;
    /// # #[netabase_definition_module(MyDef, MyKeys)]
    /// # mod models {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #     #[derive(NetabaseModel, Clone, Debug, PartialEq,
    /// #              bincode::Encode, bincode::Decode,
    /// #              serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDef)]
    /// #     pub struct User {
    /// #         #[primary_key]
    /// #         pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// # use models::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = NetabaseStore::<MyDef, SledStore<MyDef>>::new("./db")?;
    /// let store = NetabaseStore::<MyDef, RedbStore<MyDef>>::new("./db.redb")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the database file or directory
    ///
    /// # Examples
    ///
    /// ## Basic Usage with Sled Backend
    ///
    /// ```rust
    /// use netabase_store::{NetabaseStore, netabase_definition_module, NetabaseModel};
    /// use netabase_store::databases::sled_store::SledStore;
    /// use netabase_store::traits::tree::NetabaseTreeSync;
    /// use netabase_store::traits::model::NetabaseModelTrait;
    ///
    /// #[netabase_definition_module(MyDef, MyKeys)]
    /// mod my_models {
    ///     use netabase_store::{NetabaseModel, netabase};
    ///     #[derive(NetabaseModel, Clone, Debug, PartialEq,
    ///              bincode::Encode, bincode::Decode,
    ///              serde::Serialize, serde::Deserialize)]
    ///     #[netabase(MyDef)]
    ///     pub struct User {
    ///         #[primary_key]
    ///         pub id: u64,
    ///         pub name: String,
    ///     }
    /// }
    /// use my_models::*;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create store using generic constructor
    /// let temp_dir = tempfile::tempdir()?;
    /// let store = NetabaseStore::<MyDef, SledStore<MyDef>>::new(
    ///     temp_dir.path().join("test.db")
    /// )?;
    ///
    /// // Use the store normally
    /// let tree = store.open_tree::<User>();
    /// let user = User { id: 1, name: "Alice".to_string() };
    /// tree.put(user.clone())?;
    ///
    /// let retrieved = tree.get(user.primary_key())?.unwrap();
    /// assert_eq!(retrieved, user);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Using with Redb Backend
    ///
    /// ```rust
    /// use netabase_store::{NetabaseStore, netabase_definition_module, NetabaseModel};
    /// use netabase_store::databases::redb_store::RedbStore;
    /// use netabase_store::traits::tree::NetabaseTreeSync;
    /// use netabase_store::traits::model::NetabaseModelTrait;
    ///
    /// #[netabase_definition_module(MyDef, MyKeys)]
    /// mod my_models {
    ///     use netabase_store::{NetabaseModel, netabase};
    ///     #[derive(NetabaseModel, Clone, Debug, PartialEq,
    ///              bincode::Encode, bincode::Decode,
    ///              serde::Serialize, serde::Deserialize)]
    ///     #[netabase(MyDef)]
    ///     pub struct Post {
    ///         #[primary_key]
    ///         pub id: String,
    ///         pub title: String,
    ///     }
    /// }
    /// use my_models::*;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create Redb store using generic constructor
    /// let temp_dir = tempfile::tempdir()?;
    /// let store = NetabaseStore::<MyDef, RedbStore<MyDef>>::new(
    ///     temp_dir.path().join("test.redb")
    /// )?;
    ///
    /// let tree = store.open_tree::<Post>();
    /// let post = Post { id: "post-1".to_string(), title: "Hello".to_string() };
    /// tree.put(post.clone())?;
    ///
    /// let retrieved = tree.get(PostKey::Primary(post.primary_key()))?.unwrap();
    /// assert_eq!(retrieved, post);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Generic Function Example
    ///
    /// The generic constructor is useful for writing backend-agnostic code:
    ///
    /// ```rust
    /// use netabase_store::{NetabaseStore, netabase_definition_module, NetabaseModel};
    /// use netabase_store::store::BackendConstructor;
    /// use netabase_store::traits::definition::NetabaseDefinitionTrait;
    /// use netabase_store::traits::tree::NetabaseTreeSync;
    /// use netabase_store::traits::model::NetabaseModelTrait;
    ///
    /// #[netabase_definition_module(MyDef, MyKeys)]
    /// mod my_models {
    ///     use netabase_store::{NetabaseModel, netabase};
    ///     #[derive(NetabaseModel, Clone, Debug, PartialEq,
    ///              bincode::Encode, bincode::Decode,
    ///              serde::Serialize, serde::Deserialize)]
    ///     #[netabase(MyDef)]
    ///     pub struct Item {
    ///         #[primary_key]
    ///         pub id: u64,
    ///         pub data: String,
    ///     }
    /// }
    /// use my_models::*;
    ///
    /// // Example using generic new() method with concrete backend
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use netabase_store::databases::sled_store::SledStore;
    /// let temp_dir = tempfile::tempdir()?;
    /// let store = NetabaseStore::<MyDef, SledStore<MyDef>>::new(
    ///     temp_dir.path().join("test.db")
    /// )?;
    ///
    /// let tree = store.open_tree::<Item>();
    /// let item = Item { id: 1, data: "test".to_string() };
    /// tree.put(item.clone())?;
    ///
    /// let retrieved = tree.get(item.primary_key())?.unwrap();
    /// assert_eq!(retrieved, item);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, NetabaseError> {
        Ok(Self::from_backend(Backend::new_backend(path)?))
    }
}

impl<D, Backend> NetabaseStore<D, Backend>
where
    D: NetabaseDefinitionTrait,
    Backend: BackendFor<D>,
{
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
    /// ```no_run
    /// # use netabase_store::{NetabaseStore, netabase_definition_module};
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
    /// let store = NetabaseStore::sled("./db")?;
    /// let user_tree = store.open_tree::<User>();
    /// # Ok(())
    /// # }
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
    /// ```no_run
    /// # use netabase_store::{NetabaseStore, netabase_definition_module};
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
    /// #     }
    /// # }
    /// # use models::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = NetabaseStore::<MyDefinition, _>::sled("./my_database")?;
    /// # Ok(())
    /// # }
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
    /// ```no_run
    /// # use netabase_store::{NetabaseStore, netabase_definition_module};
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
    /// #     }
    /// # }
    /// # use models::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = NetabaseStore::<MyDefinition, _>::temp()?;
    /// # Ok(())
    /// # }
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
    /// ```no_run
    /// # use netabase_store::{NetabaseStore, netabase_definition_module};
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
    /// #     }
    /// # }
    /// # use models::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = NetabaseStore::<MyDefinition, _>::redb("./my_database.redb")?;
    /// # Ok(())
    /// # }
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
    /// ```
    /// # use netabase_store::{NetabaseStore, netabase_definition_module};
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
    /// #     }
    /// # }
    /// # use models::*;
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
    /// ```no_run
    /// # use netabase_store::{NetabaseStore, netabase_definition_module};
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
    /// # let store = NetabaseStore::<MyDefinition, _>::sled("./db")?;
    /// let mut txn = store.read();
    /// let tree = txn.open_tree::<User>();
    /// let user = tree.get(UserPrimaryKey(1))?;
    /// # Ok(())
    /// # }
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
    /// ```no_run
    /// # use netabase_store::{NetabaseStore, netabase_definition_module};
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
    /// # let store = NetabaseStore::sled("./db")?;
    /// # let user = User { id: 1, name: "Alice".to_string() };
    /// let mut txn = store.write();
    /// let mut tree = txn.open_tree::<User>();
    /// tree.put(user)?;
    /// txn.commit()?;
    /// # Ok(())
    /// # }
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
    /// ```no_run
    /// # use netabase_store::{NetabaseStore, netabase_definition_module};
    /// # #[netabase_definition_module(MyDefinition, MyKeys)]
    /// # mod models {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #     #[derive(NetabaseModel, Clone, Debug, PartialEq,
    /// #              bincode::Encode, bincode::Decode,
    /// #              serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDefinition)]
    /// #     pub struct Account {
    /// #         #[primary_key]
    /// #         pub id: u64,
    /// #         pub balance: i64,
    /// #     }
    /// # }
    /// # use models::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let store = NetabaseStore::sled("./db")?;
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
    /// # Ok(())
    /// # }
    /// ```
    pub fn transaction<M, F, R>(&self, f: F) -> Result<R, NetabaseError>
    where
        M: NetabaseModelTrait<D> + TryFrom<D> + Into<D>,
        D: TryFrom<M>,
        F: Fn(
            &crate::databases::sled_store::SledTransactionalTree<D, M>,
        ) -> Result<R, Box<dyn std::error::Error>>,
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

// Zero-copy Redb backend support
#[cfg(all(feature = "redb", feature = "redb-zerocopy"))]
impl<D> BackendFor<D> for crate::databases::redb_zerocopy::RedbStoreZeroCopy<D> where
    D: NetabaseDefinitionTrait
{
}

#[cfg(all(feature = "redb", feature = "redb-zerocopy"))]
impl<D> BackendConstructor<D> for crate::databases::redb_zerocopy::RedbStoreZeroCopy<D>
where
    D: NetabaseDefinitionTrait,
{
    fn new_backend<P: AsRef<Path>>(path: P) -> Result<Self, NetabaseError> {
        crate::databases::redb_zerocopy::RedbStoreZeroCopy::new(path)
    }
}

#[cfg(all(feature = "redb", feature = "redb-zerocopy"))]
impl<D> NetabaseStore<D, crate::databases::redb_zerocopy::RedbStoreZeroCopy<D>>
where
    D: NetabaseDefinitionTrait,
{
    /// Create a new zero-copy Redb-backed store at the given path.
    ///
    /// This constructor creates a store using the high-performance zero-copy redb backend.
    /// The zero-copy backend provides:
    /// - **Transaction batching**: Explicit write/read transactions for bulk operations
    /// - **Zero-copy reads**: Borrow data directly from database pages without allocation
    /// - **Performance**: Up to 10x faster bulk inserts, up to 54x faster secondary key queries
    ///
    /// # When to Use
    ///
    /// Use this backend when:
    /// - You need transaction batching (bulk operations)
    /// - Performance is critical
    /// - You want explicit transaction control
    ///
    /// Use the regular `redb()` constructor when:
    /// - Simplicity is more important than performance
    /// - Single-operation transactions are fine
    /// - You want the simplest possible API
    ///
    /// # Examples
    ///
    /// ```rust
    /// use netabase_store::{NetabaseStore, netabase_definition_module, NetabaseModel};
    /// use netabase_store::traits::model::NetabaseModelTrait;
    ///
    /// #[netabase_definition_module(MyDef, MyKeys)]
    /// mod models {
    ///     use netabase_store::{NetabaseModel, netabase};
    ///     #[derive(NetabaseModel, Clone, Debug, PartialEq,
    ///              bincode::Encode, bincode::Decode,
    ///              serde::Serialize, serde::Deserialize)]
    ///     #[netabase(MyDef)]
    ///     pub struct User {
    ///         #[primary_key]
    ///         pub id: u64,
    ///         pub name: String,
    ///         #[secondary_key]
    ///         pub email: String,
    ///     }
    /// }
    /// use models::*;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let temp_dir = tempfile::tempdir()?;
    /// let store = NetabaseStore::redb_zerocopy(temp_dir.path().join("app.redb"))?;
    ///
    /// // Write transaction - batched operations
    /// let mut txn = store.begin_write()?;
    /// let mut tree = txn.open_tree::<User>()?;
    /// tree.put(User { id: 1, name: "Alice".into(), email: "alice@example.com".into() })?;
    /// tree.put(User { id: 2, name: "Bob".into(), email: "bob@example.com".into() })?;
    /// drop(tree);
    /// txn.commit()?;  // Both inserts in one transaction
    ///
    /// // Read transaction - efficient reads
    /// let txn = store.begin_read()?;
    /// let tree = txn.open_tree::<User>()?;
    /// let user = tree.get(&UserPrimaryKey(1))?.unwrap();
    /// assert_eq!(user.name, "Alice");
    /// # Ok(())
    /// # }
    /// ```
    pub fn redb_zerocopy<P: AsRef<Path>>(path: P) -> Result<Self, NetabaseError> {
        Ok(Self::from_backend(
            crate::databases::redb_zerocopy::RedbStoreZeroCopy::new(path)?,
        ))
    }

    /// Open an existing zero-copy Redb-backed store at the given path.
    ///
    /// Unlike `redb_zerocopy()` which removes any existing database,
    /// this method opens an existing database or creates it if it doesn't exist.
    pub fn open_redb_zerocopy<P: AsRef<Path>>(path: P) -> Result<Self, NetabaseError> {
        Ok(Self::from_backend(
            crate::databases::redb_zerocopy::RedbStoreZeroCopy::open(path)?,
        ))
    }

    /// Begin a write transaction (zero-copy redb specific).
    ///
    /// Write transactions are exclusive - only one can be active at a time.
    /// The transaction must be explicitly committed or aborted.
    ///
    /// Multiple operations can be batched in a single transaction for better performance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use netabase_store::{NetabaseStore, netabase_definition_module, NetabaseModel};
    /// # #[netabase_definition_module(MyDef, MyKeys)]
    /// # mod models {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #     #[derive(NetabaseModel, Clone, Debug, PartialEq,
    /// #              bincode::Encode, bincode::Decode,
    /// #              serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDef)]
    /// #     pub struct User {
    /// #         #[primary_key]
    /// #         pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// # use models::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let temp_dir = tempfile::tempdir()?;
    /// # let store = NetabaseStore::redb_zerocopy(temp_dir.path().join("test.redb"))?;
    /// // Batch multiple writes in one transaction
    /// let mut txn = store.begin_write()?;
    /// let mut tree = txn.open_tree::<User>()?;
    /// for i in 0..1000 {
    ///     tree.put(User { id: i, name: format!("User {}", i) })?;
    /// }
    /// drop(tree);
    /// txn.commit()?;  // All 1000 inserts in one transaction!
    /// # Ok(())
    /// # }
    /// ```
    pub fn begin_write(
        &self,
    ) -> Result<crate::databases::redb_zerocopy::RedbWriteTransactionZC<'_, D>, NetabaseError> {
        self.backend.begin_write()
    }

    /// Begin a read transaction (zero-copy redb specific).
    ///
    /// Read transactions provide a consistent snapshot of the database.
    /// Multiple read transactions can be active concurrently.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use netabase_store::{NetabaseStore, netabase_definition_module, NetabaseModel};
    /// # #[netabase_definition_module(MyDef, MyKeys)]
    /// # mod models {
    /// #     use netabase_store::{NetabaseModel, netabase};
    /// #     #[derive(NetabaseModel, Clone, Debug, PartialEq,
    /// #              bincode::Encode, bincode::Decode,
    /// #              serde::Serialize, serde::Deserialize)]
    /// #     #[netabase(MyDef)]
    /// #     pub struct User {
    /// #         #[primary_key]
    /// #         pub id: u64,
    /// #         pub name: String,
    /// #     }
    /// # }
    /// # use models::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let temp_dir = tempfile::tempdir()?;
    /// # let store = NetabaseStore::redb_zerocopy(temp_dir.path().join("test.redb"))?;
    /// # let mut txn = store.begin_write()?;
    /// # let mut tree = txn.open_tree::<User>()?;
    /// # tree.put(User { id: 1, name: "Alice".into() })?;
    /// # drop(tree); txn.commit()?;
    /// let txn = store.begin_read()?;
    /// let tree = txn.open_tree::<User>()?;
    /// let user = tree.get(&UserPrimaryKey(1))?;
    /// assert!(user.is_some());
    /// # Ok(())
    /// # }
    /// ```
    pub fn begin_read(
        &self,
    ) -> Result<crate::databases::redb_zerocopy::RedbReadTransactionZC<'_, D>, NetabaseError> {
        self.backend.begin_read()
    }

    /// Insert a single model with auto-commit (convenience method).
    ///
    /// This is equivalent to `begin_write() -> open_tree() -> put() -> commit()`.
    /// Use `begin_write()` directly for better performance when inserting multiple items.
    pub fn quick_put<M>(&self, model: M) -> Result<(), NetabaseError>
    where
        M: NetabaseModelTrait<D>,
        M::Keys: crate::traits::model::NetabaseModelTraitKey<D>,
    {
        self.backend.quick_put(model)
    }

    /// Get a single model (cloned) with auto-transaction (convenience method).
    ///
    /// This is equivalent to `begin_read() -> open_tree() -> get()`.
    /// Use `begin_read()` directly for better performance when reading multiple items.
    pub fn quick_get<M>(
        &self,
        key: &<M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey,
    ) -> Result<Option<M>, NetabaseError>
    where
        M: NetabaseModelTrait<D>,
        M::Keys: crate::traits::model::NetabaseModelTraitKey<D>,
    {
        self.backend.quick_get(key)
    }

    /// Remove a single model with auto-commit (convenience method).
    ///
    /// This is equivalent to `begin_write() -> open_tree() -> remove() -> commit()`.
    /// Use `begin_write()` directly for better performance when removing multiple items.
    pub fn quick_remove<M>(
        &self,
        key: &<M::Keys as crate::traits::model::NetabaseModelTraitKey<D>>::PrimaryKey,
    ) -> Result<Option<M>, NetabaseError>
    where
        M: NetabaseModelTrait<D>,
        M::Keys: crate::traits::model::NetabaseModelTraitKey<D>,
    {
        self.backend.quick_remove(key)
    }
}
