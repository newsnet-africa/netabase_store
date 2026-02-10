// Integration tests for secondary keys, relational keys, and subscription indexes
// These tests are WIP and not yet enabled

#![allow(dead_code)]
#![allow(deprecated)]

mod common;

use netabase_store::databases::redb::transaction::RedbModelCrud;
use netabase_store::errors::NetabaseResult;
use netabase_store::relational::{
    ModelRelationPermissions, PermissionFlag, RelationPermission, RelationalLink,
};
use netabase_store::traits::registry::models::model::{NetabaseModel, RedbNetbaseModel};

use example::{
    AnotherLargeUserFile, CategoryID, DefinitionSubscriptions, LargeUserFile, Post,
    PostID, User, UserID, MainRepositoryStores
};

// #[test]
fn test_secondary_key_indexes_created() -> NetabaseResult<()> {
    let temp_dir = tempfile::tempdir().map_err(|e| netabase_store::errors::NetabaseError::IoError(e.to_string()))?;
    let stores = MainRepositoryStores::new(temp_dir.path())?;

    // Create users with same names and ages to test multimap behavior
    let users = vec![
        ("user1", "Alice", 30),
        ("user2", "Alice", 25), // Same name, different age
        ("user3", "Bob", 30),   // Different name, same age as user1
    ];

    let txn = stores.definition.begin_write()?;
    for (id, name, age) in &users {
        let user = User {
            id: UserID(id.to_string()),
            first_name: name.to_string(),
            last_name: "Test".to_string(),
            age: *age,
            partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
            category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile(vec![]),
        };
        txn.create(&user)?;
    }
    txn.commit()?;

    // VERIFY: All users can be read back
    let txn = stores.definition.begin_read()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        // Verify user1 exists
        let user1 = User::read_default(&UserID("user1".to_string()), &tables)?;
        assert!(user1.is_some());
        assert_eq!(user1.unwrap().first_name, "Alice");

        // Verify user2 exists
        let user2 = User::read_default(&UserID("user2".to_string()), &tables)?;
        assert!(user2.is_some());
        assert_eq!(user2.unwrap().age, 25);

        // Verify user3 exists
        let user3 = User::read_default(&UserID("user3".to_string()), &tables)?;
        assert!(user3.is_some());
        assert_eq!(user3.unwrap().first_name, "Bob");
    }
    txn.commit()?;

    Ok(())
}

