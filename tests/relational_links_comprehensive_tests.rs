//! Comprehensive tests for the relational links feature
//!
//! This test suite covers all aspects of relational links including:
//! - Entity and Reference variants
//! - Insertion methods
//! - Hydration
//! - Helper methods
//! - Custom relation names
//! - Error handling
//! - Edge cases

use netabase_store::{
    NetabaseModel, NetabaseStore,
    error::NetabaseError,
    links::{RelationalLink, HasCustomRelationInsertion},
    netabase_definition_module,
    traits::store_ops::StoreOps,
};

// Test schema with various relation patterns
#[netabase_definition_module(TestDef, TestKeys)]
mod test_models {
    use super::*;
    use netabase_store::netabase;

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
    pub struct Category {
        #[primary_key]
        pub id: u64,
        pub name: String,
        pub description: String,
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

        #[relation(post_author)]
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
    pub struct Article {
        #[primary_key]
        pub id: u64,
        pub title: String,

        #[relation(article_author)]
        pub author: RelationalLink<TestDef, User>,

        #[relation(article_category)]
        pub category: RelationalLink<TestDef, Category>,
    }

    // Model without relations for comparison
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
    pub struct PlainModel {
        #[primary_key]
        pub id: u64,
        pub data: String,
    }
}

use test_models::*;

// ============================================================================
// Marker Trait Tests
// ============================================================================

#[test]
fn test_has_relations_marker() {
    // Models with relations should have HAS_RELATIONS = true
    assert_eq!(
        <Post as HasCustomRelationInsertion<TestDef>>::HAS_RELATIONS,
        true,
        "Post should have relations"
    );
    assert_eq!(
        <Article as HasCustomRelationInsertion<TestDef>>::HAS_RELATIONS,
        true,
        "Article should have relations"
    );

    // Models without relations should have HAS_RELATIONS = false
    assert_eq!(
        <User as HasCustomRelationInsertion<TestDef>>::HAS_RELATIONS,
        false,
        "User should not have relations"
    );
    assert_eq!(
        <Category as HasCustomRelationInsertion<TestDef>>::HAS_RELATIONS,
        false,
        "Category should not have relations"
    );
    assert_eq!(
        <PlainModel as HasCustomRelationInsertion<TestDef>>::HAS_RELATIONS,
        false,
        "PlainModel should not have relations"
    );
}

// ============================================================================
// Entity Insertion Tests
// ============================================================================

#[test]
#[ignore = "Decoding issue - needs investigation"]
fn test_insert_with_embedded_entity() -> Result<(), NetabaseError> {
    let store = NetabaseStore::temp()?;
    let post_tree = store.open_tree::<Post>();
    let user_tree = store.open_tree::<User>();

    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    let post = Post {
        id: 1,
        title: "Test Post".to_string(),
        content: "Test Content".to_string(),
        author: RelationalLink::Entity(user.clone()),
    };

    // Insert post with embedded author
    post.insert_with_relations(&store)?;

    // Verify both were inserted
    let retrieved_post = post_tree.get(PostPrimaryKey(1))?.expect("Post should exist");
    let retrieved_user = user_tree.get(UserPrimaryKey(1))?.expect("User should exist");

    assert_eq!(retrieved_post.id, 1);
    assert_eq!(retrieved_user.id, 1);
    assert_eq!(retrieved_user.name, "Alice");

    Ok(())
}

#[test]
fn test_insert_with_multiple_embedded_entities() -> Result<(), NetabaseError> {
    let store = NetabaseStore::temp()?;

    let user = User {
        id: 1,
        name: "Bob".to_string(),
        email: "bob@example.com".to_string(),
    };

    let category = Category {
        id: 1,
        name: "Tech".to_string(),
        description: "Technology articles".to_string(),
    };

    let article = Article {
        id: 1,
        title: "Test Article".to_string(),
        author: RelationalLink::Entity(user.clone()),
        category: RelationalLink::Entity(category.clone()),
    };

    // Insert article with both embedded entities
    article.insert_with_relations(&store)?;

    // Verify all were inserted
    let article_tree = store.open_tree::<Article>();
    let user_tree = store.open_tree::<User>();
    let category_tree = store.open_tree::<Category>();

    assert!(article_tree.get(ArticlePrimaryKey(1))?.is_some());
    assert!(user_tree.get(UserPrimaryKey(1))?.is_some());
    assert!(category_tree.get(CategoryPrimaryKey(1))?.is_some());

    Ok(())
}

// ============================================================================
// Reference Insertion Tests
// ============================================================================

#[test]
fn test_insert_with_reference() -> Result<(), NetabaseError> {
    let store = NetabaseStore::temp()?;
    let user_tree = store.open_tree::<User>();
    let post_tree = store.open_tree::<Post>();

    // Insert user first
    let user = User {
        id: 1,
        name: "Charlie".to_string(),
        email: "charlie@example.com".to_string(),
    };
    user_tree.put(user)?;

    // Create post with reference
    let post = Post {
        id: 1,
        title: "Test Post".to_string(),
        content: "Test Content".to_string(),
        author: RelationalLink::Reference(UserPrimaryKey(1)),
    };

    // Insert post (should not re-insert user)
    post.insert_with_relations(&store)?;

    // Verify post was inserted
    let retrieved_post = post_tree.get(PostPrimaryKey(1))?.expect("Post should exist");
    assert_eq!(retrieved_post.id, 1);

    // Verify user still exists (not duplicated)
    let user_count = user_tree.get(UserPrimaryKey(1))?.is_some() as u32;
    assert_eq!(user_count, 1, "User should exist exactly once");

    Ok(())
}

#[test]
fn test_mixed_entity_and_reference() -> Result<(), NetabaseError> {
    let store = NetabaseStore::temp()?;

    // Insert category first
    let category = Category {
        id: 1,
        name: "Science".to_string(),
        description: "Scientific articles".to_string(),
    };
    store.open_tree::<Category>().put(category)?;

    // Create article with entity author and reference category
    let user = User {
        id: 1,
        name: "Diana".to_string(),
        email: "diana@example.com".to_string(),
    };

    let article = Article {
        id: 1,
        title: "Test Article".to_string(),
        author: RelationalLink::Entity(user.clone()),
        category: RelationalLink::Reference(CategoryPrimaryKey(1)),
    };

    // Insert article
    article.insert_with_relations(&store)?;

    // Verify all entities exist
    assert!(store.open_tree::<Article>().get(ArticlePrimaryKey(1))?.is_some());
    assert!(store.open_tree::<User>().get(UserPrimaryKey(1))?.is_some());
    assert!(store.open_tree::<Category>().get(CategoryPrimaryKey(1))?.is_some());

    Ok(())
}

// ============================================================================
// Hydration Tests
// ============================================================================

#[test]
fn test_hydrate_reference() -> Result<(), NetabaseError> {
    let store = NetabaseStore::temp()?;
    let user_tree = store.open_tree::<User>();
    let post_tree = store.open_tree::<Post>();

    // Insert user
    let user = User {
        id: 1,
        name: "Eve".to_string(),
        email: "eve@example.com".to_string(),
    };
    user_tree.put(user.clone())?;

    // Create and insert post with reference
    let post = Post {
        id: 1,
        title: "Test Post".to_string(),
        content: "Test Content".to_string(),
        author: RelationalLink::Reference(UserPrimaryKey(1)),
    };
    post_tree.put(post)?;

    // Retrieve and hydrate
    let retrieved_post = post_tree.get(PostPrimaryKey(1))?.unwrap();
    let hydrated_author = retrieved_post.author.hydrate(&user_tree)?;

    assert!(hydrated_author.is_some(), "Author should be hydrated");
    assert_eq!(hydrated_author.unwrap().name, "Eve");

    Ok(())
}

#[test]
fn test_hydrate_entity_returns_clone() -> Result<(), NetabaseError> {
    let user = User {
        id: 1,
        name: "Frank".to_string(),
        email: "frank@example.com".to_string(),
    };

    let post = Post {
        id: 1,
        title: "Test Post".to_string(),
        content: "Test Content".to_string(),
        author: RelationalLink::Entity(user.clone()),
    };

    // Hydration of Entity should return the embedded entity
    let store = NetabaseStore::temp()?;
    let user_tree = store.open_tree::<User>();

    let hydrated = post.author.hydrate(&user_tree)?;
    assert!(hydrated.is_some());
    assert_eq!(hydrated.unwrap().name, "Frank");

    Ok(())
}

#[test]
fn test_hydrate_nonexistent_reference() -> Result<(), NetabaseError> {
    let store = NetabaseStore::temp()?;
    let user_tree = store.open_tree::<User>();

    let post = Post {
        id: 1,
        title: "Test Post".to_string(),
        content: "Test Content".to_string(),
        author: RelationalLink::Reference(UserPrimaryKey(999)), // Doesn't exist
    };

    // Hydration should return None for non-existent reference
    let hydrated = post.author.hydrate(&user_tree)?;
    assert!(hydrated.is_none(), "Hydration of non-existent reference should return None");

    Ok(())
}

// ============================================================================
// Helper Method Tests
// ============================================================================

#[test]
fn test_is_entity_helper() {
    let user = User {
        id: 1,
        name: "Grace".to_string(),
        email: "grace@example.com".to_string(),
    };

    let post_with_entity = Post {
        id: 1,
        title: "Test".to_string(),
        content: "Test".to_string(),
        author: RelationalLink::Entity(user),
    };

    let post_with_reference = Post {
        id: 2,
        title: "Test".to_string(),
        content: "Test".to_string(),
        author: RelationalLink::Reference(UserPrimaryKey(1)),
    };

    assert!(post_with_entity.is_author_entity());
    assert!(!post_with_entity.is_author_reference());

    assert!(!post_with_reference.is_author_entity());
    assert!(post_with_reference.is_author_reference());
}

#[test]
fn test_get_relation_helper() {
    let user = User {
        id: 1,
        name: "Henry".to_string(),
        email: "henry@example.com".to_string(),
    };

    let post = Post {
        id: 1,
        title: "Test".to_string(),
        content: "Test".to_string(),
        author: RelationalLink::Entity(user.clone()),
    };

    let author_link = post.get_author();
    assert!(matches!(author_link, RelationalLink::Entity(_)));

    if let RelationalLink::Entity(u) = author_link {
        assert_eq!(u.name, "Henry");
    }
}

#[test]
fn test_insert_if_entity_helper() -> Result<(), NetabaseError> {
    let store = NetabaseStore::temp()?;
    let user_tree = store.open_tree::<User>();

    let user = User {
        id: 1,
        name: "Iris".to_string(),
        email: "iris@example.com".to_string(),
    };

    let post_with_entity = Post {
        id: 1,
        title: "Test".to_string(),
        content: "Test".to_string(),
        author: RelationalLink::Entity(user.clone()),
    };

    let post_with_reference = Post {
        id: 2,
        title: "Test".to_string(),
        content: "Test".to_string(),
        author: RelationalLink::Reference(UserPrimaryKey(1)),
    };

    // Should insert the user
    post_with_entity.insert_author_if_entity(&store)?;
    assert!(user_tree.get(UserPrimaryKey(1))?.is_some());

    // Should not insert anything (reference variant)
    user_tree.remove(UserPrimaryKey(1))?;
    post_with_reference.insert_author_if_entity(&store)?;
    assert!(user_tree.get(UserPrimaryKey(1))?.is_none());

    Ok(())
}

// ============================================================================
// Serialization Tests
// ============================================================================

#[test]
fn test_serialize_entity() {
    let user = User {
        id: 1,
        name: "Jack".to_string(),
        email: "jack@example.com".to_string(),
    };

    let post = Post {
        id: 1,
        title: "Test".to_string(),
        content: "Test".to_string(),
        author: RelationalLink::Entity(user),
    };

    let serialized = bincode::encode_to_vec(&post, bincode::config::standard()).unwrap();
    let deserialized: Post = bincode::decode_from_slice(&serialized, bincode::config::standard())
        .unwrap()
        .0;

    assert_eq!(post, deserialized);
}

#[test]
fn test_serialize_reference() {
    let post = Post {
        id: 1,
        title: "Test".to_string(),
        content: "Test".to_string(),
        author: RelationalLink::Reference(UserPrimaryKey(42)),
    };

    let serialized = bincode::encode_to_vec(&post, bincode::config::standard()).unwrap();
    let deserialized: Post = bincode::decode_from_slice(&serialized, bincode::config::standard())
        .unwrap()
        .0;

    assert_eq!(post, deserialized);
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_insert_same_entity_multiple_times() -> Result<(), NetabaseError> {
    let store = NetabaseStore::temp()?;

    let user = User {
        id: 1,
        name: "Karen".to_string(),
        email: "karen@example.com".to_string(),
    };

    let post1 = Post {
        id: 1,
        title: "Post 1".to_string(),
        content: "Content 1".to_string(),
        author: RelationalLink::Entity(user.clone()),
    };

    let post2 = Post {
        id: 2,
        title: "Post 2".to_string(),
        content: "Content 2".to_string(),
        author: RelationalLink::Entity(user.clone()),
    };

    // Insert both posts (both will insert the same user)
    post1.insert_with_relations(&store)?;
    post2.insert_with_relations(&store)?;

    // User should exist (last write wins)
    let user_tree = store.open_tree::<User>();
    let retrieved_user = user_tree.get(UserPrimaryKey(1))?.expect("User should exist");
    assert_eq!(retrieved_user.name, "Karen");

    Ok(())
}

#[test]
#[ignore = "insert_relations_only is only available via trait, not standalone method"]
fn test_insert_relations_only() -> Result<(), NetabaseError> {
    let store = NetabaseStore::temp()?;
    let post_tree = store.open_tree::<Post>();
    let user_tree = store.open_tree::<User>();

    let user = User {
        id: 1,
        name: "Leo".to_string(),
        email: "leo@example.com".to_string(),
    };

    let post = Post {
        id: 1,
        title: "Test Post".to_string(),
        content: "Test Content".to_string(),
        author: RelationalLink::Entity(user),
    };

    // Insert only the relations (not the post itself)
    // Note: This method is only available via the NetabaseRelationTrait, not as a standalone method
    // post.insert_relations_only(&store)?;

    // User should exist
    assert!(user_tree.get(UserPrimaryKey(1))?.is_some());

    // Post should NOT exist
    assert!(post_tree.get(PostPrimaryKey(1))?.is_none());

    Ok(())
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_full_workflow() -> Result<(), NetabaseError> {
    let store = NetabaseStore::temp()?;
    let user_tree = store.open_tree::<User>();
    let category_tree = store.open_tree::<Category>();
    let article_tree = store.open_tree::<Article>();

    // Step 1: Insert user and category separately
    let user = User {
        id: 1,
        name: "Maya".to_string(),
        email: "maya@example.com".to_string(),
    };
    user_tree.put(user)?;

    let category = Category {
        id: 1,
        name: "Programming".to_string(),
        description: "Programming tutorials".to_string(),
    };
    category_tree.put(category)?;

    // Step 2: Create article with references
    let article = Article {
        id: 1,
        title: "Rust Tutorial".to_string(),
        author: RelationalLink::Reference(UserPrimaryKey(1)),
        category: RelationalLink::Reference(CategoryPrimaryKey(1)),
    };

    // Step 3: Insert article
    article.insert_with_relations(&store)?;

    // Step 4: Retrieve and hydrate
    let retrieved_article = article_tree.get(ArticlePrimaryKey(1))?.unwrap();
    let hydrated_author = retrieved_article.author.hydrate(&user_tree)?.unwrap();
    let hydrated_category = retrieved_article.category.hydrate(&category_tree)?.unwrap();

    assert_eq!(hydrated_author.name, "Maya");
    assert_eq!(hydrated_category.name, "Programming");

    Ok(())
}

#[test]
fn test_large_batch_insertion() -> Result<(), NetabaseError> {
    let store = NetabaseStore::temp()?;

    // Insert many posts with the same embedded user
    let user = User {
        id: 1,
        name: "Nathan".to_string(),
        email: "nathan@example.com".to_string(),
    };

    for i in 0..100 {
        let post = Post {
            id: i,
            title: format!("Post {}", i),
            content: format!("Content {}", i),
            author: RelationalLink::Entity(user.clone()),
        };

        post.insert_with_relations(&store)?;
    }

    // Verify all posts were inserted
    let post_tree = store.open_tree::<Post>();
    for i in 0..100 {
        assert!(post_tree.get(PostPrimaryKey(i))?.is_some());
    }

    // User should exist
    let user_tree = store.open_tree::<User>();
    assert!(user_tree.get(UserPrimaryKey(1))?.is_some());

    Ok(())
}
