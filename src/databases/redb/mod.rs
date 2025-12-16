pub mod transaction;

use crate::errors::{NetabaseError, NetabaseResult};
pub use crate::permissions::{NetabasePermissions, RedbPermissions};
use crate::traits::registery::definition::redb_definition::RedbDefinition;
use std::path::Path;
use std::sync::Arc;
use strum::EnumDiscriminants;

pub struct RedbStore<D: RedbDefinition>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    _tree_names: D::TreeNames,
    db: Arc<RedbStorePermissions>,
    permissions: RedbPermissions<D>,
}

#[derive(EnumDiscriminants)]
#[strum_discriminants(name(DefinitionPermissions))]
pub enum RedbStorePermissions {
    ReadOnly(redb::ReadOnlyDatabase),
    ReadWrite(redb::Database),
}

impl<D: RedbDefinition> RedbStore<D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    /// Create a new RedbStore with the given permissions
    /// Always creates read-write database for now
    pub fn new<P: AsRef<Path>>(path: P, permissions: RedbPermissions<D>) -> NetabaseResult<Self>
    where
        D::TreeNames: Default,
    {
        // For now, always create read-write database
        // TODO: Implement read-only mode based on permissions
        let db = redb::Database::create(path).map_err(|e| NetabaseError::RedbError(e.into()))?;
        let db = Arc::new(RedbStorePermissions::ReadWrite(db));

        Ok(Self {
            _tree_names: Default::default(),
            db,
            permissions,
        })
    }

    /// Begin a transaction on the database
    pub fn begin_transaction(&self) -> NetabaseResult<transaction::RedbTransaction<'_, D>> {
        transaction::RedbTransaction::new(&self.db, self.permissions.clone())
    }
}
