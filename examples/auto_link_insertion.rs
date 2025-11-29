//! Example demonstrating automatic insertion of linked entities
//!
//! This example shows how the enhanced NetabaseModel derive macro automatically
//! generates code to insert linked entities when a model with RelationalLink fields
//! is inserted into the database.

use netabase_store::links::RelationalLink;
use netabase_store::store_ops::StoreOps;
use netabase_store::*;
use std::error::Error;

// Define our database schema
#[netabase_definition_module(BlogDefinition, BlogDefinitionKey)]
mod blog_schema {
    use super::*;

    /// Author model - this will be linked by Post
    #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode, PartialEq)]
    #[netabase(BlogDefinition)]
    pub struct Author {
        #[primary_key]
        pub id: u64,
        pub name: String,
        pub email: String,
        pub bio: String,
    }

    /// Category model - this will be linked by Post
    #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode, PartialEq)]
    #[netabase(BlogDefinition)]
    pub struct Category {
        #[primary_key]
        pub id: u64,
        pub name: String,
        pub description: String,
    }

    /// Tag model - this will be linked by Post
    #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode, PartialEq)]
    #[netabase(BlogDefinition)]
    pub struct Tag {
        #[primary_key]
        pub id: u64,
        pub name: String,
        pub color: String,
    }

    /// Post model with RelationalLink fields
    /// The derive macro will automatically generate insertion methods that handle linked entities
    #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode, PartialEq)]
    #[netabase(BlogDefinition)]
    pub struct Post {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,

        // RelationalLink fields - these will trigger automatic link insertion
        pub author: RelationalLink<BlogDefinition, Author>,
        pub category: RelationalLink<BlogDefinition, Category>,
        pub tags: Vec<RelationalLink<BlogDefinition, Tag>>,

        pub created_at: u64,
        pub published: bool,
    }

    /// Comment model with nested RelationalLink to Post (which has its own links)
    #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode, PartialEq)]
    #[netabase(BlogDefinition)]
    pub struct Comment {
        #[primary_key]
        pub id: u64,
        pub content: String,

        // This creates a chain of linked entities: Comment -> Post -> Author, Category, Tags
        pub post: RelationalLink<BlogDefinition, Post>,
        pub author: RelationalLink<BlogDefinition, Author>,

        pub created_at: u64,
        pub approved: bool,
    }
}

use blog_schema::*;

