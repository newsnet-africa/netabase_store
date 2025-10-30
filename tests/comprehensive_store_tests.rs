//! Comprehensive tests for netabase_store
//!
//! These tests verify that netabase_store works correctly as a standalone
//! crate without requiring netabase or paxos features.

#![cfg(feature = "sled")]

use netabase_store::{netabase, netabase_definition_module, NetabaseModel};
use netabase_store::databases::sled_store::SledStore;
use netabase_store::traits::tree::NetabaseTreeSync;
use netabase_store::traits::model::NetabaseModelTrait;

/// Test definition module with multiple models
#[netabase_definition_module(BlogDefinition, BlogKeys)]
mod blog {
    use super::*;

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
        #[secondary_key]
        pub email: String,
        pub created_at: u64,
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
        #[secondary_key]
        pub author_id: u64,
        pub created_at: u64,
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
    pub struct Comment {
        #[primary_key]
        pub id: u64,
        pub content: String,
        #[secondary_key]
        pub post_id: u64,
        #[secondary_key]
        pub author_id: u64,
        pub created_at: u64,
    }
}

use blog::*;

// ============================================================================
// Basic CRUD Operations
// ============================================================================

#[test]
fn test_create_and_read_user() {
    let db = SledStore::<BlogDefinition>::temp().unwrap();
    let tree = db.open_tree::<User>();

    let user = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        created_at: 1234567890,
    };

    // Create
    tree.put(user.clone()).unwrap();

    // Read
    let retrieved = tree.get(user.primary_key()).unwrap().unwrap();
    assert_eq!(retrieved, user);
    assert_eq!(retrieved.username, "alice");
    assert_eq!(retrieved.email, "alice@example.com");
}

#[test]
fn test_update_user() {
    let db = SledStore::<BlogDefinition>::temp().unwrap();
    let tree = db.open_tree::<User>();

    let mut user = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        created_at: 1234567890,
    };

    tree.put(user.clone()).unwrap();

    // Update
    user.email = "alice.new@example.com".to_string();
    tree.put(user.clone()).unwrap();

    // Verify update
    let retrieved = tree.get(user.primary_key()).unwrap().unwrap();
    assert_eq!(retrieved.email, "alice.new@example.com");
}

#[test]
fn test_delete_user() {
    let db = SledStore::<BlogDefinition>::temp().unwrap();
    let tree = db.open_tree::<User>();

    let user = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        created_at: 1234567890,
    };

    tree.put(user.clone()).unwrap();

    // Delete
    tree.remove(user.primary_key()).unwrap();

    // Verify deletion
    let retrieved = tree.get(user.primary_key()).unwrap();
    assert!(retrieved.is_none());
}

#[test]
fn test_multiple_models() {
    let db = SledStore::<BlogDefinition>::temp().unwrap();

    // Create instances
    let user = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        created_at: 1234567890,
    };

    let post = Post {
        id: 100,
        title: "Hello World".to_string(),
        content: "This is my first post".to_string(),
        author_id: 1,
        created_at: 1234567891,
    };

    let comment = Comment {
        id: 1000,
        content: "Great post!".to_string(),
        post_id: 100,
        author_id: 1,
        created_at: 1234567892,
    };

    // Store in different trees
    db.open_tree::<User>().put(user.clone()).unwrap();
    db.open_tree::<Post>().put(post.clone()).unwrap();
    db.open_tree::<Comment>().put(comment.clone()).unwrap();

    // Retrieve from different trees
    let retrieved_user = db.open_tree::<User>().get(user.primary_key()).unwrap().unwrap();
    let retrieved_post = db.open_tree::<Post>().get(post.primary_key()).unwrap().unwrap();
    let retrieved_comment = db.open_tree::<Comment>().get(comment.primary_key()).unwrap().unwrap();

    assert_eq!(retrieved_user, user);
    assert_eq!(retrieved_post, post);
    assert_eq!(retrieved_comment, comment);
}

// ============================================================================
// Secondary Key Queries
// ============================================================================

#[test]
fn test_secondary_key_query_single_result() {
    let db = SledStore::<BlogDefinition>::temp().unwrap();
    let tree = db.open_tree::<User>();

    let user = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        created_at: 1234567890,
    };

    tree.put(user.clone()).unwrap();

    // Query by secondary key
    let results = tree
        .get_by_secondary_key(UserSecondaryKeys::Email(EmailSecondaryKey(
            "alice@example.com".to_string(),
        )))
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0], user);
}

