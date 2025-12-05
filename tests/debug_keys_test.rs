//! Very basic test to understand TestKeys structure

#![cfg(all(feature = "redb", not(feature = "paxos")))]

use netabase_store::{NetabaseModel, netabase, netabase_definition_module};

// Test definition and models
#[netabase_definition_module(TestDef, TestKeys)]
mod test_models {
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
    #[netabase(TestDef)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub name: String,
    }
}

use test_models::*;

#[test]
fn test_key_structure() {
    // Let's see what TestKeys actually looks like
    let _user_primary = UserPrimaryKey(1);
    
    // Try to understand the TestKeys enum structure
    // This test will help us understand what variants exist
    println!("User primary key created successfully");
}