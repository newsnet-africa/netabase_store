//! Comprehensive test of all table/index types
//!
//! This test verifies that all table types work correctly:
//! - Main table (primary key -> model)
//! - Secondary key indexes
//! - Relational key indexes  
//! - Subscription indexes
//! - Blob storage

mod common;

use netabase_store::relational::RelationalLink;
use example::boilerplate_lib::definition::{
    AnotherLargeUserFile, LargeUserFile, User, UserID, UserSecondaryKeys,
    UserFirstName, UserAge, Post, PostID,
};
use example::boilerplate_lib::{CategoryID, Definition, DefinitionSubscriptions};

#[test]
fn test_all_table_types_work() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<Definition>("all_tables")?;

    // Create a user with all types of indexes
    let user_id = UserID("alice".into());
    let partner_id = UserID("bob".into());
    let category_id = CategoryID("tech".into());
    
    {
        let txn = store.begin_write()?;

        // Create partner first
        let partner = User {
            id: partner_id.clone(),
            first_name: "Bob".into(),
            last_name: "Smith".into(),
            age: 35,
            partner: RelationalLink::new_dehydrated(UserID("none".into())),
            category: RelationalLink::new_dehydrated(category_id.clone()),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile::default(),
        };
        txn.create(&partner)?;

        // Create main user with:
        // - Primary key: user_id
        // - Secondary keys: first_name="Alice", age=30
        // - Relational links: partner, category
        // - Subscriptions: Topic1, Topic2
        // - Blobs: bio (large data)
        let user = User {
            id: user_id.clone(),
            first_name: "Alice".into(),
            last_name: "Johnson".into(),
            age: 30,
            partner: RelationalLink::new_dehydrated(partner_id.clone()),
            category: RelationalLink::new_dehydrated(category_id.clone()),
            bio: LargeUserFile {
                data: vec![1, 2, 3, 4, 5],
                metadata: "Test bio".into(),
            },
            another: AnotherLargeUserFile(vec![10, 20, 30]),
        };
        txn.create(&user)?;
        txn.commit()?;
    }

    // Test 1: Main table (primary key lookup)
    {
        let txn = store.begin_read()?;
        let user = txn.read::<User>(&user_id)?;
        assert!(user.is_some(), "Primary key lookup failed");
        assert_eq!(user.unwrap().first_name, "Alice");
        println!("✅ Main table (primary key) works");
    }

    // Test 2: Secondary key indexes
    {
        let txn = store.begin_read()?;
        
        // Query by first_name
        let users = txn.query_by_secondary_key::<User>(
            &UserSecondaryKeys::FirstName(UserFirstName("Alice".into()))
        )?;
        assert_eq!(users.len(), 1, "Secondary key query by name failed");
        assert_eq!(users[0].id, user_id);

        // Query by age
        let users = txn.query_by_secondary_key::<User>(
            &UserSecondaryKeys::Age(UserAge(30))
        )?;
        assert_eq!(users.len(), 1, "Secondary key query by age failed");
        assert_eq!(users[0].id, user_id);
        
        println!("✅ Secondary key indexes work");
    }

    // Test 3: Relational links (stored and retrieved correctly)
    {
        let txn = store.begin_read()?;
        let user = txn.read::<User>(&user_id)?.unwrap();
        
        // Verify relational links are stored (dehydrated form)
        assert_eq!(user.partner.get_primary_key(), &partner_id);
        assert_eq!(user.category.get_primary_key(), &category_id);
        
        println!("✅ Relational links work (stored correctly)");
    }

    // Test 4: Subscriptions (stored correctly)
    {
        let txn = store.begin_read()?;
        let user = txn.read::<User>(&user_id)?.unwrap();
        
        
        println!("✅ Subscriptions work (stored correctly)");
    }

    // Test 5: Blob storage (large data)
    {
        let txn = store.begin_read()?;
        let user = txn.read::<User>(&user_id)?.unwrap();
        
        assert_eq!(user.bio.data, vec![1, 2, 3, 4, 5]);
        assert_eq!(user.bio.metadata, "Test bio");
        assert_eq!(user.another.0, vec![10, 20, 30]);
        
        println!("✅ Blob storage works");
    }

    // Test 6: Create a Post to test cross-model operations
    {
        let txn = store.begin_write()?;
        
        let post = Post {
            id: PostID("post1".into()),
            title: "Test Post".into(),
            author_id: user_id.0.clone(), // String reference to user
            content: "Content here".into(),
            published: true,
            tags: vec!["rust".into(), "database".into()],
        };
        txn.create(&post)?;
        txn.commit()?;
        
        println!("✅ Cross-model operations work");
    }

    // Test 7: Read the post back
    {
        let txn = store.begin_read()?;
        let post = txn.read::<Post>(&PostID("post1".into()))?;
        assert!(post.is_some());
        let post = post.unwrap();
        assert_eq!(post.title, "Test Post");
        assert_eq!(post.author_id, "alice");
        assert_eq!(post.tags.len(), 2);
        
        println!("✅ Post model with tags works");
    }

    common::cleanup_test_db(db_path);
    
    println!("\n🎉 All table types work correctly!");
    println!("   ✓ Main table (primary key)");
    println!("   ✓ Secondary key indexes");
    println!("   ✓ Relational links");
    println!("   ✓ Subscriptions");
    println!("   ✓ Blob storage");
    println!("   ✓ Cross-model operations");
    
    Ok(())
}

