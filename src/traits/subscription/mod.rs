//! Enhanced subscription system traits for netabase_store
//!
//! This module provides a comprehensive subscription system that allows tracking
//! changes to stored data and synchronizing between different nodes using merkle trees.
//!
//! The subscription system works by:
//! 1. Using `#[streams(Topic1, Topic2)]` attribute to mark modules with subscription topics
//! 2. Generating a `DefinitionStreams` enum from the topics
//! 3. Creating subscription trees that map topic -> ModelHash for efficient comparison
//! 4. Using merkle trees to detect differences between subscription states

use crate::{MaybeSend, MaybeSync, error::NetabaseError};

use strum::IntoEnumIterator;

pub mod subscription_tree;

// Re-export strum traits for macro-generated code
pub use strum::{EnumIter, IntoStaticStr, VariantArray};

// Re-export types from subscription_tree module
pub use subscription_tree::ModelHash;

/// Core trait for subscription functionality
///
/// This trait defines the subscription topics available for a given definition.
/// It's implemented automatically by the `#[netabase_definition_module]` macro
/// when the `#[streams(...)]` attribute is present.
pub trait Subscriptions: 'static + MaybeSend + MaybeSync {
    /// The enum type representing all subscription topics
    type Subscriptions: IntoEnumIterator
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + MaybeSend
        + MaybeSync
        + 'static;

    /// Get an iterator over all subscription topics
    fn subscriptions() -> <Self::Subscriptions as IntoEnumIterator>::Iterator;

    /// Get all subscription topics as a vector
    fn all_subscriptions() -> Vec<Self::Subscriptions> {
        Self::subscriptions().collect()
    }

    /// Get the topic name as a string
    fn topic_name(topic: Self::Subscriptions) -> String {
        topic.to_string()
    }
}

/// Trait for subscription tree implementations
///
/// A subscription tree tracks a specific topic's data using merkle trees
/// for efficient synchronization and comparison.
pub trait SubscriptionTree<S: Subscriptions>: 'static + MaybeSend + MaybeSync {
    /// Associated type for the specific topic this tree handles
    type Topic: Clone + Copy + std::fmt::Debug + PartialEq + Eq + std::hash::Hash;

    /// Get the subscription topic this tree tracks
    fn topic(&self) -> Self::Topic;

    /// Add an item to the subscription tree
    fn put_item(&mut self, key: Vec<u8>, hash: ModelHash) -> Result<(), NetabaseError>;

    /// Remove an item from the subscription tree
    fn remove_item(&mut self, key: &[u8]) -> Result<Option<ModelHash>, NetabaseError>;

    /// Get all hashes in the subscription tree
    fn get_all_hashes(&self) -> Result<Vec<ModelHash>, NetabaseError>;

    /// Get the current merkle root hash
    fn merkle_root(&mut self) -> Result<Option<[u8; 32]>, NetabaseError>;

    /// Get the number of items in the tree
    fn len(&self) -> usize;

    /// Check if the tree is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all items from the tree
    fn clear(&mut self) -> Result<(), NetabaseError>;

    /// Check if a key exists in the tree
    fn contains_key(&self, key: &[u8]) -> bool;

    /// Get the hash for a specific key
    fn get_hash(&self, key: &[u8]) -> Option<&ModelHash>;

    /// Get all keys in the tree
    fn get_all_keys(&self) -> Vec<Vec<u8>>;

    /// Get all key-hash pairs
    fn get_all_items(&self) -> Vec<(Vec<u8>, ModelHash)>;

    /// Rebuild the internal merkle tree (if needed)
    fn rebuild_merkle_tree(&mut self) -> Result<(), NetabaseError>;
}

/// Iterator trait for subscription tree items
pub trait SubscriptionTreeIter: 'static + MaybeSend + MaybeSync {
    type Item;

    /// Get the next item in the iterator
    fn next(&mut self) -> Option<Self::Item>;

    /// Get size hint for the iterator
    fn size_hint(&self) -> (usize, Option<usize>);

    /// Count the remaining items
    fn count(mut self) -> usize
    where
        Self: Sized,
    {
        let mut count = 0;
        while self.next().is_some() {
            count += 1;
        }
        count
    }

    /// Collect all items into a vector
    fn collect_vec(mut self) -> Vec<Self::Item>
    where
        Self: Sized,
    {
        let mut items = Vec::new();
        while let Some(item) = self.next() {
            items.push(item);
        }
        items
    }
}

/// Trait for opening subscription trees from a store
pub trait OpenSubscriptionTree<S: Subscriptions>: 'static + MaybeSend + MaybeSync {
    /// Associated type for the topic identifier
    type TopicType: Clone + Copy + std::fmt::Debug + PartialEq + Eq + std::hash::Hash;

    /// Open a subscription tree for the given topic
    fn open_subscription_tree(&self, topic: Self::TopicType) -> Result<(), NetabaseError>;
}

