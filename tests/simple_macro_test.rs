//! Minimal test to debug macro generation issues

#![cfg(all(feature = "redb", not(feature = "paxos")))]

use netabase_store::{netabase, netabase_definition_module, NetabaseModel};

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

#[test]
fn test_simple() {
    // Just test that the types exist
    let _ = simple_models::SimpleUser {
        id: 1,
        name: "test".into(),
    };
}
