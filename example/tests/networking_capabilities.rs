//! Test that the networking capabilities are generated and work correctly

use example::*;
use example::main_repository::definition::gen_definition_capabilities::*;

#[test]
fn test_definition_two_capabilities_generated() {
    // The generated module should exist
    use example::main_repository::definition_two::gen_definition_two_capabilities::*;
    
    // GeneralCapabilities struct should exist for the General subscription
    // Since Category is subscribed to General, it should have a category_capability field
    let _caps: GeneralCapabilities<DefinitionTwo> = GeneralCapabilities {
        category_capability: netabase::capabilities::Capability::new_read(),
    };
}

#[test]
fn test_definition_capabilities_generated() {
    // The generated module should exist
    use example::main_repository::definition::gen_definition_capabilities::*;
    
    // The definition should implement NetworkDefinition
    fn assert_network_definition<D: netabase::data::store::network::NetworkDefinition>() 
    where <D as netabase_store::strum::IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static {}
    assert_network_definition::<Definition>();
    
    // The DefinitionCapabilities struct should exist
    let _caps: DefinitionCapabilities = DefinitionCapabilities {
        user_capabilities: vec![],
        user_v1_capabilities: vec![],
        post_capabilities: vec![],
        post_v1_capabilities: vec![],
        heavy_model_capabilities: vec![],
        immutable_post_capabilities: vec![],
    };
}

#[test]
fn test_topic_capabilities_generated() {
    use example::main_repository::definition::gen_definition_capabilities::*;
    
    // Topic1 capabilities should include User, UserV1, ImmutablePost, and HeavyModel
    let _topic1_caps: Topic1Capabilities<Definition> = Topic1Capabilities {
        user_capability: netabase::capabilities::Capability::new_read(),
        user_v1_capability: netabase::capabilities::Capability::new_read(),
        immutable_post_capability: netabase::capabilities::Capability::new_read(),
        heavy_model_capability: netabase::capabilities::Capability::new_read(),
    };
    
    // Topic2 capabilities should include User, UserV1, ImmutablePost, and HeavyModel
    let _topic2_caps: Topic2Capabilities<Definition> = Topic2Capabilities {
        user_capability: netabase::capabilities::Capability::new_read(),
        user_v1_capability: netabase::capabilities::Capability::new_read(),
        immutable_post_capability: netabase::capabilities::Capability::new_read(),
        heavy_model_capability: netabase::capabilities::Capability::new_read(),
    };
    
    // Topic3 capabilities should include Post, PostV1, and HeavyModel
    let _topic3_caps: Topic3Capabilities<Definition> = Topic3Capabilities {
        post_capability: netabase::capabilities::Capability::new_read(),
        post_v1_capability: netabase::capabilities::Capability::new_read(),
        heavy_model_capability: netabase::capabilities::Capability::new_read(),
    };
    
    // Topic4 capabilities should include Post, PostV1, and HeavyModel
    let _topic4_caps: Topic4Capabilities<Definition> = Topic4Capabilities {
        post_capability: netabase::capabilities::Capability::new_read(),
        post_v1_capability: netabase::capabilities::Capability::new_read(),
        heavy_model_capability: netabase::capabilities::Capability::new_read(),
    };
}

#[test]
fn test_capability_access_modes() {
    let read_cap = netabase::capabilities::Capability::<Definition, User>::new_read();
    assert!(read_cap.can_read());
    assert!(!read_cap.can_write());
    
    let write_cap = netabase::capabilities::Capability::<Definition, User>::new_read_write();
    assert!(write_cap.can_read());
    assert!(write_cap.can_write());
}

#[test]
fn test_capability_serialization() {
    use netabase::capabilities::Capability;
    
    let cap = Capability::<Definition, User>::new_read_write();
    
    // Capabilities should be serializable/deserializable
    let json = serde_json::to_string(&cap).unwrap();
    let deserialized: Capability<Definition, User> = serde_json::from_str(&json).unwrap();
    
    assert_eq!(cap, deserialized);
}
