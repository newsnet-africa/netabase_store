/// Comprehensive migration system tests
///
/// This test suite validates:
/// - Version header encoding/decoding
/// - Version context usage
/// - Database CRUD with state verification
mod common;

use netabase_store::errors::NetabaseResult;
use netabase_store::traits::migration::{VersionContext, VersionHeader};
use netabase_store_examples::{Definition, Post, PostID};

/// Helper to create a test Post
fn create_post(id: &str, title: &str, content: &str, age: u32) -> Post {
    Post {
        id: PostID(id.to_string()),
        title: title.to_string(),
        author_id: format!("author_{}", age),
        content: content.to_string(),
        published: age > 25,
        subscriptions: vec![],
        tags: vec![],
    }
}

#[test]
fn test_version_header_encoding() {
    let header = VersionHeader::new(42);
    let bytes = header.to_bytes();

    // Check magic bytes
    assert_eq!(bytes[0], b'N');
    assert_eq!(bytes[1], b'V');

    // Check size
    assert_eq!(bytes.len(), VersionHeader::SIZE);

    // Decode and verify
    let decoded = VersionHeader::from_bytes(&bytes).unwrap();
    assert_eq!(decoded.version, 42);
}

#[test]
fn test_version_header_detection() {
    let versioned = VersionHeader::new(1).to_bytes();
    assert!(VersionHeader::is_versioned(&versioned));

    let unversioned = vec![0u8, 1, 2, 3, 4, 5];
    assert!(!VersionHeader::is_versioned(&unversioned));
}

#[test]
fn test_version_context_creation() {
    // Default context
    let ctx = VersionContext::default();
    assert!(ctx.auto_migrate);
    assert!(!ctx.strict);
    assert_eq!(ctx.expected_version, 0);

    // New context with specific version
    let ctx = VersionContext::new(3);
    assert_eq!(ctx.expected_version, 3);
    assert!(ctx.auto_migrate);

    // Strict context
    let strict = VersionContext::strict(5);
    assert_eq!(strict.expected_version, 5);
    assert!(!strict.auto_migrate);
    assert!(strict.strict);

    // Builder pattern
    let custom = VersionContext::new(2).with_auto_migrate(false);
    assert_eq!(custom.expected_version, 2);
    assert!(!custom.auto_migrate);
}

#[test]
fn test_version_context_needs_migration() {
    let mut ctx = VersionContext::new(3);

    // No actual version yet
    assert!(!ctx.needs_migration());

    // Same version - no migration needed
    ctx.actual_version = Some(3);
    assert!(!ctx.needs_migration());

    // Different version - migration needed
    ctx.actual_version = Some(2);
    assert!(ctx.needs_migration());
}

#[test]
fn test_version_context_delta() {
    let mut ctx = VersionContext::new(5);

    // No actual version
    assert_eq!(ctx.version_delta(), 0);

    // Same version
    ctx.actual_version = Some(5);
    assert_eq!(ctx.version_delta(), 0);

    // Upgrade needed (actual < expected)
    ctx.actual_version = Some(3);
    assert_eq!(ctx.version_delta(), 2);

    // Downgrade (actual > expected) - rare
    ctx.actual_version = Some(7);
    assert_eq!(ctx.version_delta(), -2);
}

