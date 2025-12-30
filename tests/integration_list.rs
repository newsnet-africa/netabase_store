// Integration tests for listing and counting entries

pub mod common;

use common::{cleanup_test_db, create_test_db};
use netabase_store::databases::redb::transaction::{RedbModelCrud, QueryConfig};
use netabase_store::errors::NetabaseResult;
use netabase_store::relational::{RelationalLink};
use netabase_store::traits::registery::models::model::{NetabaseModel, RedbNetbaseModel};

use netabase_store_examples::{
    AnotherLargeUserFile, CategoryID, Definition, DefinitionSubscriptions, LargeUserFile, User, UserID,
};

#[test]
fn test_count_entries() -> NetabaseResult<()> {
    println!("\n--- Starting test_count_entries ---");
    let (store, db_path) = create_test_db::<Definition>("count_entries")?;

    // Initial count should be 0
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;
        let count = User::count_entries(&tables)?;
        assert_eq!(count, 0, "Count should be 0 initially");
    }
    txn.commit()?;

    // Create 5 users
    let txn = store.begin_transaction()?;
    for i in 0..5 {
        let user = User {
            id: UserID(format!("user_{}", i)),
            name: format!("User {}", i),
            age: 20 + i as u8,
            partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
            category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
            subscriptions: vec![],
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile(vec![]),
        };
        txn.create_redb(&user)?;
    }
    txn.commit()?;

    // Verify count is 5
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;
        let count = User::count_entries(&tables)?;
        assert_eq!(count, 5, "Count should be 5 after creation");
    }
    txn.commit()?;

    cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_list_entries() -> NetabaseResult<()> {
    println!("\n--- Starting test_list_entries ---");
    let (store, db_path) = create_test_db::<Definition>("list_entries")?;

    // Create 3 users
    let txn = store.begin_transaction()?;
    for i in 0..3 {
        let user = User {
            id: UserID(format!("user_{}", i)),
            name: format!("User {}", i),
            age: 20 + i as u8,
            partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
            category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
            subscriptions: vec![],
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile(vec![]),
        };
        txn.create_redb(&user)?;
    }
    txn.commit()?;

    // Verify list returns all users
    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;
        
        let users = User::list_default(&tables)?;
        println!("Listed users: {:#?}", users);
        assert_eq!(users.len(), 3, "Should list 3 users");
        
        // Check ids are present (order might depend on key sorting)
        let ids: Vec<String> = users.iter().map(|u| u.id.0.clone()).collect();
        assert!(ids.contains(&"user_0".to_string()));
        assert!(ids.contains(&"user_1".to_string()));
        assert!(ids.contains(&"user_2".to_string()));
    }
    txn.commit()?;

    cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_list_entries_pagination() -> NetabaseResult<()> {
    println!("\n--- Starting test_list_entries_pagination ---");
    let (store, db_path) = create_test_db::<Definition>("list_pagination")?;

    // Create 10 users: user_0 to user_9
    // Lexicographically: user_0, user_1, ..., user_9
    let txn = store.begin_transaction()?;
    for i in 0..10 {
        let user = User {
            id: UserID(format!("user_{}", i)),
            name: format!("User {}", i),
            age: 20 + i as u8,
            partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
            category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
            subscriptions: vec![],
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile(vec![]),
        };
        txn.create_redb(&user)?;
    }
    txn.commit()?;

    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;
        
        // 1. First page (limit 3, offset 0)
        let page1: Vec<User> = User::list_entries(&tables, QueryConfig::new().with_limit(3))?.into_iter().map(|g| g.value()).collect();
        println!("Page 1: {:#?}", page1);
        assert_eq!(page1.len(), 3);
        assert_eq!(page1[0].id.0, "user_0");
        assert_eq!(page1[1].id.0, "user_1");
        assert_eq!(page1[2].id.0, "user_2");

        // 2. Second page (limit 3, offset 3)
        let page2: Vec<User> = User::list_entries(&tables, QueryConfig::new().with_limit(3).with_offset(3))?.into_iter().map(|g| g.value()).collect();
        println!("Page 2: {:#?}", page2);
        assert_eq!(page2.len(), 3);
        assert_eq!(page2[0].id.0, "user_3");
        assert_eq!(page2[1].id.0, "user_4");
        assert_eq!(page2[2].id.0, "user_5");
        
        // 3. Last page (limit 3, offset 9) - should return 1 item (user_9)
        let page4: Vec<User> = User::list_entries(&tables, QueryConfig::new().with_limit(3).with_offset(9))?.into_iter().map(|g| g.value()).collect();
        println!("Page 4: {:#?}", page4);
        assert_eq!(page4.len(), 1);
        assert_eq!(page4[0].id.0, "user_9");

        // 4. Out of bounds offset
        let empty_page: Vec<User> = User::list_entries(&tables, QueryConfig::new().with_limit(3).with_offset(100))?.into_iter().map(|g| g.value()).collect();
        println!("Empty Page: {:?}", empty_page);
        assert_eq!(empty_page.len(), 0);
    }
    txn.commit()?;

    cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_list_range() -> NetabaseResult<()> {
    println!("\n--- Starting test_list_range ---");
    let (store, db_path) = create_test_db::<Definition>("list_range")?;

    // Create users: a_user, b_user, c_user, d_user, e_user
    let names = vec!["a_user", "b_user", "c_user", "d_user", "e_user"];
    
    let txn = store.begin_transaction()?;
    for name in &names {
        let user = User {
            id: UserID(name.to_string()),
            name: name.to_string(),
            age: 25,
            partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
            category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
            subscriptions: vec![],
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile(vec![]),
        };
        txn.create_redb(&user)?;
    }
    txn.commit()?;

    let txn = store.begin_transaction()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;
        
        // Range: b_user to d_user (inclusive start, exclusive end)
        // Should include: b_user, c_user
        let range = UserID("b_user".to_string())..UserID("d_user".to_string());
        let result: Vec<User> = User::list_range(&tables, range, QueryConfig::default())?.into_iter().map(|g| g.value()).collect();
        println!("Range (b..d): {:#?}", result);
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id.0, "b_user");
        assert_eq!(result[1].id.0, "c_user");
        
        // Range inclusive: b_user to d_user=
        // Should include: b_user, c_user, d_user
        let range_inclusive = UserID("b_user".to_string())..=UserID("d_user".to_string());
        let result_inc: Vec<User> = User::list_range(&tables, range_inclusive, QueryConfig::default())?.into_iter().map(|g| g.value()).collect();
        println!("Range Inclusive (b..=d): {:#?}", result_inc);
        
        assert_eq!(result_inc.len(), 3);
        assert_eq!(result_inc[0].id.0, "b_user");
        assert_eq!(result_inc[1].id.0, "c_user");
        assert_eq!(result_inc[2].id.0, "d_user");
        
        // Range with pagination
        // b_user..=e_user -> b, c, d, e
        // offset 1, limit 2 -> c, d
        let range_page = UserID("b_user".to_string())..=UserID("e_user".to_string());
        let result_page: Vec<User> = User::list_range(&tables, range_page, QueryConfig::new().with_limit(2).with_offset(1))?.into_iter().map(|g| g.value()).collect();
        println!("Range Page (b..=e, skip 1, limit 2): {:#?}", result_page);
        
        assert_eq!(result_page.len(), 2);
        assert_eq!(result_page[0].id.0, "c_user");
        assert_eq!(result_page[1].id.0, "d_user");
    }
    txn.commit()?;

    cleanup_test_db(db_path);
    Ok(())
}