/// Trait for managing multiple subscription trees
pub trait SubscriptionManager<S: Subscriptions>: 'static + MaybeSend + MaybeSync {
    /// Associated types for topic-specific operations
    type TopicType: Clone + Copy + std::fmt::Debug + PartialEq + Eq + std::hash::Hash;

    /// Add an item to the appropriate subscription tree(s)
    fn subscribe_item<T>(
        &mut self,
        topic: Self::TopicType,
        key: Vec<u8>,
        data: &T,
    ) -> Result<(), NetabaseError>
    where
        T: AsRef<[u8]>;

    /// Remove an item from the subscription tree(s)
    fn unsubscribe_item(
        &mut self,
        topic: Self::TopicType,
        key: &[u8],
    ) -> Result<Option<ModelHash>, NetabaseError>;

    /// Get the merkle root for a specific topic
    fn topic_merkle_root(
        &mut self,
        topic: Self::TopicType,
    ) -> Result<Option<[u8; 32]>, NetabaseError>;

    /// Get statistics about the subscription manager
    fn stats(&self) -> SubscriptionStats;
}

/// Statistics about subscription trees
#[derive(Debug, Clone)]
pub struct SubscriptionStats {
    /// Total number of items across all topics
    pub total_items: usize,
    /// Number of topics with at least one item
    pub active_topics: usize,
}

impl SubscriptionStats {
    pub fn new() -> Self {
        Self {
            total_items: 0,
            active_topics: 0,
        }
    }

    pub fn add_topic_count(&mut self, count: usize) {
        self.total_items += count;
        if count > 0 {
            self.active_topics += 1;
        }
    }
}

impl Default for SubscriptionStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for converting data to ModelHash
pub trait IntoModelHash {
    /// Convert this data into a ModelHash
    fn into_model_hash(self) -> ModelHash;
}

impl<T: AsRef<[u8]>> IntoModelHash for T {
    fn into_model_hash(self) -> ModelHash {
        ModelHash::from_data(self)
    }
}

/// Trait for database stores that support subscription functionality
///
/// This trait enables automatic subscription tracking when implementing stores
/// integrate with the subscription system.
pub trait SubscriptionStore<S: Subscriptions>: 'static + MaybeSend + MaybeSync {
    /// The type of subscription manager used by this store
    type Manager: SubscriptionManager<S, TopicType = S::Subscriptions>;

    /// Get a reference to the subscription manager
    fn subscription_manager(&self) -> Option<&Self::Manager>;

    /// Get a mutable reference to the subscription manager
    fn subscription_manager_mut(&mut self) -> Option<&mut Self::Manager>;

    /// Check if subscription tracking is enabled for this store
    fn subscriptions_enabled(&self) -> bool {
        self.subscription_manager().is_some()
    }

    /// Subscribe an item to a topic when it's added to the store
    fn auto_subscribe<T>(
        &mut self,
        topic: <S as Subscriptions>::Subscriptions,
        key: Vec<u8>,
        data: &T,
    ) -> Result<(), NetabaseError>
    where
        T: AsRef<[u8]>,
    {
        if let Some(manager) = self.subscription_manager_mut() {
            manager.subscribe_item(topic, key, data)
        } else {
            Ok(()) // No-op if subscriptions not enabled
        }
    }

    /// Unsubscribe an item from a topic when it's removed from the store
    fn auto_unsubscribe(
        &mut self,
        topic: <S as Subscriptions>::Subscriptions,
        key: &[u8],
    ) -> Result<Option<ModelHash>, NetabaseError> {
        if let Some(manager) = self.subscription_manager_mut() {
            manager.unsubscribe_item(topic, key)
        } else {
            Ok(None) // No-op if subscriptions not enabled
        }
    }
}

/// Trait for filtering subscription data
///
/// This trait allows customizing which data gets included in subscription trees
/// based on business logic or access controls.
pub trait SubscriptionFilter<S: Subscriptions>: 'static + MaybeSend + MaybeSync {
    /// Check if data should be included in the given topic's subscription tree
    fn should_include<T>(&self, topic: S::Subscriptions, key: &[u8], data: &T) -> bool
    where
        T: AsRef<[u8]>;

    /// Get all topics that should include the given data
    fn applicable_topics<T>(&self, key: &[u8], data: &T) -> Vec<S::Subscriptions>
    where
        T: AsRef<[u8]>;
}

/// Default filter that includes all data in all topics
pub struct DefaultSubscriptionFilter;

impl<S: Subscriptions> SubscriptionFilter<S> for DefaultSubscriptionFilter {
    fn should_include<T>(&self, _topic: S::Subscriptions, _key: &[u8], _data: &T) -> bool
    where
        T: AsRef<[u8]>,
    {
        true
    }

    fn applicable_topics<T>(&self, _key: &[u8], _data: &T) -> Vec<S::Subscriptions>
    where
        T: AsRef<[u8]>,
    {
        S::all_subscriptions()
    }
}
