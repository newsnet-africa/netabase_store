//! RedbStore implementation
//!
//! This module contains the main store struct and its core implementations.

use crate::config::FileConfig;
use crate::error::NetabaseError;
use crate::traits::backend_store::{BackendStore, PathBasedBackend};
use crate::traits::convert::ToIVec;
use crate::traits::definition::NetabaseDefinitionTrait;

use redb::Database;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;
use strum::{IntoDiscriminant, IntoEnumIterator};

use super::tree::RedbStoreTree;

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
    _phantom: PhantomData<D>,
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
    #[allow(dead_code)]
    pub(crate) fn db_arc(&self) -> &Arc<Database> {
        &self.db
    }

    /// Get access to the table definitions struct
    ///
    /// This provides access to all redb TableDefinitions for models in this schema.
    /// The returned value can be used to open tables within transactions.
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
            _phantom: PhantomData,
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
            _phantom: PhantomData,
        })
    }

    /// Open a tree for a specific model type
    ///
    /// This creates a tree abstraction that wraps redb table operations.
    /// Stores models directly without Definition enum wrapping for optimal performance.
    pub fn open_tree<M>(&self) -> RedbStoreTree<'_, D, M>
    where
        M: crate::traits::model::NetabaseModelTrait<D> + std::fmt::Debug + bincode::Decode<()>,
        M::Keys: std::fmt::Debug + bincode::Decode<()> + Ord + PartialEq,
    {
        RedbStoreTree::new(Arc::clone(&self.db), M::DISCRIMINANT)
    }

    /// Get all tree names (discriminants) in the database
    pub fn tree_names(&self) -> Vec<D::Discriminant> {
        D::Discriminant::iter().collect()
    }

    /// Check database integrity
    pub fn check_integrity(&mut self) -> Result<bool, NetabaseError> {
        let db = Arc::get_mut(&mut self.db).ok_or_else(|| {
            NetabaseError::Storage(
                "Cannot check integrity: database has multiple references".to_string(),
            )
        })?;
        Ok(db.check_integrity()?)
    }

    /// Compact the database to reclaim space
    pub fn compact(&mut self) -> Result<bool, NetabaseError> {
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
            _phantom: PhantomData,
        })
    }

    fn open(config: Self::Config) -> Result<Self, NetabaseError> {
        let db = Database::open(&config.path)?;
        Ok(Self {
            db: Arc::new(db),
            #[cfg(feature = "redb")]
            tables: D::tables(),
            trees: D::Discriminant::iter().collect(),
            _phantom: PhantomData,
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
