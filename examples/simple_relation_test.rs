//! Simple relation test to verify the macro fixes
//!
//! This example demonstrates basic relational link functionality
//! with a single model to avoid macro symbol conflicts.

use netabase_store::{
    NetabaseModel, NetabaseStore, TypedTree,
    links::RelationalLink,
    netabase_definition_module,
    store_ops::{GenericStoreOps, StoreOps},
};

// Define a simple schema with one model that has relations
#[netabase_definition_module(SimpleDefinition, SimpleKeys)]
mod simple_models {
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
    #[netabase(SimpleDefinition)]
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
    #[netabase(SimpleDefinition)]
    pub struct Post {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,

        // Author relation using custom name
        #[relation(post_author)]
        pub author: RelationalLink<SimpleDefinition, User>,
    }
}

use simple_models::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Simple Relation Test ===");

    // Create a temporary store with explicit backend type
    let store: NetabaseStore<
        SimpleDefinition,
        netabase_store::databases::sled_store::SledStore<SimpleDefinition>,
    > = NetabaseStore::temp()?;

    // Create a user
    let user = User {
        id: 1,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    };

    // Create a post with the user as an embedded entity
    let post = Post {
        id: 1,
        title: "Hello World".to_string(),
        content: "This is my first post!".to_string(),
        author: RelationalLink::Entity(user.clone()),
    };

    // Test storing the user first using typed trees for better type inference
    let user_tree = store.open_typed_tree::<User>();
    user_tree.put_with_tree(user.clone())?;
    println!("✓ User stored successfully");

    // Test storing the post
    let post_tree = store.open_typed_tree::<Post>();
    post_tree.put_with_tree(post.clone())?;
    println!("✓ Post stored successfully");

    // Test retrieving the post
    if let Some(retrieved_post) = post_tree.get_with_tree(PostPrimaryKey(1))? {
        println!("✓ Post retrieved: {}", retrieved_post.title);

        // Test the relational link
        match &retrieved_post.author {
            RelationalLink::Entity(embedded_user) => {
                println!("✓ Author is embedded: {}", embedded_user.name);
            }
            RelationalLink::Reference(user_id) => {
                println!("✓ Author is referenced by ID: {:?}", user_id);

                // Hydrate the reference
                if let Ok(Some(hydrated_user)) = retrieved_post.author.hydrate(user_tree.inner()) {
                    println!("✓ Hydrated author: {}", hydrated_user.name);
                } else {
                    println!("✗ Failed to hydrate author");
                }
            }
        }
    } else {
        println!("✗ Failed to retrieve post");
    }

    // Test relation metadata (if generated correctly)
    println!("\n=== Relation Metadata ===");
    println!("Post has relations with custom names:");
    println!("- post_author -> User");

    println!("\n=== Test Complete ===");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_relation() -> Result<(), Box<dyn std::error::Error>> {
        let store: NetabaseStore<
            SimpleDefinition,
            netabase_store::databases::sled_store::SledStore<SimpleDefinition>,
        > = NetabaseStore::temp()?;

        let user = User {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };

        let post = Post {
            id: 1,
            title: "Test Post".to_string(),
            content: "Test content".to_string(),
            author: RelationalLink::Entity(user.clone()),
        };

        // Store and retrieve using typed trees
        let user_tree = store.open_typed_tree::<User>();
        let post_tree = store.open_typed_tree::<Post>();

        user_tree.put_with_tree(user)?;
        post_tree.put_with_tree(post.clone())?;

        let retrieved = post_tree.get_with_tree(1)?.unwrap();
        assert_eq!(retrieved.title, "Test Post");
        assert!(retrieved.author.is_entity());

        Ok(())
    }

    #[test]
    fn test_relation_reference() -> Result<(), Box<dyn std::error::Error>> {
        let store: NetabaseStore<
            SimpleDefinition,
            netabase_store::databases::sled_store::SledStore<SimpleDefinition>,
        > = NetabaseStore::temp()?;

        let user = User {
            id: 42,
            name: "Referenced User".to_string(),
            email: "ref@example.com".to_string(),
        };

        // Create post with reference instead of embedded entity
        let post = Post {
            id: 1,
            title: "Referenced Post".to_string(),
            content: "This post references a user".to_string(),
            author: RelationalLink::Reference(42),
        };

        // Store user and post using typed trees
        let user_tree = store.open_typed_tree::<User>();
        let post_tree = store.open_typed_tree::<Post>();

        user_tree.put_with_tree(user.clone())?;
        post_tree.put_with_tree(post)?;

        // Retrieve and test hydration
        let retrieved = post_tree.get_with_tree(1)?.unwrap();
        assert!(retrieved.author.is_reference());

        let hydrated = retrieved.author.hydrate(user_tree.inner())?;
        assert!(hydrated.is_some());
        assert_eq!(hydrated.unwrap().name, "Referenced User");

        Ok(())
    }
}
