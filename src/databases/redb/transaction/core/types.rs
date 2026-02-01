//! Core transaction type definitions.
//!
//! This module contains the fundamental transaction types used by the redb backend.
//! These are separated from the impl blocks to keep the type definitions clean.

use strum::IntoDiscriminant;

use crate::traits::registry::definition::redb_definition::RedbDefinition;

use super::super::wrappers::{NetabaseRedbReadTransaction, NetabaseRedbWriteTransaction};

/// The inner transaction container.
///
/// This struct holds either a read or write transaction and provides
/// a unified interface for database operations.
pub struct RedbTransactionInner<'txn, D: RedbDefinition>
where
    <D as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    /// The underlying transaction (read or write).
    pub(crate) transaction: RedbTransactionType<'txn, D>,
}

/// Discriminated union of transaction types.
///
/// This enum allows us to handle both read and write transactions
/// through a single interface while maintaining type safety.
pub enum RedbTransactionType<'txn, D: RedbDefinition>
where
    <D as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    /// A read-only transaction.
    Read(NetabaseRedbReadTransaction<'txn, D>),
    /// A read-write transaction.
    Write(NetabaseRedbWriteTransaction<'txn, D>),
}

/// Type alias for the main transaction type.
///
/// This is the primary type users interact with for database operations.
pub type RedbTransaction<'db, D> = RedbTransactionInner<'db, D>;
