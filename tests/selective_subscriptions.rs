/// Test selective subscription insertion
/// 
/// This tests the ability to subscribe to only specific topics when creating a model,
/// rather than automatically subscribing to all model-level topics.

use netabase_store_examples::{Definition, User, UserID, CategoryID, DefinitionSubscriptions};
use netabase_store::relational::RelationalLink;
use netabase_store_examples::{LargeUserFile, AnotherLargeUserFile};

mod common;

#[test]
fn test_selective_subscription_create() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<Definition>("selective_subs")?;

    let user_id = UserID("user1".into());
    
    // Create user subscribing only to Topic1, not all topics
    {
        let txn = store.begin_write()?;
        let user = User {
            id: user_id.clone(),
            first_name: "Selective".into(),
            last_name: "User".into(),
            age: 25,
            partner: RelationalLink::new_dehydrated(UserID("none".into())),
            category: RelationalLink::new_dehydrated(CategoryID("cat1".into())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile::default(),
        };
        
        // Subscribe only to Topic1
        let topics = vec![DefinitionSubscriptions::Topic1];
        txn.create_with_subscriptions(&user, Some(topics))?;
        txn.commit()?;
    }

    // Query Topic1 - should find the user
    {
        let txn = store.begin_read()?;
        let users = txn.query_by_subscription::<User, _>(&DefinitionSubscriptions::Topic1)?;
        assert_eq!(users.len(), 1, "User should be in Topic1 subscription");
    }

    // Query Topic2 - should NOT find the user (not subscribed)
    {
        let txn = store.begin_read()?;
        let users = txn.query_by_subscription::<User, _>(&DefinitionSubscriptions::Topic2)?;
        assert_eq!(users.len(), 0, "User should not be in Topic2 subscription");
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_default_subscription_behavior() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<Definition>("default_subs")?;

    let user_id = UserID("user2".into());
    
    // Create user with default behavior (subscribe to all topics)
    {
        let txn = store.begin_write()?;
        let user = User {
            id: user_id.clone(),
            first_name: "Default".into(),
            last_name: "User".into(),
            age: 30,
            partner: RelationalLink::new_dehydrated(UserID("none".into())),
            category: RelationalLink::new_dehydrated(CategoryID("cat1".into())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile::default(),
        };
        
        // Use None to get default behavior (all topics)
        txn.create_with_subscriptions(&user, None)?;
        txn.commit()?;
    }

    // Should be in both Topic1 and Topic2 (User subscribes to both)
    {
        let txn = store.begin_read()?;
        let users_t1 = txn.query_by_subscription::<User, _>(&DefinitionSubscriptions::Topic1)?;
        let users_t2 = txn.query_by_subscription::<User, _>(&DefinitionSubscriptions::Topic2)?;
        
        assert_eq!(users_t1.len(), 1, "User should be in Topic1 with default subscription");
        assert_eq!(users_t2.len(), 1, "User should be in Topic2 with default subscription");
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_empty_subscription_list() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<Definition>("empty_subs")?;

    let user_id = UserID("user3".into());
    
    // Create user with no subscriptions
    {
        let txn = store.begin_write()?;
        let user = User {
            id: user_id.clone(),
            first_name: "NoSub".into(),
            last_name: "User".into(),
            age: 35,
            partner: RelationalLink::new_dehydrated(UserID("none".into())),
            category: RelationalLink::new_dehydrated(CategoryID("cat1".into())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile::default(),
        };
        
        // Subscribe to nothing
        txn.create_with_subscriptions(&user, Some(vec![]))?;
        txn.commit()?;
    }

    // Should not be in any subscription topic
    {
        let txn = store.begin_read()?;
        let users_t1 = txn.query_by_subscription::<User, _>(&DefinitionSubscriptions::Topic1)?;
        let users_t2 = txn.query_by_subscription::<User, _>(&DefinitionSubscriptions::Topic2)?;
        
        assert_eq!(users_t1.len(), 0, "User should not be in Topic1");
        assert_eq!(users_t2.len(), 0, "User should not be in Topic2");
    }

    // But should still be readable by primary key
    {
        let txn = store.begin_read()?;
        let user = txn.read::<User>(&user_id)?;
        assert!(user.is_some(), "User should be readable by primary key");
    }

    common::cleanup_test_db(db_path);
    Ok(())
}
