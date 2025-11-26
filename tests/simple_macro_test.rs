//! Minimal test to debug macro generation issues

#![cfg(feature = "redb")]

use netabase_store::{NetabaseModel, netabase, netabase_definition_module};

#[netabase_definition_module(SimpleDef, SimpleKeys)]
mod simple_models {
    use super::*;

    #[derive(
        NetabaseModel,
        Debug,
        Clone,
        PartialEq,
        Eq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(SimpleDef)]
    pub struct SimpleUser {
        #[primary_key]
        pub id: u64,
        pub name: String,
    }
}

use simple_models::*;

#[test]
fn test_simple() {
    // Just test that the types exist and are usable
    let user = SimpleUser {
        id: 1,
        name: "test".into(),
    };
    assert_eq!(user.id, 1);
    assert_eq!(user.name, "test");
}
