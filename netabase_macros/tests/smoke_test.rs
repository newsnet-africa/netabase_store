use netabase_macros::{netabase_definition_module, NetabaseModel};
use netabase_store::databases::redb_store::traits::*;
use netabase_store::databases::sled_store::traits::*;

#[netabase_definition_module(MyDefinition, MyDefinitionKeys)]
pub mod my_definition {
    use super::*;

    #[derive(NetabaseModel)]
    pub struct MyModel {
        #[primary_key]
        pub id: u64,
        #[secondary_key]
        pub email: String,
    }
}

#[test]
fn test_smoke() {
    let _ = MyDefinition::MyModel(MyModel { id: 1, email: "test".to_string() });
}
