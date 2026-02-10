// Integration tests for listing and counting entries

#![allow(deprecated)]

pub mod common;

use common::cleanup_test_db;
use netabase_store::databases::redb::transaction::{CrudOptions, RedbModelCrud};
use netabase_store::errors::NetabaseResult;
use netabase_store::relational::RelationalLink;
use netabase_store::traits::registry::models::keys::{ModelKeyRange, SimpleKeyRange};
use netabase_store::traits::registry::models::model::RedbNetbaseModel;

use example::MainRepositoryStores;
use example::{
    AnotherLargeUserFile, CategoryID, LargeUserFile, User, UserID,
};
use example::boilerplate_lib::main_repository::definition::{UserSecondaryKeys, UserFirstName, UserAge};

#[test]
fn test_count_entries() -> NetabaseResult<()> {
    println!("\n--- Starting test_count_entries ---");
    let temp_dir = tempfile::tempdir().map_err(|e| netabase_store::errors::NetabaseError::IoError(e.to_string()))?;
    let stores = MainRepositoryStores::new(temp_dir.path())?;

    // Initial count should be 0
    let txn = stores.definition.begin_read()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;
        let count = User::count_entries(&tables)?;
        assert_eq!(count, 0, "Count should be 0 initially");
    }
    txn.commit()?;

    // Create 5 users
    let txn = stores.definition.begin_write()?;
    for i in 0..5 {
        let user = User {
            id: UserID(format!("user_{}", i)),
            first_name: format!("User {}", i),
            last_name: "Test".to_string(),
            age: 20 + i as u8,
            partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
            category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile(vec![]),
        };
        txn.create(&user)?;
    }
    txn.commit()?;

    // Verify count is 5
    let txn = stores.definition.begin_read()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;
        let count = User::count_entries(&tables)?;
        assert_eq!(count, 5, "Count should be 5 after creation");
    }
    txn.commit()?;

    Ok(())
}

#[test]
fn test_list_entries() -> NetabaseResult<()> {
    println!("\n--- Starting test_list_entries ---");
    let temp_dir = tempfile::tempdir().map_err(|e| netabase_store::errors::NetabaseError::IoError(e.to_string()))?;
    let stores = MainRepositoryStores::new(temp_dir.path())?;

    // Create 3 users
    let txn = stores.definition.begin_write()?;
    for i in 0..3 {
        let user = User {
            id: UserID(format!("user_{}", i)),
            first_name: format!("User {}", i),
            last_name: "Test".to_string(),
            age: 20 + i as u8,
            partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
            category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile(vec![]),
        };
        txn.create(&user)?;
    }
    txn.commit()?;

    // Verify list returns all users
    let txn = stores.definition.begin_read()?;
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

    Ok(())
}

#[test]
fn test_list_entries_pagination() -> NetabaseResult<()> {
    println!("\n--- Starting test_list_entries_pagination ---");
    let temp_dir = tempfile::tempdir().map_err(|e| netabase_store::errors::NetabaseError::IoError(e.to_string()))?;
    let stores = MainRepositoryStores::new(temp_dir.path())?;

    // Create 10 users: user_0 to user_9
    // Lexicographically: user_0, user_1, ..., user_9
    let txn = stores.definition.begin_write()?;
    for i in 0..10 {
        let user = User {
            id: UserID(format!("user_{}", i)),
            first_name: format!("User {}", i),
            last_name: "Test".to_string(),
            age: 20 + i as u8,
            partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
            category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile(vec![]),
        };
        txn.create(&user)?;
    }
    txn.commit()?;

    let txn = stores.definition.begin_read()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        // 1. First page (limit 3, offset 0)
        let page1: Vec<User> = User::list_entries(&tables, CrudOptions::new().with_limit(3))?
            .into_iter()
            .map(|g| g.value())
            .collect();
        println!("Page 1: {:#?}", page1);
        assert_eq!(page1.len(), 3);
        assert_eq!(page1[0].id.0, "user_0");
        assert_eq!(page1[1].id.0, "user_1");
        assert_eq!(page1[2].id.0, "user_2");

        // 2. Second page (limit 3, offset 3)
        let page2: Vec<User> =
            User::list_entries(&tables, CrudOptions::new().with_limit(3).with_offset(3))?
                .into_iter()
                .map(|g| g.value())
                .collect();
        println!("Page 2: {:#?}", page2);
        assert_eq!(page2.len(), 3);
        assert_eq!(page2[0].id.0, "user_3");
        assert_eq!(page2[1].id.0, "user_4");
        assert_eq!(page2[2].id.0, "user_5");

        // 3. Last page (limit 3, offset 9) - should return 1 item (user_9)
        let page4: Vec<User> =
            User::list_entries(&tables, CrudOptions::new().with_limit(3).with_offset(9))?
                .into_iter()
                .map(|g| g.value())
                .collect();
        println!("Page 4: {:#?}", page4);
        assert_eq!(page4.len(), 1);
        assert_eq!(page4[0].id.0, "user_9");

        // 4. Out of bounds offset
        let empty_page: Vec<User> =
            User::list_entries(&tables, CrudOptions::new().with_limit(3).with_offset(100))?
                .into_iter()
                .map(|g| g.value())
                .collect();
        println!("Empty Page: {:?}", empty_page);
        assert_eq!(empty_page.len(), 0);
    }
    txn.commit()?;

    Ok(())
}

