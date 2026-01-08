// Common test utilities and helpers

use netabase_store::databases::redb::RedbStore;
use netabase_store::errors::NetabaseResult;
pub use netabase_store::traits::database::store::NBStore;
use strum::IntoDiscriminant;

/// Create an in-memory database for benchmarking
/// 
/// This uses redb's InMemoryBackend to avoid disk I/O overhead,
/// measuring only the abstraction overhead, not I/O performance.
pub fn create_test_db<D>(_name: &str) -> NetabaseResult<RedbStore<D>>
where
    D: netabase_store::traits::registery::definition::redb_definition::RedbDefinition + Clone,
    D::TreeNames: Default,
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <D as IntoDiscriminant>::Discriminant: PartialEq,
{
    RedbStore::<D>::new_in_memory()
}
