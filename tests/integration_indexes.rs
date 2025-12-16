// Integration tests for secondary keys, relational keys, and subscription indexes

mod common;

use common::{cleanup_test_db, create_test_db};
use netabase_store::databases::redb::transaction::RedbModelCrud;
use netabase_store::errors::NetabaseResult;
use netabase_store::relational::RelationalLink;
use netabase_store::traits::registery::models::model::{NetabaseModel, RedbNetbaseModel};

use netabase_store_examples::models::post::{Post, PostID};
use netabase_store_examples::models::user::{User, UserID};
use netabase_store_examples::{CategoryID, Definition, DefinitionSubscriptions};

#[test]
fn test_secondary_key_indexes_created() -> NetabaseResult<()> {
    let (store, db_path) = create_test_db::<Definition>("secondary_indexes")?;

    // Create users with same names and ages to test multimap behavior
    let users = vec![
        ("user1", "Alice", 30),
        ("user2", "Alice", 25), // Same name, different age
        ("user3", "Bob", 30),   // Different name, same age as user1
    ];

    let txn = store.begin_transaction()?;
    for (id, name, age) in &users {
        let user = User {
            id: UserID(id.to_string()),
            name: name.to_string(),
            age: *age,
            partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
            category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
            subscriptions: vec![],
        };
        txn.create_redb(&user)?;
    }
    txn.commit()?;

    // VERIFY: All users can be read back
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs)?;

        // Verify user1 exists
        let user1 = User::read_entry(&UserID("user1".to_string()), &tables)?;
        assert!(user1.is_some());
        assert_eq!(user1.unwrap().name, "Alice");

        // Verify user2 exists
        let user2 = User::read_entry(&UserID("user2".to_string()), &tables)?;
        assert!(user2.is_some());
        assert_eq!(user2.unwrap().age, 25);

        // Verify user3 exists
        let user3 = User::read_entry(&UserID("user3".to_string()), &tables)?;
        assert!(user3.is_some());
        assert_eq!(user3.unwrap().name, "Bob");
    }
    txn.commit()?;

    // TODO: Once secondary key query methods are implemented, verify:
    // - Query by Name("Alice") returns [user1, user2]
    // - Query by Age(30) returns [user1, user3]
    // - Query by Name("Bob") returns [user3]

    cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_secondary_index_update() -> NetabaseResult<()> {
    let (store, db_path) = create_test_db::<Definition>("secondary_index_update")?;

    let user_id = UserID("update_secondary".to_string());

    // Create user with name "Alice"
    let user = User {
        id: user_id.clone(),
        name: "Alice".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        subscriptions: vec![],
    };

    let txn = store.begin_transaction()?;
    txn.create_redb(&user)?;
    txn.commit()?;

    // Update to name "Bob"
    let updated_user = User {
        id: user_id.clone(),
        name: "Bob".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        subscriptions: vec![],
    };

    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let mut tables = txn.open_model_tables(table_defs)?;
        updated_user.update_entry(&mut tables)?;
    }
    txn.commit()?;

    // VERIFY: User has new name
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs)?;

        let user = User::read_entry(&user_id, &tables)?;
        assert!(user.is_some());
        assert_eq!(user.unwrap().name, "Bob", "Name should be updated");
    }
    txn.commit()?;

    // TODO: Once query methods implemented, verify:
    // - Query by Name("Alice") returns empty
    // - Query by Name("Bob") returns [user_id]

    cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_relational_key_indexes_created() -> NetabaseResult<()> {
    let (store, db_path) = create_test_db::<Definition>("relational_indexes")?;

    // Create two users with partner relationship
    let user1_id = UserID("user1".to_string());
    let user2_id = UserID("user2".to_string());

    let user1 = User {
        id: user1_id.clone(),
        name: "User1".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(user2_id.clone()),
        category: RelationalLink::new_dehydrated(CategoryID("cat1".to_string())),
        subscriptions: vec![],
    };

    let user2 = User {
        id: user2_id.clone(),
        name: "User2".to_string(),
        age: 28,
        partner: RelationalLink::new_dehydrated(user1_id.clone()),
        category: RelationalLink::new_dehydrated(CategoryID("cat1".to_string())),
        subscriptions: vec![],
    };

    let txn = store.begin_transaction()?;
    txn.create_redb(&user1)?;
    txn.create_redb(&user2)?;
    txn.commit()?;

    // VERIFY: Both users exist with correct partner references
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs)?;

        let user1_read = User::read_entry(&user1_id, &tables)?;
        assert!(user1_read.is_some());
        let user1_read = user1_read.unwrap();
        assert_eq!(user1_read.partner.get_primary_key().0, "user2");

        let user2_read = User::read_entry(&user2_id, &tables)?;
        assert!(user2_read.is_some());
        let user2_read = user2_read.unwrap();
        assert_eq!(user2_read.partner.get_primary_key().0, "user1");
    }
    txn.commit()?;

    // TODO: Once relational query methods implemented, verify:
    // - Query by Partner(user2_id) returns [user1_id]
    // - Query by Partner(user1_id) returns [user2_id]
    // - Query by Category(cat1) returns [user1_id, user2_id]

    cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_post_author_relationship() -> NetabaseResult<()> {
    let (store, db_path) = create_test_db::<Definition>("post_author")?;

    // Create user (author)
    let author_id = UserID("author1".to_string());
    let author = User {
        id: author_id.clone(),
        name: "Author".to_string(),
        age: 35,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        subscriptions: vec![],
    };

    let txn = store.begin_transaction()?;
    txn.create_redb(&author)?;
    txn.commit()?;

    // Create posts by this author
    let post_ids = vec!["post1", "post2", "post3"];

    let txn = store.begin_transaction()?;
    for post_id in &post_ids {
        let post = Post {
            id: PostID(post_id.to_string()),
            title: format!("Post {}", post_id),
            author: RelationalLink::new_dehydrated(author_id.clone()),
        };
        txn.create_redb(&post)?;
    }
    txn.commit()?;

    // VERIFY: All posts exist with correct author
    let txn = store.begin_transaction()?;
    {
        let table_defs = Post::table_definitions();
        let tables = txn.open_model_tables(table_defs)?;

        for post_id in &post_ids {
            let post = Post::read_entry(&PostID(post_id.to_string()), &tables)?;
            assert!(post.is_some(), "Post {} should exist", post_id);

            let post = post.unwrap();
            assert_eq!(post.title, format!("Post {}", post_id));
            assert_eq!(
                post.author.get_primary_key().0,
                "author1",
                "Post {} should have correct author",
                post_id
            );
        }
    }
    txn.commit()?;

    // TODO: Once relational query implemented, verify:
    // - Query posts by Author(author1) returns [post1, post2, post3]

    cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_relational_key_update() -> NetabaseResult<()> {
    let (store, db_path) = create_test_db::<Definition>("relational_update")?;

    let user_id = UserID("user_rel_update".to_string());
    let old_partner_id = UserID("old_partner".to_string());
    let new_partner_id = UserID("new_partner".to_string());

    // Create with old partner
    let user = User {
        id: user_id.clone(),
        name: "User".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(old_partner_id.clone()),
        category: RelationalLink::new_dehydrated(CategoryID("cat1".to_string())),
        subscriptions: vec![],
    };

    let txn = store.begin_transaction()?;
    txn.create_redb(&user)?;
    txn.commit()?;

    // VERIFY: Has old partner
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs)?;

        let user = User::read_entry(&user_id, &tables)?;
        assert_eq!(user.unwrap().partner.get_primary_key().0, "old_partner");
    }
    txn.commit()?;

    // Update to new partner
    let updated_user = User {
        id: user_id.clone(),
        name: "User".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(new_partner_id.clone()),
        category: RelationalLink::new_dehydrated(CategoryID("cat2".to_string())),
        subscriptions: vec![],
    };

    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let mut tables = txn.open_model_tables(table_defs)?;
        updated_user.update_entry(&mut tables)?;
    }
    txn.commit()?;

    // VERIFY: Has new partner
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs)?;

        let user = User::read_entry(&user_id, &tables)?;
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

    // TODO: Once query implemented, verify:
    // - Query by Partner(old_partner_id) returns empty
    // - Query by Partner(new_partner_id) returns [user_id]

    cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_subscription_indexes_created() -> NetabaseResult<()> {
    let (store, db_path) = create_test_db::<Definition>("subscription_indexes")?;

    // Create users with different subscriptions
    let user1 = User {
        id: UserID("sub_user1".to_string()),
        name: "User1".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        subscriptions: vec![
            DefinitionSubscriptions::Topic1,
            DefinitionSubscriptions::Topic2,
        ],
    };

    let user2 = User {
        id: UserID("sub_user2".to_string()),
        name: "User2".to_string(),
        age: 25,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        subscriptions: vec![DefinitionSubscriptions::Topic1], // Only Topic1
    };

    let txn = store.begin_transaction()?;
    txn.create_redb(&user1)?;
    txn.create_redb(&user2)?;
    txn.commit()?;

    // VERIFY: Users exist with correct subscriptions
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs)?;

        let user1_read = User::read_entry(&UserID("sub_user1".to_string()), &tables)?;
        assert!(user1_read.is_some());
        assert_eq!(user1_read.unwrap().subscriptions.len(), 2);

        let user2_read = User::read_entry(&UserID("sub_user2".to_string()), &tables)?;
        assert!(user2_read.is_some());
        assert_eq!(user2_read.unwrap().subscriptions.len(), 1);
    }
    txn.commit()?;

    // TODO: Once subscription query methods implemented, verify:
    // - Query subscribers to Topic1 returns [sub_user1, sub_user2]
    // - Query subscribers to Topic2 returns [sub_user1]

    cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_subscription_update() -> NetabaseResult<()> {
    let (store, db_path) = create_test_db::<Definition>("subscription_update")?;

    let user_id = UserID("sub_update_user".to_string());

    // Create with Topic1 subscription
    let user = User {
        id: user_id.clone(),
        name: "User".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        subscriptions: vec![DefinitionSubscriptions::Topic1],
    };

    let txn = store.begin_transaction()?;
    txn.create_redb(&user)?;
    txn.commit()?;

    // Update to Topic2 subscription
    let updated_user = User {
        id: user_id.clone(),
        name: "User".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
        subscriptions: vec![DefinitionSubscriptions::Topic2],
    };

    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let mut tables = txn.open_model_tables(table_defs)?;
        updated_user.update_entry(&mut tables)?;
    }
    txn.commit()?;

    // VERIFY: Has new subscription
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs)?;

        let user = User::read_entry(&user_id, &tables)?;
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.subscriptions.len(), 1);
        assert!(matches!(
            user.subscriptions[0],
            DefinitionSubscriptions::Topic2
        ));
    }
    txn.commit()?;

    // TODO: Once query implemented, verify:
    // - Query subscribers to Topic1 returns empty
    // - Query subscribers to Topic2 returns [user_id]

    cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_delete_cleans_all_indexes() -> NetabaseResult<()> {
    let (store, db_path) = create_test_db::<Definition>("delete_indexes")?;

    let user_id = UserID("delete_all_indexes".to_string());

    // Create user with all index types
    let user = User {
        id: user_id.clone(),
        name: "Delete Me".to_string(),
        age: 40,
        partner: RelationalLink::new_dehydrated(UserID("partner".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("cat".to_string())),
        subscriptions: vec![
            DefinitionSubscriptions::Topic1,
            DefinitionSubscriptions::Topic2,
        ],
    };

    let txn = store.begin_transaction()?;
    txn.create_redb(&user)?;
    txn.commit()?;

    // VERIFY: User exists
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs)?;

        assert!(User::read_entry(&user_id, &tables)?.is_some());
    }
    txn.commit()?;

    // Delete user
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let mut tables = txn.open_model_tables(table_defs)?;

        User::delete_entry(&user_id, &mut tables)?;
    }
    txn.commit()?;

    // VERIFY: User is gone
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs)?;

        assert!(
            User::read_entry(&user_id, &tables)?.is_none(),
            "User should be deleted from main table"
        );
    }
    txn.commit()?;

    // TODO: Once query methods implemented, verify:
    // - Query by Name("Delete Me") returns empty (secondary index cleaned)
    // - Query by Age(40) returns empty (secondary index cleaned)
    // - Query by Partner("partner") returns empty (relational index cleaned)
    // - Query by Category("cat") returns empty (relational index cleaned)
    // - Query subscribers to Topic1 returns empty (subscription index cleaned)
    // - Query subscribers to Topic2 returns empty (subscription index cleaned)

    cleanup_test_db(db_path);
    Ok(())
}
