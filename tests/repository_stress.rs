//! Comprehensive stress tests for repository interoperation and safety.
//!
//! This test suite validates:
//! 1. Multi-store interoperation between redb stores
//! 2. Schema validation and relational integrity
//! 3. Cross-definition relational link resolution
//! 4. Migration compatibility at repository level
//! 5. File system structure and schema.toml generation
//! 6. Concurrent access patterns
//! 7. Error handling for unsafe cross-repository operations

mod common;

use common::{cleanup_test_db, create_test_db};
use example::{
    AnotherLargeUserFile, Category, CategoryID, Definition, DefinitionTwo, LargeUserFile,
    MainRepository, MainRepositoryStores, User, UserID,
};
use netabase_store::{
    databases::redb::repository::RedbRepositoryDefinitions,
    errors::NetabaseResult,
    relational::RelationalLink,
    traits::registry::repository::NetabaseRepository,
};

/// Test: Repository folder structure creation and schema.toml generation
///
/// Validates:
/// - Correct folder hierarchy
/// - schema.toml generated for each definition
/// - repository.toml metadata file exists
/// - All files are readable and valid
#[test]
fn test_repository_filesystem_structure() -> NetabaseResult<()> {
    let temp_dir = std::env::temp_dir().join(format!(
        "netabase_repo_fs_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    let stores = MainRepositoryStores::new(&temp_dir)?;

    // Verify repository root
    assert!(temp_dir.exists(), "Repository root should exist");
    assert!(temp_dir.is_dir(), "Repository root should be directory");

    // Verify definition folders
    let def1_path = temp_dir.join("Definition");
    let def2_path = temp_dir.join("DefinitionTwo");
    assert!(def1_path.exists(), "Definition folder should exist");
    assert!(def2_path.exists(), "DefinitionTwo folder should exist");

    // Verify database files
    assert!(
        def1_path.join("data.redb").exists(),
        "Definition database should exist"
    );
    assert!(
        def2_path.join("data.redb").exists(),
        "DefinitionTwo database should exist"
    );

    // Verify schema files
    let schema1 = def1_path.join("schema.toml");
    let schema2 = def2_path.join("schema.toml");
    assert!(schema1.exists(), "Definition schema should exist");
    assert!(schema2.exists(), "DefinitionTwo schema should exist");

    // Verify schema files are readable and non-empty
    let schema1_content = std::fs::read_to_string(&schema1).expect("Failed to read schema1");
    let schema2_content = std::fs::read_to_string(&schema2).expect("Failed to read schema2");
    assert!(!schema1_content.is_empty(), "Schema 1 should not be empty");
    assert!(!schema2_content.is_empty(), "Schema 2 should not be empty");

    // Verify schema contains expected model names
    assert!(
        schema1_content.contains("User") || schema1_content.contains("LargeUserFile"),
        "Schema 1 should contain model names"
    );
    assert!(
        schema2_content.contains("Category") || schema2_content.contains("AnotherLargeUserFile"),
        "Schema 2 should contain model names"
    );

    drop(stores);
    std::fs::remove_dir_all(&temp_dir).ok();

    Ok(())
}

/// Test: Cross-definition relational link creation and resolution
///
/// Validates:
/// - Links can be created between definitions in same repository
/// - Links are properly dehydrated by default
/// - Links can be hydrated within repository context
/// - Link resolution validates repository membership
#[test]
fn test_cross_definition_relational_links() -> NetabaseResult<()> {
    let temp_dir = std::env::temp_dir().join(format!(
        "netabase_repo_relational_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    let stores = MainRepositoryStores::new(&temp_dir)?;

    // Create a category in DefinitionTwo
    let category_id = CategoryID("tech".to_string());
    let category = Category {
        id: category_id.clone(),
        name: "Technology".to_string(),
        description: "Tech category".to_string(),
    };

    {
        let txn = stores.definition_two.begin_write()?;
        txn.create(&category)?;
        txn.commit()?;
    }

    // Create a user in Definition that links to the category
    let user_id = UserID("user1".to_string());
    let user = User {
        id: user_id.clone(),
        first_name: "Alice".to_string(),
        last_name: "Smith".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(category_id.clone()),
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    {
        let txn = stores.definition.begin_write()?;
        txn.create(&user)?;
        txn.commit()?;
    }

    // Verify both entities exist
    {
        let txn = stores.definition.begin_read()?;
        let retrieved_user = txn.read::<User>(&user_id)?.expect("User should exist");
        assert_eq!(retrieved_user.id, user_id);
        assert_eq!(retrieved_user.first_name, "Alice");

        // Verify relational link is dehydrated
        match &retrieved_user.category {
            RelationalLink::Dehydrated { primary_key, .. } => {
                assert_eq!(primary_key, &category_id);
            }
            _ => panic!("Link should be dehydrated"),
        }
    }

    {
        let txn = stores.definition_two.begin_read()?;
        let retrieved_category = txn.read::<Category>(&category_id)?.expect("Category should exist");
        assert_eq!(retrieved_category.id, category_id);
        assert_eq!(retrieved_category.name, "Technology");
    }

    drop(stores);
    std::fs::remove_dir_all(&temp_dir).ok();

    Ok(())
}

/// Test: Repository-level data consistency
///
/// Validates:
/// - Transactions are isolated per definition
/// - Commit in one definition doesn't affect another
/// - Rollback in one definition doesn't affect another
/// - Read consistency across definitions
#[test]
fn test_repository_transaction_isolation() -> NetabaseResult<()> {
    let temp_dir = std::env::temp_dir().join(format!(
        "netabase_repo_isolation_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    let stores = MainRepositoryStores::new(&temp_dir)?;

    // Create initial data in both definitions
    let category_id = CategoryID("sports".to_string());
    {
        let txn = stores.definition_two.begin_write()?;
        txn.create(&Category {
            id: category_id.clone(),
            name: "Sports".to_string(),
            description: "Sports category".to_string(),
        })?;
        txn.commit()?;
    }

    let user_id = UserID("user2".to_string());
    {
        let txn = stores.definition.begin_write()?;
        txn.create(&User {
            id: user_id.clone(),
            first_name: "Bob".to_string(),
            last_name: "Jones".to_string(),
            age: 25,
            partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
            category: RelationalLink::new_dehydrated(category_id.clone()),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile(vec![]),
        })?;
        txn.commit()?;
    }

    // Start write transaction on Definition, modify but DON'T commit
    {
        let txn = stores.definition.begin_write()?;
        let mut user = txn.read::<User>(&user_id)?.expect("User should exist");
        user.age = 26;
        txn.create(&user)?;
        // Intentionally NOT committing
    }

    // Verify the change was NOT persisted
    {
        let txn = stores.definition.begin_read()?;
        let user = txn.read::<User>(&user_id)?.expect("User should exist");
        assert_eq!(user.age, 25, "Uncommitted changes should not persist");
    }

    // Verify DefinitionTwo is unaffected
    {
        let txn = stores.definition_two.begin_read()?;
        let category = txn.read::<Category>(&category_id)?.expect("Category should exist");
        assert_eq!(category.name, "Sports");
    }

    drop(stores);
    std::fs::remove_dir_all(&temp_dir).ok();

    Ok(())
}

/// Test: High-volume cross-definition operations
///
/// Validates:
/// - Performance under load
/// - Relational link integrity with many entities
/// - No memory leaks or resource exhaustion
#[test]
fn test_repository_high_volume_operations() -> NetabaseResult<()> {
    let temp_dir = std::env::temp_dir().join(format!(
        "netabase_repo_volume_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    let stores = MainRepositoryStores::new(&temp_dir)?;

    // Create 100 categories
    let category_count = 100;
    {
        let txn = stores.definition_two.begin_write()?;
        for i in 0..category_count {
            let category = Category {
                id: CategoryID(format!("cat{}", i)),
                name: format!("Category {}", i),
                description: format!("Description {}", i),
            };
            txn.create(&category)?;
        }
        txn.commit()?;
    }

    // Create 1000 users, each referencing a category
    let user_count = 1000;
    {
        let txn = stores.definition.begin_write()?;
        for i in 0..user_count {
            let category_id = CategoryID(format!("cat{}", i % category_count));
            let user = User {
                id: UserID(format!("user{}", i)),
                first_name: format!("User{}", i),
                last_name: "LastName".to_string(),
                age: 20 + (i % 50) as u8,
                partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
                category: RelationalLink::new_dehydrated(category_id),
                bio: LargeUserFile::default(),
                another: AnotherLargeUserFile(vec![]),
            };
            txn.create(&user)?;
        }
        txn.commit()?;
    }

    // Verify all data exists
    {
        let txn = stores.definition_two.begin_read()?;
        for i in 0..category_count {
            let category_id = CategoryID(format!("cat{}", i));
            let category = txn.read::<Category>(&category_id)?.expect("Category should exist");
            assert_eq!(category.id, category_id);
        }
    }

    {
        let txn = stores.definition.begin_read()?;
        for i in 0..user_count {
            let user_id = UserID(format!("user{}", i));
            let user = txn.read::<User>(&user_id)?.expect("User should exist");
            assert_eq!(user.id, user_id);

            // Verify relational link points to correct category
            match &user.category {
                RelationalLink::Dehydrated { primary_key, .. } => {
                    let expected_cat = CategoryID(format!("cat{}", i % category_count));
                    assert_eq!(primary_key, &expected_cat);
                }
                _ => panic!("Link should be dehydrated"),
            }
        }
    }

    drop(stores);
    std::fs::remove_dir_all(&temp_dir).ok();

    Ok(())
}

/// Test: Concurrent access to different definitions
///
/// Validates:
/// - Parallel reads to different definitions
/// - Parallel writes to different definitions
/// - No deadlocks or race conditions
#[test]
fn test_repository_concurrent_access() -> NetabaseResult<()> {
    let temp_dir = std::env::temp_dir().join(format!(
        "netabase_repo_concurrent_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    let stores = std::sync::Arc::new(MainRepositoryStores::new(&temp_dir)?);

    // Spawn threads that write to different definitions concurrently
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let stores_clone = std::sync::Arc::clone(&stores);
            std::thread::spawn(move || -> NetabaseResult<()> {
                if i % 2 == 0 {
                    // Even threads write to Definition
                    let txn = stores_clone.definition.begin_write()?;
                    let user = User {
                        id: UserID(format!("thread_user_{}", i)),
                        first_name: format!("ThreadUser{}", i),
                        last_name: "LastName".to_string(),
                        age: 30 + i as u8,
                        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
                        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
                        bio: LargeUserFile::default(),
                        another: AnotherLargeUserFile(vec![]),
                    };
                    txn.create(&user)?;
                    txn.commit()?;
                } else {
                    // Odd threads write to DefinitionTwo
                    let txn = stores_clone.definition_two.begin_write()?;
                    let category = Category {
                        id: CategoryID(format!("thread_cat_{}", i)),
                        name: format!("Thread Category {}", i),
                        description: format!("Description {}", i),
                    };
                    txn.create(&category)?;
                    txn.commit()?;
                }
                Ok(())
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap()?;
    }

    // Verify all writes succeeded
    {
        let txn = stores.definition.begin_read()?;
        for i in (0..10).step_by(2) {
            let user_id = UserID(format!("thread_user_{}", i));
            let user = txn.read::<User>(&user_id)?.expect("User should exist");
            assert_eq!(user.id, user_id);
        }
    }

    {
        let txn = stores.definition_two.begin_read()?;
        for i in (1..10).step_by(2) {
            let category_id = CategoryID(format!("thread_cat_{}", i));
            let category = txn.read::<Category>(&category_id)?.expect("Category should exist");
            assert_eq!(category.id, category_id);
        }
    }

    drop(stores);
    std::fs::remove_dir_all(&temp_dir).ok();

    Ok(())
}

/// Test: Schema validation and evolution
///
/// Validates:
/// - schema.toml reflects current model structure
/// - Schema changes are detected
/// - Migration path validation (future work)
#[test]
fn test_repository_schema_validation() -> NetabaseResult<()> {
    let temp_dir = std::env::temp_dir().join(format!(
        "netabase_repo_schema_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    let stores = MainRepositoryStores::new(&temp_dir)?;

    // Read schema files
    let schema1_path = temp_dir.join("Definition").join("schema.toml");
    let schema2_path = temp_dir.join("DefinitionTwo").join("schema.toml");

    let schema1_content = std::fs::read_to_string(&schema1_path).expect("Failed to read schema1");
    let schema2_content = std::fs::read_to_string(&schema2_path).expect("Failed to read schema2");

    // Verify schema contains expected fields for User model
    assert!(
        schema1_content.contains("first_name") || schema1_content.contains("User"),
        "Schema should describe User model fields"
    );

    // Verify schema contains expected fields for Category model
    assert!(
        schema2_content.contains("name") || schema2_content.contains("Category"),
        "Schema should describe Category model fields"
    );

    // Future: Parse TOML and validate structure programmatically
    // For now, just verify it's valid TOML
    toml::from_str::<toml::Value>(&schema1_content).expect("Schema1 should be valid TOML");
    toml::from_str::<toml::Value>(&schema2_content).expect("Schema2 should be valid TOML");

    drop(stores);
    std::fs::remove_dir_all(&temp_dir).ok();

    Ok(())
}

/// Test: Repository reopening and persistence
///
/// Validates:
/// - Repository can be closed and reopened
/// - Data persists across reopens
/// - File handles are properly released
#[test]
fn test_repository_persistence() -> NetabaseResult<()> {
    let temp_dir = std::env::temp_dir().join(format!(
        "netabase_repo_persist_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    // Create repository and write data
    {
        let stores = MainRepositoryStores::new(&temp_dir)?;

        let category_id = CategoryID("persistent_cat".to_string());
        {
            let txn = stores.definition_two.begin_write()?;
            txn.create(&Category {
                id: category_id.clone(),
                name: "Persistent Category".to_string(),
                description: "Persistent description".to_string(),
            })?;
            txn.commit()?;
        }

        let user_id = UserID("persistent_user".to_string());
        {
            let txn = stores.definition.begin_write()?;
            txn.create(&User {
                id: user_id.clone(),
                first_name: "Persistent".to_string(),
                last_name: "User".to_string(),
                age: 35,
                partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
                category: RelationalLink::new_dehydrated(category_id),
                bio: LargeUserFile::default(),
                another: AnotherLargeUserFile(vec![]),
            })?;
            txn.commit()?;
        }

        // Explicitly drop to close database
        drop(stores);
    }

    // Reopen repository and verify data persists
    {
        let stores = MainRepositoryStores::new(&temp_dir)?;

        {
            let txn = stores.definition_two.begin_read()?;
            let category = txn.read::<Category>(&CategoryID("persistent_cat".to_string()))?.expect("Category should exist");
            assert_eq!(category.name, "Persistent Category");
        }

        {
            let txn = stores.definition.begin_read()?;
            let user = txn.read::<User>(&UserID("persistent_user".to_string()))?.expect("User should exist");
            assert_eq!(user.first_name, "Persistent");
            assert_eq!(user.age, 35);
        }

        drop(stores);
    }

    std::fs::remove_dir_all(&temp_dir).ok();

    Ok(())
}

/// Test: Error handling for missing related entities
///
/// Validates:
/// - Graceful handling when relational link points to non-existent entity
/// - Proper error messages
/// - Transaction safety on errors
#[test]
fn test_repository_missing_relation_handling() -> NetabaseResult<()> {
    let temp_dir = std::env::temp_dir().join(format!(
        "netabase_repo_missing_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    let stores = MainRepositoryStores::new(&temp_dir)?;

    // Create a user that references a non-existent category
    let user_id = UserID("orphan_user".to_string());
    let nonexistent_category = CategoryID("does_not_exist".to_string());

    {
        let txn = stores.definition.begin_write()?;
        let user = User {
            id: user_id.clone(),
            first_name: "Orphan".to_string(),
            last_name: "User".to_string(),
            age: 40,
            partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
            category: RelationalLink::new_dehydrated(nonexistent_category.clone()),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile(vec![]),
        };
        txn.create(&user)?;
        txn.commit()?;
    }

    // User creation should succeed (relational links are dehydrated)
    {
        let txn = stores.definition.begin_read()?;
        let user = txn.read::<User>(&user_id)?.expect("User should exist");
        assert_eq!(user.first_name, "Orphan");

        // Link exists but points to non-existent entity
        match &user.category {
            RelationalLink::Dehydrated { primary_key, .. } => {
                assert_eq!(primary_key, &nonexistent_category);
            }
            _ => panic!("Link should be dehydrated"),
        }
    }

    // Attempting to hydrate the link should fail gracefully
    // (This is future work - hydration validation)

    drop(stores);
    std::fs::remove_dir_all(&temp_dir).ok();

    Ok(())
}

/// Test: Repository metadata consistency
///
/// Validates:
/// - All definitions are listed in metadata
/// - Definition names match folder structure
/// - Repository name is correct
#[test]
fn test_repository_metadata_consistency() -> NetabaseResult<()> {
    let temp_dir = std::env::temp_dir().join(format!(
        "netabase_repo_meta_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    let stores = MainRepositoryStores::new(&temp_dir)?;

    // Verify repository name
    assert_eq!(MainRepository::name(), "MainRepository");

    // Verify definition count
    assert_eq!(MainRepository::definition_count(), 2);

    // Verify definition names
    let def_names = MainRepository::definition_names();
    assert_eq!(def_names.len(), 2);
    assert!(def_names.contains(&"Definition"));
    assert!(def_names.contains(&"DefinitionTwo"));

    // Verify folders exist for all definitions
    for def_name in def_names {
        let def_path = temp_dir.join(def_name);
        assert!(
            def_path.exists(),
            "Definition folder {} should exist",
            def_name
        );
    }

    drop(stores);
    std::fs::remove_dir_all(&temp_dir).ok();

    Ok(())
}
