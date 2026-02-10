//! Integration tests for merkle tree subscription synchronization.
//!
//! Tests the integration between:
//! - Subscription tables storing model hashes
//! - Merkle tree construction from subscription data
//! - P2P sync diff computation

#[path = "common/mod.rs"]
mod common;

use netabase_store::databases::redb::transaction::RedbModelCrud;
use netabase_store::relational::RelationalLink;
use netabase_store::schema::subscription_hash::{ModelHash, SubscriptionMerkleTree};
use netabase_store::traits::registry::models::model::NetabaseModel;
use example::boilerplate_lib::main_repository::definition::{
    AnotherLargeUserFile, LargeUserFile, User, UserID,
};
use example::boilerplate_lib::{CategoryID, Definition, DefinitionSubscriptions};

/// Test that we can build a merkle tree from subscription query results
#[test]
fn test_merkle_tree_from_subscription_data() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<Definition>("merkle_sub_basic")?;
    
    // Create multiple users (all subscribed to Topic1 and Topic2 via trait)
    {
        let txn = store.begin_write()?;
        for i in 1u8..=5 {
            let user = User {
                id: UserID(format!("user_{}", i)),
                first_name: format!("First{}", i),
                last_name: format!("Last{}", i),
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
    
    // Query subscription table to get model hashes directly
    {
        let txn = store.begin_read()?;
        let tables = txn.prepare_model::<User>()?;
        
        // query_by_subscription returns Vec<ModelHash> directly
        let hashes = User::query_by_subscription(&DefinitionSubscriptions::Topic1, &tables)?;
        assert_eq!(hashes.len(), 5, "Should have 5 users subscribed to Topic1");
        
        // Build merkle tree directly from the hashes
        let tree = SubscriptionMerkleTree::from_hashes(hashes.clone());
        
        assert_eq!(tree.len(), 5);
        assert!(tree.root().is_some(), "Tree should have a root");
        
        // Verify we can generate and verify proofs for each hash
        for hash in &hashes {
            let proof = tree.proof(hash).expect("Should generate proof");
            assert!(tree.verify_proof(hash, &proof), "Proof should verify");
        }
    }
    
    common::cleanup_test_db(db_path);
    Ok(())
}

/// Test merkle tree diff computation for sync
#[test]
fn test_merkle_diff_for_sync() -> Result<(), Box<dyn std::error::Error>> {
    // Simulate two nodes with different data using simple hash arrays
    // (avoiding the complexity of creating full User objects)
    
    // Node A has hashes for users 1, 2, 3
    let hashes_a: Vec<ModelHash> = vec![
        ModelHash::new([1u8; 32]),
        ModelHash::new([2u8; 32]),
        ModelHash::new([3u8; 32]),
    ];
    
    // Node B has hashes for users 2, 3, 4, 5
    let hashes_b: Vec<ModelHash> = vec![
        ModelHash::new([2u8; 32]),
        ModelHash::new([3u8; 32]),
        ModelHash::new([4u8; 32]),
        ModelHash::new([5u8; 32]),
    ];
    
    let tree_a = SubscriptionMerkleTree::from_hashes(hashes_a.clone());
    let tree_b = SubscriptionMerkleTree::from_hashes(hashes_b.clone());
    
    // Different roots mean different data
    assert_ne!(tree_a.root(), tree_b.root());
    
    // Compute diff from A's perspective
    let diff = tree_a.diff(&tree_b);
    
    // A has hash [1] that B doesn't have
    assert_eq!(diff.missing_in_other.len(), 1, "B is missing 1 record from A");
    
    // B has hashes [4] and [5] that A doesn't have  
    assert_eq!(diff.missing_in_self.len(), 2, "A is missing 2 records from B");
    
    assert!(diff.has_differences());
    assert_eq!(diff.diff_count(), 3);
    
    Ok(())
}

/// Test that identical subscription data produces identical merkle roots
#[test]
fn test_merkle_root_determinism() -> Result<(), Box<dyn std::error::Error>> {
    let (store1, db_path1) = common::create_test_db::<Definition>("merkle_determ_1")?;
    let (store2, db_path2) = common::create_test_db::<Definition>("merkle_determ_2")?;
    
    // Create identical users
    let users: Vec<User> = (1u8..=3)
        .map(|i| User {
            id: UserID(format!("user_{}", i)),
            first_name: format!("First{}", i),
            last_name: format!("Last{}", i),
            age: 20 + i,
            partner: RelationalLink::new_dehydrated(UserID("none".into())),
            category: RelationalLink::new_dehydrated(CategoryID("cat1".into())),
            bio: LargeUserFile::default(),
            another: AnotherLargeUserFile::default(),
        })
        .collect();
    
    // Store 1: insert in order 1, 2, 3
    {
        let txn = store1.begin_write()?;
        for user in &users {
            txn.create(user)?;
        }
        txn.commit()?;
    }
    
    // Store 2: insert in order 3, 1, 2
    {
        let txn = store2.begin_write()?;
        txn.create(&users[2])?;
        txn.create(&users[0])?;
        txn.create(&users[1])?;
        txn.commit()?;
    }
    
    // Build trees from both stores
    let get_tree = |store: &netabase_store::databases::redb::RedbStore<Definition>| -> SubscriptionMerkleTree {
        let txn = store.begin_read().unwrap();
        let tables = txn.prepare_model::<User>().unwrap();
        let hashes = User::query_by_subscription(&DefinitionSubscriptions::Topic1, &tables).unwrap();
        SubscriptionMerkleTree::from_hashes(hashes)
    };
    
    let tree1 = get_tree(&store1);
    let tree2 = get_tree(&store2);
    
    // Both should have same root (trees are sorted internally)
    assert_eq!(tree1.root(), tree2.root(), "Identical data should produce identical merkle roots");
    
    // Diff should show no differences
    let diff = tree1.diff(&tree2);
    assert!(!diff.has_differences(), "Identical stores should have no diff");
    
    common::cleanup_test_db(db_path1);
    common::cleanup_test_db(db_path2);
    Ok(())
}

/// Test proof serialization for P2P transfer
#[test]
fn test_merkle_proof_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let hashes: Vec<ModelHash> = (1..=5)
        .map(|i| ModelHash::new([i as u8; 32]))
        .collect();
    
    let tree = SubscriptionMerkleTree::from_hashes(hashes.clone());
    
    // Get proof for first hash
    let hash = &hashes[0];
    let proof = tree.proof(hash).expect("Should generate proof");
    
    // Serialize proof (rs_merkle proofs are serializable)
    let proof_bytes = proof.to_bytes();
    assert!(!proof_bytes.is_empty(), "Proof should serialize to bytes");
    
    // Verify we can recreate and verify
    // Note: MerkleProof::from_bytes requires knowing the number of leaves
    // In practice, this would be sent along with the proof
    
    Ok(())
}
