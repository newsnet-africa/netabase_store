//! Basic tests for the new relation architecture
//!
//! This tests the type-safe relation system with generated relation enums

use netabase_store::{
    links::RelationalLink,
    netabase_definition_module,
    traits::{
        model::NetabaseModelTrait,
        relation::{NetabaseRelationDiscriminant, NetabaseRelationTrait},
    },
};

#[netabase_definition_module(TestDefinition, TestKeys)]
mod test_models {
    use super::*;
    use netabase_store::{NetabaseModel, netabase};

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(TestDefinition)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub name: String,
    }

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(TestDefinition)]
    pub struct Post {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub author: RelationalLink<TestDefinition, User, PostRelations>,
    }
}

use test_models::*;

/*
#[test]
fn test_relational_link_creation() {
    let user = User {
        id: 1,
        name: "Alice".to_string(),
    };

    // Test creating links with entities
    let author_link: RelationalLink<TestDefinition, User, PostRelations> =
        RelationalLink::from_entity(user.clone());

    // Test creating links with references
    let author_ref: RelationalLink<TestDefinition, User, PostRelations> =
        RelationalLink::from_key(UserPrimaryKey(1u64));

    // Verify the links work as expected
    assert!(author_link.is_entity());
    assert!(!author_link.is_reference());
    assert_eq!(author_link.key(), UserPrimaryKey(1u64));

    assert!(!author_ref.is_entity());
    assert!(author_ref.is_reference());
    assert_eq!(author_ref.key(), UserPrimaryKey(1u64));
}

#[test]
fn test_relational_link_basic_methods() {
    let user = User {
        id: 42,
        name: "Bob".to_string(),
    };

    // Test From trait
    let link: RelationalLink<TestDefinition, User, PostRelations> = user.clone().into();
    assert!(link.is_entity());
    assert_eq!(link.key(), UserPrimaryKey(42u64));

    // Test converting to reference
    let ref_link = link.to_reference();
    assert!(ref_link.is_reference());
    assert_eq!(ref_link.key(), UserPrimaryKey(42u64));
}

#[test]
fn test_relation_discriminant_basic() {
    // Test that the generated relation enums work
    let all_variants = PostRelations::all_variants();
    assert_eq!(all_variants.len(), 1); // Only Author

    // Test field names
    let author_variant = PostRelations::Author;
    assert_eq!(author_variant.field_name(), "author");

    // Test target model names
    assert_eq!(author_variant.target_model_name(), "User");
}

#[test]
fn test_post_has_relations() {
    let user = User {
        id: 1,
        name: "Alice".to_string(),
    };

    let post = Post {
        id: 1,
        title: "Test Post".to_string(),
        author: RelationalLink::from_entity(user),
    };

    // Test that the model implements NetabaseRelationTrait
    assert!(post.has_relations());

    let relations = post.relations();
    assert_eq!(relations.len(), 1);

    // Verify the relations map contains the expected discriminants
    assert!(relations.contains_key(&PostRelations::Author.into()));
}

#[test]
fn test_simple_serialization() {
    let user = User {
        id: 1,
        name: "Alice".to_string(),
    };

    // Test serializing entity link
    let entity_link: RelationalLink<TestDefinition, User, PostRelations> =
        RelationalLink::from_entity(user.clone());
    let encoded = bincode::encode_to_vec(&entity_link, bincode::config::standard()).unwrap();
    let decoded: RelationalLink<TestDefinition, User, PostRelations> =
        bincode::decode_from_slice(&encoded, bincode::config::standard())
            .unwrap()
            .0;

    assert!(decoded.is_entity());
    assert_eq!(decoded.key(), UserPrimaryKey(1u64));

    // Test serializing reference link
    let ref_link: RelationalLink<TestDefinition, User, PostRelations> =
        RelationalLink::from_key(UserPrimaryKey(42u64));
    let encoded = bincode::encode_to_vec(&ref_link, bincode::config::standard()).unwrap();
    let decoded: RelationalLink<TestDefinition, User, PostRelations> =
        bincode::decode_from_slice(&encoded, bincode::config::standard())
            .unwrap()
            .0;

    assert!(decoded.is_reference());
    assert_eq!(decoded.key(), UserPrimaryKey(42u64));
}
*/

#[test]
fn test_model_without_relations() {
    // Test that models without relations work normally
    let user = User {
        id: 1,
        name: "Alice".to_string(),
    };

    // User doesn't have relations, so this should be fine
    assert_eq!(user.primary_key(), UserPrimaryKey(1u64));
    assert_eq!(User::discriminant_name(), "User");
}
