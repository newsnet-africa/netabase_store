use serde::{Deserialize, Serialize};
use netabase_store::traits::registery::definition::NetabaseDefinition;

#[netabase_macros::netabase_definition(Libp2pTestDef)]
pub mod libp2p_test_def {
    use super::*;

    #[derive(
        netabase_macros::NetabaseModel,
        Debug,
        Clone,
        Serialize,
        Deserialize,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
    )]
    #[netabase_libp2p]
    pub struct Libp2pModel {
        #[primary_key]
        pub id: String,
        pub data: String,
    }

    #[derive(
        netabase_macros::NetabaseModel,
        Debug,
        Clone,
        Serialize,
        Deserialize,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
    )]
    pub struct NormalModel {
        #[primary_key]
        pub id: String,
        pub data: String,
    }
}

#[test]
fn test_libp2p_schema_generation() {
    use libp2p_test_def::*;
    
    let schema = Libp2pTestDef::schema();
    let toml = Libp2pTestDef::export_toml();
    
    println!("Generated TOML:\n{}", toml);

    // Check internal schema structure
    let libp2p_model = schema.models.iter().find(|m| m.name == "Libp2pModel").expect("Libp2pModel not found");
    assert!(libp2p_model.is_libp2p_enabled, "Libp2pModel should have is_libp2p_enabled = true");
    
    let normal_model = schema.models.iter().find(|m| m.name == "NormalModel").expect("NormalModel not found");
    assert!(!normal_model.is_libp2p_enabled, "NormalModel should have is_libp2p_enabled = false");
    
    // Check TOML output
    // Note: Serde might not serialize false by default if we used #[serde(default)] without specific setting, 
    // but looking at schema.rs we just have #[serde(default)], so it will be false by default on read, 
    // but on write it depends on skip_serializing_if. We didn't add skip_serializing_if.
    // So false should be present or implied. 
    // However, we definitely expect "is_libp2p_enabled = true" for the enabled one.
    assert!(toml.contains("is_libp2p_enabled = true"), "TOML should contain is_libp2p_enabled = true for Libp2pModel");
}

#[test]
fn test_record_conversion() {
    use libp2p_test_def::*;
    use netabase_store::traits::libp2p::libp2p_model::Libp2pMetadata;
    use std::time::SystemTime;

    let model = Libp2pModel {
        id: Libp2pModelID("test_id".to_string()),
        data: "some data".to_string(),
        libp2p_metadata: Some(Libp2pMetadata {
            publisher: None,
            expires: Some(SystemTime::now() + std::time::Duration::from_secs(3600)),
            extra: None,
        }),
    };

    // Convert to RecordWrapper (effectively (Def, Meta))
    // Note: The struct name is Libp2pTestDefRecord
    let wrapper: Libp2pTestDefRecord = model.clone().into();
    
    // Check wrapper contents
    // wrapper.1 is metadata, no longer has key field.
    
    // Convert to libp2p Record
    let record: netabase_store::libp2p::kad::Record = wrapper.into();
    
    // The key should be derived from the model ID "test_id"
    // The ID is Libp2pModelID("test_id")
    // It is serialized using postcard.
    let expected_key = netabase_store::postcard::to_allocvec(&model.id).unwrap();
    assert_eq!(record.key.as_ref(), expected_key);
    assert!(record.expires.is_some());
}