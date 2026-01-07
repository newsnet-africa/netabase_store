//! Subscription hash utilities for merkle tree-based syncing
//!
//! This module provides hashing and merkle tree functionality for subscription-based
//! peer-to-peer synchronization.

use sha2::{Sha256, Digest};
use rs_merkle::{MerkleTree, Hasher as MerkleHasher, MerkleProof};
use serde::{Serialize, Deserialize};
use std::borrow::Borrow;

/// SHA-256 hasher for rs_merkle
#[derive(Clone)]
pub struct Sha256Hasher;

impl MerkleHasher for Sha256Hasher {
    type Hash = [u8; 32];

    fn hash(data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }
}

/// Hash of a model for subscription tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ModelHash(pub [u8; 32]);

impl ModelHash {
    /// Create a new model hash from bytes
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Create a model hash from serializable data
    pub fn from_data<T: Serialize>(data: &T) -> Result<Self, Box<dyn std::error::Error>> {
        let serialized = bincode::serialize(data)?;
        let mut hasher = Sha256::new();
        hasher.update(&serialized);
        Ok(Self(hasher.finalize().into()))
    }

    /// Get the hash as bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Parse from hex string
    pub fn from_hex(s: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(s)?;
        if bytes.len() != 32 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&bytes);
        Ok(Self(hash))
    }
}

// Implement redb::Key for ModelHash
impl redb::Key for ModelHash {
    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
        data1.cmp(data2)
    }
}

impl redb::Value for ModelHash {
    type SelfType<'a> = ModelHash;
    type AsBytes<'a> = [u8; 32];

    fn fixed_width() -> Option<usize> {
        Some(32)
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(data);
        ModelHash(bytes)
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        value.0
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("ModelHash")
    }
}