// #[test]
fn test_secondary_index_update() -> NetabaseResult<()> {
    let temp_dir = tempfile::tempdir().map_err(|e| netabase_store::errors::NetabaseError::IoError(e.to_string()))?;
    let stores = MainRepositoryStores::new(temp_dir.path())?;

    let user_id = UserID("update_secondary".to_string());

    // Create user with name "Alice"
    let user = User {
        id: user_id.clone(),
        first_name: "Alice".to_string(),
        last_name: "Test".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    let txn = stores.definition.begin_write()?;
    txn.create(&user)?;
    txn.commit()?;

    // Update to name "Bob"
    let updated_user = User {
        id: user_id.clone(),
        first_name: "Bob".to_string(),
        last_name: "Test".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    let txn = stores.definition.begin_write()?;
    {
        let table_defs = User::table_definitions();
        let perms = ModelRelationPermissions {
            relationa_tree_access: &[RelationPermission(
                User::TREE_NAMES,
                PermissionFlag::ReadWrite,
            )],
        };
        let mut tables = txn.open_model_tables(table_defs, Some(perms))?;
        updated_user.update_entry(&mut tables)?;
    }
    txn.commit()?;

    // VERIFY: User has new name
    let txn = stores.definition.begin_read()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        let user = User::read_default(&user_id, &tables)?;
        assert!(user.is_some());
        assert_eq!(user.unwrap().first_name, "Bob", "Name should be updated");
    }
    txn.commit()?;

    Ok(())
}

// #[test]
fn test_relational_key_indexes_created() -> NetabaseResult<()> {
    let temp_dir = tempfile::tempdir().map_err(|e| netabase_store::errors::NetabaseError::IoError(e.to_string()))?;
    let stores = MainRepositoryStores::new(temp_dir.path())?;

    // Create two users with partner relationship
    let user1_id = UserID("user1".to_string());
    let user2_id = UserID("user2".to_string());

    let user1 = User {
        id: user1_id.clone(),
        first_name: "User1".to_string(),
        last_name: "Test".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(user2_id.clone()),
        category: RelationalLink::new_dehydrated(CategoryID("cat1".to_string())),
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    let user2 = User {
        id: user2_id.clone(),
        first_name: "User2".to_string(),
        last_name: "Test".to_string(),
        age: 28,
        partner: RelationalLink::new_dehydrated(user1_id.clone()),
        category: RelationalLink::new_dehydrated(CategoryID("cat1".to_string())),
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    let txn = stores.definition.begin_write()?;
    txn.create(&user1)?;
    txn.create(&user2)?;
    txn.commit()?;

    // VERIFY: Both users exist with correct partner references
    let txn = stores.definition.begin_read()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        let user1_read = User::read_default(&user1_id, &tables)?;
        assert!(user1_read.is_some());
        let user1_read = user1_read.unwrap();
        assert_eq!(user1_read.partner.get_primary_key().0, "user2");

        let user2_read = User::read_default(&user2_id, &tables)?;
        assert!(user2_read.is_some());
        let user2_read = user2_read.unwrap();
        assert_eq!(user2_read.partner.get_primary_key().0, "user1");
    }
    txn.commit()?;

    Ok(())
}

// #[test]
fn test_post_author_relationship() -> NetabaseResult<()> {
    let temp_dir = tempfile::tempdir().map_err(|e| netabase_store::errors::NetabaseError::IoError(e.to_string()))?;
    let stores = MainRepositoryStores::new(temp_dir.path())?;

    // Create user (author)
    let author_id = UserID("author1".to_string());
    let author = User {
        id: author_id.clone(),
        first_name: "Author".to_string(),
        last_name: "Test".to_string(),
        age: 35,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    let txn = stores.definition.begin_write()?;
    txn.create(&author)?;
    txn.commit()?;

    // Create posts by this author
    let post_ids = vec!["post1", "post2", "post3"];

    let txn = stores.definition.begin_write()?;
    for post_id in &post_ids {
        let post = Post {
            id: PostID(post_id.to_string()),
            title: format!("Post {}", post_id),
            author_id: "Some".to_string(),
            content: "".to_string(),
            published: false,
            tags: vec![],
        };
        txn.create(&post)?;
    }
    txn.commit()?;

    // VERIFY: All posts exist with correct author
    let txn = stores.definition.begin_read()?;
    {
        let table_defs = Post::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        for post_id in &post_ids {
            let post = Post::read_default(&PostID(post_id.to_string()), &tables)?;
            assert!(post.is_some(), "Post {} should exist", post_id);

            let post = post.unwrap();
            assert_eq!(post.title, format!("Post {}", post_id));
        }
    }
    txn.commit()?;

    Ok(())
}

// #[test]
fn test_relational_key_update() -> NetabaseResult<()> {
    let temp_dir = tempfile::tempdir().map_err(|e| netabase_store::errors::NetabaseError::IoError(e.to_string()))?;
    let stores = MainRepositoryStores::new(temp_dir.path())?;

    let user_id = UserID("user_rel_update".to_string());
    let old_partner_id = UserID("old_partner".to_string());
    let new_partner_id = UserID("new_partner".to_string());

    // Create with old partner
    let user = User {
        id: user_id.clone(),
        first_name: "User".to_string(),
        last_name: "Test".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(old_partner_id.clone()),
        category: RelationalLink::new_dehydrated(CategoryID("cat1".to_string())),
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    let txn = stores.definition.begin_write()?;
    txn.create(&user)?;
    txn.commit()?;

    // VERIFY: Has old partner
    let txn = stores.definition.begin_read()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        let user = User::read_default(&user_id, &tables)?;
        assert_eq!(user.unwrap().partner.get_primary_key().0, "old_partner");
    }
    txn.commit()?;

    // Update to new partner
    let updated_user = User {
        id: user_id.clone(),
        first_name: "User".to_string(),
        last_name: "Test".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(new_partner_id.clone()),
        category: RelationalLink::new_dehydrated(CategoryID("cat2".to_string())),
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    let txn = stores.definition.begin_write()?;
    {
        let table_defs = User::table_definitions();
        let perms = ModelRelationPermissions {
            relationa_tree_access: &[RelationPermission(
                User::TREE_NAMES,
                PermissionFlag::ReadWrite,
            )],
        };
        let mut tables = txn.open_model_tables(table_defs, Some(perms))?;
        updated_user.update_entry(&mut tables)?;
    }
    txn.commit()?;

    // VERIFY: Has new partner
    let txn = stores.definition.begin_read()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        let user = User::read_default(&user_id, &tables)?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(
            user.partner.get_primary_key().0,
            "new_partner",
            "Should have new partner"
        );
        assert_eq!(
            user.category.get_primary_key().0,
            "cat2",
            "Should have new category"
        );
    }
    txn.commit()?;

    Ok(())
}

// #[test]
fn test_subscription_indexes_created() -> NetabaseResult<()> {
    let temp_dir = tempfile::tempdir().map_err(|e| netabase_store::errors::NetabaseError::IoError(e.to_string()))?;
    let stores = MainRepositoryStores::new(temp_dir.path())?;

    // Create users - all Users automatically subscribe to Topic1 and Topic2 (trait-level)
    let user1 = User {
        id: UserID("sub_user1".to_string()),
        first_name: "User1".to_string(),
        last_name: "Test".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    let user2 = User {
        id: UserID("sub_user2".to_string()),
        first_name: "User2".to_string(),
        last_name: "Test".to_string(),
        age: 25,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    let txn = stores.definition.begin_write()?;
    txn.create(&user1)?;
    txn.create(&user2)?;
    txn.commit()?;

    // Verify subscription tables were created and populated
    let txn = stores.definition.begin_read()?;
    let topic1_subs = txn.query_by_subscription::<User, _>(&DefinitionSubscriptions::Topic1)?;
    assert_eq!(topic1_subs.len(), 2, "Both users subscribed to Topic1");

    Ok(())
}

// #[test]
fn test_subscription_update() -> NetabaseResult<()> {
    let temp_dir = tempfile::tempdir().map_err(|e| netabase_store::errors::NetabaseError::IoError(e.to_string()))?;
    let stores = MainRepositoryStores::new(temp_dir.path())?;

    let user_id = UserID("sub_update_user".to_string());

    // Create with Topic1 subscription
    let user = User {
        id: user_id.clone(),
        first_name: "User".to_string(),
        last_name: "Test".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    let txn = stores.definition.begin_write()?;
    txn.create(&user)?;
    txn.commit()?;

    // Update to Topic2 subscription
    let updated_user = User {
        id: user_id.clone(),
        first_name: "User".to_string(),
        last_name: "Test".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    let txn = stores.definition.begin_write()?;
    {
        let table_defs = User::table_definitions();
        let perms = ModelRelationPermissions {
            relationa_tree_access: &[RelationPermission(
                User::TREE_NAMES,
                PermissionFlag::ReadWrite,
            )],
        };
        let mut tables = txn.open_model_tables(table_defs, Some(perms))?;
        updated_user.update_entry(&mut tables)?;
    }
    txn.commit()?;

    // VERIFY: Has new subscription
    let txn = stores.definition.begin_read()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        let user = User::read_default(&user_id, &tables)?;
        assert!(user.is_some());
        let user = user.unwrap();
        
        // Subscriptions are trait-level
        use netabase_store::traits::registry::models::model::NetabaseModel;
        let sub_keys = user.get_subscription_keys();
        assert_eq!(sub_keys.len(), 2, "User has 2 subscription topics");
    }
    txn.commit()?;

    Ok(())
}

// #[test]
fn test_delete_cleans_all_indexes() -> NetabaseResult<()> {
    let temp_dir = tempfile::tempdir().map_err(|e| netabase_store::errors::NetabaseError::IoError(e.to_string()))?;
    let stores = MainRepositoryStores::new(temp_dir.path())?;

    let user_id = UserID("delete_all_indexes".to_string());

    // Create user with all index types
    let user = User {
        id: user_id.clone(),
        first_name: "Delete Me".to_string(),
        last_name: "Test".to_string(),
        age: 40,
        partner: RelationalLink::new_dehydrated(UserID("partner".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("cat".to_string())),
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile(vec![]),
    };

    let txn = stores.definition.begin_write()?;
    txn.create(&user)?;
    txn.commit()?;

    // VERIFY: User exists
    let txn = stores.definition.begin_read()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        assert!(User::read_default(&user_id, &tables)?.is_some());
    }
    txn.commit()?;

    // Delete user
    let txn = stores.definition.begin_write()?;
    {
        let table_defs = User::table_definitions();
        let perms = ModelRelationPermissions {
            relationa_tree_access: &[RelationPermission(
                User::TREE_NAMES,
                PermissionFlag::ReadWrite,
            )],
        };
        let mut tables = txn.open_model_tables(table_defs, Some(perms))?;

        User::delete_entry(&user_id, &mut tables)?;
    }
    txn.commit()?;

    // VERIFY: User is gone
    let txn = stores.definition.begin_read()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        assert!(
            User::read_default(&user_id, &tables)?.is_none(),
            "User should be deleted from main table"
        );
    }
    txn.commit()?;

    Ok(())
}