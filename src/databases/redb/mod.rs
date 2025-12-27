pub mod transaction;

use crate::errors::{NetabaseError, NetabaseResult};
use crate::traits::registery::definition::redb_definition::RedbDefinition;
use std::path::Path;
use std::sync::Arc;
use strum::IntoDiscriminant;

pub struct RedbStore<D: RedbDefinition>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    _tree_names: D::TreeNames,
    db: Arc<redb::Database>,
}

impl<D: RedbDefinition> RedbStore<D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    /// Begin a transaction on the database
    pub fn begin_transaction(&self) -> NetabaseResult<transaction::RedbTransaction<'_, D>> {
        transaction::RedbTransaction::new(&self.db)
    }
}

use crate::traits::database::store::NBStore;

impl<D: RedbDefinition> NBStore<D> for RedbStore<D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug + PartialEq,
    <D as IntoDiscriminant>::Discriminant: PartialEq,
    D: Clone,
{
    /// Create a new RedbStore
    fn new<P: AsRef<Path>>(path: P) -> NetabaseResult<Self>
    where
        D::TreeNames: Default,
    {
        let path_ref = path.as_ref();
        let db =
            redb::Database::create(path_ref).map_err(|e| NetabaseError::RedbError(e.into()))?;

        let schema_path = path_ref.join(".netabase_schema.toml");
        let toml = D::export_toml();

        std::fs::write(&schema_path, &toml);

        println!(
            "Written toml to path: {:?}.\n\tToml File: {:?}",
            schema_path, toml
        );

        Ok(Self {
            _tree_names: Default::default(),
            db: Arc::new(db),
        })
    }

    fn execute_transaction<F: Fn()>(f: F) {
        f()
    }
}
