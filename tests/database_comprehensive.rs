/// Comprehensive database tests with full state inspection before and after operations.
///
/// These tests verify:
/// - Transaction isolation
/// - CRUD operations correctness
/// - Query operations
/// - Error handling
/// - Rollback behavior
mod common;

use netabase_store::errors::NetabaseResult;
use netabase_store::query::{QueryConfig, QueryResult};

use netabase_store_examples::{Definition, Post, PostID};

/// Helper to create a test Post
fn create_post(id: &str, title: &str, content: &str, published: bool) -> Post {
    Post {
        id: PostID(id.to_string()),
        title: title.to_string(),
        author_id: "test_author".to_string(),
        content: content.to_string(),
        published,
        subscriptions: vec![],
    }
}

#[test]
fn test_empty_database_initial_state() -> NetabaseResult<()> {
    let (store, db_path) = common::create_test_db::<Definition>("db_empty_state")?;

    // Verify empty database
    let txn = store.begin_read()?;

    // Reading non-existent key returns None
    let post_id = PostID("nonexistent".to_string());
    let result = txn.read::<Post>(&post_id)?;
    assert!(
        result.is_none(),
        "New database should not contain any records"
    );

    let post_id2 = PostID("also_nonexistent".to_string());
    let result2 = txn.read::<Post>(&post_id2)?;
    assert!(
        result2.is_none(),
        "New database should not contain any records"
    );

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_create_single_record() -> NetabaseResult<()> {
    let (store, db_path) = common::create_test_db::<Definition>("db_create_single")?;

    let post_id = PostID("post1".to_string());
    let post = create_post("post1", "My First Post", "Hello World!", true);

    // State before: record doesn't exist
    {
        let txn = store.begin_read()?;
        let result = txn.read::<Post>(&post_id)?;
        assert!(result.is_none());
    }

    // Create record
    {
        let txn = store.begin_write()?;
        txn.create(&post)?;
        txn.commit()?;
    }

    // State after: record exists with exact data
    {
        let txn = store.begin_read()?;
        let result = txn.read::<Post>(&post_id)?;
        assert!(result.is_some(), "Record should exist after create");

        let retrieved = result.unwrap();
        assert_eq!(retrieved.id.0, "post1");
        assert_eq!(retrieved.title, "My First Post");
        assert_eq!(retrieved.content, "Hello World!");
        assert!(retrieved.published);
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_create_duplicate_overwrites() -> NetabaseResult<()> {
    let (store, db_path) = common::create_test_db::<Definition>("db_create_duplicate")?;

    let post_id = PostID("dup_post".to_string());
    let post = create_post("dup_post", "First Version", "Original content", false);

    // First create succeeds
    {
        let txn = store.begin_write()?;
        txn.create(&post)?;
        txn.commit()?;
    }

    // Second create with same ID overwrites (this is the current behavior)
    {
        let txn = store.begin_write()?;
        let post2 = create_post("dup_post", "Second Version", "Updated content", true);
        txn.create(&post2)?;
        txn.commit()?;
    }

    // Verify the new version is stored
    {
        let txn = store.begin_read()?;
        let retrieved = txn.read::<Post>(&post_id)?.unwrap();
        assert_eq!(
            retrieved.title, "Second Version",
            "Record should be overwritten"
        );
        assert_eq!(retrieved.content, "Updated content");
        assert!(retrieved.published);
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_update_existing_record() -> NetabaseResult<()> {
    let (store, db_path) = common::create_test_db::<Definition>("db_update_existing")?;

    let post_id = PostID("update_post".to_string());

    // Create initial record
    {
        let txn = store.begin_write()?;
        let post = create_post("update_post", "Original Title", "Original content", false);
        txn.create(&post)?;
        txn.commit()?;
    }

    // State before update
    {
        let txn = store.begin_read()?;
        let post = txn.read::<Post>(&post_id)?.unwrap();
        assert_eq!(post.title, "Original Title");
        assert!(!post.published);
    }

    // Update record
    {
        let txn = store.begin_write()?;
        let mut post = txn.read::<Post>(&post_id)?.unwrap();

        post.title = "Updated Title".to_string();
        post.published = true;

        txn.update(&post)?;
        txn.commit()?;
    }

    // State after update: changes applied
    {
        let txn = store.begin_read()?;
        let post = txn.read::<Post>(&post_id)?.unwrap();
        assert_eq!(post.title, "Updated Title", "Title should be updated");
        assert!(post.published, "Published should be updated");
        assert_eq!(post.content, "Original content", "Content unchanged");
        assert_eq!(post.id.0, "update_post", "ID should be unchanged");
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_delete_existing_record() -> NetabaseResult<()> {
    let (store, db_path) = common::create_test_db::<Definition>("db_delete_existing")?;

    let post_id = PostID("delete_me".to_string());

    // Create record
    {
        let txn = store.begin_write()?;
        let post = create_post("delete_me", "Temporary", "Will be deleted", false);
        txn.create(&post)?;
        txn.commit()?;
    }

    // State before delete: record exists
    {
        let txn = store.begin_read()?;
        let result = txn.read::<Post>(&post_id)?;
        assert!(result.is_some(), "Record should exist before delete");
    }

    // Delete record
    {
        let txn = store.begin_write()?;
        txn.delete::<Post>(&post_id)?;
        txn.commit()?;
    }

    // State after delete: record doesn't exist
    {
        let txn = store.begin_read()?;
        let result = txn.read::<Post>(&post_id)?;
        assert!(result.is_none(), "Record should not exist after delete");
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_delete_nonexistent_record_succeeds() -> NetabaseResult<()> {
    let (store, db_path) = common::create_test_db::<Definition>("db_delete_nonexistent")?;

    let post_id = PostID("never_existed".to_string());

    // Delete non-existent record (should succeed as no-op)
    {
        let txn = store.begin_write()?;
        let result = txn.delete::<Post>(&post_id);
        assert!(
            result.is_ok(),
            "Deleting non-existent record should succeed (no-op)"
        );
        txn.commit()?;
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_transaction_rollback_on_drop() -> NetabaseResult<()> {
    let (store, db_path) = common::create_test_db::<Definition>("db_rollback_drop")?;

    let post_id = PostID("rollback_test".to_string());

    // Create initial record
    {
        let txn = store.begin_write()?;
        let post = create_post("rollback_test", "Original", "Original content", false);
        txn.create(&post)?;
        txn.commit()?;
    }

    // Modify in transaction but don't commit (implicit rollback on drop)
    {
        let txn = store.begin_write()?;
        let mut post = txn.read::<Post>(&post_id)?.unwrap();
        post.title = "Modified".to_string();
        post.content = "Modified content".to_string();
        txn.update(&post)?;
        // Transaction drops here without commit - should rollback
    }

    // Verify original state preserved
    {
        let txn = store.begin_read()?;
        let post = txn.read::<Post>(&post_id)?.unwrap();
        assert_eq!(post.title, "Original", "Changes should be rolled back");
        assert_eq!(
            post.content, "Original content",
            "Changes should be rolled back"
        );
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_multiple_records_crud() -> NetabaseResult<()> {
    let (store, db_path) = common::create_test_db::<Definition>("db_multiple_records")?;

    let num_records = 10;

    // Create multiple records
    {
        let txn = store.begin_write()?;
        for i in 1..=num_records {
            let post = create_post(
                &format!("post_{}", i),
                &format!("Title {}", i),
                &format!("Content {}", i),
                i % 2 == 0, // Even posts are published
            );
            txn.create(&post)?;
        }
        txn.commit()?;
    }

    // Verify all records exist with correct data
    {
        let txn = store.begin_read()?;
        for i in 1..=num_records {
            let post_id = PostID(format!("post_{}", i));
            let post = txn.read::<Post>(&post_id)?;
            assert!(post.is_some(), "Record {} should exist", i);

            let post = post.unwrap();
            assert_eq!(post.id.0, format!("post_{}", i));
            assert_eq!(post.title, format!("Title {}", i));
            assert_eq!(post.content, format!("Content {}", i));
            assert_eq!(post.published, i % 2 == 0);
        }
    }

    // Update selective records (posts 2, 4, 6, 8)
    {
        let txn = store.begin_write()?;
        for i in [2, 4, 6, 8] {
            let post_id = PostID(format!("post_{}", i));
            let mut post = txn.read::<Post>(&post_id)?.unwrap();
            post.title = format!("Updated Title {}", i);
            txn.update(&post)?;
        }
        txn.commit()?;
    }

    // Verify selective updates
    {
        let txn = store.begin_read()?;

        // Updated records
        for i in [2, 4, 6, 8] {
            let post_id = PostID(format!("post_{}", i));
            let post = txn.read::<Post>(&post_id)?.unwrap();
            assert_eq!(
                post.title,
                format!("Updated Title {}", i),
                "Record {} should be updated",
                i
            );
        }

        // Non-updated records
        for i in [1, 3, 5, 7, 9, 10] {
            let post_id = PostID(format!("post_{}", i));
            let post = txn.read::<Post>(&post_id)?.unwrap();
            assert_eq!(
                post.title,
                format!("Title {}", i),
                "Record {} should be unchanged",
                i
            );
        }
    }

    // Delete selective records (posts 1, 5, 9)
    {
        let txn = store.begin_write()?;
        for i in [1, 5, 9] {
            let post_id = PostID(format!("post_{}", i));
            txn.delete::<Post>(&post_id)?;
        }
        txn.commit()?;
    }

    // Verify selective deletions
    {
        let txn = store.begin_read()?;

        // Deleted records
        for i in [1, 5, 9] {
            let post_id = PostID(format!("post_{}", i));
            let result = txn.read::<Post>(&post_id)?;
            assert!(result.is_none(), "Record {} should be deleted", i);
        }

        // Remaining records
        for i in [2, 3, 4, 6, 7, 8, 10] {
            let post_id = PostID(format!("post_{}", i));
            let result = txn.read::<Post>(&post_id)?;
            assert!(result.is_some(), "Record {} should still exist", i);
        }
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_transaction_isolation() -> NetabaseResult<()> {
    let (store, db_path) = common::create_test_db::<Definition>("db_isolation")?;

    let post_id = PostID("shared_post".to_string());

    // Create initial record
    {
        let txn = store.begin_write()?;
        let post = create_post("shared_post", "Original", "Shared content", true);
        txn.create(&post)?;
        txn.commit()?;
    }

    // Start write transaction but don't commit yet
    let write_txn = store.begin_write()?;
    let mut post = write_txn.read::<Post>(&post_id)?.unwrap();
    post.title = "Modified".to_string();
    write_txn.update(&post)?;

    // Read transaction should see original state (not the uncommitted change)
    {
        let read_txn = store.begin_read()?;
        let post = read_txn.read::<Post>(&post_id)?.unwrap();
        assert_eq!(
            post.title, "Original",
            "Read transaction should see original value"
        );
    }

    // Commit write transaction
    write_txn.commit()?;

    // Now read transaction should see new state
    {
        let read_txn = store.begin_read()?;
        let post = read_txn.read::<Post>(&post_id)?.unwrap();
        assert_eq!(
            post.title, "Modified",
            "Read transaction should see committed value"
        );
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_query_config_helpers() {
    use std::ops::RangeFull;

    // Test all query config helper methods
    let all = QueryConfig::all();
    assert_eq!(all.range, RangeFull);

    let first = QueryConfig::first();
    assert_eq!(first.pagination.limit, Some(1));

    let dump = QueryConfig::dump_all();
    assert!(dump.fetch_options.include_blobs);
    assert_eq!(dump.fetch_options.hydration_depth, 0);

    let inspect = QueryConfig::inspect_range(0u64..100u64);
    assert_eq!(inspect.range, 0u64..100u64);
    assert!(inspect.fetch_options.include_blobs);

    // Test builder pattern
    let custom = QueryConfig::default()
        .with_limit(10)
        .with_offset(5)
        .no_blobs()
        .no_hydration()
        .reversed();

    assert_eq!(custom.pagination.limit, Some(10));
    assert_eq!(custom.pagination.offset, Some(5));
    assert!(!custom.fetch_options.include_blobs);
    assert_eq!(custom.fetch_options.hydration_depth, 0);
    assert!(custom.reversed);
}

#[test]
fn test_query_result_methods() {
    // Single variant
    let single = QueryResult::Single(Some(42));
    assert_eq!(single.len(), 1);
    assert!(!single.is_empty());
    assert_eq!(single.as_single(), Some(&42));
    assert_eq!(single.clone().unwrap_single(), 42);

    // Single None
    let empty: QueryResult<i32> = QueryResult::Single(None);
    assert_eq!(empty.len(), 0);
    assert!(empty.is_empty());
    assert_eq!(empty.as_single(), None);

    // Multiple variant
    let multiple = QueryResult::Multiple(vec![1, 2, 3, 4, 5]);
    assert_eq!(multiple.len(), 5);
    assert!(!multiple.is_empty());
    assert_eq!(multiple.as_multiple(), Some(&vec![1, 2, 3, 4, 5]));
    assert_eq!(multiple.clone().into_vec(), vec![1, 2, 3, 4, 5]);

    // Count variant
    let count: QueryResult<i32> = QueryResult::Count(100);
    assert_eq!(count.len(), 100);
    assert!(!count.is_empty());
    assert_eq!(count.count(), Some(100));
}
