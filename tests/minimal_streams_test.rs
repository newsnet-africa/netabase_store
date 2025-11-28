//! Minimal test case for streams functionality

use netabase_store::{
    NetabaseModel, netabase, netabase_definition_module, streams,
    traits::subscription::{SubscriptionManager, SubscriptionTree, Subscriptions},
};

#[netabase_definition_module(TestDef, TestKeys)]
#[streams(Topic1)]
mod test_module {
    use super::*;

    #[derive(
        NetabaseModel,
        bincode::Encode,
        bincode::Decode,
        Clone,
        Debug,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(TestDef)]
    pub struct SimpleModel {
        #[primary_key]
        pub id: u64,
        pub name: String,
    }
}

use test_module::*;

#[test]
fn test_basic_compilation() {
    // Test basic module compilation
    let model = SimpleModel {
        id: 1,
        name: "test".to_string(),
    };
    let _def = TestDef::SimpleModel(model);
}

#[test]
fn test_basic_store() {
    // Test basic store functionality without streams for now
    println!("Model compiled successfully");
}

#[test]
fn test_streams_compilation() {
    // Test that the streams macro generates the expected types
    let _topic = TestDefSubscriptions::Topic1;
    let mut _manager = TestDefSubscriptionManager::new();

    // Test that subscription functionality works
    let mut tree = Topic1SubscriptionTree::new();
    assert_eq!(tree.len(), 0);

    // Test that the manager can be used
    let stats = _manager.stats();
    assert_eq!(stats.total_items, 0);
}
