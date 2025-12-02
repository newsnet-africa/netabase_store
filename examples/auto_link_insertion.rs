//! Example demonstrating relational links in netabase_store
//!
//! This example shows how to work with RelationalLink fields in models
//! and demonstrates the basic insertion and retrieval patterns.

use netabase_store::links::RelationalLink;
use netabase_store::*;
use std::error::Error;

// Define our database schema
#[netabase_definition_module(BlogDefinition, BlogDefinitionKey)]
mod blog_schema {
    use super::*;

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
        pub bio: String,
    }

    /// Category model - this will be linked by Post
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
    pub struct Category {
        #[primary_key]
        pub id: u64,
        pub name: String,
        pub description: String,
    }

    /// Tag model - this will be linked by Post
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
    pub struct Tag {
        #[primary_key]
        pub id: u64,
        pub name: String,
        pub color: String,
    }

    /// Post model with RelationalLink fields
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

        // RelationalLink fields with the 3-parameter form
        #[relation(author)]
        pub author: RelationalLink<BlogDefinition, Author>,
        #[relation(category)]
        pub category: RelationalLink<BlogDefinition, Category>,
        #[relation(tags)]
        pub tags: Vec<RelationalLink<BlogDefinition, Tag>>,

        pub created_at: u64,
        pub published: bool,
    }

    /// Comment model with RelationalLink to Post
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
    pub struct Comment {
        #[primary_key]
        pub id: u64,
        pub content: String,

        // RelationalLink to Post and Author
        #[relation(post)]
        pub post: RelationalLink<BlogDefinition, Post>,
        #[relation(comment_author)]
        pub author: RelationalLink<BlogDefinition, Author>,

        pub created_at: u64,
        pub approved: bool,
    }
}

use blog_schema::*;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Relational Links Example ===\n");

    // Create a temporary database with explicit type annotation
    let store: NetabaseStore<BlogDefinition, _> = NetabaseStore::temp()?;

    // Create some entities
    let author = Author {
        id: 1,
        name: "Alice Johnson".to_string(),
        email: "alice@example.com".to_string(),
        bio: "Tech writer and blogger".to_string(),
    };

    let category = Category {
        id: 1,
        name: "Technology".to_string(),
        description: "Posts about technology trends".to_string(),
    };

    let tag1 = Tag {
        id: 1,
        name: "Rust".to_string(),
        color: "#CE422B".to_string(),
    };

    let tag2 = Tag {
        id: 2,
        name: "Database".to_string(),
        color: "#4A90E2".to_string(),
    };

    // Insert the linked entities first
    println!("1. Inserting linked entities:");

    let author_tree = store.open_tree::<Author>();
    author_tree.put(author.clone())?;
    println!("   ✓ Inserted author: {}", author.name);

    let category_tree = store.open_tree::<Category>();
    category_tree.put(category.clone())?;
    println!("   ✓ Inserted category: {}", category.name);

    let tag_tree = store.open_tree::<Tag>();
    tag_tree.put(tag1.clone())?;
    tag_tree.put(tag2.clone())?;
    println!("   ✓ Inserted tags: {} and {}", tag1.name, tag2.name);

    // Create a post with relational links
    let post = Post {
        id: 1,
        title: "Building Databases with Rust".to_string(),
        content: "In this post, we'll explore how to build efficient databases using Rust..."
            .to_string(),

        // Using key-based links (referencing existing entities)
        author: RelationalLink::from_key(author.primary_key()),
        category: RelationalLink::from_key(category.primary_key()),
        tags: vec![
            RelationalLink::from_key(tag1.primary_key()),
            RelationalLink::from_key(tag2.primary_key()),
        ],

        created_at: 1640995200, // 2022-01-01
        published: true,
    };

    println!("\n2. Inserting post with relational links:");
    let post_tree = store.open_tree::<Post>();
    post_tree.put(post.clone())?;
    println!("   ✓ Inserted post: {}", post.title);

    // Retrieve and display the post with its linked data
    println!("\n3. Retrieving post and resolving links:");
    if let Some(retrieved_post) = post_tree.get(post.primary_key())? {
        println!("   Retrieved post: {}", retrieved_post.title);

        // Demonstrate link resolution by fetching related entities
        let linked_author_key = retrieved_post.author.key();
        if let Some(linked_author) = author_tree.get(linked_author_key)? {
            println!("   ✓ Linked author: {}", linked_author.name);
        }

        let linked_category_key = retrieved_post.category.key();
        if let Some(linked_category) = category_tree.get(linked_category_key)? {
            println!("   ✓ Linked category: {}", linked_category.name);
        }

        println!("   ✓ Linked tags:");
        for tag_link in &retrieved_post.tags {
            let tag_key = tag_link.key();
            if let Some(linked_tag) = tag_tree.get(tag_key)? {
                println!("     - {}: {}", linked_tag.name, linked_tag.color);
            }
        }
    }

    // Create a comment that links to the post
    let comment = Comment {
        id: 1,
        content: "Great post! Very informative about Rust databases.".to_string(),
        post: RelationalLink::from_key(post.primary_key()),
        author: RelationalLink::from_key(author.primary_key()),
        created_at: 1641000000,
        approved: true,
    };

    println!("\n4. Inserting comment with links:");
    let comment_tree = store.open_tree::<Comment>();
    comment_tree.put(comment.clone())?;
    println!("   ✓ Inserted comment: {}", comment.content);

    // Retrieve and show the comment with its linked post
    if let Some(retrieved_comment) = comment_tree.get(comment.primary_key())? {
        println!("   Retrieved comment: {}", retrieved_comment.content);

        let post_key = retrieved_comment.post.key();
        if let Some(linked_post) = post_tree.get(post_key)? {
            println!("   ✓ Comment is on post: {}", linked_post.title);
        }

        let author_key = retrieved_comment.author.key();
        if let Some(comment_author) = author_tree.get(author_key)? {
            println!("   ✓ Comment author: {}", comment_author.name);
        }
    }

    println!("\n5. Demonstrating different RelationalLink variants:");

    // Create a post using entity links (embedding the full entity)
    let embedded_author = Author {
        id: 2,
        name: "Bob Smith".to_string(),
        email: "bob@example.com".to_string(),
        bio: "Database expert".to_string(),
    };

    let post_with_embedded = Post {
        id: 2,
        title: "Advanced Database Techniques".to_string(),
        content: "Deep dive into database optimization...".to_string(),
        author: RelationalLink::Entity(embedded_author.clone()),
        category: RelationalLink::from_key(category.primary_key()),
        tags: vec![RelationalLink::from_key(tag1.primary_key())],
        created_at: 1641081600,
        published: true,
    };

    post_tree.put(post_with_embedded.clone())?;
    println!("   ✓ Inserted post with embedded author entity");

    // When we retrieve this post, we can access the embedded author directly
    if let Some(post_with_embedded_retrieved) = post_tree.get(post_with_embedded.primary_key())? {
        match &post_with_embedded_retrieved.author {
            RelationalLink::Entity(embedded_author) => {
                println!("   ✓ Retrieved embedded author: {}", embedded_author.name);
            }
            RelationalLink::Reference(_) => {
                println!("   ✓ Author is stored as key reference");
            }
        }
    }

    println!("\n✅ Relational links example completed successfully!");
    Ok(())
}
