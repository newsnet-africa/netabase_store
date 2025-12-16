pub mod transaction;

use crate::traits::registery::definition::redb_definition::RedbDefinition;
pub use crate::permissions::{NetabasePermissions, RedbPermissions};
use strum::EnumDiscriminants;

pub struct RedbStore<D: RedbDefinition>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    tree_names: D::TreeNames,
    db: RedbStorePermissions,
}

#[derive(EnumDiscriminants)]
#[strum_discriminants(name(DefinitionPermissions))]
pub enum RedbStorePermissions {
    ReadOnly(redb::ReadOnlyDatabase),
    ReadWrite(redb::Database),
}
