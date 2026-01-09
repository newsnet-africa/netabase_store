// Example: Merkle Tree and P2P Synchronization
//
// This example demonstrates using Merkle trees with subscription queries
// for efficient peer-to-peer synchronization.

use netabase_store::databases::redb::RedbStore;
use netabase_store::relational::RelationalLink;
use netabase_store::subscription_hash::SubscriptionMerkleTree;
use netabase_store::traits::database::store::NBStore;
use netabase_store_examples::boilerplate_lib::definition::{AnotherLargeUserFile, LargeUserFile};
use netabase_store_examples::boilerplate_lib::{
    CategoryID, Definition, DefinitionSubscriptions, ImmutablePost, ImmutablePostEnvelope, User,
    UserID,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Merkle Tree & P2P Sync Example ===\n");

    // Create two in-memory databases simulating two peers
    let (peer1_store, _temp1) = RedbStore::<Definition>::new_temporary()?;
    let (peer2_store, _temp2) = RedbStore::<Definition>::new_temporary()?;

    println!("1. Populating Peer 1 database\n");

    // Add users to Peer 1
    {
        let txn = peer1_store.begin_write()?;

        let users = vec![
            ("alice", "Alice", "Smith", 30),
            ("bob", "Bob", "Jones", 25),
            ("charlie", "Charlie", "Brown", 35),
        ];

        for (id, first, last, age) in users {
            let user = User {
                id: UserID(id.into()),
                first_name: first.into(),
                last_name: last.into(),
                age,
                partner: RelationalLink::new_dehydrated(UserID("none".into())),
                category: RelationalLink::new_dehydrated(CategoryID("tech".into())),
                bio: LargeUserFile::default(),
                another: AnotherLargeUserFile::default(),
            };
            txn.create::<User>(&user)?;
        }

        txn.commit()?;
        println!("  ✓ Added 3 users to Peer 1");
    }

    println!("\n2. Populating Peer 2 database\n");

    // Add some users to Peer 2 (partially overlapping with Peer 1)
    {
        let txn = peer2_store.begin_write()?;

        let users = vec![
            ("alice", "Alice", "Smith", 30), // Same as Peer 1
            ("bob", "Bob", "Jones", 25),     // Same as Peer 1
            ("dave", "Dave", "Wilson", 40),  // Only in Peer 2
        ];

        for (id, first, last, age) in users {
            let user = User {
                id: UserID(id.into()),
                first_name: first.into(),
                last_name: last.into(),
                age,
                partner: RelationalLink::new_dehydrated(UserID("none".into())),
                category: RelationalLink::new_dehydrated(CategoryID("tech".into())),
                bio: LargeUserFile::default(),
                another: AnotherLargeUserFile::default(),
            };
            txn.create::<User>(&user)?;
        }

        txn.commit()?;
        println!("  ✓ Added 3 users to Peer 2");
    }

    println!("\n3. Building Merkle trees for each peer\n");

    // Build Merkle tree for Peer 1
    let peer1_tree = {
        let txn = peer1_store.begin_read()?;
        let results = txn.query_by_subscription::<User, _>(&DefinitionSubscriptions::Topic1)?;
        println!("  Peer 1: {} users in Topic1", results.len());

        let hashes = results.clone();
        SubscriptionMerkleTree::from_hashes(hashes)
    };

    // Build Merkle tree for Peer 2
    let peer2_tree = {
        let txn = peer2_store.begin_read()?;
        let results = txn.query_by_subscription::<User, _>(&DefinitionSubscriptions::Topic1)?;
        println!("  Peer 2: {} users in Topic1", results.len());

        let hashes = results.clone();
        SubscriptionMerkleTree::from_hashes(hashes)
    };

    println!("\n4. Comparing Merkle roots (quick sync check)\n");

    let peer1_root = peer1_tree.root().unwrap();
    let peer2_root = peer2_tree.root().unwrap();

    println!("  Peer 1 root: {}", hex::encode(peer1_root));
    println!("  Peer 2 root: {}", hex::encode(peer2_root));

    if peer1_root == peer2_root {
        println!("\n  ✓ Roots match - peers are in sync!");
    } else {
        println!("\n  ⚠ Roots differ - sync needed!");
    }

    println!("\n5. Finding differences between peers\n");

    let diff = peer1_tree.diff(&peer2_tree);

    if diff.has_differences() {
        println!("  Synchronization required:");
        println!(
            "    - Missing in Peer 2: {} items",
            diff.missing_in_other.len()
        );
        println!(
            "    - Missing in Peer 1: {} items",
            diff.missing_in_self.len()
        );

        // Show which items are missing
        if !diff.missing_in_other.is_empty() {
            println!("\n  Items in Peer 1 but not in Peer 2:");
            for hash in &diff.missing_in_other {
                println!("    - {}...", &hash.to_hex()[..16]);
            }
        }

        if !diff.missing_in_self.is_empty() {
            println!("\n  Items in Peer 2 but not in Peer 1:");
            for hash in &diff.missing_in_self {
                println!("    - {}...", &hash.to_hex()[..16]);
            }
        }
    } else {
        println!("  ✓ No differences - trees are identical!");
    }

    println!("\n6. Merkle proof verification\n");

    // Get a hash from Peer 1 and generate proof
    let peer1_hashes = peer1_tree.hashes();
    if let Some(hash) = peer1_hashes.first() {
        println!("  Testing proof for hash: {}...", &hash.to_hex()[..16]);

        // Generate proof
        let proof = peer1_tree.proof(hash).expect("Hash should be in tree");
        println!(
            "  ✓ Proof generated (size: {} hashes)",
            proof.proof_hashes().len()
        );

        // Verify proof
        let valid = peer1_tree.verify_proof(hash, &proof);
        if valid {
            println!("  ✓ Proof verified successfully!");
        } else {
            println!("  ✗ Proof verification failed!");
        }

        // Try to verify with wrong hash
        println!("\n  Testing invalid proof:");
        let fake_hash = peer1_hashes.get(1).unwrap_or(hash);
        let invalid = peer1_tree.verify_proof(fake_hash, &proof);
        if !invalid {
            println!("  ✓ Invalid proof correctly rejected!");
        } else {
            println!("  ✗ Invalid proof incorrectly accepted!");
        }
    }

    println!("\n7. Simulating P2P sync workflow\n");

    println!("Typical P2P sync process:");
    println!("  1. Exchange Merkle roots");
    println!("     - If roots match → already in sync ✓");
    println!("     - If roots differ → need sync");
    println!();
    println!("  2. Compare trees to find differences");
    println!("     - tree.diff(&peer_tree)");
    println!();
    println!("  3. Request missing items from peer");
    println!("     - Peer sends: (model_data, merkle_proof)");
    println!();
    println!("  4. Verify proof before accepting");
    println!("     - if peer_tree.verify_proof(&hash, &proof) {{");
    println!("         txn.create(&model)?;");
    println!("       }}");
    println!();
    println!("  5. Send our missing items to peer");
    println!("     - Generate proofs for items peer doesn't have");
    println!("     - Peer verifies before accepting");

    println!("\n8. Tree statistics\n");

    println!("  Peer 1 tree:");
    println!("    - Leaves: {}", peer1_tree.len());
    println!("    - Root: {}", hex::encode(peer1_root));
    println!("    - Is empty: {}", peer1_tree.is_empty());

    println!("\n  Peer 2 tree:");
    println!("    - Leaves: {}", peer2_tree.len());
    println!("    - Root: {}", hex::encode(peer2_root));
    println!("    - Is empty: {}", peer2_tree.is_empty());

    println!("\n9. Summary\n");
    println!("Merkle Tree Benefits:");
    println!("  ✓ O(1) sync check (compare roots)");
    println!("  ✓ O(log n) proof generation & verification");
    println!("  ✓ O(n) diff calculation");
    println!("  ✓ Cryptographic proof of inclusion");
    println!("  ✓ Efficient bandwidth usage");
    println!("\nUse Cases:");
    println!("  • P2P data synchronization");
    println!("  • Distributed databases");
    println!("  • Blockchain-like verification");
    println!("  • Content-addressed storage");

    // Verify sync of content-addressed models (ImmutablePost)
    println!("\n--- Syncing Content-Addressed Models ---");

    let post = ImmutablePost {
        author: "SyncUser".to_string(),
        content: "Synced Content".to_string(),
        timestamp: 12345,
    };

    // Create post in Peer 1
    {
        let txn = peer1_store.begin_write()?;
        let envelope = ImmutablePostEnvelope::from(&post);
        txn.create(&envelope)?;
        txn.commit()?;
    }

    // Sync Peer 1 -> Peer 2
    // In a real Merkle sync, we would compare root hashes.
    // Here we simulate detecting the missing hash and transferring it.
    use netabase_store_examples::boilerplate_lib::definition::ImmutablePostID;
    use netabase_store_examples::boilerplate_lib::models::hash_model;

    let hash = ImmutablePostID(hash_model(&post));
    println!("Detected new content hash: {}", hash);

    // Transfer logic: Read from Peer 1, Write to Peer 2
    {
        let txn_1 = peer1_store.begin_read()?;
        // Read the envelope using the wrapper ID
        let envelope = txn_1.read::<ImmutablePostEnvelope>(&hash)?.unwrap();

        let txn_2 = peer2_store.begin_write()?;
        // Insert into Peer 2 (idempotent)
        txn_2.create(&envelope)?;
        txn_2.commit()?;
    }

    // Verify Peer 2 has the post
    {
        let txn = peer2_store.begin_read()?;
        // We can check existence by reading
        let result = txn.read::<ImmutablePostEnvelope>(&hash)?;
        assert!(result.is_some(), "Store B should have the synced post");
        println!("Store B successfully synced content-addressed post!");
    }

    Ok(())
}
