//! Comprehensive test suite for all table types and query features
//!
//! Tests primary keys, secondary keys, relational links, and blob storage.
//! Note: Subscription queries currently blocked by macro issue - see SUBSCRIPTION_HASH_IMPLEMENTATION.md

mod common;

use netabase_store::relational::RelationalLink;
use netabase_store::subscription_hash::{ModelHash, SubscriptionMerkleTree};
use example::boilerplate_lib::definition::{
    AnotherLargeUserFile, LargeUserFile, User, UserAge, UserCategory, UserFirstName, UserID,
    UserLastName, UserRelationalKeys, UserSecondaryKeys,
};
use example::boilerplate_lib::{CategoryID, Definition};

#[test]
fn test_primary_key_crud() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<Definition>("primary_key")?;

    let user_id = UserID("user1".into());

    // Create
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

    // Read
    {
        let txn = store.begin_read()?;
        let user = txn.read::<User>(&user_id)?.expect("User should exist");
        assert_eq!(user.first_name, "Alice");
        assert_eq!(user.age, 30);
    }

    // Update
    {
        let txn = store.begin_write()?;
        let mut user = txn.read::<User>(&user_id)?.unwrap();
        user.age = 31;
        txn.update(&user)?;
        txn.commit()?;
    }

    // Verify update
    {
        let txn = store.begin_read()?;
        let user = txn.read::<User>(&user_id)?.unwrap();
        assert_eq!(user.age, 31);
    }

    // Delete
    {
        let txn = store.begin_write()?;
        txn.delete::<User>(&user_id)?;
        txn.commit()?;
    }

    // Verify deletion
    {
        let txn = store.begin_read()?;
        let user = txn.read::<User>(&user_id)?;
        assert!(user.is_none(), "User should be deleted");
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_secondary_key_queries() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<Definition>("secondary_key")?;

    // Create multiple users
    {
        let txn = store.begin_write()?;

        let user1 = User {
            id: UserID("user1".into()),
            first_name: "Alice".into(),
            last_name: "Smith".into(),
            age: 30,
            partner: RelationalLink::new_dehydrated(UserID("none".into())),
            category: RelationalLink::new_dehydrated(CategoryID("cat1".into())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile::default(),
        };

        let user2 = User {
            id: UserID("user2".into()),
            first_name: "Alice".into(), // Same name
            last_name: "Johnson".into(),
            age: 25,
            partner: RelationalLink::new_dehydrated(UserID("none".into())),
            category: RelationalLink::new_dehydrated(CategoryID("cat1".into())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile::default(),
        };

        let user3 = User {
            id: UserID("user3".into()),
            first_name: "Bob".into(),
            last_name: "Smith".into(), // Same last name as user1
            age: 30,                   // Same age as user1
            partner: RelationalLink::new_dehydrated(UserID("none".into())),
            category: RelationalLink::new_dehydrated(CategoryID("cat1".into())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile::default(),
        };

        txn.create(&user1)?;
        txn.create(&user2)?;
        txn.create(&user3)?;
        txn.commit()?;
    }

    // Query by first_name
    {
        let txn = store.begin_read()?;
        let users = txn.query_by_secondary_key::<User>(&UserSecondaryKeys::FirstName(
            UserFirstName("Alice".into()),
        ))?;

        assert_eq!(users.len(), 2, "Should find 2 users named Alice");
        assert!(users.iter().all(|u| u.first_name == "Alice"));
    }

    // Query by last_name
    {
        let txn = store.begin_read()?;
        let users = txn.query_by_secondary_key::<User>(&UserSecondaryKeys::LastName(
            UserLastName("Smith".into()),
        ))?;

        assert_eq!(users.len(), 2, "Should find 2 users with last name Smith");
    }

    // Query by age
    {
        let txn = store.begin_read()?;
        let users = txn.query_by_secondary_key::<User>(&UserSecondaryKeys::Age(UserAge(30)))?;

        assert_eq!(users.len(), 2, "Should find 2 users aged 30");
    }

    // Query non-existent
    {
        let txn = store.begin_read()?;
        let users = txn.query_by_secondary_key::<User>(&UserSecondaryKeys::FirstName(
            UserFirstName("Charlie".into()),
        ))?;

        assert_eq!(users.len(), 0, "Should find no users named Charlie");
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_relational_queries_forward() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<Definition>("relational_key")?;

    let cat1 = CategoryID("tech".into());
    let cat2 = CategoryID("sports".into());

    // Create users with different categories
    {
        let txn = store.begin_write()?;

        for i in 1..=3 {
            let user = User {
                id: UserID(format!("user{}", i)),
                first_name: format!("User{}", i),
                last_name: "Test".into(),
                age: 20 + i,
                partner: RelationalLink::new_dehydrated(UserID("none".into())),
                category: RelationalLink::new_dehydrated(if i < 3 {
                    cat1.clone()
                } else {
                    cat2.clone()
                }),
                bio: LargeUserFile::default(),
                another: AnotherLargeUserFile::default(),
            };
            txn.create(&user)?;
        }
        txn.commit()?;
    }

    // Query relations
    {
        let txn = store.begin_read()?;

        // User 1 -> Tech
        let rels1 = txn.query_relations::<User>(&UserID("user1".into()))?;
        assert!(
            rels1
                .iter()
                .any(|r| r == &UserRelationalKeys::Category(UserCategory(cat1.clone())))
        );

        // User 2 -> Tech
        let rels2 = txn.query_relations::<User>(&UserID("user2".into()))?;
        assert!(
            rels2
                .iter()
                .any(|r| r == &UserRelationalKeys::Category(UserCategory(cat1.clone())))
        );

        // User 3 -> Sports
        let rels3 = txn.query_relations::<User>(&UserID("user3".into()))?;
        assert!(
            rels3
                .iter()
                .any(|r| r == &UserRelationalKeys::Category(UserCategory(cat2.clone())))
        );
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_blob_storage() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<Definition>("blob_storage")?;

    let user_id = UserID("blob_user".into());
    let large_data = vec![42u8; 100_000]; // 100KB

    // Create with blob
    {
        let txn = store.begin_write()?;
        let user = User {
            id: user_id.clone(),
            first_name: "BlobUser".into(),
            last_name: "Test".into(),
            age: 30,
            partner: RelationalLink::new_dehydrated(UserID("none".into())),
            category: RelationalLink::new_dehydrated(CategoryID("cat1".into())),
            bio: LargeUserFile {
                data: large_data.clone(),
                metadata: "Large bio data".into(),
            },
            another: AnotherLargeUserFile(vec![1, 2, 3]),
        };
        txn.create(&user)?;
        txn.commit()?;
    }

    // Read and verify blob
    {
        let txn = store.begin_read()?;
        let user = txn.read::<User>(&user_id)?.unwrap();

        assert_eq!(user.bio.data.len(), 100_000);
        assert_eq!(user.bio.data[0], 42);
        assert_eq!(user.bio.metadata, "Large bio data");
        assert_eq!(user.another.0, vec![1, 2, 3]);
    }

    // Update blob
    {
        let txn = store.begin_write()?;
        let mut user = txn.read::<User>(&user_id)?.unwrap();
        user.bio.data = vec![99u8; 50_000];
        user.bio.metadata = "Updated bio".into();
        txn.update(&user)?;
        txn.commit()?;
    }

    // Verify update
    {
        let txn = store.begin_read()?;
        let user = txn.read::<User>(&user_id)?.unwrap();

        assert_eq!(user.bio.data.len(), 50_000);
        assert_eq!(user.bio.data[0], 99);
        assert_eq!(user.bio.metadata, "Updated bio");
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_model_hash_computation() -> Result<(), Box<dyn std::error::Error>> {
    let (_store, db_path) = common::create_test_db::<Definition>("model_hash")?;

    let user1 = User {
        id: UserID("user1".into()),
        first_name: "Alice".into(),
        last_name: "Smith".into(),
        age: 30,
        partner: RelationalLink::new_dehydrated(UserID("none".into())),
        category: RelationalLink::new_dehydrated(CategoryID("cat1".into())),
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile::default(),
    };

    let user2 = user1.clone();
    let mut user3 = user1.clone();
    user3.age = 31;

    // Test hash computation
    use netabase_store::traits::registery::models::model::NetabaseModel;

    let hash1 = user1.compute_hash();
    let hash2 = user2.compute_hash();
    let hash3 = user3.compute_hash();

    // Same data should have same hash
    assert_eq!(hash1, hash2, "Identical models should have same hash");

    // Different data should have different hash
    assert_ne!(hash1, hash3, "Different models should have different hash");

    // Hash should be deterministic
    assert_eq!(user1.compute_hash(), user1.compute_hash());

    // Test hex conversion
    let hex = hash1.to_hex();
    assert_eq!(hex.len(), 64); // 32 bytes = 64 hex chars

    let parsed = ModelHash::from_hex(&hex)?;
    assert_eq!(hash1, parsed);

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_merkle_tree_construction() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<Definition>("merkle_tree")?;

    // Create multiple users
    let mut hashes = Vec::new();
    {
        let txn = store.begin_write()?;

        for i in 1..=5 {
            let user = User {
                id: UserID(format!("user{}", i)),
                first_name: format!("User{}", i),
                last_name: "Test".into(),
                age: 20 + i as u8,
                partner: RelationalLink::new_dehydrated(UserID("none".into())),
                category: RelationalLink::new_dehydrated(CategoryID("cat1".into())),
                bio: LargeUserFile::default(),
                another: AnotherLargeUserFile::default(),
            };

            use netabase_store::traits::registery::models::model::NetabaseModel;
            hashes.push(user.compute_hash());
            txn.create(&user)?;
        }
        txn.commit()?;
    }

    // Build merkle tree
    let tree = SubscriptionMerkleTree::from_hashes(hashes.clone());

    assert_eq!(tree.len(), 5);
    assert!(tree.root().is_some(), "Tree should have a root");

    // Test proof generation and verification
    let hash = hashes[0];
    let proof = tree.proof(&hash).expect("Should generate proof");
    assert!(tree.verify_proof(&hash, &proof), "Proof should verify");
    println!("✓ Merkle proof verified successfully");

    // Test tree diff
    let mut hashes2 = hashes.clone();
    hashes2.pop(); // Remove one
    hashes2.push(ModelHash::new([99u8; 32])); // Add different one

    let tree2 = SubscriptionMerkleTree::from_hashes(hashes2);
    let diff = tree.diff(&tree2);

    assert!(diff.has_differences());
    assert_eq!(diff.missing_in_other.len(), 1);
    assert_eq!(diff.missing_in_self.len(), 1);

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_index_maintenance_on_update() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<Definition>("index_maintenance")?;

    let user_id = UserID("user1".into());

    // Create
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

    // Verify initial indexes
    {
        let txn = store.begin_read()?;

        let by_name = txn.query_by_secondary_key::<User>(&UserSecondaryKeys::FirstName(
            UserFirstName("Alice".into()),
        ))?;
        assert_eq!(by_name.len(), 1);

        let rels = txn.query_relations::<User>(&user_id)?;
        // User has 2 relational fields: 'partner' and 'category'. Even if 'partner' is "none", it's still a link.
        assert_eq!(rels.len(), 2, "Should have 2 relations (category and partner)");
        assert!(
            rels.iter().any(
                |r| r == &UserRelationalKeys::Category(UserCategory(CategoryID("cat1".into())))
            )
        );

        // Test query_relations_by_type (filtering)
        use strum::IntoDiscriminant;
        
        // Construct a dummy key to get the discriminant
        let dummy_cat_key = UserRelationalKeys::Category(UserCategory(CategoryID("dummy".into())));
        let cat_discriminant = dummy_cat_key.discriminant();
        
        let cat_rels = txn.query_relations_by_type::<User>(
            &user_id, 
            cat_discriminant
        )?;
        assert_eq!(cat_rels.len(), 1, "Should find exactly 1 Category relation");
        assert_eq!(cat_rels[0], UserRelationalKeys::Category(UserCategory(CategoryID("cat1".into()))));
    }

    // Update - change secondary and relational keys
    {
        let txn = store.begin_write()?;
        let mut user = txn.read::<User>(&user_id)?.unwrap();
        user.first_name = "Alicia".into();
        user.category = RelationalLink::new_dehydrated(CategoryID("cat2".into()));
        txn.update(&user)?;
        txn.commit()?;
    }

    // Verify indexes updated
    {
        let txn = store.begin_read()?;

        let by_old_name = txn.query_by_secondary_key::<User>(&UserSecondaryKeys::FirstName(
            UserFirstName("Alice".into()),
        ))?;
        assert_eq!(by_old_name.len(), 0, "Old name index should be removed");

        // Verify relations updated
        let rels = txn.query_relations::<User>(&user_id)?;
        assert_eq!(rels.len(), 2, "Should have 2 relations");
        assert!(
            rels.iter().any(
                |r| r == &UserRelationalKeys::Category(UserCategory(CategoryID("cat2".into())))
            )
        );
        
        // Verify via filtered query
        use strum::IntoDiscriminant;
        let dummy_cat_key = UserRelationalKeys::Category(UserCategory(CategoryID("dummy".into())));
        let cat_discriminant = dummy_cat_key.discriminant();

        let cat_rels = txn.query_relations_by_type::<User>(
            &user_id, 
            cat_discriminant
        )?;
        assert_eq!(cat_rels.len(), 1);
        assert_eq!(cat_rels[0], UserRelationalKeys::Category(UserCategory(CategoryID("cat2".into()))));
    }

    // Verify new indexes added (Secondary)
    {
        let txn = store.begin_read()?;

        let by_new_name = txn.query_by_secondary_key::<User>(&UserSecondaryKeys::FirstName(
            UserFirstName("Alicia".into()),
        ))?;
        assert_eq!(by_new_name.len(), 1, "New name index should exist");
    }

    common::cleanup_test_db(db_path);
    Ok(())
}
