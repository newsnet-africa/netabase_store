//! Test subscription and relational key query functionality

mod common;

use netabase_store::relational::RelationalLink;
use netabase_store_examples::boilerplate_lib::definition::{
    AnotherLargeUserFile, LargeUserFile, User, UserID, UserRelationalKeys,
    UserCategory,
};
use netabase_store_examples::boilerplate_lib::{CategoryID, Definition, DefinitionSubscriptions};

#[test]
fn test_query_by_subscription() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<Definition>("subscription_query")?;

    // Subscriptions are trait-level:
    // - User model subscribes to Topic1 and Topic2 (via #[subscribe(Topic1, Topic2)])
    // - Post model subscribes to Topic3 and Topic4 (via #[subscribe(Topic3, Topic4)])
    
    // Create users - ALL Users subscribe to Topic1 and Topic2
    {
        let txn = store.begin_write()?;

        for i in 1..=3 {
            let user = User {
                id: UserID(format!("user{}", i)),
                first_name: format!("User{}", i),
                last_name: "Smith".into(),
                age: 20 + i,
                partner: RelationalLink::new_dehydrated(UserID("none".into())),
                category: RelationalLink::new_dehydrated(CategoryID("cat1".into())),
                bio: LargeUserFile::default(),
                another: AnotherLargeUserFile::default(),
            };
            txn.create(&user)?;
        }
        txn.commit()?;
    }

    // Query by Topic1 - all Users subscribe
    {
        let txn = store.begin_read()?;
        let results = txn.query_by_subscription::<User, _>(&DefinitionSubscriptions::Topic1)?;
        
        assert_eq!(results.len(), 3, "All 3 users subscribed to Topic1");
        
        // Verify hashes are present and unique
        let hashes: std::collections::HashSet<_> = results.iter().map(|h| h.to_hex()).collect();
        assert_eq!(hashes.len(), 3, "Each user has unique hash");
    }

    // Query by Topic2 - all Users also subscribe to this
    {
        let txn = store.begin_read()?;
        let results = txn.query_by_subscription::<User, _>(&DefinitionSubscriptions::Topic2)?;
        
        assert_eq!(results.len(), 3, "All 3 users subscribed to Topic2");
    }

    // Query by Topic3 - Users don't subscribe to this (only Posts do)
    {
        let txn = store.begin_read()?;
        let results = txn.query_by_subscription::<User, _>(&DefinitionSubscriptions::Topic3)?;
        
        assert_eq!(results.len(), 0, "Users not subscribed to Topic3");
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_query_by_relational_key() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<Definition>("relational_query")?;

    let cat1 = CategoryID("tech".into());
    let cat2 = CategoryID("sports".into());

    // Create users linked to different categories
    {
        let txn = store.begin_write()?;

        let user1 = User {
            id: UserID("user1".into()),
            first_name: "Alice".into(),
            last_name: "Smith".into(),
            age: 30,
            partner: RelationalLink::new_dehydrated(UserID("none".into())),
            category: RelationalLink::new_dehydrated(cat1.clone()),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile::default(),
        };

        let user2 = User {
            id: UserID("user2".into()),
            first_name: "Bob".into(),
            last_name: "Jones".into(),
            age: 25,
            partner: RelationalLink::new_dehydrated(UserID("none".into())),
            category: RelationalLink::new_dehydrated(cat1.clone()),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile::default(),
        };

        let user3 = User {
            id: UserID("user3".into()),
            first_name: "Charlie".into(),
            last_name: "Brown".into(),
            age: 35,
            partner: RelationalLink::new_dehydrated(UserID("none".into())),
            category: RelationalLink::new_dehydrated(cat2.clone()),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile::default(),
        };

        txn.create(&user1)?;
        txn.create(&user2)?;
        txn.create(&user3)?;
        txn.commit()?;
    }

    // Query by category "tech"
    {
        let txn = store.begin_read()?;
        let users = txn.query_by_relational_key::<User>(
            &UserRelationalKeys::Category(UserCategory(cat1.clone()))
        )?;
        
        assert_eq!(users.len(), 2, "Should find 2 users in tech category");
        
        let names: Vec<_> = users.iter().map(|u| u.first_name.as_str()).collect();
        assert!(names.contains(&"Alice"));
        assert!(names.contains(&"Bob"));
    }

    // Query by category "sports"
    {
        let txn = store.begin_read()?;
        let users = txn.query_by_relational_key::<User>(
            &UserRelationalKeys::Category(UserCategory(cat2.clone()))
        )?;
        
        assert_eq!(users.len(), 1, "Should find 1 user in sports category");
        assert_eq!(users[0].first_name, "Charlie");
    }

    // Query by non-existent category
    {
        let txn = store.begin_read()?;
        let users = txn.query_by_relational_key::<User>(
            &UserRelationalKeys::Category(UserCategory(CategoryID("music".into())))
        )?;
        
        assert_eq!(users.len(), 0, "Should find no users in music category");
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_subscription_trait_level() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<Definition>("subscription_trait")?;

    // Subscriptions are trait-level - ALL Users subscribe to Topic1 and Topic2
    // This is defined by #[subscribe(Topic1, Topic2)] on the User struct
    
    // Create multiple users
    {
        let txn = store.begin_write()?;
        for i in 1..=3 {
            let user = User {
                id: UserID(format!("user{}", i)),
                first_name: format!("User{}", i),
                last_name: "Test".into(),
                age: 20 + i,
                partner: RelationalLink::new_dehydrated(UserID("none".into())),
                category: RelationalLink::new_dehydrated(CategoryID("cat1".into())),
                bio: LargeUserFile::default(),
                another: AnotherLargeUserFile::default(),
            };
            txn.create(&user)?;
        }
        txn.commit()?;
    }

    // All Users are subscribed to Topic1 (trait-level subscription)
    {
        let txn = store.begin_read()?;
        let results = txn.query_by_subscription::<User, _>(&DefinitionSubscriptions::Topic1)?;
        assert_eq!(results.len(), 3, "All 3 users subscribed to Topic1");
    }

    // All Users are also subscribed to Topic2
    {
        let txn = store.begin_read()?;
        let results = txn.query_by_subscription::<User, _>(&DefinitionSubscriptions::Topic2)?;
        assert_eq!(results.len(), 3, "All 3 users subscribed to Topic2");
    }

    // Users are NOT subscribed to Topic3 (Post subscription)
    {
        let txn = store.begin_read()?;
        let results = txn.query_by_subscription::<User, _>(&DefinitionSubscriptions::Topic3)?;
        assert_eq!(results.len(), 0, "Users not subscribed to Topic3");
    }
    
    // Update a user - subscription membership is maintained
    {
        let txn = store.begin_write()?;
        let mut user = txn.read::<User>(&UserID("user1".into()))?.unwrap();
        user.age = 99;
        txn.update(&user)?;
        txn.commit()?;
    }

        // Still subscribed after update
        {
            let txn = store.begin_read()?;
            let results = txn.query_by_subscription::<User, _>(&DefinitionSubscriptions::Topic1)?;
            assert_eq!(results.len(), 3, "Still all 3 users after update");
            
            // Verify the updated user is there with new hash
            // let updated = results.iter().find(|(u, _)| u.id.0 == "user1").unwrap();
            // assert_eq!(updated.0.age, 99);
        }
    
    // Delete a user - removed from subscription
    {
        let txn = store.begin_write()?;
        txn.delete::<User>(&UserID("user2".into()))?;
        txn.commit()?;
    }

    // Only 2 users remain in subscription
    {
        let txn = store.begin_read()?;
        let results = txn.query_by_subscription::<User, _>(&DefinitionSubscriptions::Topic1)?;
        assert_eq!(results.len(), 2, "2 users remain after delete");
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_relational_key_update() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<Definition>("relational_update")?;

    let user_id = UserID("user1".into());
    let cat1 = CategoryID("tech".into());
    let cat2 = CategoryID("sports".into());

    // Create user in tech category
    {
        let txn = store.begin_write()?;
        let user = User {
            id: user_id.clone(),
            first_name: "Alice".into(),
            last_name: "Smith".into(),
            age: 30,
            partner: RelationalLink::new_dehydrated(UserID("none".into())),
            category: RelationalLink::new_dehydrated(cat1.clone()),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile::default(),
        };
        txn.create(&user)?;
        txn.commit()?;
    }

    // Verify in tech category
    {
        let txn = store.begin_read()?;
        let users = txn.query_by_relational_key::<User>(
            &UserRelationalKeys::Category(UserCategory(cat1.clone()))
        )?;
        assert_eq!(users.len(), 1);
    }

    // Update to sports category
    {
        let txn = store.begin_write()?;
        let mut user = txn.read::<User>(&user_id)?.unwrap();
        user.category = RelationalLink::new_dehydrated(cat2.clone());
        txn.update(&user)?;
        txn.commit()?;
    }

    // Verify old category removed
    {
        let txn = store.begin_read()?;
        let users = txn.query_by_relational_key::<User>(
            &UserRelationalKeys::Category(UserCategory(cat1.clone()))
        )?;
        assert_eq!(users.len(), 0, "Should no longer be in tech category");
    }

    // Verify new category works
    {
        let txn = store.begin_read()?;
        let users = txn.query_by_relational_key::<User>(
            &UserRelationalKeys::Category(UserCategory(cat2.clone()))
        )?;
        assert_eq!(users.len(), 1, "Should be in sports category");
        assert_eq!(users[0].id, user_id);
    }

    common::cleanup_test_db(db_path);
    Ok(())
}
