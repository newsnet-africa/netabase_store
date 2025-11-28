//! Enhanced subscription tree implementation using merkle trees for efficient synchronization
//!
//! This module provides the core implementation for subscription trees that track
//! data changes using merkle trees. It allows for efficient comparison and
//! synchronization between different nodes.

use std::collections::{BTreeMap, HashMap};
use std::marker::PhantomData;

use netabase_deps::blake3;
use rs_merkle::{MerkleTree, algorithms::Sha256};

use crate::{
    error::NetabaseError,
    traits::subscription::{
        SubscriptionManager, SubscriptionStats, SubscriptionTree, Subscriptions,
        subscription_tree::ModelHash,
    },
};

/// Represents the difference between two subscription trees
#[derive(Debug, Clone)]
pub struct SubscriptionDiff<S: Subscriptions> {
    /// Items that are missing in the first tree but present in the second
    pub missing_in_self: Vec<(Vec<u8>, ModelHash)>,
    /// Items that are missing in the second tree but present in the first
    pub missing_in_other: Vec<(Vec<u8>, ModelHash)>,
    /// Items that exist in both trees but with different values
    pub different_values: Vec<(Vec<u8>, ModelHash, ModelHash)>, // key, their_hash, our_hash
    /// Topic this diff relates to
    pub topic: S::Subscriptions,
}

impl<S: Subscriptions> SubscriptionDiff<S> {
    pub fn new(topic: S::Subscriptions) -> Self {
        Self {
            missing_in_self: Vec::new(),
            missing_in_other: Vec::new(),
            different_values: Vec::new(),
            topic,
        }
    }

    /// Check if there are any differences
    pub fn has_differences(&self) -> bool {
        !self.missing_in_self.is_empty()
            || !self.missing_in_other.is_empty()
            || !self.different_values.is_empty()
    }

    /// Get the total number of differences
    pub fn total_differences(&self) -> usize {
        self.missing_in_self.len() + self.missing_in_other.len() + self.different_values.len()
    }

    /// Get all keys that need to be synchronized from the other tree to self
    pub fn keys_needed_by_self(&self) -> Vec<&Vec<u8>> {
        self.missing_in_self.iter().map(|(key, _)| key).collect()
    }

    /// Get all keys that need to be synchronized from self to the other tree
    pub fn keys_needed_by_other(&self) -> Vec<&Vec<u8>> {
        self.missing_in_other.iter().map(|(key, _)| key).collect()
    }

    /// Get all keys that have conflicting values
    pub fn conflicting_keys(&self) -> Vec<&Vec<u8>> {
        self.different_values
            .iter()
            .map(|(key, _, _)| key)
            .collect()
    }

    /// Get a summary of the differences
    pub fn summary(&self) -> String {
        format!(
            "Topic: {:?}, Missing in self: {}, Missing in other: {}, Different values: {}",
            self.topic,
            self.missing_in_self.len(),
            self.missing_in_other.len(),
            self.different_values.len()
        )
    }

    /// Merge this diff with another diff for the same topic
    pub fn merge(&mut self, other: SubscriptionDiff<S>) {
        assert_eq!(
            self.topic, other.topic,
            "Cannot merge diffs for different topics"
        );

        self.missing_in_self.extend(other.missing_in_self);
        self.missing_in_other.extend(other.missing_in_other);
        self.different_values.extend(other.different_values);
    }
}

/// A merkle tree-based subscription tree for a specific topic
#[derive(Clone)]
pub struct MerkleSubscriptionTree<S: Subscriptions> {
    topic: S::Subscriptions,
    /// Map from model key to model hash
    items: BTreeMap<Vec<u8>, ModelHash>,
    /// Cached merkle tree root - rebuilt when items change
    merkle_root: Option<[u8; 32]>,
    /// Merkle tree for efficient comparison
    merkle_tree: Option<MerkleTree<Sha256>>,
    /// Flag to track if merkle tree needs rebuilding
    needs_rebuild: bool,
    _phantom: PhantomData<S>,
}

impl<S: Subscriptions> MerkleSubscriptionTree<S> {
    /// Create a new subscription tree for the given topic
    pub fn new(topic: S::Subscriptions) -> Self {
        Self {
            topic,
            items: BTreeMap::new(),
            merkle_root: None,
            merkle_tree: None,
            needs_rebuild: false,
            _phantom: PhantomData,
        }
    }

    /// Get the topic this tree tracks
    pub fn topic(&self) -> S::Subscriptions {
        self.topic
    }

    /// Mark the merkle tree as needing to be rebuilt
    fn mark_dirty(&mut self) {
        self.needs_rebuild = true;
        self.merkle_root = None;
    }