#[test]
fn test_list_range() -> NetabaseResult<()> {
    println!("\n--- Starting test_list_range ---");
    let temp_dir = tempfile::tempdir().map_err(|e| netabase_store::errors::NetabaseError::IoError(e.to_string()))?;
    let stores = MainRepositoryStores::new(temp_dir.path())?;

    // Create users: a_user, b_user, c_user, d_user, e_user
    let names = vec!["a_user", "b_user", "c_user", "d_user", "e_user"];

    let txn = stores.definition.begin_write()?;
    for name in &names {
        let user = User {
            id: UserID(name.to_string()),
            first_name: name.to_string(),
            last_name: "Test".to_string(),
            age: 25,
            partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
            category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile(vec![]),
        };
        txn.create(&user)?;
    }
    txn.commit()?;

    let txn = stores.definition.begin_read()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        // Range: b_user to d_user (inclusive start, exclusive end)
        // Should include: b_user, c_user
        let range = UserID("b_user".to_string())..UserID("d_user".to_string());
        let result: Vec<User> = User::list_range(&tables, range, CrudOptions::default())?
            .into_iter()
            .map(|g| g.value())
            .collect();
        println!("Range (b..d): {:#?}", result);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id.0, "b_user");
        assert_eq!(result[1].id.0, "c_user");

        // Range inclusive: b_user to d_user=
        // Should include: b_user, c_user, d_user
        let range_inclusive = UserID("b_user".to_string())..=UserID("d_user".to_string());
        let result_inc: Vec<User> =
            User::list_range(&tables, range_inclusive, CrudOptions::default())?
                .into_iter()
                .map(|g| g.value())
                .collect();
        println!("Range Inclusive (b..=d): {:#?}", result_inc);

        assert_eq!(result_inc.len(), 3);
        assert_eq!(result_inc[0].id.0, "b_user");
        assert_eq!(result_inc[1].id.0, "c_user");
        assert_eq!(result_inc[2].id.0, "d_user");

        // Range with pagination
        // b_user..=e_user -> b, c, d, e
        // offset 1, limit 2 -> c, d
        let range_page = UserID("b_user".to_string())..=UserID("e_user".to_string());
        let result_page: Vec<User> = User::list_range(
            &tables,
            range_page,
            CrudOptions::new().with_limit(2).with_offset(1),
        )?
        .into_iter()
        .map(|g| g.value())
        .collect();
        println!("Range Page (b..=e, skip 1, limit 2): {:#?}", result_page);

        assert_eq!(result_page.len(), 2);
        assert_eq!(result_page[0].id.0, "c_user");
        assert_eq!(result_page[1].id.0, "d_user");
    }
    txn.commit()?;

    Ok(())
}

