/// Test to demonstrate relational query performance
/// 
/// This test shows the difference between O(n) table scan and O(log n) index lookup
/// for relational queries.

use netabase_store_examples::{Definition, User, UserID, CategoryID, UserRelationalKeys, UserCategory};
use netabase_store::relational::RelationalLink;
use netabase_store_examples::{LargeUserFile, AnotherLargeUserFile};

mod common;

#[test]
fn test_relational_query_performance() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<Definition>("relational_perf")?;

    let cat_tech = CategoryID("tech".into());
    let cat_sports = CategoryID("sports".into());
    let cat_news = CategoryID("news".into());

    // Create a larger dataset to test performance
    let num_users = 100;
    {
        let txn = store.begin_write()?;

        for i in 1..=num_users {
            let category = match i % 3 {
                0 => cat_tech.clone(),
                1 => cat_sports.clone(),
                _ => cat_news.clone(),
            };

            let user = User {
                id: UserID(format!("user{}", i)),
                first_name: format!("User{}", i),
                last_name: "Test".into(),
                age: 20 + (i % 50),
                partner: RelationalLink::new_dehydrated(UserID("none".into())),
                category: RelationalLink::new_dehydrated(category),
                bio: LargeUserFile::default(),
                another: AnotherLargeUserFile::default(),
            };
            txn.create(&user)?;
        }
        txn.commit()?;
    }

    // Query by category - this should use an index, not a table scan
    {
        let txn = store.begin_read()?;
        
        // Query for tech category
        let tech_users = txn.query_by_relational_key::<User>(
            &UserRelationalKeys::Category(UserCategory(cat_tech.clone()))
        )?;
        
        // Should find approximately 1/3 of users
        assert!(tech_users.len() >= 30 && tech_users.len() <= 35, 
            "Should find roughly 1/3 of users in tech category, found {}", tech_users.len());
    }

    // Query for sports category
    {
        let txn = store.begin_read()?;
        let sports_users = txn.query_by_relational_key::<User>(
            &UserRelationalKeys::Category(UserCategory(cat_sports.clone()))
        )?;
        
        assert!(sports_users.len() >= 30 && sports_users.len() <= 35,
            "Should find roughly 1/3 of users in sports category, found {}", sports_users.len());
    }

    // Query for news category  
    {
        let txn = store.begin_read()?;
        let news_users = txn.query_by_relational_key::<User>(
            &UserRelationalKeys::Category(UserCategory(cat_news.clone()))
        )?;
        
        assert!(news_users.len() >= 30 && news_users.len() <= 40,
            "Should find roughly 1/3 of users in news category, found {}", news_users.len());
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_relational_query_with_vec() -> Result<(), Box<dyn std::error::Error>> {
    // This test demonstrates querying when models have Vec<RelationalLink<T>>
    // Currently not implemented in the example schema, but this shows the API we want
    
    let (_store, db_path) = common::create_test_db::<Definition>("relational_vec")?;

    let _cat1 = CategoryID("cat1".into());
    
    // In the future, if User had `tags: Vec<RelationalLink<Category>>`,
    // we should be able to query: "find all users with tag cat1"
    // This would require the inverse index: RelationalKey -> Vec<PrimaryKey>

    common::cleanup_test_db(db_path);
    Ok(())
}
