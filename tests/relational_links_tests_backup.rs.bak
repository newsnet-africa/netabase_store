//! Comprehensive tests for RelationalLink functionality
//!
//! This test suite covers all aspects of RelationalLink usage including:
//! - Basic creation and manipulation
//! - Hydration from references to full entities
//! - Type safety and compile-time guarantees
//! - Performance characteristics
//! - Edge cases and error handling
//! - Integration with different storage backends

use netabase_store::links::RelationalLink;
use netabase_store::traits::store_ops::StoreOps;
use netabase_store::*;
use std::collections::HashMap;

// Test schema with relationships
#[netabase_definition_module(TestDefinition, TestDefinitionKey)]
mod test_schema {
    use super::*;

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        PartialEq,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(TestDefinition)]
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
        bincode::Encode,
        bincode::Decode,
        PartialEq,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(TestDefinition)]
    pub struct Post {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,
        pub author_id: u64, // Traditional foreign key
    }

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        PartialEq,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(TestDefinition)]
    pub struct Comment {
        #[primary_key]
        pub id: u64,
        pub content: String,
        pub post_id: u64,
        pub author_id: u64,
    }

    // Models using RelationalLink
    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        PartialEq,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(TestDefinition)]
    pub struct PostWithLinks {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,
        pub author: RelationalLink<TestDefinition, User, PostWithLinksRelations>,
    }

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        PartialEq,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(TestDefinition)]
    pub struct CommentWithLinks {
        #[primary_key]
        pub id: u64,
        pub content: String,
        pub post: RelationalLink<TestDefinition, Post, CommentWithLinksRelations>,
        pub author: RelationalLink<TestDefinition, User, CommentWithLinksRelations>,
    }

    // Complex model with multiple link types
    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        bincode::Encode,
        bincode::Decode,
        PartialEq,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(TestDefinition)]
    pub struct BlogArticle {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,
        pub author: RelationalLink<TestDefinition, User, BlogArticleRelations>,
        pub related_posts: Vec<RelationalLink<TestDefinition, Post, BlogArticleRelations>>,
        pub comments: Vec<RelationalLink<TestDefinition, Comment, BlogArticleRelations>>,
    }
}

use test_schema::*;

#[cfg(feature = "native")]
mod native_tests {
    use super::*;
    use netabase_store::databases::sled_store::SledStore;

    #[test]
    fn test_relational_link_creation_and_variants() {
        let user = User {
            id: 1,
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
        };

        // Test Entity variant creation
        let entity_link: RelationalLink<TestDefinition, User, PostWithLinksRelations> =
            RelationalLink::Entity(user.clone());
        let entity_link_from: RelationalLink<TestDefinition, User, PostWithLinksRelations> =
            user.clone().into();
        assert_eq!(entity_link, entity_link_from);

        // Test Reference variant creation
        let ref_link = RelationalLink::from_key(user.primary_key());

        // Verify they're different variants
        assert!(matches!(entity_link, RelationalLink::Entity(_)));
        assert!(matches!(ref_link, RelationalLink::Reference(_)));
        assert_ne!(entity_link, ref_link);
    }

    #[test]
    fn test_relational_link_hydration() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempfile::tempdir()?;
        let store = SledStore::<TestDefinition>::new(temp_dir.path())?;

        // Setup test data
        let user = User {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        };

        // Insert user into store
        let user_tree = store.open_tree();
        user_tree.put_raw(user.clone())?;

        // Test hydration of Entity variant (should return the entity directly)
        let entity_link: RelationalLink<TestDefinition, User, PostWithLinksRelations> =
            RelationalLink::Entity(user.clone());
        let hydrated = entity_link.hydrate(&user_tree)?;
        assert_eq!(hydrated, Some(user.clone()));

        // Test hydration of Reference variant (should fetch from store)
        let ref_link: RelationalLink<TestDefinition, User, PostWithLinksRelations> =
            RelationalLink::from_key(user.primary_key());
        let hydrated = ref_link.hydrate(&user_tree)?;
        assert_eq!(hydrated, Some(user));

