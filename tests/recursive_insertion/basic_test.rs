//! Basic test for recursive relation insertion functionality

use netabase_store::{
    links::{RecursionLevel, RelationalLink},
    netabase_definition_module,
    store::{NetabaseStore, RelationalLinksNetabaseStore},
    traits::{model::NetabaseModelTrait, tree::NetabaseTreeSync},
};

#[netabase_definition_module(TestDef, TestKeys)]
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
    #[netabase(TestDef)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub name: String,
        pub email: String,
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
    #[netabase(TestDef)]
    pub struct Post {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,
        #[relation(author)]
        pub author: RelationalLink<TestDef, User>,
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
    #[netabase(TestDef)]
    pub struct Comment {
        #[primary_key]
        pub id: u64,
        pub text: String,
        #[relation(post)]
        pub post: RelationalLink<TestDef, Post>,
        #[relation(author)]
        pub author: RelationalLink<TestDef, User>,
    }
}

use test_models::*;

#[test]
fn test_basic_recursive_insertion() {
    // Create a temporary store
    let store = NetabaseStore::<TestDef, _>::temp().expect("Failed to create temp store");

    // Create test data with nested relations
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    let post = Post {
        id: 1,
        title: "Hello World".to_string(),
        content: "This is my first post".to_string(),
        author: RelationalLink::Entity(user.clone()),
    };

    let comment = Comment {
        id: 1,
        text: "Great post!".to_string(),
        post: RelationalLink::Entity(post),
        author: RelationalLink::Reference(user.primary_key()),
    };

    // Test basic insertion with relations
    store
        .put_with_links(&comment)
        .expect("Failed to insert comment with links");

    // Verify that all related entities were inserted
    let user_tree = store.open_tree::<User>();
    let post_tree = store.open_tree::<Post>();
    let comment_tree = store.open_tree::<Comment>();

    // Check that user was inserted
    let retrieved_user = user_tree
        .get(user.primary_key())
        .expect("Failed to get user");
    assert!(retrieved_user.is_some(), "User should have been inserted");
    assert_eq!(retrieved_user.unwrap(), user);

    // Check that post was inserted
    let retrieved_post = post_tree
        .get(PostPrimaryKey(1u64))
        .expect("Failed to get post");
    assert!(retrieved_post.is_some(), "Post should have been inserted");

    // Check that comment was inserted
    let retrieved_comment = comment_tree
        .get(CommentPrimaryKey(1u64))
        .expect("Failed to get comment");
    assert!(
        retrieved_comment.is_some(),
        "Comment should have been inserted"
    );
}

#[test]
fn test_recursive_insertion_with_depth_control() {
    // Create a temporary store
    let store = NetabaseStore::<TestDef, _>::temp().expect("Failed to create temp store");

    // Create test data
    let user = User {
        id: 2,
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
    };

    let post = Post {
        id: 2,
        title: "Another Post".to_string(),
        content: "This is another post".to_string(),
        author: RelationalLink::Entity(user.clone()),
    };

    let comment = Comment {
        id: 2,
        text: "Nice post!".to_string(),
        post: RelationalLink::Entity(post),
        author: RelationalLink::Entity(user.clone()),
    };

    // Test recursive insertion with depth limit
    store
        .put_with_relations_recursive(&comment, RecursionLevel::Value(2))
        .expect("Failed to insert with recursion");

    // Verify entities were inserted
    let user_tree = store.open_tree::<User>();
    let post_tree = store.open_tree::<Post>();
    let comment_tree = store.open_tree::<Comment>();

    assert!(
        user_tree
            .get(user.primary_key())
            .expect("Failed to get user")
            .is_some()
    );
    assert!(
        post_tree
            .get(PostPrimaryKey(2u64))
            .expect("Failed to get post")
            .is_some()
    );
    assert!(
        comment_tree
            .get(CommentPrimaryKey(2u64))
            .expect("Failed to get comment")
            .is_some()
    );
}

#[test]
fn test_no_recursion_insertion() {
    // Create a temporary store
    let store = NetabaseStore::<TestDef, _>::temp().expect("Failed to create temp store");

    // Create test data
    let user = User {
        id: 3,
        name: "Charlie".to_string(),
        email: "charlie@example.com".to_string(),
    };

    let post = Post {
        id: 3,
        title: "Third Post".to_string(),
        content: "This is the third post".to_string(),
        author: RelationalLink::Entity(user.clone()),
    };

    // Test insertion with no recursion
    store
        .put_with_relations_recursive(&post, RecursionLevel::None)
        .expect("Failed to insert without recursion");

    // Verify only the main model was inserted
    let user_tree = store.open_tree::<User>();
    let post_tree = store.open_tree::<Post>();

    // Post should be inserted
    assert!(
        post_tree
            .get(PostPrimaryKey(3u64))
            .expect("Failed to get post")
            .is_some()
    );

    // User should NOT be inserted because recursion is disabled
    assert!(
        user_tree
            .get(user.primary_key())
            .expect("Failed to get user")
            .is_none()
    );
}
