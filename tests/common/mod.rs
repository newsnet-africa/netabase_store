// Common test utilities and helpers

use std::path::PathBuf;
use netabase_store::databases::redb::{RedbStore, RedbPermissions};
use netabase_store::errors::NetabaseResult;

/// Create a temporary database for testing
pub fn create_test_db<D>(name: &str) -> NetabaseResult<(RedbStore<D>, PathBuf)>
where
    D: netabase_store::traits::registery::definition::redb_definition::RedbDefinition + Clone,
    D::TreeNames: Default,
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    let db_path = PathBuf::from(format!("/tmp/netabase_test_{}.redb", name));

    // Clean up any existing database
    if db_path.exists() {
        std::fs::remove_file(&db_path).ok();
    }

    let permissions = RedbPermissions::<D>::allow_all();
    let store = RedbStore::<D>::new(&db_path, permissions)?;

    Ok((store, db_path))
}

/// Clean up test database
pub fn cleanup_test_db(path: PathBuf) {
    std::fs::remove_file(&path).ok();
}