#[test]
fn test_database_create_and_read_inspection() -> NetabaseResult<()> {
    let (store, db_path) = common::create_test_db::<Definition>("migration_inspect")?;

    let post = create_post("1", "Grace's Post", "Hello world", 28);

    {
        let txn = store.begin_write()?;
        txn.create(&post)?;
        txn.commit()?;
    }

    // Verify after commit
    {
        let txn = store.begin_read()?;
        let read_post = txn.read::<Post>(&PostID("1".to_string()))?;

        assert!(read_post.is_some());
        let read_post = read_post.unwrap();
        assert_eq!(read_post.title, "Grace's Post");
        assert_eq!(read_post.content, "Hello world");
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_database_update_and_verify_state() -> NetabaseResult<()> {
    let (store, db_path) = common::create_test_db::<Definition>("migration_update")?;

    let post_id = PostID("helen".to_string());
    let post = create_post("helen", "Original Title", "Original content", 30);

    // Create initial state
    {
        let txn = store.begin_write()?;
        txn.create(&post)?;
        txn.commit()?;
    }

    // Verify initial state
    {
        let txn = store.begin_read()?;
        let read = txn.read::<Post>(&post_id)?;
        assert_eq!(read.unwrap().title, "Original Title");
    }

    // Update
    {
        let txn = store.begin_write()?;
        let mut post = txn.read::<Post>(&post_id)?.expect("Post not found");
        post.title = "Updated Title".to_string();
        post.content = "Updated content".to_string();
        txn.update(&post)?;
        txn.commit()?;
    }

    // Verify updated state
    {
        let txn = store.begin_read()?;
        let post = txn.read::<Post>(&post_id)?.unwrap();
        assert_eq!(post.title, "Updated Title");
        assert_eq!(post.content, "Updated content");
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_database_delete_and_verify() -> NetabaseResult<()> {
    let (store, db_path) = common::create_test_db::<Definition>("migration_delete")?;

    let post_id = PostID("ivy".to_string());
    let post = create_post("ivy", "Ivy's Post", "Will be deleted", 27);

    // Create
    {
        let txn = store.begin_write()?;
        txn.create(&post)?;
        txn.commit()?;
    }

    // Verify exists
    {
        let txn = store.begin_read()?;
        let read = txn.read::<Post>(&post_id)?;
        assert!(read.is_some());
    }

    // Delete
    {
        let txn = store.begin_write()?;
        txn.delete::<Post>(&post_id)?;
        txn.commit()?;
    }

    // Verify deleted
    {
        let txn = store.begin_read()?;
        let read = txn.read::<Post>(&post_id)?;
        assert!(read.is_none());
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_multiple_records_state_consistency() -> NetabaseResult<()> {
    let (store, db_path) = common::create_test_db::<Definition>("migration_multi")?;

    // Create multiple records
    {
        let txn = store.begin_write()?;
        for i in 1..=5 {
            let post = create_post(
                &format!("post_{}", i),
                &format!("Title {}", i),
                &format!("Content {}", i),
                20 + i,
            );
            txn.create(&post)?;
        }
        txn.commit()?;
    }

    // Verify all records exist
    {
        let txn = store.begin_read()?;
        for i in 1..=5 {
            let post_id = PostID(format!("post_{}", i));
            let post = txn.read::<Post>(&post_id)?;
            assert!(post.is_some(), "Post {} should exist", i);
            let post = post.unwrap();
            assert_eq!(post.title, format!("Title {}", i));
        }
    }

    // Update selective records
    {
        let txn = store.begin_write()?;
        for i in [2, 4] {
            let post_id = PostID(format!("post_{}", i));
            let mut post = txn.read::<Post>(&post_id)?.expect("Post not found");
            post.title = format!("Updated Title {}", i);
            txn.update(&post)?;
        }
        txn.commit()?;
    }

    // Verify selective updates
    {
        let txn = store.begin_read()?;

        // Unchanged
        let post1 = txn.read::<Post>(&PostID("post_1".to_string()))?.unwrap();
        assert_eq!(post1.title, "Title 1");

        // Changed
        let post2 = txn.read::<Post>(&PostID("post_2".to_string()))?.unwrap();
        assert_eq!(post2.title, "Updated Title 2");

        // Unchanged
        let post3 = txn.read::<Post>(&PostID("post_3".to_string()))?.unwrap();
        assert_eq!(post3.title, "Title 3");

        // Changed
        let post4 = txn.read::<Post>(&PostID("post_4".to_string()))?.unwrap();
        assert_eq!(post4.title, "Updated Title 4");

        // Unchanged
        let post5 = txn.read::<Post>(&PostID("post_5".to_string()))?.unwrap();
        assert_eq!(post5.title, "Title 5");
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_transaction_rollback_preserves_state() -> NetabaseResult<()> {
    let (store, db_path) = common::create_test_db::<Definition>("migration_rollback")?;

    let post_id = PostID("jack".to_string());
    let post = create_post("jack", "Original", "Original content", 30);

    // Create initial record
    {
        let txn = store.begin_write()?;
        txn.create(&post)?;
        txn.commit()?;
    }

    // Start a transaction but don't commit (implicit rollback)
    {
        let txn = store.begin_write()?;
        let mut post = txn.read::<Post>(&post_id)?.expect("Post not found");
        post.title = "Modified".to_string();
        txn.update(&post)?;
        // Don't commit - transaction drops and rolls back
    }

    // Verify state unchanged
    {
        let txn = store.begin_read()?;
        let post = txn.read::<Post>(&post_id)?;
        assert_eq!(post.unwrap().title, "Original");
    }

    common::cleanup_test_db(db_path);
    Ok(())
}
