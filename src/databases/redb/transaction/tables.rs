//! Low-level table access and management.
//!
//! This module provides infrastructure for opening and managing database tables.
//! Most users should use the high-level transaction API instead of interacting with these types directly.
//!
//! # Design
//!
//! Netabase models can have multiple tables:
//! - **Main table**: Primary key → Model data
//! - **Secondary tables**: Secondary key → Primary key lookups
//! - **Blob tables**: Blob key → Blob chunk data
//! - **Relational tables**: Primary key → Related model keys
//! - **Subscription tables**: Topic → Subscriber primary keys
//!
//! ## Table Permissions
//!
//! Tables can be opened with different permission levels:
//! - `ReadOnly`: Can only read data
//! - `ReadWrite`: Can read and write data
//! - `ReadOnlyWrite`: Opened as writable but used read-only (optimization)
//!
//! ## Performance Considerations
//!
//! Opening tables has overhead. For batch operations, use `prepare_model` from the transaction
//! to keep tables open across multiple operations.
//!
//! # Safety
//!
//! Table handles must not outlive their parent transaction. The lifetime system enforces this.

use crate::traits::registery::{
    definition::redb_definition::RedbDefinition,
    models::{
        keys::{NetabaseModelKeys, blob::NetabaseModelBlobKey},
        model::{NetabaseModel, redb_model::RedbNetbaseModel},
    },
};
use strum::IntoDiscriminant;

/// Collection of open tables for a specific model.
///
/// This struct holds references to all the tables needed for CRUD operations on a model.
/// It's returned by `prepare_model` and can be passed to low-level CRUD methods.
///
/// # Lifetimes
///
/// - `'txn`: The lifetime of the table handles (must not outlive transaction)
/// - `'db`: The lifetime of the database
///
/// # Example
///
/// ```rust,ignore
/// let txn = store.begin_write()?;
/// let tables = txn.prepare_model::<User>()?;
///
/// // Use tables for batch operations
/// for user in users {
///     User::create_entry(&user, &tables)?;
/// }
///
/// txn.commit()?;
/// ```
pub struct ModelOpenTables<'txn, 'db, D: RedbDefinition, M: RedbNetbaseModel<'db, D> + redb::Key>
where
    'db: 'txn,
    D::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key + 'static,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key + 'static,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key + 'static,
    D::SubscriptionKeys: redb::Key + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: 'static,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
{
    pub main: TablePermission<'txn, <M::Keys as NetabaseModelKeys<D, M>>::Primary, M::TableV>,

    pub secondary: Vec<(
        TablePermission<'txn, <M::Keys as NetabaseModelKeys<D, M>>::Secondary, <M::Keys as NetabaseModelKeys<D, M>>::Primary>,
        &'db str,
    )>,

    pub blob: Vec<(
        TablePermission<'txn, <M::Keys as NetabaseModelKeys<D, M>>::Blob, <<M::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem>,
        &'db str,
    )>,

    pub relational: Vec<(
        TablePermission<'txn, <M::Keys as NetabaseModelKeys<D, M>>::Primary, <M::Keys as NetabaseModelKeys<D, M>>::Relational>,
        &'db str,
    )>,

    pub subscription: Vec<(
        TablePermission<'txn, D::SubscriptionKeys, <M::Keys as NetabaseModelKeys<D, M>>::Primary>,
        &'db str,
    )>,
}

/// Read-only table handle.
///
/// Represents either a regular table or multimap table opened in read-only mode.
/// Used in read transactions and for read-only access within write transactions.
pub enum TableType<K, V>
where
    K: redb::Key + 'static,
    V: redb::Value + redb::Key + 'static,
{
    /// Regular key-value table (one value per key)
    Table(redb::ReadOnlyTable<K, V>),
    /// Multimap table (multiple values per key)
    MultimapTable(redb::ReadOnlyMultimapTable<K, V>),
}

/// Read-write table handle.
///
/// Represents either a regular table or multimap table opened in read-write mode.
/// Only available in write transactions.
pub enum ReadWriteTableType<'a, K, V>
where
    K: redb::Key + 'static,
    V: redb::Value + redb::Key + 'static,
{
    /// Regular key-value table (one value per key)
    Table(redb::Table<'a, K, V>),
    /// Multimap table (multiple values per key)
    MultimapTable(redb::MultimapTable<'a, K, V>),
}

/// Table with associated permissions.
///
/// Encodes whether a table is opened for reading only or reading and writing.
/// The type system ensures you can't write to a read-only table.
///
/// # Variants
///
/// - `ReadOnly`: Table opened in a read transaction
/// - `ReadWrite`: Table opened in a write transaction with write intent
/// - `ReadOnlyWrite`: Table opened in a write transaction but used read-only (optimization)
pub enum TablePermission<'a, K, V>
where
    K: redb::Key + 'static,
    V: redb::Value + redb::Key + 'static,
{
    /// Read-only access (cannot modify)
    ReadOnly(TableType<K, V>),
    /// Full read-write access
    ReadWrite(ReadWriteTableType<'a, K, V>),
    /// Opened as writable but used read-only (avoids unnecessary write locks)
    ReadOnlyWrite(ReadWriteTableType<'a, K, V>),
}
