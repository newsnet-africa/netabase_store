/// Integration tests for blob storage with large data
///
/// These tests verify that large blob data is correctly stored and retrieved.
/// The low-level blob query APIs (read_blob_items, list_blob_keys, etc.) are internal
/// trait methods. See BLOB_QUERY_METHODS.md for architectural details.
mod common;

use netabase_store::errors::NetabaseResult;
use netabase_store::relational::RelationalLink;
use netabase_store_examples::boilerplate_lib::definition::{
    AnotherLargeUserFile, LargeUserFile, User, UserID,
};
use netabase_store_examples::boilerplate_lib::{CategoryID, Definition};

#[test]
fn test_blob_storage_single_large_field() -> NetabaseResult<()> {
    let (store, db_path) = common::create_test_db::<Definition>("blob_single")?;

    // Create a user with large blob data
    let large_data = vec![42u8; 100_000]; // 100KB of data
    let user = User {
        id: UserID("user_with_blobs".to_string()),
        first_name: "Blob".to_string(),
        last_name: "User".to_string(),
        age: 25,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        bio: LargeUserFile {
            data: large_data.clone(),
            metadata: "Test metadata".to_string(),
        },
        another: AnotherLargeUserFile(vec![]),
        subscriptions: vec![],
    };

    // Create the user
    {
        let txn = store.begin_write()?;
        txn.create(&user)?;
        txn.commit()?;
    }

    // Read back and verify blob data integrity
    {
        let txn = store.begin_read()?;
        let retrieved: Option<User> = txn.read(&UserID("user_with_blobs".to_string()))?;

        assert!(retrieved.is_some(), "User should exist");
        let retrieved_user = retrieved.unwrap();

        println!(
            "Read user with {} bytes of blob data",
            retrieved_user.bio.data.len()
        );
        assert_eq!(
            retrieved_user.bio.data.len(),
            100_000,
            "Blob data should be intact"
        );
        assert_eq!(
            retrieved_user.bio.data, large_data,
            "Blob data should match original"
        );
        assert_eq!(retrieved_user.bio.metadata, "Test metadata");
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_blob_storage_multiple_users() -> NetabaseResult<()> {
    let (store, db_path) = common::create_test_db::<Definition>("blob_multiple")?;

    // Create multiple users with varying blob sizes
    for i in 0..5 {
        let user = User {
            id: UserID(format!("user_{}", i)),
            first_name: format!("User{}", i),
            last_name: "Test".to_string(),
            age: (20 + i) as u8,
            partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
            category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
            bio: LargeUserFile {
                data: vec![i as u8; 50_000],
                metadata: format!("User {} metadata", i),
            },
            another: AnotherLargeUserFile(vec![i as u8; 30_000]),
            subscriptions: vec![],
        };

        let txn = store.begin_write()?;
        txn.create(&user)?;
        txn.commit()?;
    }

    // Verify all users can be read back with intact blob data
    {
        let txn = store.begin_read()?;

        for i in 0..5 {
            let user: Option<User> = txn.read(&UserID(format!("user_{}", i)))?;
            assert!(user.is_some(), "User {} should exist", i);

            let user = user.unwrap();
            assert_eq!(user.bio.data.len(), 50_000, "User {} bio should be 50KB", i);
            assert_eq!(
                user.another.0.len(),
                30_000,
                "User {} another should be 30KB",
                i
            );
            assert_eq!(user.bio.metadata, format!("User {} metadata", i));
        }

        println!("Successfully verified 5 users with multiple blob fields");
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_blob_storage_very_large_data() -> NetabaseResult<()> {
    let (store, db_path) = common::create_test_db::<Definition>("blob_very_large")?;

    // Create a user with very large blob data (will be chunked)
    let large_bio = vec![1u8; 200_000]; // 200KB
    let large_another = vec![2u8; 150_000]; // 150KB

    let user = User {
        id: UserID("large_user".to_string()),
        first_name: "Large".to_string(),
        last_name: "User".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        bio: LargeUserFile {
            data: large_bio.clone(),
            metadata: "Very large metadata".to_string(),
        },
        another: AnotherLargeUserFile(large_another.clone()),
        subscriptions: vec![],
    };

    {
        let txn = store.begin_write()?;
        txn.create(&user)?;
        txn.commit()?;
    }

    // Verify large blobs are correctly stored and retrieved
    {
        let txn = store.begin_read()?;
        let retrieved: Option<User> = txn.read(&UserID("large_user".to_string()))?;

        assert!(retrieved.is_some());
        let retrieved_user = retrieved.unwrap();

        println!(
            "Retrieved user with {}KB + {}KB of blob data",
            retrieved_user.bio.data.len() / 1000,
            retrieved_user.another.0.len() / 1000
        );

        assert_eq!(retrieved_user.bio.data, large_bio, "Bio blob should match");
        assert_eq!(
            retrieved_user.another.0, large_another,
            "Another blob should match"
        );
        assert_eq!(retrieved_user.bio.metadata, "Very large metadata");
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_blob_storage_update_workflow() -> NetabaseResult<()> {
    let (store, db_path) = common::create_test_db::<Definition>("blob_update")?;

    let user_id = UserID("updatable_user".to_string());

    // Create initial user with small blobs
    {
        let user = User {
            id: user_id.clone(),
            first_name: "Initial".to_string(),
            last_name: "User".to_string(),
            age: 25,
            partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
            category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
            bio: LargeUserFile {
                data: vec![1u8; 1000],
                metadata: "Initial".to_string(),
            },
            another: AnotherLargeUserFile(vec![]),
            subscriptions: vec![],
        };

        let txn = store.begin_write()?;
        txn.create(&user)?;
        txn.commit()?;
    }

    // Update with larger blobs
    {
        let updated_user = User {
            id: user_id.clone(),
            first_name: "Updated".to_string(),
            last_name: "User".to_string(),
            age: 26,
            partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
            category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
            bio: LargeUserFile {
                data: vec![2u8; 80_000],
                metadata: "Updated".to_string(),
            },
            another: AnotherLargeUserFile(vec![3u8; 60_000]),
            subscriptions: vec![],
        };

        let txn = store.begin_write()?;
        txn.update(&updated_user)?;
        txn.commit()?;
    }

    // Verify updated data
    {
        let txn = store.begin_read()?;
        let user: Option<User> = txn.read(&user_id)?;

        assert!(user.is_some());
        let user = user.unwrap();

        assert_eq!(user.first_name, "Updated");
        assert_eq!(user.age, 26);
        assert_eq!(user.bio.data.len(), 80_000);
        assert_eq!(user.bio.metadata, "Updated");
        assert_eq!(user.another.0.len(), 60_000);

        println!("Successfully updated blob data from 1KB to 80KB + 60KB");
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_blob_storage_empty_data() -> NetabaseResult<()> {
    let (store, db_path) = common::create_test_db::<Definition>("blob_empty")?;

    // Create user with empty blob fields
    let user = User {
        id: UserID("empty_blobs".to_string()),
        first_name: "Empty".to_string(),
        last_name: "User".to_string(),
        age: 20,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        bio: LargeUserFile {
            data: vec![],
            metadata: "Empty data".to_string(),
        },
        another: AnotherLargeUserFile(vec![]),
        subscriptions: vec![],
    };

    {
        let txn = store.begin_write()?;
        txn.create(&user)?;
        txn.commit()?;
    }

    // Verify empty blobs are handled correctly
    {
        let txn = store.begin_read()?;
        let retrieved: Option<User> = txn.read(&UserID("empty_blobs".to_string()))?;

        assert!(retrieved.is_some());
        let retrieved_user = retrieved.unwrap();

        assert_eq!(retrieved_user.bio.data.len(), 0);
        assert_eq!(retrieved_user.another.0.len(), 0);
        assert_eq!(retrieved_user.bio.metadata, "Empty data");

        println!("Empty blob data handled correctly");
    }

    common::cleanup_test_db(db_path);
    Ok(())
}
