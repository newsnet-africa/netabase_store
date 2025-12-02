//! Example demonstrating relation insertion with custom names
//!
//! This example shows how to:
//! - Define models with RelationalLink fields
//! - Use custom relation names with the #[relation(name)] attribute
//! - Insert models with their related entities

use netabase_store::{
    NetabaseModel, NetabaseStore,
    links::RelationalLink,
    netabase_definition_module,
};

// Define our models with relations
#[netabase_definition_module(BlogDefinition, BlogKeys)]
mod blog_models {
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
    #[netabase(BlogDefinition)]
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
    #[netabase(BlogDefinition)]
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
    #[netabase(BlogDefinition)]
    pub struct Post {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,

        // Author of the post with custom relation name
        #[relation(post_author)]
        pub author: RelationalLink<BlogDefinition, User>,

        // Category of the post
        #[relation(post_category)]
        pub category: RelationalLink<BlogDefinition, Category>,
    }
}

use blog_models::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Recursive Relations Example ===\n");

    // Create a temporary store
    let store = NetabaseStore::temp()?;

    // Create sample data
    let user = User {
        id: 1,
        name: "Alice Johnson".to_string(),
        email: "alice@example.com".to_string(),
    };

    let category = Category {
        id: 1,
        name: "Technology".to_string(),
        description: "Tech-related posts".to_string(),
    };

    // Create a post with both author and category as entities
    let post = Post {
        id: 1,
        title: "Introduction to Rust".to_string(),
        content: "Rust is a systems programming language...".to_string(),
        author: RelationalLink::Entity(user.clone()),
        category: RelationalLink::Entity(category.clone()),
    };

    println!("1. Testing relation insertion with embedded entities...");

    // Insert the post with its relations
    post.insert_with_relations(&store)?;
    println!("   ✓ Post and all related entities inserted");

    // Verify all entities were inserted
    let user_tree = store.open_tree::<User>();
    let category_tree = store.open_tree::<Category>();
    let post_tree = store.open_tree::<Post>();

    assert!(user_tree.get(UserPrimaryKey(1))?.is_some());
    assert!(category_tree.get(CategoryPrimaryKey(1))?.is_some());
    assert!(post_tree.get(PostPrimaryKey(1))?.is_some());
    println!("   ✓ All entities verified in store");

    println!("\n2. Testing relation insertion with references...");

    // Create another user and category
    let user2 = User {
        id: 2,
        name: "Bob Smith".to_string(),
        email: "bob@example.com".to_string(),
    };

    let category2 = Category {
        id: 2,
        name: "Science".to_string(),
        description: "Scientific posts".to_string(),
    };

    // Insert them first
    user_tree.put(user2.clone())?;
    category_tree.put(category2.clone())?;

    // Create a post with references
    let post2 = Post {
        id: 2,
        title: "Quantum Computing".to_string(),
        content: "An introduction to quantum computing...".to_string(),
        author: RelationalLink::Reference(UserPrimaryKey(2)),
        category: RelationalLink::Reference(CategoryPrimaryKey(2)),
    };

    // Insert the post (references won't be re-inserted)
    post2.insert_with_relations(&store)?;
    println!("   ✓ Post with references inserted");

    // Verify the post
    let retrieved_post = post_tree.get(PostPrimaryKey(2))?.unwrap();
    println!("   ✓ Post retrieved: {}", retrieved_post.title);

    println!("\n3. Testing mixed relations (entity + reference)...");

    let user3 = User {
        id: 3,
        name: "Charlie Brown".to_string(),
        email: "charlie@example.com".to_string(),
    };

    // Mix: new author as entity, existing category as reference
    let post3 = Post {
        id: 3,
        title: "Machine Learning Basics".to_string(),
        content: "An overview of ML concepts...".to_string(),
        author: RelationalLink::Entity(user3),
        category: RelationalLink::Reference(CategoryPrimaryKey(1)), // Reuse Technology category
    };

    post3.insert_with_relations(&store)?;
    println!("   ✓ Post with mixed relations inserted");

    // Verify
    assert!(user_tree.get(UserPrimaryKey(3))?.is_some());
    assert!(post_tree.get(PostPrimaryKey(3))?.is_some());
    println!("   ✓ Mixed relations verified");

    println!("\n=== Test Summary ===");
    println!("• Entity insertion: ✓");
    println!("• Reference insertion: ✓");
    println!("• Mixed relations: ✓");
    println!("• Custom relation names: ✓");

    Ok(())
}
