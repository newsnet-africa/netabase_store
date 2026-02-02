//! Integration tests for immutable model hashing and verification
//!
//! These tests verify that:
//! 1. Models stored in the database have their content hashes correctly indexed in subscription tables.
//! 2. `query_by_subscription` returns these hashes correctly.
//! 3. The `create_entry_with_hash` API works as expected.
//! 4. These hashes can be used to build Merkle trees for sync.

mod common;

use netabase_store::errors::NetabaseResult;
use netabase_store::relational::RelationalLink;
use netabase_store::subscription_hash::{ModelHash, SubscriptionMerkleTree};
use netabase_store::traits::registry::models::model::NetabaseModel;
use example::boilerplate_lib::definition::{
    AnotherLargeUserFile, DefinitionSubscriptions, LargeUserFile, User, UserID,
};
use example::boilerplate_lib::{CategoryID, Definition};

#[test]
fn test_subscription_query_returns_correct_hashes() -> NetabaseResult<()> {
    let (store, db_path) = common::create_test_db::<Definition>("sub_hashes")?;

    // Create a set of users
    let mut expected_hashes = Vec::new();
    let count = 5;

    {
        let txn = store.begin_write()?;
        for i in 0..count {
            let user = User {
                id: UserID(format!("user_{}", i)),
                first_name: format!("User {}", i),
                last_name: "Test".to_string(),
                age: 20 + i as u8,
                partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
                category: RelationalLink::new_dehydrated(CategoryID("cat1".to_string())),
                bio: LargeUserFile::default(),
                another: AnotherLargeUserFile::default(),
            };

            // Compute expected hash
            expected_hashes.push(user.compute_hash());

            // Create in DB
            txn.create(&user)?;
        }
        txn.commit()?;
    }

    // Verify hashes via query_by_subscription
    {
        let txn = store.begin_read()?;
        
        // Query Topic1 (default subscription)
        let stored_hashes = txn.query_by_subscription::<User, _>(&DefinitionSubscriptions::Topic1)?;
        
        assert_eq!(stored_hashes.len(), count, "Should find all users in subscription");

        // Sort both lists for comparison (order is not guaranteed by DB query)
        let mut sorted_expected = expected_hashes.clone();
        sorted_expected.sort();
        
        let mut sorted_stored = stored_hashes.clone();
        sorted_stored.sort();

        for (i, (expected, stored)) in sorted_expected.iter().zip(sorted_stored.iter()).enumerate() {
            assert_eq!(expected, stored, "Hash mismatch at index {}", i);
        }
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_create_entry_with_explicit_hash() -> NetabaseResult<()> {
    let (store, db_path) = common::create_test_db::<Definition>("explicit_hash")?;

    let user_id = UserID("explicit_hash_user".to_string());
    let user = User {
        id: user_id.clone(),
        first_name: "Explicit".to_string(),
        last_name: "Hash".to_string(),
        age: 30,
        partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: RelationalLink::new_dehydrated(CategoryID("cat1".to_string())),
        bio: LargeUserFile::default(),
        another: AnotherLargeUserFile::default(),
    };

    // Pre-calculate hash
    let computed_hash = user.compute_hash();

    // Create using the explicit hash API
    {
        let txn = store.begin_write()?;
        // This avoids re-computing the hash inside create_entry
        txn.create_with_hash(&user, &computed_hash)?;
        txn.commit()?;
    }

    // Verify
    {
        let txn = store.begin_read()?;
        let stored_hashes = txn.query_by_subscription::<User, _>(&DefinitionSubscriptions::Topic1)?;
        
        assert_eq!(stored_hashes.len(), 1);
        assert_eq!(stored_hashes[0], computed_hash, "Stored hash should match explicit hash");
        
        // Verify model data is correct
        let read_user = txn.read::<User>(&user_id)?.expect("User should exist");
        assert_eq!(read_user.first_name, "Explicit");
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

#[test]
fn test_merkle_sync_simulation() -> NetabaseResult<()> {
    let (store, db_path) = common::create_test_db::<Definition>("merkle_sync_sim")?;

    // 1. Populate initial state (Node A)
    let users_a = vec![
        User {
            id: UserID("user_1".to_string()),
            first_name: "Alice".to_string(),
            last_name: "A".to_string(),
            age: 20,
            partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
            category: RelationalLink::new_dehydrated(CategoryID("cat1".to_string())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile::default(),
        },
        User {
            id: UserID("user_2".to_string()),
            first_name: "Bob".to_string(),
            last_name: "B".to_string(),
            age: 21,
            partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
            category: RelationalLink::new_dehydrated(CategoryID("cat1".to_string())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile::default(),
        },
    ];

    {
        let txn = store.begin_write()?;
        for user in &users_a {
            txn.create(user)?;
        }
        txn.commit()?;
    }

    // 2. Compute Merkle Root for Node A from DB
    let root_a = {
        let txn = store.begin_read()?;
        let hashes = txn.query_by_subscription::<User, _>(&DefinitionSubscriptions::Topic1)?;
        let tree = SubscriptionMerkleTree::from_hashes(hashes);
        tree.root_hex()
    };
    
    // 3. Update state (Node B simulation: Node A + new user - old user)
    // We'll simulate this by modifying the DB in place
    {
        let txn = store.begin_write()?;
        // Remove user_1
        txn.delete::<User>(&UserID("user_1".to_string()))?;
        
        // Add user_3
        let user_3 = User {
            id: UserID("user_3".to_string()),
            first_name: "Charlie".to_string(),
            last_name: "C".to_string(),
            age: 22,
            partner: RelationalLink::new_dehydrated(UserID("none".to_string())),
            category: RelationalLink::new_dehydrated(CategoryID("cat1".to_string())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile::default(),
        };
        txn.create(&user_3)?;
        txn.commit()?;
    }

    // 4. Compute Merkle Root for Node B (current DB state)
    let root_b = {
        let txn = store.begin_read()?;
        let hashes = txn.query_by_subscription::<User, _>(&DefinitionSubscriptions::Topic1)?;
        let tree = SubscriptionMerkleTree::from_hashes(hashes);
        tree.root_hex()
    };

    // Roots should be different
    assert_ne!(root_a, root_b, "Merkle roots should differ after state change");

    // 5. Verify the difference
    // Reconstruct Tree A (in memory for simulation)
    let tree_a = {
        let hashes: Vec<ModelHash> = users_a.iter().map(|u| u.compute_hash()).collect();
        SubscriptionMerkleTree::from_hashes(hashes)
    };

    // Reconstruct Tree B (from DB)
    let tree_b = {
        let txn = store.begin_read()?;
        let hashes = txn.query_by_subscription::<User, _>(&DefinitionSubscriptions::Topic1)?;
        SubscriptionMerkleTree::from_hashes(hashes)
    };

    let diff = tree_a.diff(&tree_b);

    // Tree A has user_1 (missing in B)
    assert_eq!(diff.missing_in_other.len(), 1); 
    
    // Tree B has user_3 (missing in A)
    assert_eq!(diff.missing_in_self.len(), 1);

    common::cleanup_test_db(db_path);
    Ok(())
}