        Ok(())
    }

    #[test]
    fn test_relational_link_hydration_missing_entity() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempfile::tempdir()?;
        let store = SledStore::<TestDefinition>::new(temp_dir.path())?;

        // Test hydration of reference to non-existent entity
        let ref_link = RelationalLink::<TestDefinition, User, PostWithLinksRelations>::from_key(
            UserPrimaryKey(999),
        );
        let user_tree = store.open_tree();
        let hydrated = ref_link.hydrate(&user_tree)?;
        assert_eq!(hydrated, None);

        Ok(())
    }

    #[test]
    fn test_complex_relational_structures() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempfile::tempdir()?;
        let store = SledStore::<TestDefinition>::new(temp_dir.path())?;

        // Setup test data
        let author = User {
            id: 1,
            name: "Author".to_string(),
            email: "author@example.com".to_string(),
        };
        let post = Post {
            id: 2,
            title: "Test Post".to_string(),
            content: "Content".to_string(),
            author_id: author.id,
        };
        let comment = Comment {
            id: 3,
            content: "Great post!".to_string(),
            post_id: post.id,
            author_id: author.id,
        };

        // Insert entities
        let user_tree = store.open_tree();
        let post_tree = store.open_tree();
        let comment_tree = store.open_tree();

        user_tree.put_raw(author.clone())?;
        post_tree.put_raw(post.clone())?;
        comment_tree.put_raw(comment.clone())?;

        // Create complex linked structure
        let article = BlogArticle {
            id: 1,
            title: "Complex Article".to_string(),
            content: "This article has many relationships".to_string(),
            author: RelationalLink::Entity(author.clone()),
            related_posts: vec![
                RelationalLink::Entity(post.clone()),
                RelationalLink::from_key(post.primary_key()), // Mix of entity and reference
            ],
            comments: vec![RelationalLink::from_key(comment.primary_key())],
        };

        // Verify the structure
        assert!(matches!(article.author, RelationalLink::Entity(_)));
        assert_eq!(article.related_posts.len(), 2);
        assert!(matches!(
            article.related_posts[0],
            RelationalLink::Entity(_)
        ));
        assert!(matches!(
            article.related_posts[1],
            RelationalLink::Reference(_)
        ));

        Ok(())
    }

    #[test]
    fn test_relational_link_type_safety() {
        let user = User {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };

        // These should compile fine
        let _user_link: RelationalLink<TestDefinition, User, PostWithLinksRelations> =
            RelationalLink::Entity(user.clone());
        let _user_ref: RelationalLink<TestDefinition, User, PostWithLinksRelations> =
            RelationalLink::from_key(UserPrimaryKey(1u64));

        // This should not compile (different model type)
        // let invalid_link: RelationalLink<TestDefinition, Post> = RelationalLink::Entity(user);
    }

    #[test]
    fn test_relational_link_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let user = User {
            id: 1,
            name: "Serializable User".to_string(),
            email: "serial@example.com".to_string(),
        };

        // Test Entity variant serialization
        let entity_link = RelationalLink::Entity(user.clone());
        let encoded = bincode::encode_to_vec(&entity_link, bincode::config::standard())?;
        let decoded: RelationalLink<TestDefinition, User, PostWithLinksRelations> =
            bincode::decode_from_slice(&encoded, bincode::config::standard())?.0;
        assert_eq!(entity_link, decoded);

        // Test Reference variant serialization
        let ref_link = RelationalLink::<TestDefinition, User, PostWithLinksRelations>::from_key(
            UserPrimaryKey(42),
        );
        let encoded = bincode::encode_to_vec(&ref_link, bincode::config::standard())?;
        let decoded: RelationalLink<TestDefinition, User, PostWithLinksRelations> =
            bincode::decode_from_slice(&encoded, bincode::config::standard())?.0;
        assert_eq!(ref_link, decoded);

        Ok(())
    }

    #[test]
    fn test_relational_link_collections() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempfile::tempdir()?;
        let store = SledStore::<TestDefinition>::new(temp_dir.path())?;

        // Create multiple users
        let users: Vec<User> = (1..=5)
            .map(|i| User {
                id: i,
                name: format!("User {}", i),
                email: format!("user{}@example.com", i),
            })
            .collect();

        // Insert users
        let user_tree = store.open_tree();
        for user in &users {
            user_tree.put_raw(user.clone())?;
        }

        // Create collection with mixed Entity/Reference links
        let user_links: Vec<RelationalLink<TestDefinition, User, PostWithLinksRelations>> = vec![
            RelationalLink::Entity(users[0].clone()),
            RelationalLink::from_key(users[1].primary_key()),
            RelationalLink::Entity(users[2].clone()),
            RelationalLink::from_key(users[3].primary_key()),
            RelationalLink::from_key(users[4].primary_key()),
        ];

        // Test hydration of the entire collection
        let mut hydrated_users = Vec::new();
        for link in user_links {
            if let Some(user) = link.hydrate(&user_tree)? {
                hydrated_users.push(user);
            }
        }

        assert_eq!(hydrated_users.len(), 5);
        for (i, user) in hydrated_users.iter().enumerate() {
            assert_eq!(user.id, (i + 1) as u64);
        }

        Ok(())
    }

    #[test]
    fn test_relational_link_memory_efficiency() {
        // Test that Reference variant uses less memory than Entity variant
        let user = User {
            id: 1,
            name: "Large User Name That Takes More Memory".to_string(),
            email: "very.long.email.address@example-domain.com".to_string(),
        };

        let entity_link: RelationalLink<TestDefinition, User, PostWithLinksRelations> =
            RelationalLink::Entity(user.clone());
        let ref_link = RelationalLink::<TestDefinition, User, PostWithLinksRelations>::from_key(
            UserPrimaryKey(1),
        );

        // Serialize to compare sizes
        let entity_size = bincode::encode_to_vec(&entity_link, bincode::config::standard())
            .unwrap()
            .len();
        let ref_size = bincode::encode_to_vec(&ref_link, bincode::config::standard())
            .unwrap()
            .len();

        // Reference should be significantly smaller
        assert!(ref_size < entity_size);
        println!("Entity link size: {} bytes", entity_size);
        println!("Reference link size: {} bytes", ref_size);
    }

    #[test]
    fn test_relational_link_conversion_patterns() {
        let user = User {
            id: 42,
            name: "Convert User".to_string(),
            email: "convert@example.com".to_string(),
        };

        // Test conversion from entity to reference
        let entity_link: RelationalLink<TestDefinition, User, PostWithLinksRelations> =
            RelationalLink::Entity(user.clone());
        let extracted_key = match &entity_link {
            RelationalLink::Entity(u) => u.primary_key(),
            RelationalLink::Reference(key) => key.clone(),
            RelationalLink::_RelationMarker(_) => {
                unreachable!("RelationMarker should never be used")
            }
        };
        assert_eq!(extracted_key, user.primary_key());

        // Test utility functions from link_utils
        use netabase_store::traits::links::link_utils;

        assert!(link_utils::is_entity(&entity_link));
        assert_eq!(link_utils::extract_entity(&entity_link), Some(&user));
        assert_eq!(link_utils::extract_key(&entity_link), user.primary_key());

        let ref_link = RelationalLink::<TestDefinition, User, PostWithLinksRelations>::from_key(
            user.primary_key(),
        );
        assert!(!link_utils::is_entity(&ref_link));
        assert_eq!(link_utils::extract_entity(&ref_link), None);
        assert_eq!(link_utils::extract_key(&ref_link), user.primary_key());
    }

    #[test]
    fn test_relational_link_error_handling() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempfile::tempdir()?;
        let store = SledStore::<TestDefinition>::new(temp_dir.path())?;

        // Test hydration with corrupted store (we can't easily simulate this)
        // But we can test with empty store
        let post_tree = store.open_tree();
        let ref_link = RelationalLink::<TestDefinition, Post, BlogArticleRelations>::from_key(
            PostPrimaryKey(42),
        );

        // Should return None for missing entity, not error
        let result = ref_link.hydrate(&post_tree)?;
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_relational_link_with_different_backends() -> Result<(), Box<dyn std::error::Error>> {
        // Test with Sled
        {
            let temp_dir = tempfile::tempdir()?;
            let store = SledStore::<TestDefinition>::new(temp_dir.path())?;
            test_backend_compatibility(store)?;
        }

        // Test with Redb if available
        #[cfg(feature = "redb")]
        {
            let temp_dir = tempfile::tempdir()?;
            let store = netabase_store::databases::redb_store::RedbStore::<TestDefinition>::new(
                temp_dir.path().join("test.redb"),
            )?;
            test_backend_compatibility(store)?;
        }

        Ok(())
    }

    fn test_backend_compatibility<Store>(store: Store) -> Result<(), Box<dyn std::error::Error>>
    where
        Store: netabase_store::traits::store_ops::OpenTree<TestDefinition, User>,
    {
        let user = User {
            id: 1,
            name: "Backend Test".to_string(),
            email: "backend@example.com".to_string(),
        };

        let user_tree = store.open_tree();
        user_tree.put_raw(user.clone())?;

        // Test both variants work with this backend
        let entity_link: RelationalLink<TestDefinition, User, PostWithLinksRelations> =
            RelationalLink::Entity(user.clone());
        let ref_link: RelationalLink<TestDefinition, User, PostWithLinksRelations> =
            RelationalLink::from_key(user.primary_key());

        let user_tree1 = store.open_tree();
        let hydrated1 = entity_link.hydrate(&user_tree1)?;

        // Create a fresh tree for the second hydrate since it consumes self
        let user_tree2 = store.open_tree();
        let hydrated2 = ref_link.hydrate(&user_tree2)?;

        assert_eq!(hydrated1, Some(user.clone()));
        assert_eq!(hydrated2, Some(user));

        Ok(())
    }

    #[test]
    fn test_relational_link_clone_and_equality() {
        let user = User {
            id: 1,
            name: "Clone Test".to_string(),
            email: "clone@example.com".to_string(),
        };

        let entity_link = RelationalLink::Entity(user.clone());
        let entity_link_clone = entity_link.clone();
        assert_eq!(entity_link, entity_link_clone);

        let ref_link = RelationalLink::<TestDefinition, User, PostWithLinksRelations>::from_key(
            user.primary_key(),
        );
        let ref_link_clone = ref_link.clone();
        assert_eq!(ref_link, ref_link_clone);

        // Different variants with same logical entity should not be equal
        assert_ne!(entity_link, ref_link);
    }
}