#[test]
fn test_secondary_key_query_multiple_results() {
    let db = SledStore::<BlogDefinition>::temp().unwrap();
    let tree = db.open_tree::<Post>();

    // Create multiple posts by same author
    for i in 1..=5 {
        let post = Post {
            id: i,
            title: format!("Post {}", i),
            content: format!("Content {}", i),
            author_id: 1, // Same author
            created_at: 1234567890 + i,
        };
        tree.put(post).unwrap();
    }

    // Query all posts by author
    let results = tree
        .get_by_secondary_key(PostSecondaryKeys::AuthorId(AuthorIdSecondaryKey(1)))
        .unwrap();

    assert_eq!(results.len(), 5);

    // Verify all posts belong to author 1
    for post in results {
        assert_eq!(post.author_id, 1);
    }
}

#[test]
fn test_secondary_key_query_no_results() {
    let db = SledStore::<BlogDefinition>::temp().unwrap();
    let tree = db.open_tree::<User>();

    // Query non-existent email
    let results = tree
        .get_by_secondary_key(UserSecondaryKeys::Email(EmailSecondaryKey(
            "nonexistent@example.com".to_string(),
        )))
        .unwrap();

    assert_eq!(results.len(), 0);
}

#[test]
fn test_multiple_secondary_keys() {
    let db = SledStore::<BlogDefinition>::temp().unwrap();
    let tree = db.open_tree::<Comment>();

    // Create comments with different post_id and author_id combinations
    let comment1 = Comment {
        id: 1,
        content: "Comment 1".to_string(),
        post_id: 100,
        author_id: 1,
        created_at: 1234567890,
    };

    let comment2 = Comment {
        id: 2,
        content: "Comment 2".to_string(),
        post_id: 100,
        author_id: 2,
        created_at: 1234567891,
    };

    let comment3 = Comment {
        id: 3,
        content: "Comment 3".to_string(),
        post_id: 200,
        author_id: 1,
        created_at: 1234567892,
    };

    tree.put(comment1.clone()).unwrap();
    tree.put(comment2.clone()).unwrap();
    tree.put(comment3.clone()).unwrap();

    // Query by post_id
    let post_comments = tree
        .get_by_secondary_key(CommentSecondaryKeys::PostId(PostIdSecondaryKey(100)))
        .unwrap();
    assert_eq!(post_comments.len(), 2);

    // Query by author_id
    let author_comments = tree
        .get_by_secondary_key(CommentSecondaryKeys::AuthorId(AuthorIdSecondaryKey(1)))
        .unwrap();
    assert_eq!(author_comments.len(), 2);
}

// ============================================================================
// Batch Operations
// ============================================================================

#[test]
fn test_batch_insert() {
    let db = SledStore::<BlogDefinition>::temp().unwrap();
    let tree = db.open_tree::<User>();

    // Insert 100 users
    for i in 1..=100 {
        let user = User {
            id: i,
            username: format!("user{}", i),
            email: format!("user{}@example.com", i),
            created_at: 1234567890 + i,
        };
        tree.put(user).unwrap();
    }

    // Verify all were inserted
    for i in 1..=100 {
        let retrieved = tree.get(UserPrimaryKey(i)).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, i);
    }
}