    /// Rebuild the merkle tree from current items if needed
    fn rebuild_if_needed(&mut self) -> Result<(), NetabaseError> {
        if !self.needs_rebuild {
            return Ok(());
        }

        if self.items.is_empty() {
            self.merkle_tree = None;
            self.merkle_root = None;
            self.needs_rebuild = false;
            return Ok(());
        }

        // Create leaves from all item hashes, sorted by key for deterministic ordering
        let mut leaves: Vec<[u8; 32]> = self
            .items
            .iter()
            .map(|(key, hash)| {
                // Combine key and hash for the leaf
                let mut hasher = blake3::Hasher::new();
                hasher.update(key);
                hasher.update(hash.as_bytes());
                *hasher.finalize().as_bytes()
            })
            .collect();

        // Sort leaves for deterministic tree structure
        leaves.sort();

        // Build merkle tree
        let tree = MerkleTree::<Sha256>::from_leaves(&leaves);
        self.merkle_root = tree.root().map(|root| {
            let mut root_bytes = [0u8; 32];
            root_bytes.copy_from_slice(&root);
            root_bytes
        });
        self.merkle_tree = Some(tree);
        self.needs_rebuild = false;

        Ok(())
    }

    /// Get all items as an iterator
    pub fn iter(&self) -> impl Iterator<Item = (&Vec<u8>, &ModelHash)> {
        self.items.iter()
    }

    /// Get all hashes
    pub fn all_hashes(&self) -> Vec<ModelHash> {
        self.items.values().cloned().collect()
    }

    /// Get statistics about this tree
    pub fn stats(&self) -> TreeStats {
        TreeStats {
            item_count: self.items.len(),
            has_merkle_root: self.merkle_root.is_some(),
            needs_rebuild: self.needs_rebuild,
        }
    }

    /// Compare this tree with another tree and return their differences
    pub fn compare_with(
        &mut self,
        other: &mut MerkleSubscriptionTree<S>,
    ) -> Result<SubscriptionDiff<S>, NetabaseError> {
        let our_root = self.merkle_root()?;
        let their_root = other.merkle_root()?;

        let mut diff = SubscriptionDiff::new(self.topic);

        // If roots are the same, no differences
        if our_root == their_root {
            return Ok(diff);
        }

        let our_items = self.get_all_items();
        let their_items = other.get_all_items();

        let our_map: HashMap<Vec<u8>, ModelHash> = our_items.into_iter().collect();
        let their_map: HashMap<Vec<u8>, ModelHash> = their_items.into_iter().collect();

        // Find missing in self
        for (key, hash) in &their_map {
            if !our_map.contains_key(key) {
                diff.missing_in_self.push((key.clone(), hash.clone()));
            }
        }

        // Find missing in other
        for (key, hash) in &our_map {
            if !their_map.contains_key(key) {
                diff.missing_in_other.push((key.clone(), hash.clone()));
            }
        }

        // Find different values
        for (key, our_hash) in &our_map {
            if let Some(their_hash) = their_map.get(key) {
                if our_hash != their_hash {
                    diff.different_values
                        .push((key.clone(), their_hash.clone(), our_hash.clone()));
                }
            }
        }

        Ok(diff)
    }
}

impl<S: Subscriptions> SubscriptionTree<S> for MerkleSubscriptionTree<S> {
    type Topic = S::Subscriptions;

    fn topic(&self) -> Self::Topic {
        self.topic
    }

    fn put_item(&mut self, key: Vec<u8>, hash: ModelHash) -> Result<(), NetabaseError> {
        self.items.insert(key, hash);
        self.mark_dirty();
        Ok(())
    }

    fn remove_item(&mut self, key: &[u8]) -> Result<Option<ModelHash>, NetabaseError> {
        let result = self.items.remove(key);
        if result.is_some() {
            self.mark_dirty();
        }
        Ok(result)
    }

    fn get_all_hashes(&self) -> Result<Vec<ModelHash>, NetabaseError> {
        Ok(self.all_hashes())
    }

