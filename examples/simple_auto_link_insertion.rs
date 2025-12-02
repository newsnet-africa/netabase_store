//! Simple example demonstrating relational links in netabase_store
//!
//! This example shows how to work with RelationalLink fields in models
//! and demonstrates basic insertion and retrieval patterns.

use netabase_store::links::RelationalLink;
use netabase_store::traits::store_ops::StoreOps;
use netabase_store::*;
use std::error::Error;

// Define our database schema
#[netabase_definition_module(BlogDefinition, BlogDefinitionKey)]
mod blog_schema {
    use super::*;
    use netabase_store::netabase;

    /// Author model - this will be linked by Post
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
    #[netabase(BlogDefinition)]
    pub struct Author {
        #[primary_key]
        pub id: u64,
        pub name: String,
        pub email: String,
    }

    /// Post model with RelationalLink field
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
    #[netabase(BlogDefinition)]
    pub struct Post {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,

        // RelationalLink field with 2-parameter form
        #[relation(author)]
        pub author: RelationalLink<BlogDefinition, Author>,
    }
}

use blog_schema::*;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Simple Relational Links Example ===\n");

    // Create a temporary database with explicit type annotation
    let store: NetabaseStore<
        BlogDefinition,
        netabase_store::databases::sled_store::SledStore<BlogDefinition>,
    > = NetabaseStore::temp()?;

    // Create an author entity
    let author = Author {
        id: 1,
        name: "Alice Johnson".to_string(),
        email: "alice@example.com".to_string(),
    };

    println!("1. Inserting author:");
    let author_tree = store.open_tree::<Author>();
    author_tree.put_raw(author.clone())?;
    println!("   ✓ Inserted author: {}", author.name);

    // Create a post that references the author by key
    let post_with_key = Post {
        id: 1,
        title: "My First Post".to_string(),
        content: "This is my first blog post!".to_string(),
        author: RelationalLink::from_key(author.primary_key()),
    };

    println!("\n2. Inserting post with author key reference:");
    let post_tree = store.open_tree::<Post>();
    post_tree.put_raw(post_with_key.clone())?;
    println!("   ✓ Inserted post: {}", post_with_key.title);

    // Verify we can retrieve the post and resolve the author link
    println!("\n3. Retrieving post and resolving author:");
    if let Some(retrieved_post) = post_tree.get_raw(post_with_key.primary_key())? {
        println!("   Retrieved post: {}", retrieved_post.title);

        let author_key = retrieved_post.author.key();
        if let Some(linked_author) = author_tree.get_raw(author_key)? {
            println!("   ✓ Resolved author: {}", linked_author.name);
        }
    }

    // Create a post with an embedded author entity
    let embedded_author = Author {
        id: 2,
        name: "Bob Smith".to_string(),
        email: "bob@example.com".to_string(),
    };

    let post_with_entity = Post {
        id: 2,
        title: "Second Post".to_string(),
        content: "This post embeds the author entity.".to_string(),
        author: RelationalLink::Entity(embedded_author.clone()),
    };

    println!("\n4. Inserting post with embedded author entity:");
    post_tree.put_raw(post_with_entity.clone())?;
    println!(
        "   ✓ Inserted post with embedded author: {}",
        post_with_entity.title
    );

    // When we retrieve this post, we can access the embedded author directly
    if let Some(post_with_embedded_retrieved) = post_tree.get_raw(post_with_entity.primary_key())? {
        match &post_with_embedded_retrieved.author {
            RelationalLink::Entity(embedded_author) => {
                println!("   ✓ Retrieved embedded author: {}", embedded_author.name);
            }
            RelationalLink::Reference(_) => {
                println!("   ✓ Author is stored as key reference");
            }
        }
    }

    println!("\n5. Verification - listing all posts:");
    for post_result in post_tree.iter() {
        let (_key, post) = post_result?;
        println!(
            "   Post: {} (author key: {:?})",
            post.title,
            post.author.key()
        );
    }

    println!("\n6. Verification - listing all authors:");
    for author_result in author_tree.iter() {
        let (_key, author) = author_result?;
        println!("   Author: {} ({})", author.name, author.email);
    }

    println!("\n✅ Simple relational links example completed successfully!");
    println!("\nKey concepts demonstrated:");
    println!("• RelationalLink<D, M> with 3-parameter form");
    println!("• Key-based references using RelationalLink::from_key()");
    println!("• Entity embedding using RelationalLink::Entity()");
    println!("• Link resolution using the .key() method");
    println!("• Pattern matching on RelationalLink variants");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relational_link_variants() {
        let author = Author {
            id: 1,
            name: "Test Author".to_string(),
            email: "test@example.com".to_string(),
        };

        // Test Entity variant
        let entity_link = RelationalLink::Entity(author.clone());
        assert!(matches!(entity_link, RelationalLink::Entity(_)));

        // Test Reference variant
        let ref_link = RelationalLink::from_key(1u64);
        assert!(matches!(ref_link, RelationalLink::Reference(_)));
    }

    #[test]
    fn test_basic_insertion() -> Result<(), Box<dyn std::error::Error>> {
        let store: NetabaseStore<
            BlogDefinition,
            netabase_store::databases::sled_store::SledStore<BlogDefinition>,
        > = NetabaseStore::temp()?;

        let author = Author {
            id: 1,
            name: "Test Author".to_string(),
            email: "test@example.com".to_string(),
        };

        let post = Post {
            id: 1,
            title: "Test Post".to_string(),
            content: "Test content".to_string(),
            author: RelationalLink::Entity(author.clone()),
        };

        // Test basic insertion
        let author_tree = store.open_tree::<Author>();
        author_tree.put_raw(author.clone())?;

        let post_tree = store.open_tree::<Post>();
        post_tree.put_raw(post.clone())?;

        // Verify insertion worked
        let stored_author = author_tree
            .get_raw(author.primary_key())?
            .expect("Author should exist");
        assert_eq!(stored_author.name, "Test Author");

        let stored_post = post_tree
            .get_raw(post.primary_key())?
            .expect("Post should exist");
        assert_eq!(stored_post.title, "Test Post");

        Ok(())
    }

    #[test]
    fn test_link_resolution() -> Result<(), Box<dyn std::error::Error>> {
        let store: NetabaseStore<
            BlogDefinition,
            netabase_store::databases::sled_store::SledStore<BlogDefinition>,
        > = NetabaseStore::temp()?;

        let author = Author {
            id: 1,
            name: "Test Author".to_string(),
            email: "test@example.com".to_string(),
        };

        // Insert author first
        let author_tree = store.open_tree::<Author>();
        author_tree.put_raw(author.clone())?;

        // Create post with key reference
        let post = Post {
            id: 1,
            title: "Test Post".to_string(),
            content: "Test content".to_string(),
            author: RelationalLink::from_key(author.primary_key()),
        };

        let post_tree = store.open_tree::<Post>();
        post_tree.put_raw(post.clone())?;

        // Retrieve and resolve link
        let retrieved_post = post_tree
            .get_raw(post.primary_key())?
            .expect("Post should exist");
        let author_key = retrieved_post.author.key();
        let linked_author = author_tree
            .get_raw(author_key)?
            .expect("Linked author should exist");

        assert_eq!(linked_author.name, "Test Author");

        Ok(())
    }
}