#[cfg(feature = "wasm")]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn test_relational_links_in_wasm() {
        // Basic functionality tests for WASM environment
        let user = User {
            id: 1,
            name: "WASM User".to_string(),
            email: "wasm@example.com".to_string(),
        };

        let entity_link = RelationalLink::Entity(user.clone());
        let user_link = RelationalLink::from_key(user.primary_key());

        // Test serialization works in WASM
        let encoded = bincode::encode_to_vec(&entity_link, bincode::config::standard()).unwrap();
        let _decoded: RelationalLink<TestDefinition, User> =
            bincode::decode_from_slice(&encoded, bincode::config::standard())
                .unwrap()
                .0;

        assert!(true); // If we get here, basic functionality works
    }
}

// Benchmark-style performance tests
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_relational_link_creation_performance() {
        let user = User {
            id: 1,
            name: "Performance Test".to_string(),
            email: "perf@example.com".to_string(),
        };

        let iterations = 100_000;

        // Benchmark Entity variant creation
        let start = Instant::now();
        for _ in 0..iterations {
            let _link: RelationalLink<TestDefinition, User, PostWithLinksRelations> =
                RelationalLink::Entity(user.clone());
        }
        let entity_duration = start.elapsed();

        // Benchmark Reference variant creation
        let start = Instant::now();
        for _ in 0..iterations {
            let _link = RelationalLink::<TestDefinition, User, PostWithLinksRelations>::from_key(
                user.primary_key(),
            );
        }
        let ref_duration = start.elapsed();

        println!(
            "Entity creation: {:?} for {} iterations",
            entity_duration, iterations
        );
        println!(
            "Reference creation: {:?} for {} iterations",
            ref_duration, iterations
        );

        // Reference creation should be faster (no cloning)
        assert!(ref_duration < entity_duration);
    }

    #[test]
    fn test_relational_link_serialization_performance() -> Result<(), Box<dyn std::error::Error>> {
        let user = User {
            id: 1,
            name: "Serialization Test".to_string(),
            email: "serialize@example.com".to_string(),
        };

        let entity_link: RelationalLink<TestDefinition, User, PostWithLinksRelations> =
            RelationalLink::Entity(user.clone());
        let ref_link = RelationalLink::<TestDefinition, User, PostWithLinksRelations>::from_key(
            user.primary_key(),
        );

        let iterations = 10_000;

        // Benchmark Entity serialization
        let start = Instant::now();
        for _ in 0..iterations {
            let _encoded = bincode::encode_to_vec(&entity_link, bincode::config::standard())?;
        }
        let entity_ser_duration = start.elapsed();

        // Benchmark Reference serialization
        let start = Instant::now();
        for _ in 0..iterations {
            let _encoded = bincode::encode_to_vec(&ref_link, bincode::config::standard())?;
        }
        let ref_ser_duration = start.elapsed();

        println!(
            "Entity serialization: {:?} for {} iterations",
            entity_ser_duration, iterations
        );
        println!(
            "Reference serialization: {:?} for {} iterations",
            ref_ser_duration, iterations
        );

        // Reference serialization should be faster
        assert!(ref_ser_duration < entity_ser_duration);

        Ok(())
    }

    #[cfg(feature = "native")]
    #[test]
    fn test_relational_link_hydration_performance() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempfile::tempdir()?;
        let store = netabase_store::databases::sled_store::SledStore::<TestDefinition>::new(
            temp_dir.path(),
        )?;

        // Setup test data
        let users: Vec<User> = (1..=1000)
            .map(|i| User {
                id: i,
                name: format!("User {}", i),
                email: format!("user{}@example.com", i),
            })
            .collect();

        let user_tree = store.open_tree();
        for user in &users {
            user_tree.put_raw(user.clone())?;
        }

        // Create test links
        let entity_links: Vec<RelationalLink<TestDefinition, User, PostWithLinksRelations>> = users
            .iter()
            .take(100)
            .map(|u| RelationalLink::Entity(u.clone()))
            .collect();

        let ref_links: Vec<RelationalLink<TestDefinition, User, PostWithLinksRelations>> = users
            .iter()
            .take(100)
            .map(|u| RelationalLink::from_key(u.primary_key()))
            .collect();

        // Benchmark Entity hydration (should be instant)
        let start = Instant::now();
        for link in entity_links {
            let user_tree_instance = store.open_tree();
            let _hydrated = link.hydrate(&user_tree_instance)?;
        }
        let entity_hydration_duration = start.elapsed();

        // Benchmark Reference hydration (requires store lookup)
        let start = Instant::now();
        for link in ref_links {
            let user_tree_instance = store.open_tree();
            let _result = link.hydrate(&user_tree_instance)?;
        }
        let ref_hydration_duration = start.elapsed();

        println!(
            "Entity hydration: {:?} for 100 items",
            entity_hydration_duration
        );
        println!(
            "Reference hydration: {:?} for 100 items",
            ref_hydration_duration
        );

        // Entity hydration should be much faster
        assert!(entity_hydration_duration < ref_hydration_duration);

        Ok(())
    }
}

