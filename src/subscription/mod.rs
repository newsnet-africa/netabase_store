//! Subscription system for tracking data changes and synchronization
//!
//! This module provides the core subscription functionality that allows tracking
//! changes to stored data and synchronizing between different nodes using merkle trees.

pub mod subscription_tree;

// Re-export the Subscriptions trait from the traits module for convenience
pub use crate::traits::subscription::Subscriptions;

// Re-export ModelHash from canonical traits location
pub use crate::traits::subscription::subscription_tree::ModelHash;
