//! Shared types and utilities for sled_store module.

use std::marker::PhantomData;

use crate::traits::definition::NetabaseDefinitionTrait;
use crate::traits::model::NetabaseModelTrait;

/// Operation to be performed on secondary index after transaction commits.
///
/// This enum is used to defer secondary key operations until after a transaction
/// has successfully committed, preventing multi-tree deadlocks in sled.
#[derive(Debug, Clone)]
pub enum SecondaryKeyOp {
    /// Insert a secondary key entry
    Insert(Vec<u8>),
    /// Remove a secondary key entry
    Remove(Vec<u8>),
}

/// Helper to maintain phantom data for unused generic parameters.
pub(super) struct Phantom<D, M> {
    pub(super) _d: PhantomData<D>,
    pub(super) _m: PhantomData<M>,
}

impl<D, M> Default for Phantom<D, M> {
    fn default() -> Self {
        Self {
            _d: PhantomData,
            _m: PhantomData,
        }
    }
}