// Integration tests with the full stack
#[cfg(feature = "native")]
mod integration_tests {
    use super::*;
    use netabase_store::databases::sled_store::SledStore;

    #[test]
    fn test_relational_links_end_to_end_workflow() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempfile::tempdir()?;
        let store = SledStore::<TestDefinition>::new(temp_dir.path())?;

        // Step 1: Create and store base entities
        let author = User {
            id: 1,
            name: "Article Author".to_string(),
            email: "author@example.com".to_string(),
        };

        let reviewer = User {
            id: 2,
            name: "Code Reviewer".to_string(),
            email: "reviewer@example.com".to_string(),
        };

        let post1 = Post {
            id: 1,
            title: "First Post".to_string(),
            content: "Content of first post".to_string(),
            author_id: author.id,
        };

        let post2 = Post {
            id: 2,
            title: "Second Post".to_string(),
            content: "Content of second post".to_string(),
            author_id: author.id,
        };

        // Store entities
        let user_tree = store.open_tree();
        let post_tree = store.open_tree();

        user_tree.put_raw(author.clone())?;
        user_tree.put_raw(reviewer.clone())?;
        post_tree.put_raw(post1.clone())?;
        post_tree.put_raw(post2.clone())?;

        // Step 2: Create complex linked structure
        let article = BlogArticle {
            id: 1,
            title: "Comprehensive Article".to_string(),
            content: "This article links to everything".to_string(),
            author: RelationalLink::Entity(author.clone()),
            related_posts: vec![
                RelationalLink::from_key(post1.primary_key()),
                RelationalLink::Entity(post2.clone()),
            ],
            comments: vec![], // No comments yet
        };

