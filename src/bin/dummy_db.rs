use netabase_macros::infer_netabase_definition;
use netabase_store::{prelude::RedbStore, traits::database::store::NBStore};

infer_netabase_definition!("src/bin/simple_repo.toml");
use SimpleDefinitionModule::*;

fn main() {
    let store = RedbStore::<SimpleDefinition>::new("./tmp/dummy_db");
}
