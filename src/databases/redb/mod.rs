pub mod transaction;

use crate::errors::{NetabaseError, NetabaseResult};
pub use crate::permissions::{NetabasePermissions, RedbPermissions};
use crate::traits::registery::definition::redb_definition::RedbDefinition;
use std::path::Path;
use std::sync::Arc;
use strum::{EnumDiscriminants, IntoDiscriminant};

pub struct RedbStore<D: RedbDefinition>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    _tree_names: D::TreeNames,
    db: Arc<RedbStorePermissions>,
    permissions: RedbPermissions<D>,
}

pub enum RedbStorePermissions {
    ReadOnly(redb::ReadOnlyDatabase),
    ReadWrite(redb::Database),
}

impl<D: RedbDefinition> RedbStore<D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    /// Begin a transaction on the database
    pub fn begin_transaction(&self) -> NetabaseResult<transaction::RedbTransaction<'_, D>> {
        transaction::RedbTransaction::new(&self.db, self.permissions.clone())
    }
}

use crate::traits::{database::store::NBStore, permissions::DefinitionPermissions};

impl<D: RedbDefinition> NBStore<D> for RedbStore<D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug + PartialEq,
    <D as IntoDiscriminant>::Discriminant: PartialEq,
    D: Clone,
{
    /// Create a new RedbStore with the given permissions
    /// Opens database as read-only or read-write based on permissions
    fn new<P: AsRef<Path>>(
        path: P,
        def_permissions: DefinitionPermissions<'static, D>,
    ) -> NetabaseResult<Self>
    where
        D::TreeNames: Default,
    {
        // Determine if we need write access based on permissions
        let needs_write = def_permissions.requires_write_access();

        let db = if needs_write {
            let db =
                redb::Database::create(path).map_err(|e| NetabaseError::RedbError(e.into()))?;
            Arc::new(RedbStorePermissions::ReadWrite(db))
        } else {
            // For read-only, attempt to open existing database
            let db = redb::ReadOnlyDatabase::open(path)
                .map_err(|e| NetabaseError::RedbError(e.into()))?;
            Arc::new(RedbStorePermissions::ReadOnly(db))
        };

        // Convert DefinitionPermissions to RedbPermissions (NetabasePermissions)
        // For now, we'll use a simple conversion - create allow_all since the real
        // permission checking happens at the definition level
        let permissions = RedbPermissions::allow_all();

        Ok(Self {
            _tree_names: Default::default(),
            db,
            permissions,
        })
    }

    fn execute_transaction<F: Fn()>(f: F) {
        f()
    }
}