fn main() -> Result<(), Box<dyn Error>> {
    // Create a temporary database
    let temp_dir = tempfile::tempdir()?;
    let store = SledBackend::open(temp_dir.path(), "blog_db")?;

    // Create some entities with full data (Entity variants)
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

    // Create a post with linked entities
    let post = Post {
        id: 1,
        title: "Building Databases with Rust".to_string(),
        content: "In this post, we'll explore...".to_string(),

        // Using Entity variants - these will be automatically inserted
        author: RelationalLink::Entity(author.clone()),
        category: RelationalLink::Entity(category.clone()),
        tags: vec![
            RelationalLink::Entity(tag1.clone()),
            RelationalLink::Entity(tag2.clone()),
        ],

        created_at: 1640995200, // 2022-01-01
        published: true,
    };

    println!("=== Demonstrating Automatic Link Insertion ===\n");

    // Example 1: Manual insertion (traditional approach)
    println("1. Traditional manual insertion:");
    {
        // Manually insert each linked entity first
        let author_tree = store.open_tree();
        author_tree.put_raw(author.clone())?;
        println!("   ✓ Manually inserted author: {}", author.name);

        let category_tree = store.open_tree();
        category_tree.put_raw(category.clone())?;
        println!("   ✓ Manually inserted category: {}", category.name);

        let tag_tree = store.open_tree();
        tag_tree.put_raw(tag1.clone())?;
        tag_tree.put_raw(tag2.clone())?;
        println!(
            "   ✓ Manually inserted tags: {} and {}",
            tag1.name, tag2.name
        );

        // Finally insert the post
        let post_tree = store.open_tree();
        post_tree.put_raw(post.clone())?;
        println!("   ✓ Manually inserted post: {}", post.title);
    }
    println!();

    // Example 2: Automatic insertion using the generated methods
    println!("2. Automatic insertion with generated link methods:");
    {
        use netabase_store::links::InsertWithLinks;

        // The insert_with_links method was automatically generated by the derive macro
        // It will recursively insert all linked entities before inserting the post
        post.insert_with_links(&store)?;
        println!("   ✓ Automatically inserted post and all linked entities!");

        // Verify that all entities were inserted
        let author_tree = store.open_tree();
        if let Some(stored_author) = author_tree.get_raw(author.id)? {
            println!("   ✓ Verified author was inserted: {}", stored_author.name);
        }

        let category_tree = store.open_tree();
        if let Some(stored_category) = category_tree.get_raw(category.id)? {
            println!(
                "   ✓ Verified category was inserted: {}",
                stored_category.name
            );
        }
    }
    println!();

    // Example 3: Individual field insertion methods
    println!("3. Individual field insertion methods:");
    {
        // The derive macro also generates methods for individual fields

        // Only insert the author if it's an Entity variant
        post.insert_author_if_entity(&store)?;
        println!("   ✓ Conditionally inserted author");

        // Only insert the category if it's an Entity variant
        post.insert_category_if_entity(&store)?;
        println!("   ✓ Conditionally inserted category");

        // Check if fields contain Entity variants (compile-time + runtime)
        if post.is_author_entity() {
            println!("   ✓ Author field contains Entity variant");
        }

        if post.is_category_entity() {
            println!("   ✓ Category field contains Entity variant");
        }
    }
    println!();

    // Example 4: Nested link insertion (Comment -> Post -> Author/Category/Tags)
    println!("4. Nested link insertion:");
    {
        let comment = Comment {
            id: 1,
            content: "Great post! Very informative.".to_string(),

            // Link to the post (which has its own links)
            post: RelationalLink::Entity(post.clone()),

            // Link to the same author
            author: RelationalLink::Entity(author.clone()),

            created_at: 1641000000,
            approved: true,
        };

        use netabase_store::links::InsertWithLinks;

        // This will recursively insert:
        // 1. The comment's linked post (if it's an Entity)
        // 2. The post's linked author, category, and tags (if they're Entities)
        // 3. The comment's linked author (if it's an Entity)
        // 4. Finally, the comment itself
        comment.insert_with_links(&store)?;
        println!("   ✓ Inserted comment with nested entity links!");

        // Verify the comment was inserted
        let comment_tree = store.open_tree();
        if let Some(stored_comment) = comment_tree.get_raw(comment.id)? {
            println!(
                "   ✓ Verified comment was inserted: {}",
                stored_comment.content.chars().take(30).collect::<String>()
            );
        }
    }
    println!();

    // Example 5: Reference-only links (no insertion needed)
    println!("5. Reference-only links:");
    {
        let post_with_refs = Post {
            id: 2,
            title: "Another Post".to_string(),
            content: "This post uses references...".to_string(),

            // Using Reference variants - these won't trigger insertion
            author: RelationalLink::from_key(author.id),
            category: RelationalLink::from_key(category.id),
            tags: vec![
                RelationalLink::from_key(tag1.id),
                RelationalLink::from_key(tag2.id),
            ],

            created_at: 1641081600,
            published: true,
        };

        use netabase_store::links::InsertWithLinks;

        // Only the post itself will be inserted (linked entities are references)
        post_with_refs.insert_with_links(&store)?;
        println!("   ✓ Inserted post with reference-only links (no linked entity insertion)");

        // Verify only the post was inserted (linked entities already existed)
        let post_tree = store.open_tree();
        if let Some(stored_post) = post_tree.get_raw(post_with_refs.id)? {
            println!(
                "   ✓ Verified reference-only post was inserted: {}",
                stored_post.title
            );
        }
    }
    println!();

    // Example 6: Using the link insertion macros directly
    println!("6. Direct macro usage:");
    {
        let another_author = Author {
            id: 2,
            name: "Bob Smith".to_string(),
            email: "bob@example.com".to_string(),
            bio: "Database expert".to_string(),
        };

        let author_link = RelationalLink::Entity(another_author);

        // Use the generated macro for conditional insertion
        insert_if_relational_link!(&author_link, &store, Author, BlogDefinition);

        println!("   ✓ Used insert_if_relational_link! macro directly");

        // Check if it's a RelationalLink type at compile time
        if is_relational_link!(RelationalLink<BlogDefinition, Author>) {
            println!("   ✓ Compile-time type check confirmed RelationalLink");
        }
    }

    println!("\n=== Summary ===");
    println!("The enhanced NetabaseModel derive macro automatically generated:");
    println!("• InsertWithLinks trait implementation");
    println!("• Individual field insertion methods (insert_*_if_entity)");
    println!("• Field type checking methods (is_*_entity)");
    println!("• Support for nested link insertion");
    println!("• Compile-time conditional code based on actual model types");
    println!("• Helper macros for manual link insertion");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_type_detection() {
        let author = Author {
            id: 1,
            name: "Test Author".to_string(),
            email: "test@example.com".to_string(),
            bio: "Test bio".to_string(),
        };

        // Test Entity variant
        let entity_link = RelationalLink::Entity(author.clone());
        assert!(matches!(entity_link, RelationalLink::Entity(_)));

        // Test Reference variant
        let ref_link = RelationalLink::from_key(1u64);
        assert!(matches!(ref_link, RelationalLink::Reference(_)));
    }

    #[test]
    fn test_automatic_insertion() -> Result<(), Box<dyn Error>> {
        let temp_dir = tempfile::tempdir()?;
        let store = SledBackend::open(temp_dir.path(), "test_db")?;

        let author = Author {
            id: 1,
            name: "Test Author".to_string(),
            email: "test@example.com".to_string(),
            bio: "Test bio".to_string(),
        };

        let post = Post {
            id: 1,
            title: "Test Post".to_string(),
            content: "Test content".to_string(),
            author: RelationalLink::Entity(author.clone()),
            category: RelationalLink::from_key(1u64), // Reference only
            tags: vec![],
            created_at: 0,
            published: true,
        };

        // Use the automatically generated insertion method
        use netabase_store::links::InsertWithLinks;
        post.insert_with_links(&store)?;

        // Verify both post and linked author were inserted
        let post_tree = store.open_tree();
        let stored_post = post_tree
            .get_raw(post.id)?
            .expect("Post should be inserted");
        assert_eq!(stored_post.title, post.title);

        let author_tree = store.open_tree();
        let stored_author = author_tree
            .get_raw(author.id)?
            .expect("Author should be inserted");
        assert_eq!(stored_author.name, author.name);

        Ok(())
    }
}
