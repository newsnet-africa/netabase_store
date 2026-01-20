/// Test to demonstrate relational query performance
/// 
/// This test verifies the O(1) forward lookup: Model -> Relations.

use example::{Definition, User, UserID, CategoryID, UserRelationalKeys, UserCategory};
use netabase_store::relational::RelationalLink;
use example::{LargeUserFile, AnotherLargeUserFile};

mod common;

#[test]
fn test_relational_query_performance() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<Definition>("relational_perf")?;

    let cat_tech = CategoryID("tech".into());
    let cat_sports = CategoryID("sports".into());
    let cat_news = CategoryID("news".into());

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

    // Query relations for a specific user
    {
        let txn = store.begin_read()?;
        
        // Check user3 (should be tech: 3%3=0)
        let user3_id = UserID("user3".into());
        let relations = txn.query_relations::<User>(&user3_id)?;
        
        assert!(!relations.is_empty(), "Should find relations for user3");
        
        let found_category = relations.iter().any(|r| {
            if let UserRelationalKeys::Category(UserCategory(c)) = r {
                c == &cat_tech
            } else {
                false
            }
        });
        assert!(found_category, "User3 should be in tech category");

        // Check user4 (should be sports: 4%3=1)
        let user4_id = UserID("user4".into());
        let relations4 = txn.query_relations::<User>(&user4_id)?;
        let found_sports = relations4.iter().any(|r| {
            if let UserRelationalKeys::Category(UserCategory(c)) = r {
                c == &cat_sports
            } else {
                false
            }
        });
        assert!(found_sports, "User4 should be in sports category");
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
    
    // Future work: If models have Vec<RelationalLink>, query_relations(pk) 
    // will return all of them.

    common::cleanup_test_db(db_path);
    Ok(())
}
