//! Iterator types for Redb backend
//!
//! This module provides iterator implementations for traversing stored models.

use crate::error::NetabaseError;

/// Simple iterator wrapper for redb results
///
/// This wraps a Vec into an iterator that yields Results for consistency
/// with other backend iterators.
pub struct RedbIter<M> {
    items: std::vec::IntoIter<M>,
}

impl<M> RedbIter<M> {
    /// Create a new RedbIter from a vector of items
    pub fn new(items: Vec<M>) -> Self {
        Self {
            items: items.into_iter(),
        }
    }

    /// Create an empty RedbIter
    pub fn empty() -> Self {
        Self {
            items: Vec::new().into_iter(),
        }
    }
}

impl<M> Iterator for RedbIter<M> {
    type Item = Result<M, NetabaseError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.next().map(Ok)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.items.size_hint()
    }
}

impl<M> ExactSizeIterator for RedbIter<M> {
    fn len(&self) -> usize {
        self.items.len()
    }
}

/// Iterator for subscription tree items in Redb
pub struct RedbSubscriptionTreeIter {
    items: Vec<crate::subscription::ModelHash>,
    current: usize,
}

impl RedbSubscriptionTreeIter {
    /// Create a new subscription tree iterator
    pub fn new(items: Vec<crate::subscription::ModelHash>) -> Self {
        Self { items, current: 0 }
    }
}

impl crate::traits::subscription::SubscriptionTreeIter for RedbSubscriptionTreeIter {
    type Item = crate::subscription::ModelHash;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.items.len() {
            let item = self.items[self.current].clone();
            self.current += 1;
            Some(item)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.items.len().saturating_sub(self.current);
        (remaining, Some(remaining))
    }
}