#[test]
fn test_model_key_range_primary_only() -> NetabaseResult<()> {
    println!("\n--- Starting test_model_key_range_primary_only ---");
    let temp_dir = tempfile::tempdir().map_err(|e| netabase_store::errors::NetabaseError::IoError(e.to_string()))?;
    let stores = MainRepositoryStores::new(temp_dir.path())?;

    let names = vec!["a_user", "b_user", "c_user", "d_user", "e_user"];

    let txn = stores.definition.begin_write()?;
    for name in &names {
        let user = User {
            id: UserID(name.to_string()),
            first_name: name.to_string(),
            last_name: "Test".to_string(),
            age: 25,
            partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
            category: RelationalLink::new_dehydrated(CategoryID("none".to_string())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile(vec![]),
        };
        txn.create(&user)?;
    }
    txn.commit()?;

    let txn = stores.definition.begin_read()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        // Test primary-only range query
        let ranges = ModelKeyRange::<example::Definition, User>::with_primary(SimpleKeyRange::Between {
            start: UserID("b_user".to_string()),
            end: UserID("d_user".to_string()),
            start_inclusive: true,
            end_inclusive: false,
        });

        let result: Vec<User> = User::list_with_key_ranges(&tables, &ranges, CrudOptions::default())?
            .into_iter()
            .map(|g| g.value())
            .collect();

        println!("Range query result: {:?}", result.iter().map(|u| &u.id.0).collect::<Vec<_>>());

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id.0, "b_user");
        assert_eq!(result[1].id.0, "c_user");
    }
    txn.commit()?;

    Ok(())
}