        // Step 3: Work with the linked structure

        // Verify author is immediately available
        if let RelationalLink::Entity(article_author) = &article.author {
            assert_eq!(article_author.name, author.name);
        }

        // Hydrate related posts
        let mut hydrated_posts = Vec::new();
        for post_link in article.related_posts.clone() {
            let post_tree_instance = store.open_tree();
            if let Some(post) = post_link.hydrate(&post_tree_instance)? {
                hydrated_posts.push(post);
            }
        }
        assert_eq!(hydrated_posts.len(), 2);

        // Step 4: Transform links between variants
        let mut article_with_refs = article.clone();

        // Convert author from Entity to Reference
        article_with_refs.author = RelationalLink::from_key(author.primary_key());

        // Convert related posts to all References
        article_with_refs.related_posts = article
            .related_posts
            .iter()
            .map(|link| {
                let key = match link {
                    RelationalLink::Entity(post) => post.primary_key(),
                    RelationalLink::Reference(key) => key.clone(),
                    RelationalLink::_RelationMarker(_) => {
                        unreachable!("RelationMarker should never be used")
                    }
                };
                RelationalLink::from_key(key)
            })
            .collect();

        // Verify the transformed structure
        assert!(matches!(
            article_with_refs.author,
            RelationalLink::Reference(_)
        ));
        for post_link in &article_with_refs.related_posts {
            assert!(matches!(post_link, RelationalLink::Reference(_)));
        }

