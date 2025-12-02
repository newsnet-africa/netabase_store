//! Database Introspection Traits
//!
//! This module provides traits for inspecting the internal structure of databases,
//! including all trees (user-defined models, secondary indexes, and system trees).

use crate::error::NetabaseError;
use crate::traits::definition::NetabaseDefinitionTrait;

/// Represents information about a tree in the database
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeInfo {
    /// The name of the tree as stored in the database
    pub name: String,
    /// The type/category of the tree
    pub tree_type: TreeType,
    /// Estimated number of entries (if available)
    pub entry_count: Option<usize>,
    /// Estimated size in bytes (if available)
    pub size_bytes: Option<u64>,
}

/// Categories of trees that can exist in a database
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeType {
    /// Primary model tree (stores model instances by primary key)
    PrimaryModel,
    /// Secondary index tree (maps secondary keys to primary keys)
    SecondaryIndex,
    /// libp2p provider records (when libp2p feature is enabled)
    LibP2PProviders,
    /// libp2p provided keys (when libp2p feature is enabled)
    LibP2PProvided,
    /// Subscription/sync trees
    Subscription,
    /// Unknown/system tree
    System,
}

impl TreeType {
    /// Check if this is a user-visible tree type
    pub fn is_user_visible(&self) -> bool {
        matches!(self, TreeType::PrimaryModel | TreeType::SecondaryIndex)
    }

    /// Check if this is a system-internal tree
    pub fn is_system_tree(&self) -> bool {
        matches!(
            self,
            TreeType::LibP2PProviders
                | TreeType::LibP2PProvided
                | TreeType::Subscription
                | TreeType::System
        )
    }
}

/// Trait for introspecting database internals
///
/// This trait provides methods to inspect all trees in a database, including:
/// - User-defined model trees
/// - Secondary key index trees
/// - System trees (libp2p, subscriptions, etc.)
///
/// This is useful for:
/// - Testing and verification
/// - Debugging database state
/// - Database diagnostics and monitoring
/// - Migration and backup tools
pub trait DatabaseIntrospection<D: NetabaseDefinitionTrait> {
    /// List all trees in the database
    ///
    /// Returns a vector of `TreeInfo` describing each tree, including:
    /// - User-defined model trees
    /// - Secondary key indexes
    /// - System trees (libp2p, etc.)
    fn list_all_trees(&self) -> Result<Vec<TreeInfo>, NetabaseError>;

    /// List only user-defined model trees
    ///
    /// This is equivalent to the existing `tree_names()` but returns full TreeInfo
    fn list_model_trees(&self) -> Result<Vec<TreeInfo>, NetabaseError> {
        Ok(self
            .list_all_trees()?
            .into_iter()
            .filter(|info| info.tree_type == TreeType::PrimaryModel)
            .collect())
    }

    /// List only secondary index trees
    fn list_secondary_trees(&self) -> Result<Vec<TreeInfo>, NetabaseError> {
        Ok(self
            .list_all_trees()?
            .into_iter()
            .filter(|info| info.tree_type == TreeType::SecondaryIndex)
            .collect())
    }

    /// List only system trees
    fn list_system_trees(&self) -> Result<Vec<TreeInfo>, NetabaseError> {
        Ok(self
            .list_all_trees()?
            .into_iter()
            .filter(|info| info.tree_type.is_system_tree())
            .collect())
    }

    /// Get the number of entries in a specific tree by name
    ///
    /// This provides a way to count entries in any tree, including system trees
    fn tree_entry_count(&self, tree_name: &str) -> Result<usize, NetabaseError>;

    /// Get all keys in a tree as raw bytes
    ///
    /// This is a low-level method that returns raw key bytes from any tree.
    /// Useful for debugging and verification.
    fn tree_keys_raw(&self, tree_name: &str) -> Result<Vec<Vec<u8>>, NetabaseError>;

    /// Get all key-value pairs in a tree as raw bytes
    ///
    /// This is a low-level method that returns raw data from any tree.
    /// Useful for debugging, backup, and migration.
    fn tree_contents_raw(&self, tree_name: &str) -> Result<Vec<(Vec<u8>, Vec<u8>)>, NetabaseError>;

    /// Check if a tree exists
    fn tree_exists(&self, tree_name: &str) -> Result<bool, NetabaseError> {
        Ok(self
            .list_all_trees()?
            .iter()
            .any(|info| info.name == tree_name))
    }

    /// Get detailed statistics about the database
    fn database_stats(&self) -> Result<DatabaseStats, NetabaseError> {
        let all_trees = self.list_all_trees()?;
        let total_entries: usize = all_trees
            .iter()
            .filter_map(|t| t.entry_count)
            .sum();
        let total_size: u64 = all_trees
            .iter()
            .filter_map(|t| t.size_bytes)
            .sum();

        Ok(DatabaseStats {
            total_trees: all_trees.len(),
            model_trees: all_trees.iter().filter(|t| t.tree_type == TreeType::PrimaryModel).count(),
            secondary_trees: all_trees.iter().filter(|t| t.tree_type == TreeType::SecondaryIndex).count(),
            system_trees: all_trees.iter().filter(|t| t.tree_type.is_system_tree()).count(),
            total_entries,
            total_size_bytes: total_size,
        })
    }
}

/// Database statistics
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseStats {
    /// Total number of trees
    pub total_trees: usize,
    /// Number of model trees
    pub model_trees: usize,
    /// Number of secondary index trees
    pub secondary_trees: usize,
    /// Number of system trees
    pub system_trees: usize,
    /// Total entries across all trees
    pub total_entries: usize,
    /// Total size in bytes
    pub total_size_bytes: u64,
}