#[test]
fn test_batch_delete() {
    let db = SledStore::<BlogDefinition>::temp().unwrap();
    let tree = db.open_tree::<Post>();

    // Insert posts
    for i in 1..=50 {
        let post = Post {
            id: i,
            title: format!("Post {}", i),
            content: format!("Content {}", i),
            author_id: 1,
            created_at: 1234567890 + i,
        };
        tree.put(post).unwrap();
    }

    // Delete first 25
    for i in 1..=25 {
        tree.remove(PostPrimaryKey(i)).unwrap();
    }

    // Verify deletions
    for i in 1..=25 {
        assert!(tree.get(PostPrimaryKey(i)).unwrap().is_none());
    }

    // Verify remaining
    for i in 26..=50 {
        assert!(tree.get(PostPrimaryKey(i)).unwrap().is_some());
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_empty_database() {
    let db = SledStore::<BlogDefinition>::temp().unwrap();
    let tree = db.open_tree::<User>();

    // Query non-existent record
    let result = tree.get(UserPrimaryKey(999)).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_duplicate_primary_key_overwrites() {
    let db = SledStore::<BlogDefinition>::temp().unwrap();
    let tree = db.open_tree::<User>();

    let user1 = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        created_at: 1234567890,
    };

    let user2 = User {
        id: 1, // Same ID
        username: "alice_updated".to_string(),
        email: "alice_new@example.com".to_string(),
        created_at: 1234567891,
    };

    tree.put(user1).unwrap();
    tree.put(user2.clone()).unwrap();

    // Should have overwritten
    let retrieved = tree.get(UserPrimaryKey(1)).unwrap().unwrap();
    assert_eq!(retrieved, user2);
    assert_eq!(retrieved.username, "alice_updated");
}

#[test]
fn test_large_string_fields() {
    let db = SledStore::<BlogDefinition>::temp().unwrap();
    let tree = db.open_tree::<Post>();

    let large_content = "a".repeat(10_000); // 10KB content

    let post = Post {
        id: 1,
        title: "Large Post".to_string(),
        content: large_content.clone(),
        author_id: 1,
        created_at: 1234567890,
    };

    tree.put(post.clone()).unwrap();

    let retrieved = tree.get(post.primary_key()).unwrap().unwrap();
    assert_eq!(retrieved.content.len(), 10_000);
    assert_eq!(retrieved.content, large_content);
}

#[test]
fn test_unicode_strings() {
    let db = SledStore::<BlogDefinition>::temp().unwrap();
    let tree = db.open_tree::<User>();

    let user = User {
        id: 1,
        username: "用户名".to_string(), // Chinese
        email: "ユーザー@example.com".to_string(), // Japanese
        created_at: 1234567890,
    };

    tree.put(user.clone()).unwrap();

    let retrieved = tree.get(user.primary_key()).unwrap().unwrap();
    assert_eq!(retrieved.username, "用户名");
    assert_eq!(retrieved.email, "ユーザー@example.com");
}

#[test]
fn test_zero_values() {
    let db = SledStore::<BlogDefinition>::temp().unwrap();
    let tree = db.open_tree::<User>();

    let user = User {
        id: 0, // Zero ID
        username: "".to_string(), // Empty username
        email: "test@example.com".to_string(),
        created_at: 0, // Zero timestamp
    };

    tree.put(user.clone()).unwrap();

    let retrieved = tree.get(UserPrimaryKey(0)).unwrap().unwrap();
    assert_eq!(retrieved.id, 0);
    assert_eq!(retrieved.username, "");
    assert_eq!(retrieved.created_at, 0);
}

// ============================================================================
// Concurrency (Single-threaded sequential tests)
// ============================================================================

#[test]
fn test_sequential_reads_and_writes() {
    let db = SledStore::<BlogDefinition>::temp().unwrap();
    let tree = db.open_tree::<User>();

    // Interleave reads and writes
    for i in 1..=10 {
        let user = User {
            id: i,
            username: format!("user{}", i),
            email: format!("user{}@example.com", i),
            created_at: 1234567890 + i,
        };
        tree.put(user).unwrap();

        // Read immediately after write
        let retrieved = tree.get(UserPrimaryKey(i)).unwrap().unwrap();
        assert_eq!(retrieved.id, i);
    }
}

// ============================================================================
// Summary Test
// ============================================================================

#[test]
fn test_comprehensive_workflow() {
    println!("\n🧪 Running comprehensive workflow test...\n");

    let db = SledStore::<BlogDefinition>::temp().unwrap();

    // 1. Create a user
    let user = User {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        created_at: 1234567890,
    };
    db.open_tree::<User>().put(user.clone()).unwrap();
    println!("✅ Created user: {}", user.username);

    // 2. Create multiple posts
    for i in 1..=3 {
        let post = Post {
            id: i,
            title: format!("Post {}", i),
            content: format!("Content for post {}", i),
            author_id: 1,
            created_at: 1234567890 + i,
        };
        db.open_tree::<Post>().put(post).unwrap();
        println!("✅ Created post {}", i);
    }

    // 3. Create comments on posts
    for post_id in 1..=3 {
        for comment_id in 1..=2 {
            let comment = Comment {
                id: (post_id * 10) + comment_id,
                content: format!("Comment {} on post {}", comment_id, post_id),
                post_id,
                author_id: 1,
                created_at: 1234567890 + post_id + comment_id,
            };
            db.open_tree::<Comment>().put(comment).unwrap();
            println!("✅ Created comment on post {}", post_id);
        }
    }

    // 4. Query all posts by author
    let posts = db
        .open_tree::<Post>()
        .get_by_secondary_key(PostSecondaryKeys::AuthorId(AuthorIdSecondaryKey(1)))
        .unwrap();
    assert_eq!(posts.len(), 3);
    println!("✅ Queried {} posts by author", posts.len());

    // 5. Query comments by post
    let comments = db
        .open_tree::<Comment>()
        .get_by_secondary_key(CommentSecondaryKeys::PostId(PostIdSecondaryKey(1)))
        .unwrap();
    assert_eq!(comments.len(), 2);
    println!("✅ Queried {} comments for post 1", comments.len());

    // 6. Update user email
    let mut user = db.open_tree::<User>().get(UserPrimaryKey(1)).unwrap().unwrap();
    user.email = "alice.new@example.com".to_string();
    db.open_tree::<User>().put(user).unwrap();
    println!("✅ Updated user email");

    // 7. Delete a post
    db.open_tree::<Post>().remove(PostPrimaryKey(2)).unwrap();
    println!("✅ Deleted post 2");

    // 8. Verify deletion
    let deleted_post = db.open_tree::<Post>().get(PostPrimaryKey(2)).unwrap();
    assert!(deleted_post.is_none());
    println!("✅ Verified post deletion");

    println!("\n🎉 Comprehensive workflow test PASSED!\n");
}