    fn merkle_root(&mut self) -> Result<Option<[u8; 32]>, NetabaseError> {
        self.rebuild_if_needed()?;
        Ok(self.merkle_root)
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    fn clear(&mut self) -> Result<(), NetabaseError> {
        self.items.clear();
        self.mark_dirty();
        Ok(())
    }

    fn contains_key(&self, key: &[u8]) -> bool {
        self.items.contains_key(key)
    }

    fn get_hash(&self, key: &[u8]) -> Option<&ModelHash> {
        self.items.get(key)
    }

    fn get_all_keys(&self) -> Vec<Vec<u8>> {
        self.items.keys().cloned().collect()
    }

    fn get_all_items(&self) -> Vec<(Vec<u8>, ModelHash)> {
        self.items
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    fn rebuild_merkle_tree(&mut self) -> Result<(), NetabaseError> {
        self.needs_rebuild = true;
        self.rebuild_if_needed()
    }
}

/// Statistics about a subscription tree
#[derive(Debug, Clone)]
pub struct TreeStats {
    pub item_count: usize,
    pub has_merkle_root: bool,
    pub needs_rebuild: bool,
}

/// Default subscription manager implementation
pub struct DefaultSubscriptionManager<S: Subscriptions> {
    trees: HashMap<S::Subscriptions, MerkleSubscriptionTree<S>>,
    _phantom: PhantomData<S>,
}

impl<S: Subscriptions> DefaultSubscriptionManager<S> {
    /// Create a new subscription manager with all topics initialized
    pub fn new() -> Self {
        let mut trees = HashMap::new();
        for topic in S::subscriptions() {
            trees.insert(topic, MerkleSubscriptionTree::new(topic));
        }
        Self {
            trees,
            _phantom: PhantomData,
        }
    }

    /// Compare this manager with another manager and return differences for all topics
    pub fn compare_with(
        &mut self,
        other: &mut DefaultSubscriptionManager<S>,
    ) -> Result<std::collections::HashMap<S::Subscriptions, SubscriptionDiff<S>>, NetabaseError>
    {
        let mut diffs = std::collections::HashMap::new();

        for topic in S::subscriptions() {
            if let (Some(our_tree), Some(their_tree)) =
                (self.trees.get_mut(&topic), other.trees.get_mut(&topic))
            {
                let diff = our_tree.compare_with(their_tree)?;
                if diff.has_differences() {
                    diffs.insert(topic, diff);
                }
            }
        }

        Ok(diffs)
    }

    /// Initialize only specific topics
    pub fn with_topics(topics: &[S::Subscriptions]) -> Self {
        let mut trees = HashMap::new();
        for &topic in topics {
            trees.insert(topic, MerkleSubscriptionTree::new(topic));
        }
        Self {
            trees,
            _phantom: PhantomData,
        }
    }
}

impl<S: Subscriptions> Default for DefaultSubscriptionManager<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Subscriptions> SubscriptionManager<S> for DefaultSubscriptionManager<S> {
    type TopicType = S::Subscriptions;

    fn subscribe_item<T>(
        &mut self,
        topic: S::Subscriptions,
        key: Vec<u8>,
        data: &T,
    ) -> Result<(), NetabaseError>
    where
        T: AsRef<[u8]>,
    {
        let hash = ModelHash::from_key_and_data(&key, data);
        if let Some(tree) = self.trees.get_mut(&topic) {
            tree.put_item(key, hash)?;
        }
        Ok(())
    }

    fn unsubscribe_item(
        &mut self,
        topic: S::Subscriptions,
        key: &[u8],
    ) -> Result<Option<ModelHash>, NetabaseError> {
        if let Some(tree) = self.trees.get_mut(&topic) {
            tree.remove_item(key)
        } else {
            Ok(None)
        }
    }

    fn topic_merkle_root(
        &mut self,
        topic: S::Subscriptions,
    ) -> Result<Option<[u8; 32]>, NetabaseError> {
        if let Some(tree) = self.trees.get_mut(&topic) {
            tree.merkle_root()
        } else {
            Ok(None)
        }
    }

    fn stats(&self) -> SubscriptionStats {
        let mut stats = SubscriptionStats::new();
        for (_topic, tree) in &self.trees {
            stats.add_topic_count(tree.len());
        }
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumIter)]
    pub enum TestTopics {
        Users,
        Posts,
    }

    impl std::fmt::Display for TestTopics {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                TestTopics::Users => write!(f, "Users"),
                TestTopics::Posts => write!(f, "Posts"),
            }
        }
    }

    struct TestDef;

    impl Subscriptions for TestDef {
        type Subscriptions = TestTopics;

        fn subscriptions() -> <Self::Subscriptions as IntoEnumIterator>::Iterator {
            TestTopics::iter()
        }
    }

    #[test]
    fn test_model_hash_creation() {
        let data = b"test data";
        let hash1 = ModelHash::from_data(data);
        let hash2 = ModelHash::from_data(data);
        assert_eq!(hash1, hash2);

        let different_data = b"different data";
        let hash3 = ModelHash::from_data(different_data);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_subscription_diff() {
        let mut tree1 = MerkleSubscriptionTree::<TestDef>::new(TestTopics::Users);
        let mut tree2 = MerkleSubscriptionTree::<TestDef>::new(TestTopics::Users);

        // Tree1 has user1, tree2 has user2
        tree1
            .put_item(b"user1".to_vec(), ModelHash::from_data(b"data1"))
            .unwrap();
        tree2
            .put_item(b"user2".to_vec(), ModelHash::from_data(b"data2"))
            .unwrap();

        let diff = tree1.compare_with(&mut tree2).unwrap();

        assert!(diff.has_differences());
        assert_eq!(diff.total_differences(), 2);
        assert_eq!(diff.missing_in_self.len(), 1);
        assert_eq!(diff.missing_in_other.len(), 1);
        assert_eq!(diff.different_values.len(), 0);

        // user2 is missing in tree1
        assert_eq!(diff.missing_in_self[0].0, b"user2");
        // user1 is missing in tree2
        assert_eq!(diff.missing_in_other[0].0, b"user1");
    }

    #[test]
    fn test_subscription_diff_conflicting_values() {
        let mut tree1 = MerkleSubscriptionTree::<TestDef>::new(TestTopics::Users);
        let mut tree2 = MerkleSubscriptionTree::<TestDef>::new(TestTopics::Users);

        // Both trees have user1 but with different data
        tree1
            .put_item(b"user1".to_vec(), ModelHash::from_data(b"data1"))
            .unwrap();
        tree2
            .put_item(b"user1".to_vec(), ModelHash::from_data(b"data2"))
            .unwrap();

        let diff = tree1.compare_with(&mut tree2).unwrap();

        assert!(diff.has_differences());
        assert_eq!(diff.total_differences(), 1);
        assert_eq!(diff.missing_in_self.len(), 0);
        assert_eq!(diff.missing_in_other.len(), 0);
        assert_eq!(diff.different_values.len(), 1);

        let (key, their_hash, our_hash) = &diff.different_values[0];
        assert_eq!(key, b"user1");
        assert_eq!(*their_hash, ModelHash::from_data(b"data2"));
        assert_eq!(*our_hash, ModelHash::from_data(b"data1"));
    }

    #[test]
    fn test_subscription_tree_basic_operations() {
        let mut tree = MerkleSubscriptionTree::<TestDef>::new(TestTopics::Users);

        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);

        let key1 = b"user1".to_vec();
        let hash1 = ModelHash::from_data(b"user1_data");

        tree.put_item(key1.clone(), hash1.clone()).unwrap();
        assert_eq!(tree.len(), 1);
        assert!(!tree.is_empty());
        assert!(tree.contains_key(&key1));
        assert_eq!(tree.get_hash(&key1), Some(&hash1));

        let removed = tree.remove_item(&key1).unwrap();
        assert_eq!(removed, Some(hash1));
        assert!(tree.is_empty());
        assert!(!tree.contains_key(&key1));
    }

    #[test]
    fn test_subscription_manager() {
        let mut manager = DefaultSubscriptionManager::<TestDef>::new();

        // Test initial state
        let stats = manager.stats();
        assert_eq!(stats.total_items, 0);
        assert_eq!(stats.active_topics, 0);

        // Add item to Users topic
        manager
            .subscribe_item(TestTopics::Users, b"user1".to_vec(), b"user_data")
            .unwrap();

        let stats = manager.stats();
        assert_eq!(stats.total_items, 1);
        assert_eq!(stats.active_topics, 1);

        // Remove item
        let removed = manager
            .unsubscribe_item(TestTopics::Users, b"user1")
            .unwrap();
        assert!(removed.is_some());

        let stats = manager.stats();
        assert_eq!(stats.total_items, 0);
        assert_eq!(stats.active_topics, 0);
    }

    #[test]
    fn test_manager_comparison() {
        let mut manager1 = DefaultSubscriptionManager::<TestDef>::new();
        let mut manager2 = DefaultSubscriptionManager::<TestDef>::new();

        // Add different data to each manager
        manager1
            .subscribe_item(TestTopics::Users, b"user1".to_vec(), b"data1")
            .unwrap();

        manager2
            .subscribe_item(TestTopics::Users, b"user2".to_vec(), b"data2")
            .unwrap();

        let diffs = manager1.compare_with(&mut manager2).unwrap();
        assert_eq!(diffs.len(), 1);

        let user_diff = &diffs[&TestTopics::Users];
        assert!(user_diff.has_differences());
        assert_eq!(user_diff.total_differences(), 2);
        assert_eq!(user_diff.missing_in_self.len(), 1);
        assert_eq!(user_diff.missing_in_other.len(), 1);
    }
}
