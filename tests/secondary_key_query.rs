//! Test secondary key querying functionality

mod common;

use netabase_store::relational::RelationalLink;
use example::boilerplate_lib::definition::{
    AnotherLargeUserFile, LargeUserFile, User, UserID, UserSecondaryKeys,
    UserFirstName, UserAge,
};
use example::boilerplate_lib::{CategoryID, Definition};

#[test]
fn test_secondary_key_query_basic() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<Definition>("secondary_key_basic")?;

    // Create users with different names and ages
    {
        let txn = store.begin_write()?;

        let alice = User {
            id: UserID("alice1".into()),
            first_name: "Alice".into(),
            last_name: "Smith".into(),
            age: 30,
            partner: RelationalLink::new_dehydrated(UserID("none".into())),
            category: RelationalLink::new_dehydrated(CategoryID("cat1".into())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile::default(),
        };

        let alice2 = User {
            id: UserID("alice2".into()),
            first_name: "Alice".into(),
            last_name: "Jones".into(),
            age: 25,
            partner: RelationalLink::new_dehydrated(UserID("none".into())),
            category: RelationalLink::new_dehydrated(CategoryID("cat1".into())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile::default(),
        };

        let bob = User {
            id: UserID("bob1".into()),
            first_name: "Bob".into(),
            last_name: "Brown".into(),
            age: 30,
            partner: RelationalLink::new_dehydrated(UserID("none".into())),
            category: RelationalLink::new_dehydrated(CategoryID("cat1".into())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile::default(),
        };

        txn.create(&alice)?;
        txn.create(&alice2)?;
        txn.create(&bob)?;
        txn.commit()?;
    }

    // Query by first_name (secondary key)
    {
        let txn = store.begin_read()?;

        // Find all users named "Alice"
        let alices = txn.query_by_secondary_key::<User>(
            &UserSecondaryKeys::FirstName(UserFirstName("Alice".into()))
        )?;

        assert_eq!(alices.len(), 2, "Should find 2 users named Alice");
        
        let names: Vec<_> = alices.iter().map(|u| u.last_name.as_str()).collect();
        assert!(names.contains(&"Smith"));
        assert!(names.contains(&"Jones"));

        // Find users named "Bob"
        let bobs = txn.query_by_secondary_key::<User>(
            &UserSecondaryKeys::FirstName(UserFirstName("Bob".into()))
        )?;

        assert_eq!(bobs.len(), 1, "Should find 1 user named Bob");
        assert_eq!(bobs[0].last_name, "Brown");

        // Query for non-existent name
        let charlies = txn.query_by_secondary_key::<User>(
            &UserSecondaryKeys::FirstName(UserFirstName("Charlie".into()))
        )?;

        assert_eq!(charlies.len(), 0, "Should find no users named Charlie");
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_secondary_key_query_by_age() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<Definition>("secondary_key_age")?;

    // Create users with different ages
    {
        let txn = store.begin_write()?;

        for i in 0..5 {
            let user = User {
                id: UserID(format!("user{}", i)),
                first_name: format!("User{}", i),
                last_name: "Test".into(),
                age: 25 + (i % 3) * 5, // Ages: 25, 30, 35, 25, 30
                partner: RelationalLink::new_dehydrated(UserID("none".into())),
                category: RelationalLink::new_dehydrated(CategoryID("cat1".into())),
                bio: LargeUserFile::default(),
                another: AnotherLargeUserFile::default(),
            };
            txn.create(&user)?;
        }

        txn.commit()?;
    }

    // Query by age
    {
        let txn = store.begin_read()?;

        // Find all 25-year-olds
        let age25 = txn.query_by_secondary_key::<User>(
            &UserSecondaryKeys::Age(UserAge(25))
        )?;
        assert_eq!(age25.len(), 2, "Should find 2 users aged 25");

        // Find all 30-year-olds
        let age30 = txn.query_by_secondary_key::<User>(
            &UserSecondaryKeys::Age(UserAge(30))
        )?;
        assert_eq!(age30.len(), 2, "Should find 2 users aged 30");

        // Find all 35-year-olds
        let age35 = txn.query_by_secondary_key::<User>(
            &UserSecondaryKeys::Age(UserAge(35))
        )?;
        assert_eq!(age35.len(), 1, "Should find 1 user aged 35");

        // Query for non-existent age
        let age40 = txn.query_by_secondary_key::<User>(
            &UserSecondaryKeys::Age(UserAge(40))
        )?;
        assert_eq!(age40.len(), 0, "Should find no users aged 40");
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_secondary_key_query_update() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<Definition>("secondary_key_update")?;

    let user_id = UserID("user1".into());

    // Create a user
    {
        let txn = store.begin_write()?;
        let user = User {
            id: user_id.clone(),
            first_name: "Alice".into(),
            last_name: "Smith".into(),
            age: 30,
            partner: RelationalLink::new_dehydrated(UserID("none".into())),
            category: RelationalLink::new_dehydrated(CategoryID("cat1".into())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile::default(),
        };
        txn.create(&user)?;
        txn.commit()?;
    }

    // Verify we can query by original name
    {
        let txn = store.begin_read()?;
        let alices = txn.query_by_secondary_key::<User>(
            &UserSecondaryKeys::FirstName(UserFirstName("Alice".into()))
        )?;
        assert_eq!(alices.len(), 1);
    }

    // Update the user's name
    {
        let txn = store.begin_write()?;
        let mut user = txn.read::<User>(&user_id)?.unwrap();
        user.first_name = "Bob".into();
        txn.update(&user)?;
        txn.commit()?;
    }

    // Verify old name returns nothing and new name returns the user
    {
        let txn = store.begin_read()?;
        
        let alices = txn.query_by_secondary_key::<User>(
            &UserSecondaryKeys::FirstName(UserFirstName("Alice".into()))
        )?;
        assert_eq!(alices.len(), 0, "Old name should not be found");

        let bobs = txn.query_by_secondary_key::<User>(
            &UserSecondaryKeys::FirstName(UserFirstName("Bob".into()))
        )?;
        assert_eq!(bobs.len(), 1, "New name should be found");
        assert_eq!(bobs[0].id, user_id);
    }

    common::cleanup_test_db(db_path);
    Ok(())
}
