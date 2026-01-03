// Common test utilities and helpers

use netabase_store::databases::redb::RedbStore;
use netabase_store::errors::NetabaseResult;
pub use netabase_store::traits::database::store::NBStore;
use std::path::PathBuf;
use strum::IntoDiscriminant;

/// Create a temporary database for testing
pub fn create_test_db<D>(name: &str) -> NetabaseResult<(RedbStore<D>, PathBuf)>
where
    D: netabase_store::traits::registery::definition::redb_definition::RedbDefinition + Clone,
    D::TreeNames: Default,
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <D as IntoDiscriminant>::Discriminant: PartialEq,
{
    let db_path = PathBuf::from(format!("/tmp/netabase_test_{}.redb", name));

    // Clean up any existing database
    if db_path.exists() {
        std::fs::remove_file(&db_path).ok();
    }

    let store = RedbStore::<D>::new(&db_path)?;

    Ok((store, db_path))
}

/// Clean up test database
#[allow(dead_code)]
pub fn cleanup_test_db(path: PathBuf) {
    std::fs::remove_file(&path).ok();
}
