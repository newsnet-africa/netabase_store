#![allow(deprecated)]

/// Comprehensive test that validates README examples work correctly.
///
/// This test file verifies that the core CRUD and query functionality works.
/// Uses the boilerplate `Definition` with `User` and `Post` models.
mod common;

use netabase_store::query::QueryResult;
use netabase_store::relational::RelationalLink;
use netabase_store_examples::boilerplate_lib::definition::{
    AnotherLargeUserFile, LargeUserFile, Post, PostID, User, UserID,
};
use netabase_store_examples::boilerplate_lib::{CategoryID, Definition};

// ============================================================================
// Quick Start Example
// ============================================================================

#[test]
fn test_readme_quick_start() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<Definition>("readme_quick_start")?;

    // Create records in a write transaction
    {
        let txn = store.begin_write()?;

        let user = User {
            id: UserID("user1".into()),
            name: "alice".into(),
            age: 30,
            partner: RelationalLink::new_dehydrated(UserID("partner1".into())),
            category: RelationalLink::new_dehydrated(CategoryID("cat1".into())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile::default(),
            subscriptions: vec![],
        };

        txn.create(&user)?;

        let post = Post {
            id: PostID("post1".into()),
            title: "Hello World".into(),
            author_id: "user1".into(),
            content: "My first post!".into(),
            published: true,
            subscriptions: vec![],
        };

        txn.create(&post)?;
        txn.commit()?;
    }

    // Query records in a read transaction
    {
        let txn = store.begin_read()?;

        // Read by primary key
        let user: Option<User> = txn.read(&UserID("user1".into()))?;
        assert!(user.is_some());
        assert_eq!(user.as_ref().unwrap().name, "alice");

        // Read posts
        let post: Option<Post> = txn.read(&PostID("post1".into()))?;
        assert!(post.is_some());
        assert_eq!(post.as_ref().unwrap().title, "Hello World");
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

// ============================================================================
// CRUD Operations
// ============================================================================

#[test]
fn test_readme_crud_operations() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<Definition>("readme_crud")?;

    // Create
    {
        let txn = store.begin_write()?;

        let user = User {
            id: UserID("user1".into()),
            name: "Test User".into(),
            age: 25,
            partner: RelationalLink::new_dehydrated(UserID("partner1".into())),
            category: RelationalLink::new_dehydrated(CategoryID("cat1".into())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile::default(),
            subscriptions: vec![],
        };

        txn.create(&user)?;
        txn.commit()?;
    }

    // Read
    {
        let txn = store.begin_read()?;

        let user: Option<User> = txn.read(&UserID("user1".into()))?;
        assert!(user.is_some());
        assert_eq!(user.unwrap().name, "Test User");
    }

    // Update
    {
        let txn = store.begin_write()?;

        let mut user: User = txn.read(&UserID("user1".into()))?.expect("User not found");
        user.name = "Updated User".into();
        user.age = 26;

        txn.update(&user)?;
        txn.commit()?;
    }

    // Verify update
    {
        let txn = store.begin_read()?;

        let user: User = txn.read(&UserID("user1".into()))?.unwrap();
        assert_eq!(user.name, "Updated User");
        assert_eq!(user.age, 26);
    }

    // Delete
    {
        let txn = store.begin_write()?;
        txn.delete::<User>(&UserID("user1".into()))?;
        txn.commit()?;
    }

    // Verify deletion
    {
        let txn = store.begin_read()?;
        let user: Option<User> = txn.read(&UserID("user1".into()))?;
        assert!(user.is_none());
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

// ============================================================================
// Query Results
// ============================================================================

#[test]
fn test_readme_query_results() {
    // Single variant
    let single = QueryResult::Single(Some(42));
    assert_eq!(single.len(), 1);
    assert!(!single.is_empty());
    assert_eq!(single.as_single(), Some(&42));
    assert_eq!(single.clone().unwrap_single(), 42);

    // Multiple variant
    let multiple = QueryResult::Multiple(vec![1, 2, 3, 4, 5]);
    assert_eq!(multiple.len(), 5);
    assert!(!multiple.is_empty());
    assert_eq!(multiple.as_multiple(), Some(&vec![1, 2, 3, 4, 5]));

    // Count variant
    let count: QueryResult<i32> = QueryResult::Count(100);
    assert_eq!(count.len(), 100);
    assert!(!count.is_empty());
    assert_eq!(count.count(), Some(100));

    // Conversion to vec
    let vec = QueryResult::Multiple(vec![10, 20, 30]);
    assert_eq!(vec.into_vec(), vec![10, 20, 30]);
}
