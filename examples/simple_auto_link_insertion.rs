//! Simple example demonstrating automatic insertion of linked entities
//!
//! This example shows how the enhanced NetabaseModel derive macro automatically
//! generates code to insert linked entities when a model with RelationalLink fields
//! is inserted into the database.

use netabase_store::databases::sled_store::SledStore;
use netabase_store::store_ops::StoreOps;
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
    }

    /// Post model with RelationalLink fields
    /// The derive macro will automatically generate insertion methods that handle linked entities
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

        // RelationalLink field - this will trigger automatic link insertion
        pub author: RelationalLink<BlogDefinition, Author>,
    }
}

use blog_schema::*;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Simple Auto Link Insertion Example ===\n");

    // Create a temporary database
    let temp_dir = tempfile::tempdir()?;
    let store = SledStore::new(temp_dir.path().join("simple_blog_db"))?;

    // Create an author entity
    let author = Author {
        id: 1,
        name: "Alice Johnson".to_string(),
        email: "alice@example.com".to_string(),
    };

    // Create a post that references the author as an Entity (not just a key reference)
    let post = Post {
        id: 1,
        title: "My First Post".to_string(),
        content: "This is my first blog post!".to_string(),

        // Using Entity variant - this will be automatically inserted
        author: RelationalLink::Entity(author.clone()),
    };

    println!("1. Traditional approach - manual insertion:");
    {
        // Manually insert the author first
        let author_tree = store.open_tree();
        author_tree.put_raw(author.clone())?;
        println!("   ✓ Manually inserted author: {}", author.name);

        // Then insert the post
        let post_tree = store.open_tree();
        post_tree.put_raw(post.clone())?;
        println!("   ✓ Manually inserted post: {}", post.title);
    }
    println!();

    println!("2. Enhanced approach - automatic link insertion:");
    {
        use netabase_store::links::InsertWithLinks;

        // The derive macro generated insert_with_links method
        // It automatically detects RelationalLink::Entity variants and inserts them
        match post.insert_with_links(&store) {
            Ok(_) => {
                println!("   ✓ Successfully used automatic link insertion!");
                println!("   ✓ Post and linked author were both inserted automatically");
            }
            Err(e) => {
                println!("   ✗ Error with automatic insertion: {}", e);
                // Fallback to manual insertion
                println!("   → Falling back to manual insertion approach");

                let author_tree = store.open_tree();
                author_tree.put_raw(author.clone())?;

                let post_tree = store.open_tree();
                post_tree.put_raw(post.clone())?;

                println!("   ✓ Manual fallback completed");
            }
        }
    }
    println!();

    println!("3. Using compile-time generated helper methods:");
    {
        // The derive macro also generated field-specific methods

        // Check if the author field contains an Entity (compile-time + runtime check)
        if post.is_author_entity() {
            println!("   ✓ Author field contains Entity variant");

            // Insert only the author if it's an Entity
            if let Err(e) = post.insert_author_if_entity(&store) {
                println!("   ✗ Error inserting author: {}", e);
            } else {
                println!("   ✓ Author inserted using field-specific method");
            }
        }
    }
    println!();

    println!("4. Reference-only links (no insertion needed):");
    {
        let post_with_ref = Post {
            id: 2,
            title: "Second Post".to_string(),
            content: "This post uses a reference...".to_string(),

            // Using Reference variant - this won't trigger insertion
            author: RelationalLink::from_key(AuthorPrimaryKey(1u64)),
        };

        use netabase_store::links::InsertWithLinks;

        // Only the post itself will be inserted (author is just a reference)
        match post_with_ref.insert_with_links(&store) {
            Ok(_) => println!("   ✓ Post with reference-only link inserted (no author insertion)"),
            Err(e) => {
                println!("   ✗ Error: {}", e);
                // Manual fallback
                let post_tree = store.open_tree();
                post_tree.put_raw(post_with_ref)?;
                println!("   ✓ Manual insertion completed");
            }
        }
    }
    println!();

    println!("5. Verification:");
    {
        // Verify all entities were inserted correctly
        let author_tree = store.open_tree();
        if let Some(stored_author) = author_tree.get_raw(AuthorPrimaryKey(1u64))? {
            println!("   ✓ Author found in database: {}", stored_author.name);
        } else {
            println!("   ✗ Author not found in database");
        }

        let post_tree = store.open_tree();
        if let Some(stored_post) = post_tree.get_raw(PostPrimaryKey(1u64))? {
            println!("   ✓ First post found in database: {}", stored_post.title);
        } else {
            println!("   ✗ First post not found in database");
        }

        if let Some(stored_post2) = post_tree.get_raw(PostPrimaryKey(2u64))? {
            println!("   ✓ Second post found in database: {}", stored_post2.title);
        } else {
            println!("   ✗ Second post not found in database");
        }
    }

    println!("\n=== Summary ===");
    println!("The enhanced NetabaseModel derive macro provides:");
    println!("• Automatic detection of RelationalLink<D, M> fields");
    println!("• InsertWithLinks trait implementation");
    println!("• Conditional insertion based on Entity vs Reference variants");
    println!("• Individual field insertion methods (insert_*_if_entity)");
    println!("• Field type checking methods (is_*_entity)");
    println!("• Compile-time code generation based on actual field types");
    println!("• Graceful fallback when automatic insertion fails");

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

        // Test conversion
        let entity_from_author: RelationalLink<BlogDefinition, Author> = author.into();
        assert!(matches!(entity_from_author, RelationalLink::Entity(_)));
    }

    #[test]
    fn test_basic_insertion() -> Result<(), Box<dyn Error>> {
        let temp_dir = tempfile::tempdir()?;
        let store = SledBackend::open(temp_dir.path(), "test_db")?;

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

        // Test manual insertion as baseline
        let author_tree = store.open_tree();
        author_tree.put_raw(author.clone())?;

        let post_tree = store.open_tree();
        post_tree.put_raw(post.clone())?;

        // Verify insertion worked
        let stored_author = author_tree.get_raw(1u64)?.expect("Author should exist");
        assert_eq!(stored_author.name, "Test Author");

        let stored_post = post_tree.get_raw(1u64)?.expect("Post should exist");
        assert_eq!(stored_post.title, "Test Post");

        Ok(())
    }

    #[test]
    fn test_generated_methods_exist() {
        let author = Author {
            id: 1,
            name: "Test Author".to_string(),
            email: "test@example.com".to_string(),
        };

        let post = Post {
            id: 1,
            title: "Test Post".to_string(),
            content: "Test content".to_string(),
            author: RelationalLink::Entity(author),
        };

        // These methods should be generated by the derive macro
        assert!(post.is_author_entity());

        // The insert_author_if_entity method should exist (we can't easily test it without a store)
        // but we can at least verify it compiles
    }
}
