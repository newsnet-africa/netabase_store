pub mod transaction;

use strum::EnumDiscriminants;
use crate::traits::registery::definition::redb_definition::RedbDefinition;
pub use crate::permissions::{NetabasePermissions, ModelOperationPermission, TablePermissionLevel};

pub struct RedbStore<D: RedbDefinition>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    tree_names: D::TreeNames,
    db: RedbStorePermissions,
}

#[derive(Debug, Clone)]
pub enum RedbPermissions<D: RedbDefinition> 
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    Database {
        inner: NetabasePermissions,
        tables: Vec<D::ModelTableDefinition<'static>>,
    },
    Model {
        inner: NetabasePermissions,
        table: D::ModelTableDefinition<'static>,
    },
}

impl<D: RedbDefinition> RedbPermissions<D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    pub fn inner(&self) -> &NetabasePermissions {
        match self {
            Self::Database { inner, .. } => inner,
            Self::Model { inner, .. } => inner,
        }
    }

    pub fn can_perform_operation(&self, operation: &ModelOperationPermission) -> bool {
        self.inner().can_perform_operation(operation)
    }

    pub fn can_write(&self) -> bool {
        self.inner().can_write()
    }
}

#[derive(EnumDiscriminants)]
#[strum_discriminants(name(DefinitionPermissions))]
pub enum RedbStorePermissions {
    ReadOnly(redb::ReadOnlyDatabase),
    ReadWrite(redb::Database),
}