/// Test secondary key range intersection.
///
/// Creates users with varying ages and names, then queries using secondary key ranges
/// to verify that the intersection logic works correctly.
#[test]
fn test_model_key_range_secondary_intersection() -> NetabaseResult<()> {
    println!("\n--- Starting test_model_key_range_secondary_intersection ---");
    let temp_dir = tempfile::tempdir().map_err(|e| netabase_store::errors::NetabaseError::IoError(e.to_string()))?;
    let stores = MainRepositoryStores::new(temp_dir.path())?;

    // Create users with varying ages and first names
    // age: 20, 25, 30, 35, 40
    // first_name: alice, bob, carol, dave, eve
    let users_data = vec![
        ("user_alice", "alice", 20u8),
        ("user_bob", "bob", 25),
        ("user_carol", "carol", 30),
        ("user_dave", "dave", 35),
        ("user_eve", "eve", 40),
    ];

    let txn = stores.definition.begin_write()?;
    for (id, first_name, age) in &users_data {
        let user = User {
            id: UserID(id.to_string()),
            first_name: first_name.to_string(),
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

    // Test 1: Secondary key range only (age 25-35 inclusive)
    println!("\n--- Test 1: Secondary key range only (age 25-35) ---");
    let txn = stores.definition.begin_read()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        let ranges = ModelKeyRange::<example::Definition, User>::new()
            .and_secondary(SimpleKeyRange::Between {
                start: UserSecondaryKeys::Age(UserAge(25)),
                end: UserSecondaryKeys::Age(UserAge(35)),
                start_inclusive: true,
                end_inclusive: true,
            });

        let result: Vec<User> = User::list_with_key_ranges(&tables, &ranges, CrudOptions::default())?
            .into_iter()
            .map(|g| g.value())
            .collect();

        println!("Age 25-35 result: {:?}", result.iter().map(|u| (&u.id.0, u.age)).collect::<Vec<_>>());

        // Should match bob(25), carol(30), dave(35)
        assert_eq!(result.len(), 3, "Expected 3 users with age 25-35");
        let ages: Vec<u8> = result.iter().map(|u| u.age).collect();
        assert!(ages.contains(&25));
        assert!(ages.contains(&30));
        assert!(ages.contains(&35));
    }
    txn.commit()?;

    // Test 2: Primary + Secondary intersection
    println!("\n--- Test 2: Primary (user_bob..=user_eve) + Secondary (age < 35) ---");
    let txn = stores.definition.begin_read()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        let ranges = ModelKeyRange::<example::Definition, User>::with_primary(
            SimpleKeyRange::Between {
                start: UserID("user_bob".to_string()),
                end: UserID("user_eve".to_string()),
                start_inclusive: true,
                end_inclusive: true,
            }
        ).and_secondary(SimpleKeyRange::To {
            end: UserSecondaryKeys::Age(UserAge(35)),
            inclusive: false,
        });

        let result: Vec<User> = User::list_with_key_ranges(&tables, &ranges, CrudOptions::default())?
            .into_iter()
            .map(|g| g.value())
            .collect();

        println!("Primary bob..eve + age<35 result: {:?}", result.iter().map(|u| (&u.id.0, u.age)).collect::<Vec<_>>());

        // Primary range: bob, carol, dave, eve
        // Age < 35: alice(20), bob(25), carol(30)
        // Intersection: bob(25), carol(30)
        assert_eq!(result.len(), 2, "Expected 2 users in intersection");
        let ids: Vec<&str> = result.iter().map(|u| u.id.0.as_str()).collect();
        assert!(ids.contains(&"user_bob"));
        assert!(ids.contains(&"user_carol"));
    }
    txn.commit()?;

    // Test 3: Multiple secondary ranges (intersection of age ranges)
    println!("\n--- Test 3: Two secondary ranges (age >= 25 AND age <= 35) ---");
    let txn = stores.definition.begin_read()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        let ranges = ModelKeyRange::<example::Definition, User>::new()
            .and_secondary(SimpleKeyRange::From {
                start: UserSecondaryKeys::Age(UserAge(25)),
            })
            .and_secondary(SimpleKeyRange::To {
                end: UserSecondaryKeys::Age(UserAge(35)),
                inclusive: true,
            });

        let result: Vec<User> = User::list_with_key_ranges(&tables, &ranges, CrudOptions::default())?
            .into_iter()
            .map(|g| g.value())
            .collect();

        println!("age>=25 AND age<=35 result: {:?}", result.iter().map(|u| (&u.id.0, u.age)).collect::<Vec<_>>());

        // age >= 25: bob(25), carol(30), dave(35), eve(40)
        // age <= 35: alice(20), bob(25), carol(30), dave(35)
        // Intersection: bob(25), carol(30), dave(35)
        assert_eq!(result.len(), 3, "Expected 3 users in intersection");
    }
    txn.commit()?;

    // Test 4: Secondary range with first_name
    println!("\n--- Test 4: Secondary range by first_name (bob..dave) ---");
    let txn = stores.definition.begin_read()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        let ranges = ModelKeyRange::<example::Definition, User>::new()
            .and_secondary(SimpleKeyRange::Between {
                start: UserSecondaryKeys::FirstName(UserFirstName("bob".to_string())),
                end: UserSecondaryKeys::FirstName(UserFirstName("dave".to_string())),
                start_inclusive: true,
                end_inclusive: true,
            });

        let result: Vec<User> = User::list_with_key_ranges(&tables, &ranges, CrudOptions::default())?
            .into_iter()
            .map(|g| g.value())
            .collect();

        println!("first_name bob..dave result: {:?}", result.iter().map(|u| (&u.id.0, &u.first_name)).collect::<Vec<_>>());

        // first_name in [bob, dave]: bob, carol, dave (alphabetically)
        assert_eq!(result.len(), 3, "Expected 3 users with first_name in [bob..dave]");
        let names: Vec<&str> = result.iter().map(|u| u.first_name.as_str()).collect();
        assert!(names.contains(&"bob"));
        assert!(names.contains(&"carol"));
        assert!(names.contains(&"dave"));
    }
    txn.commit()?;

    // Test 5: Empty intersection (no matching results)
    println!("\n--- Test 5: Empty intersection (primary alice..bob + age > 40) ---");
    let txn = stores.definition.begin_read()?;
    {
        let table_defs = User::table_definitions();
        let tables = txn.open_model_tables(table_defs, None)?;

        let ranges = ModelKeyRange::<example::Definition, User>::with_primary(
            SimpleKeyRange::Between {
                start: UserID("user_alice".to_string()),
                end: UserID("user_bob".to_string()),
                start_inclusive: true,
                end_inclusive: true,
            }
        ).and_secondary(SimpleKeyRange::From {
            start: UserSecondaryKeys::Age(UserAge(41)),
        });

        let result: Vec<User> = User::list_with_key_ranges(&tables, &ranges, CrudOptions::default())?
            .into_iter()
            .map(|g| g.value())
            .collect();

        println!("Empty intersection result: {:?}", result.len());

        // Primary: alice, bob (ages 20, 25)
        // Age > 40: none in this primary range
        assert_eq!(result.len(), 0, "Expected empty result for non-overlapping ranges");
    }
    txn.commit()?;

    Ok(())
}