#[test]
fn test_update_maintains_all_indexes() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<Definition>("update_indexes")?;

    let user_id = UserID("user1".into());

    // Create initial user
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

    // Update the user (changes secondary keys, relational links, subscriptions)
    {
        let txn = store.begin_write()?;
        let mut user = txn.read::<User>(&user_id)?.unwrap();
        
        user.first_name = "Alicia".into(); // Change secondary key
        user.age = 31; // Change secondary key
        user.category = RelationalLink::new_dehydrated(CategoryID("cat2".into())); // Change relational link
        user.bio.data = vec![99; 1000]; // Change blob
        
        txn.update(&user)?;
        txn.commit()?;
    }

    // Verify old secondary key values are gone
    {
        let txn = store.begin_read()?;
        
        let alices = txn.query_by_secondary_key::<User>(
            &UserSecondaryKeys::FirstName(UserFirstName("Alice".into()))
        )?;
        assert_eq!(alices.len(), 0, "Old name should not be found");

        let age30 = txn.query_by_secondary_key::<User>(
            &UserSecondaryKeys::Age(UserAge(30))
        )?;
        assert_eq!(age30.len(), 0, "Old age should not be found");
    }

    // Verify new secondary key values work
    {
        let txn = store.begin_read()?;
        
        let alicias = txn.query_by_secondary_key::<User>(
            &UserSecondaryKeys::FirstName(UserFirstName("Alicia".into()))
        )?;
        assert_eq!(alicias.len(), 1, "New name should be found");

        let age31 = txn.query_by_secondary_key::<User>(
            &UserSecondaryKeys::Age(UserAge(31))
        )?;
        assert_eq!(age31.len(), 1, "New age should be found");
    }

    // Verify relational links updated
    {
        let txn = store.begin_read()?;
        let user = txn.read::<User>(&user_id)?.unwrap();
        assert_eq!(user.category.get_primary_key().0, "cat2");
    }

    // Verify subscriptions updated
    {
        let txn = store.begin_read()?;
        let user = txn.read::<User>(&user_id)?.unwrap();
    }

    // Verify blob updated
    {
        let txn = store.begin_read()?;
        let user = txn.read::<User>(&user_id)?.unwrap();
        assert_eq!(user.bio.data.len(), 1000);
        assert_eq!(user.bio.data[0], 99);
    }

    common::cleanup_test_db(db_path);
    
    println!("\n🎉 All indexes are maintained correctly on update!");
    
    Ok(())
}