impl Borrow<[u8; 32]> for ModelHash {
    fn borrow(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Value stored in subscription tables: (Primary Key, Model Hash)
/// This is stored as a fixed-size 64-byte array: 32 bytes PK + 32 bytes hash
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SubscriptionValue {
    /// Primary key of the subscribed model (as 32-byte hash/ID)
    pub primary_key_hash: [u8; 32],
    /// Hash of the model content
    pub model_hash: ModelHash,
}

impl SubscriptionValue {
    pub fn new(primary_key_hash: [u8; 32], model_hash: ModelHash) -> Self {
        Self { primary_key_hash, model_hash }
    }
}

impl redb::Value for SubscriptionValue {
    type SelfType<'a> = SubscriptionValue;
    type AsBytes<'a> = [u8; 64];

    fn fixed_width() -> Option<usize> {
        Some(64)
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        let mut pk_hash = [0u8; 32];
        let mut model_hash = [0u8; 32];
        pk_hash.copy_from_slice(&data[0..32]);
        model_hash.copy_from_slice(&data[32..64]);
        SubscriptionValue {
            primary_key_hash: pk_hash,
            model_hash: ModelHash(model_hash),
        }
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        let mut bytes = [0u8; 64];
        bytes[0..32].copy_from_slice(&value.primary_key_hash);
        bytes[32..64].copy_from_slice(&value.model_hash.0);
        bytes
    }

    fn type_name() -> redb::TypeName {
        redb::TypeName::new("SubscriptionValue")
    }
}

/// Subscription merkle tree for efficient comparison
pub struct SubscriptionMerkleTree {
    tree: MerkleTree<Sha256Hasher>,
    hashes: Vec<ModelHash>,
}

impl SubscriptionMerkleTree {
    /// Build a merkle tree from model hashes
    pub fn from_hashes(mut hashes: Vec<ModelHash>) -> Self {
        // Sort hashes for deterministic tree construction
        hashes.sort();
        
        let leaves: Vec<[u8; 32]> = hashes.iter().map(|h| h.0).collect();
        let tree = MerkleTree::<Sha256Hasher>::from_leaves(&leaves);
        
        Self { tree, hashes }
    }

    /// Get the merkle root
    pub fn root(&self) -> Option<[u8; 32]> {
        self.tree.root()
    }

    /// Get the root as a hex string
    pub fn root_hex(&self) -> Option<String> {
        self.root().map(|r| hex::encode(r))
    }

    /// Get the number of leaves
    pub fn len(&self) -> usize {
        self.hashes.len()
    }

    /// Check if tree is empty
    pub fn is_empty(&self) -> bool {
        self.hashes.is_empty()
    }

    /// Get all hashes
    pub fn hashes(&self) -> &[ModelHash] {
        &self.hashes
    }

    /// Generate a proof for a specific hash
    pub fn proof(&self, hash: &ModelHash) -> Option<MerkleProof<Sha256Hasher>> {
        let index = self.hashes.iter().position(|h| h == hash)?;
        let indices = vec![index];
        Some(self.tree.proof(&indices))
    }

    /// Verify a proof for a specific hash
    pub fn verify_proof(&self, hash: &ModelHash, proof: &MerkleProof<Sha256Hasher>) -> bool {
        if let Some(root) = self.root() {
            // Find the index of the hash in the sorted hashes
            if let Some(index) = self.hashes.iter().position(|h| h == hash) {
                proof.verify(root, &[index], &[hash.0], self.hashes.len())
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Compare with another tree and find differences
    pub fn diff(&self, other: &SubscriptionMerkleTree) -> SubscriptionDiff {
        let mut missing_in_other = Vec::new();
        let mut missing_in_self = Vec::new();

        // Find hashes in self but not in other
        for hash in &self.hashes {
            if !other.hashes.contains(hash) {
                missing_in_other.push(*hash);
            }
        }

        // Find hashes in other but not in self
        for hash in &other.hashes {
            if !self.hashes.contains(hash) {
                missing_in_self.push(*hash);
            }
        }

        SubscriptionDiff {
            missing_in_other,
            missing_in_self,
        }
    }
}

/// Difference between two subscription merkle trees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionDiff {
    /// Hashes present in the first tree but not in the second
    pub missing_in_other: Vec<ModelHash>,
    /// Hashes present in the second tree but not in the first
    pub missing_in_self: Vec<ModelHash>,
}

impl SubscriptionDiff {
    /// Check if there are any differences
    pub fn has_differences(&self) -> bool {
        !self.missing_in_other.is_empty() || !self.missing_in_self.is_empty()
    }

    /// Get total number of differences
    pub fn diff_count(&self) -> usize {
        self.missing_in_other.len() + self.missing_in_self.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_hash_creation() {
        let hash1 = ModelHash::new([1u8; 32]);
        let hash2 = ModelHash::new([1u8; 32]);
        let hash3 = ModelHash::new([2u8; 32]);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_model_hash_from_data() {
        let data = "test data";
        let hash1 = ModelHash::from_data(&data).unwrap();
        let hash2 = ModelHash::from_data(&data).unwrap();
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_model_hash_hex() {
        let hash = ModelHash::new([0xAB; 32]);
        let hex = hash.to_hex();
        let parsed = ModelHash::from_hex(&hex).unwrap();
        
        assert_eq!(hash, parsed);
    }

    #[test]
    fn test_merkle_tree_creation() {
        let hashes = vec![
            ModelHash::new([1u8; 32]),
            ModelHash::new([2u8; 32]),
            ModelHash::new([3u8; 32]),
        ];

        let tree = SubscriptionMerkleTree::from_hashes(hashes.clone());
        
        assert_eq!(tree.len(), 3);
        assert!(tree.root().is_some());
    }

    #[test]
    fn test_merkle_tree_diff() {
        let hashes1 = vec![
            ModelHash::new([1u8; 32]),
            ModelHash::new([2u8; 32]),
            ModelHash::new([3u8; 32]),
        ];

        let hashes2 = vec![
            ModelHash::new([2u8; 32]),
            ModelHash::new([3u8; 32]),
            ModelHash::new([4u8; 32]),
        ];

        let tree1 = SubscriptionMerkleTree::from_hashes(hashes1);
        let tree2 = SubscriptionMerkleTree::from_hashes(hashes2);

        let diff = tree1.diff(&tree2);
        
        assert_eq!(diff.missing_in_other.len(), 1); // hash [1u8; 32]
        assert_eq!(diff.missing_in_self.len(), 1);  // hash [4u8; 32]
        assert!(diff.has_differences());
    }

    #[test]
    fn test_merkle_proof() {
        let hashes = vec![
            ModelHash::new([1u8; 32]),
            ModelHash::new([2u8; 32]),
            ModelHash::new([3u8; 32]),
        ];

        let tree = SubscriptionMerkleTree::from_hashes(hashes.clone());
        let hash = hashes[0];
        
        let proof = tree.proof(&hash).unwrap();
        assert!(tree.verify_proof(&hash, &proof));
    }

    #[test]
    fn test_merkle_proof_all_leaves() {
        // Test proof verification for all leaves
        let hashes = vec![
            ModelHash::new([1u8; 32]),
            ModelHash::new([2u8; 32]),
            ModelHash::new([3u8; 32]),
            ModelHash::new([4u8; 32]),
            ModelHash::new([5u8; 32]),
        ];

        let tree = SubscriptionMerkleTree::from_hashes(hashes.clone());
        
        // Verify proof for each hash
        for hash in &hashes {
            let proof = tree.proof(hash).expect("Should generate proof");
            assert!(tree.verify_proof(hash, &proof), "Proof should verify for hash");
        }
    }

    #[test]
    fn test_merkle_proof_invalid() {
        let hashes = vec![
            ModelHash::new([1u8; 32]),
            ModelHash::new([2u8; 32]),
            ModelHash::new([3u8; 32]),
        ];

        let tree = SubscriptionMerkleTree::from_hashes(hashes.clone());
        
        // Try to verify a hash that's not in the tree
        let wrong_hash = ModelHash::new([99u8; 32]);
        assert!(!tree.verify_proof(&wrong_hash, &tree.proof(&hashes[0]).unwrap()));
    }
}
