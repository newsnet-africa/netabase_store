// Integration tests for CRUD operations with full verification

pub mod common;

use common::{cleanup_test_db, create_test_db};
use netabase_store::databases::redb::transaction::RedbModelCrud;
use netabase_store::errors::NetabaseResult;
use netabase_store::relational::{RelationalLink, ModelRelationPermissions, RelationPermission, PermissionFlag};
use netabase_store::traits::registery::models::model::{NetabaseModel, RedbNetbaseModel};

// Use boilerplate models from examples
use netabase_store_examples::{User, UserID, LargeUserFile, AnotherLargeUserFile};
use netabase_store_examples::{CategoryID, Definition, DefinitionSubscriptions};

#[test]
fn test_create_and_verify() -> NetabaseResult<()> {
    let (store, db_path) = create_test_db::<Definition>("create_verify")?;

    // Create a user
    let user_id = UserID("test_user_1".to_string());
    let user = User {
        id: user_id.clone(),
        name: "Alice".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        subscriptions: vec![DefinitionSubscriptions::Topic1],
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    // Create in database
    let txn = store.begin_transaction()?;
    txn.create_redb(&user)?;
    txn.commit()?;

    // VERIFY: Read back and check all fields
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        let read_user = User::read_default(&user_id, &tables)?;

        // Verify user exists
        assert!(read_user.is_some(), "User should exist after creation");

        let read_user = read_user.unwrap();

        // Verify all fields match
        assert_eq!(read_user.id.0, "test_user_1", "User ID should match");
        assert_eq!(read_user.name, "Alice", "User name should match");
        assert_eq!(read_user.age, 30, "User age should match");
        assert_eq!(
            read_user.subscriptions.len(),
            1,
            "Should have 1 subscription"
        );
        assert!(matches!(
            read_user.subscriptions[0],
            DefinitionSubscriptions::Topic1
        ));
    }
    txn.commit()?;

    cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_create_duplicate_should_overwrite() -> NetabaseResult<()> {
    let (store, db_path) = create_test_db::<Definition>("create_duplicate")?;

    let user_id = UserID("dup_user".to_string());

    // Create first version
    let user_v1 = User {
        id: user_id.clone(),
        name: "Version 1".to_string(),
        age: 25,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        subscriptions: vec![],
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    let txn = store.begin_transaction()?;
    txn.create_redb(&user_v1)?;
    txn.commit()?;

    // Create second version with same ID
    let user_v2 = User {
        id: user_id.clone(),
        name: "Version 2".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        subscriptions: vec![],
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    let txn = store.begin_transaction()?;
    txn.create_redb(&user_v2)?;
    txn.commit()?;

    // VERIFY: Should have the second version
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        let read_user = User::read_default(&user_id, &tables)?;
        assert!(read_user.is_some());

        let read_user = read_user.unwrap();
        assert_eq!(read_user.name, "Version 2", "Should have latest version");
        assert_eq!(read_user.age, 30, "Should have latest age");
    }
    txn.commit()?;

    cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_read_nonexistent() -> NetabaseResult<()> {
    let (store, db_path) = create_test_db::<Definition>("read_nonexistent")?;

    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        let result = User::read_default(&UserID("does_not_exist".to_string()), &tables)?;

        // VERIFY: Should return None for nonexistent user
        assert!(result.is_none(), "Should return None for nonexistent user");
    }
    txn.commit()?;

    cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_update_and_verify() -> NetabaseResult<()> {
    let (store, db_path) = create_test_db::<Definition>("update_verify")?;

    let user_id = UserID("update_user".to_string());

    // Create initial user
    let user = User {
        id: user_id.clone(),
        name: "Original".to_string(),
        age: 25,
        partner: RelationalLink::new_dehydrated(UserID("partner_old".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("cat_old".to_string())),
        subscriptions: vec![DefinitionSubscriptions::Topic1],
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    let txn = store.begin_transaction()?;
    txn.create_redb(&user)?;
    txn.commit()?;

    // VERIFY: Initial state
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        let read = User::read_default(&user_id, &tables)?;
        assert!(read.is_some());
        assert_eq!(read.unwrap().name, "Original");
    }
    txn.commit()?;

    // Update user
    let updated_user = User {
        id: user_id.clone(),
        name: "Updated".to_string(),
        age: 26,
        partner: RelationalLink::new_dehydrated(UserID("partner_new".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("cat_new".to_string())),
        subscriptions: vec![DefinitionSubscriptions::Topic2],
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let perms = ModelRelationPermissions {
            relationa_tree_access: &[RelationPermission(User::TREE_NAMES, PermissionFlag::ReadWrite)]
        };
        let mut tables = txn.open_model_tables(table_defs, Some(perms))?;
        updated_user.update_entry(&mut tables)?;
    }
    txn.commit()?;

    // VERIFY: Updated state
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        let read = User::read_default(&user_id, &tables)?;
        assert!(read.is_some(), "User should still exist after update");

        let read = read.unwrap();
        assert_eq!(read.name, "Updated", "Name should be updated");
        assert_eq!(read.age, 26, "Age should be updated");
        assert_eq!(read.subscriptions.len(), 1);
        assert!(matches!(
            read.subscriptions[0],
            DefinitionSubscriptions::Topic2
        ));
    }
    txn.commit()?;

    cleanup_test_db(db_path);
    Ok(())
}

#[test]
#[ignore] // TODO: update_entry should check if entry exists before updating
fn test_update_nonexistent_should_fail() -> NetabaseResult<()> {
    let (store, db_path) = create_test_db::<Definition>("update_nonexistent")?;

    let user_id = UserID("does_not_exist".to_string());
    let user = User {
        id: user_id.clone(),
        name: "Test".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        subscriptions: vec![],
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let perms = ModelRelationPermissions {
            relationa_tree_access: &[RelationPermission(User::TREE_NAMES, PermissionFlag::ReadWrite)]
        };
        let mut tables = txn.open_model_tables(table_defs, Some(perms))?;

        let result = user.update_entry(&mut tables);

        // VERIFY: Update should fail for nonexistent entry
        assert!(result.is_err(), "Update should fail for nonexistent entry");
    }
    txn.commit()?;

    cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_delete_and_verify() -> NetabaseResult<()> {
    let (store, db_path) = create_test_db::<Definition>("delete_verify")?;

    let user_id = UserID("delete_user".to_string());

    // Create user
    let user = User {
        id: user_id.clone(),
        name: "To Delete".to_string(),
        age: 40,
        partner: RelationalLink::new_dehydrated(UserID("partner".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("cat".to_string())),
        subscriptions: vec![
            DefinitionSubscriptions::Topic1,
            DefinitionSubscriptions::Topic2,
        ],
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    let txn = store.begin_transaction()?;
    txn.create_redb(&user)?;
    txn.commit()?;

    // VERIFY: User exists before deletion
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        let exists = User::read_default(&user_id, &tables)?;
        assert!(exists.is_some(), "User should exist before deletion");
    }
    txn.commit()?;

    // Delete user
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let perms = ModelRelationPermissions {
            relationa_tree_access: &[RelationPermission(User::TREE_NAMES, PermissionFlag::ReadWrite)]
        };
        let mut tables = txn.open_model_tables(table_defs, Some(perms))?;

        User::delete_entry(&user_id, &mut tables)?;
    }
    txn.commit()?;

    // VERIFY: User no longer exists
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        let exists = User::read_default(&user_id, &tables)?;
        assert!(exists.is_none(), "User should not exist after deletion");
    }
    txn.commit()?;

    cleanup_test_db(db_path);
    Ok(())
}

#[test]
#[ignore] // TODO: delete_entry should check if entry exists before deleting
fn test_delete_nonexistent_should_fail() -> NetabaseResult<()> {
    let (store, db_path) = create_test_db::<Definition>("delete_nonexistent")?;

    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let perms = ModelRelationPermissions {
            relationa_tree_access: &[RelationPermission(User::TREE_NAMES, PermissionFlag::ReadWrite)]
        };
        let mut tables = txn.open_model_tables(table_defs, Some(perms))?;

        let result = User::delete_entry(&UserID("does_not_exist".to_string()), &mut tables);

        // VERIFY: Delete should fail for nonexistent entry
        assert!(result.is_err(), "Delete should fail for nonexistent entry");
    }
    txn.commit()?;

    cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_multiple_creates_and_verify_all() -> NetabaseResult<()> {
    let (store, db_path) = create_test_db::<Definition>("multiple_creates")?;

    let users = vec![
        ("user1", "Alice", 30),
        ("user2", "Bob", 25),
        ("user3", "Charlie", 35),
    ];

    // Create all users
    let txn = store.begin_transaction()?;
    for (id, name, age) in &users {
        let user = User {
            id: UserID(id.to_string()),
            name: name.to_string(),
            age: *age,
            partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
            category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
            subscriptions: vec![],
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile(vec![]),
        };
        txn.create_redb(&user)?;
    }
    txn.commit()?;

    // VERIFY: All users exist with correct data
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        for (id, expected_name, expected_age) in &users {
            let read = User::read_default(&UserID(id.to_string()), &tables)?;

            assert!(read.is_some(), "User {} should exist", id);
            let read = read.unwrap();
            assert_eq!(read.name, *expected_name, "User {} name should match", id);
            assert_eq!(read.age, *expected_age, "User {} age should match", id);
        }
    }
    txn.commit()?;

    cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_transaction_rollback_on_drop() -> NetabaseResult<()> {
    let (store, db_path) = create_test_db::<Definition>("rollback")?;

    let user_id = UserID("rollback_user".to_string());
    let user = User {
        id: user_id.clone(),
        name: "Should Not Persist".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        subscriptions: vec![],
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    // Create but don't commit (drop transaction)
    {
        let txn = store.begin_transaction()?;
        txn.create_redb(&user)?;
        // Transaction dropped here without commit
    }

    // VERIFY: User should not exist (transaction rolled back)
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        let exists = User::read_default(&user_id, &tables)?;
        assert!(exists.is_none(), "User should not exist after rollback");
    }
    txn.commit()?;

    cleanup_test_db(db_path);
    Ok(())
}