        // Step 5: Hydrate everything back to entities
        let user_tree_instance = store.open_tree();
        let hydrated_author = article_with_refs
            .author
            .clone()
            .hydrate(&user_tree_instance)?;
        assert_eq!(hydrated_author, Some(author));

        Ok(())
    }

    #[test]
    fn test_relational_links_lazy_loading_pattern() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempfile::tempdir()?;
        let store = SledStore::<TestDefinition>::new(temp_dir.path())?;

        // Create a large dataset
        let users: Vec<User> = (1..=100)
            .map(|i| User {
                id: i,
                name: format!("User {}", i),
                email: format!("user{}@example.com", i),
            })
            .collect();

        let user_tree = store.open_tree();
        for user in &users {
            user_tree.put_raw(user.clone())?;
        }

        // Simulate lazy loading: start with references, load entities on demand
        let user_refs: Vec<RelationalLink<TestDefinition, User, PostWithLinksRelations>> = users
            .iter()
            .map(|u| RelationalLink::from_key(u.primary_key()))
            .collect();

        // Simulate accessing only some users (lazy loading)
        let indices_to_load = vec![5, 15, 25, 35, 45];
        let mut loaded_users = HashMap::new();

        for &index in &indices_to_load {
            let user_tree_instance = store.open_tree();
            if let Some(user) = user_refs[index].clone().hydrate(&user_tree_instance)? {
                loaded_users.insert(user.id, user);
            }
        }

        assert_eq!(loaded_users.len(), 5);
        for &index in &indices_to_load {
            let expected_id = (index + 1) as u64;
            assert!(loaded_users.contains_key(&expected_id));
        }

        Ok(())
    }
}
