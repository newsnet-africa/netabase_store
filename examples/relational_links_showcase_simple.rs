//! Simplified RelationalLink Showcase
//!
//! This example demonstrates RelationalLink functionality with Vec, Option, and Box support

use netabase_store::{
    NetabaseModel, NetabaseStore,
    links::RelationalLink,
    netabase_definition_module,
    traits::store_ops::StoreOps,
};
use std::error::Error;

#[netabase_definition_module(BlogDefinition, BlogKeys)]
mod models {
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
        pub username: String,
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
    pub struct Tag {
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
    #[netabase(BlogDefinition)]
    pub struct Post {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,

        // Single relation
        #[relation(post_author)]
        pub author: RelationalLink<BlogDefinition, User>,

        // Optional relation
        #[relation(post_category)]
        pub category: Option<RelationalLink<BlogDefinition, Category>>,

        // Vec of relations (tags)
        #[relation(post_tags)]
        pub tags: Vec<RelationalLink<BlogDefinition, Tag>>,

        // Box relation
        #[relation(post_editor)]
        pub editor: Box<RelationalLink<BlogDefinition, User>>,
    }
}

use models::*;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== RelationalLink Showcase (Vec/Option/Box Support) ===\n");

    let store = NetabaseStore::temp()?;

    // Create entities
    let author = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
    };

    let editor = User {
        id: 2,
        username: "bob".to_string(),
        email: "bob@example.com".to_string(),
    };

    let category = Category {
        id: 1,
        name: "Technology".to_string(),
        description: "Tech articles".to_string(),
    };

    let tag1 = Tag {
        id: 1,
        name: "rust".to_string(),
    };

    let tag2 = Tag {
        id: 2,
        name: "programming".to_string(),
    };

    println!("1. Testing post with wrapped RelationalLinks...");

    let post = Post {
        id: 1,
        title: "Introduction to Rust".to_string(),
        content: "Rust is a systems programming language...".to_string(),
        author: RelationalLink::Entity(author),
        category: Some(RelationalLink::Entity(category)),
        tags: vec![
            RelationalLink::Entity(tag1),
            RelationalLink::Entity(tag2),
        ],
        editor: Box::new(RelationalLink::Entity(editor)),
    };

    // Note: Full insertion support for Vec/Option/Box is still being developed
    // For now, we can demonstrate the type detection works
    println!("   ✓ Post with wrapped relations created successfully");
    println!("   - Author: RelationalLink (direct)");
    println!("   - Category: Option<RelationalLink>");
    println!("   - Tags: Vec<RelationalLink> ({} tags)", post.tags.len());
    println!("   - Editor: Box<RelationalLink>");

    // Test basic insertion (without automatic relation insertion for Vec/Option/Box yet)
    let post_tree = store.open_tree::<Post>();
    post_tree.put(post.clone())?;
    println!("\n2. Basic insertion works");

    // Retrieve
    let retrieved = post_tree.get(PostPrimaryKey(1))?.expect("Post should exist");
    println!("   ✓ Post retrieved: {}", retrieved.title);

    // Test category (Option<RelationalLink>)
    match &retrieved.category {
        Some(RelationalLink::Entity(cat)) => {
            println!("   ✓ Category (Option, Entity): {}", cat.name);
        }
        Some(RelationalLink::Reference(cat_id)) => {
            println!("   ✓ Category (Option, Reference): {:?}", cat_id);
        }
        None => {
            println!("   ✓ Category (Option): None");
        }
    }

    // Test tags (Vec<RelationalLink>)
    println!("   ✓ Tags (Vec): {} tags", retrieved.tags.len());
    for (i, tag_link) in retrieved.tags.iter().enumerate() {
        match tag_link {
            RelationalLink::Entity(tag) => {
                println!("     - Tag {}: {}", i + 1, tag.name);
            }
            RelationalLink::Reference(tag_id) => {
                println!("     - Tag {} (Reference): {:?}", i + 1, tag_id);
            }
        }
    }

    // Test editor (Box<RelationalLink>)
    match &*retrieved.editor {
        RelationalLink::Entity(ed) => {
            println!("   ✓ Editor (Box, Entity): {}", ed.username);
        }
        RelationalLink::Reference(ed_id) => {
            println!("   ✓ Editor (Box, Reference): {:?}", ed_id);
        }
    }

    println!("\n=== Summary ===");
    println!("✓ Type detection works for Vec/Option/Box<RelationalLink>");
    println!("✓ Models compile with wrapped relations");
    println!("✓ Serialization/deserialization works");
    println!("✓ Basic storage operations work");
    println!("\nNote: Automatic relation insertion for Vec/Option/Box is a future enhancement.");
    println!("Currently, you need to insert related entities manually.");

    Ok(())
}
