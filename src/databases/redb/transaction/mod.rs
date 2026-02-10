#![allow(clippy::type_complexity)]
//! Transaction layer for redb database operations.
//!
//! This module provides the core transaction infrastructure for interacting with redb databases.
//! Transactions are the primary mechanism for reading and writing data, with full ACID guarantees.
//!
//! # Transaction Types
//!
//! - **Read transactions** ([`NetabaseRedbReadTransaction`]) - Read-only access to the database
//! - **Write transactions** ([`NetabaseRedbWriteTransaction`]) - Read/write access with commit/rollback support
//!
//! # Design Patterns
//!
//! ## Basic CRUD Pattern
//!
//! The simplest pattern for database operations:
//!
//! ```rust,no_run
//! use netabase_store::prelude::*;
//! use netabase_store::traits::database::store::NBStore;
//! use serde::{Serialize, Deserialize};
//!
//! #[netabase_macros::netabase_definition(MyApp)]
//! mod models {
//!     use super::*;
//!
//!     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
//!     pub struct MyModel {
//!         #[primary_key]
//!         pub id: String,
//!         pub data: String,
//!     }
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use models::*;
//!
//! let (store, _temp) = RedbStore::<MyApp>::new_temporary()?;
//!
//! // Write data
//! let txn = store.begin_write()?;
//! txn.create(&MyModel { id: MyModelID("1".into()), data: "test".into() })?;
//! txn.commit()?;
//!
//! // Read data
//! let txn = store.begin_read()?;
//! let result: Option<MyModel> = txn.read(&MyModelID("1".into()))?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Batch Operations Pattern
//!
//! For better performance when processing many records:
//!
//! ```rust,no_run
//! use netabase_store::prelude::*;
//! use netabase_store::traits::database::store::NBStore;
//! use serde::{Serialize, Deserialize};
//!
//! #[netabase_macros::netabase_definition(MyApp)]
//! mod models {
//!     use super::*;
//!
//!     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
//!     pub struct Item {
//!         #[primary_key]
//!         pub id: u64,
//!         pub value: String,
//!     }
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use models::*;
//!
//! let (store, _temp) = RedbStore::<MyApp>::new_temporary()?;
//! let txn = store.begin_write()?;
//!
//! for i in 0..10 {
//!     txn.create(&Item { id: ItemID(i), value: format!("item_{}", i) })?;
//! }
//!
//! txn.commit()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Query Pattern
//!
//! For listing and querying data with pagination and filtering:
//!
//! ```rust,no_run
//! use netabase_store::prelude::*;
//! use netabase_store::traits::database::store::NBStore;
//! use serde::{Serialize, Deserialize};
//!
//! #[netabase_macros::netabase_definition(MyApp)]
//! mod models {
//!     use super::*;
//!
//!     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
//!     pub struct Product {
//!         #[primary_key]
//!         pub sku: String,
//!         pub name: String,
//!         #[secondary_key]
//!         pub category: String,
//!     }
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use models::*;
//!
//! let (store, _temp) = RedbStore::<MyApp>::new_temporary()?;
//!
//! let txn = store.begin_read()?;
//! let results = txn.list::<Product>()?;
//! # Ok(())
//! # }
//! ```
//!
//! # Rules and Limitations
//!
//! 1. **Transaction Scope**: Transactions must be committed explicitly. Uncommitted write transactions are rolled back on drop.
//! 2. **Concurrency**: Multiple read transactions can run concurrently. Write transactions are exclusive.
//! 3. **Lifetime Management**: Table handles borrowed from transactions cannot outlive the transaction.
//! 4. **Error Handling**: All database operations return `NetabaseResult<T>`. Always check for errors before commit.
//! 5. **Performance**: Opening/closing tables has overhead. For batch operations, reuse transactions for multiple creates.
//!
//! # See Also
//!
//! - [`crud`] - CRUD operation implementations
//! - [`options`] - Configuration options for operations
//! - [`tables`] - Low-level table access
//! - [`wrappers`] - Transaction wrapper types
//! - [`core`] - Core types and trait bounds

pub mod core;
pub mod crud;
pub mod iterators;
pub mod options;
pub mod range_query;
pub mod tables;
pub mod value_wrappers;
pub mod wrappers;

use redb::{ReadableDatabase, TransactionError};
use strum::IntoDiscriminant;

use crate::{
    errors::{NetabaseError, NetabaseResult},
    relational::{ModelRelationPermissions, PermissionFlag, RelationPermission},
    traits::{
        registry::{
            definition::{NetabaseDefinition, redb_definition::RedbDefinition},
            models::{
                keys::{ModelKeyRange, NetabaseModelKeys, blob::NetabaseModelBlobKey},
                model::{
                    NetabaseModel,
                    redb_model::{RedbModelTableDefinitions, RedbNetbaseModel},
                },
            },
        },
    },
};

pub use self::core::{RedbTransaction, RedbTransactionInner, RedbTransactionType};
pub use self::crud::RedbModelCrud;
pub use self::iterators::{IteratorConfig, KeyIterator, ModelIterator, ModelIteratorExt};
pub use self::options::*;
pub use self::tables::{ModelOpenTables, ReadWriteTableType, TablePermission, TableType};
pub use self::wrappers::{NetabaseRedbReadTransaction, NetabaseRedbWriteTransaction};

// Re-export bound helpers for users who need them
pub use self::core::{DiscriminantBounds, RedbKeyBounds};

impl<'db, D: RedbDefinition> RedbTransaction<'db, D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    /// Create a new write transaction.
    pub fn new_write(db: &redb::Database) -> NetabaseResult<Self> {
        let write_txn = db
            .begin_write()
            .map_err(|e: TransactionError| NetabaseError::RedbTransactionError(e))?;
        let transaction = RedbTransactionType::Write(NetabaseRedbWriteTransaction::new(write_txn));

        Ok(RedbTransactionInner { transaction })
    }

    /// Create a new read-only transaction.
    pub fn new_read(db: &redb::Database) -> NetabaseResult<Self> {
        let read_txn = db
            .begin_read()
            .map_err(|e: TransactionError| NetabaseError::RedbTransactionError(e))?;
        let transaction = RedbTransactionType::Read(NetabaseRedbReadTransaction::new(read_txn));

        Ok(RedbTransactionInner { transaction })
    }

    /// Prepare model tables for batch operations.
    /// Returns a `ModelOpenTables` struct that holds open table handles.
    /// Use this with `RedbModelCrud` methods (like `create_entry`) for better performance in loops.
    pub fn prepare_model<'txn, M>(&'txn self) -> NetabaseResult<ModelOpenTables<'txn, 'db, D, M>>
    where
        'db: 'txn,
        M: RedbNetbaseModel<'db, D> + redb::Key,
        D::Discriminant: 'static + std::fmt::Debug,
        D: Clone + 'static,
        D::SubscriptionKeys: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key + 'static,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
    {
        // For batch operations, we default to ReadWrite permissions for the model being prepared
        let perms = ModelRelationPermissions {
            relationa_tree_access: &[RelationPermission(M::TREE_NAMES, PermissionFlag::ReadWrite)],
        };
        self.open_model_tables(M::table_definitions(), Some(perms))
    }

    /// Open tables for a specific model (concrete implementation)
    ///
    /// Opens all tables defined in M::TREE_NAMES for the given model.
    pub fn open_model_tables<'txn, 'data, 'perms, M>(
        &'txn self,
        definitions: RedbModelTableDefinitions<'data, M, D>,
        relational_permissions: Option<ModelRelationPermissions<'perms, 'static, D, M>>
    ) -> NetabaseResult<ModelOpenTables<'txn, 'data, D, M>>
    where
        M: RedbNetbaseModel<'data, D> + redb::Key,
        D::Discriminant: 'static + std::fmt::Debug,
        D: Clone,
        D::SubscriptionKeys: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key + 'static,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
    {
        let _table_definitions = definitions; // Keep for future use

        // Use static table names from M::TREE_NAMES
        let main_def = redb::TableDefinition::new(M::TREE_NAMES.main.table_name);

        match &self.transaction {
            RedbTransactionType::Read(read_txn) => {
                // For read transactions, open read-only tables
                let main_table = {
                    read_txn
                        .open_table(main_def)
                        .map(|table| TablePermission::ReadOnly(TableType::Table(table)))?
                };

                let secondary_tables: Result<Vec<_>, NetabaseError> = M::TREE_NAMES
                    .secondary
                    .iter()
                    .map(|disc_table| -> Result<_, NetabaseError> {
                        let def = redb::MultimapTableDefinition::new(disc_table.table_name);
                        read_txn.open_multimap_table(def).map(|table| {
                            (
                                TablePermission::ReadOnly(TableType::MultimapTable(table)),
                                disc_table.table_name,
                            )
                        })
                    })
                    .collect();

                let blob_tables: Result<Vec<_>, NetabaseError> = M::TREE_NAMES
                    .blob
                    .iter()
                    .map(|disc_table| -> Result<_, NetabaseError> {
                        let def = redb::MultimapTableDefinition::new(disc_table.table_name);
                        read_txn.open_multimap_table(def).map(|table| {
                            (
                                TablePermission::ReadOnly(TableType::MultimapTable(table)),
                                disc_table.table_name,
                            )
                        })
                    })
                    .collect();

                let relational_tables: Result<Vec<_>, NetabaseError> = M::TREE_NAMES
                    .relational
                    .iter()
                    .map(|disc_table| -> Result<_, NetabaseError> {
                        let def = redb::MultimapTableDefinition::new(disc_table.table_name);
                        read_txn.open_multimap_table(def).map(|table| {
                            (
                                TablePermission::ReadOnly(TableType::MultimapTable(table)),
                                disc_table.table_name,
                            )
                        })
                    })
                    .collect();

                let subscription_tables: Result<Vec<_>, NetabaseError> = match M::TREE_NAMES
                    .subscription
                {
                    Some(subs) => subs
                        .iter()
                        .map(|disc_table| -> Result<_, NetabaseError> {
                            let def = redb::MultimapTableDefinition::<
                                D::SubscriptionKeys,
                                crate::subscription_hash::ModelHash,
                            >::new(disc_table.table_name);
                            read_txn.open_multimap_table(def).map(|table| {
                                (
                                    TablePermission::ReadOnly(TableType::MultimapTable(table)),
                                    disc_table.table_name,
                                )
                            })
                        })
                        .collect(),
                    None => Ok(Vec::new()),
                };

                Ok(ModelOpenTables {
                    main: main_table,
                    secondary: secondary_tables?,
                    blob: blob_tables?,
                    relational: relational_tables?,
                    subscription: subscription_tables?,
                })
            }
            RedbTransactionType::Write(write_txn) => {
                use crate::relational::PermissionFlag;

                // For write transactions, open read-write tables
                let main_table = {
                    write_txn
                        .open_table(main_def)
                        .map(|table| TablePermission::ReadWrite(ReadWriteTableType::Table(table)))?
                };

                let secondary_tables: Result<Vec<_>, NetabaseError> = M::TREE_NAMES
                    .secondary
                    .iter()
                    .map(|disc_table| -> Result<_, NetabaseError> {
                        let def = redb::MultimapTableDefinition::new(disc_table.table_name);
                        write_txn.open_multimap_table(def).map(|table| {
                            (
                                TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(
                                    table,
                                )),
                                disc_table.table_name,
                            )
                        })
                    })
                    .collect();

                let blob_tables: Result<Vec<_>, NetabaseError> = M::TREE_NAMES
                    .blob
                    .iter()
                    .map(|disc_table| -> Result<_, NetabaseError> {
                        let def = redb::MultimapTableDefinition::new(disc_table.table_name);
                        write_txn.open_multimap_table(def).map(|table| {
                            (
                                TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(
                                    table,
                                )),
                                disc_table.table_name,
                            )
                        })
                    })
                    .collect();

                let relational_tables: Result<Vec<_>, NetabaseError> = M::TREE_NAMES
                    .relational
                    .iter()
                    .map(|disc_table| {
                        let permission_flag = if let Some(perms) = &relational_permissions {
                            perms
                                .relationa_tree_access
                                .iter()
                                .find(|p| {
                                    p.0.relational
                                        .iter()
                                        .any(|r| r.table_name == disc_table.table_name)
                                })
                                .map(|p| &p.1)
                                .unwrap_or(&PermissionFlag::ReadOnly)
                        } else {
                            &PermissionFlag::ReadOnly
                        };

                        let def = redb::MultimapTableDefinition::new(disc_table.table_name);

                        write_txn.open_multimap_table(def).map(|table| {
                            let table_perm = match permission_flag {
                                PermissionFlag::ReadWrite => TablePermission::ReadWrite(
                                    ReadWriteTableType::MultimapTable(table),
                                ),
                                PermissionFlag::ReadOnly => TablePermission::ReadOnlyWrite(
                                    ReadWriteTableType::MultimapTable(table),
                                ),
                            };
                            (table_perm, disc_table.table_name)
                        })
                    })
                    .collect();

                let subscription_tables: Result<Vec<_>, NetabaseError> = match M::TREE_NAMES
                    .subscription
                {
                    Some(subs) => subs
                        .iter()
                        .map(|disc_table| -> Result<_, NetabaseError> {
                            let def = redb::MultimapTableDefinition::<
                                D::SubscriptionKeys,
                                crate::subscription_hash::ModelHash,
                            >::new(disc_table.table_name);
                            write_txn.open_multimap_table(def).map(|table| {
                                (
                                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(
                                        table,
                                    )),
                                    disc_table.table_name,
                                )
                            })
                        })
                        .collect(),
                    None => Ok(Vec::new()),
                };

                Ok(ModelOpenTables {
                    main: main_table,
                    secondary: secondary_tables?,
                    blob: blob_tables?,
                    relational: relational_tables?,
                    subscription: subscription_tables?,
                })
            }
        }
    }

    /// Execute a function with the raw read transaction (limited scope)
    pub fn with_read_transaction<F, R>(&self, f: F) -> NetabaseResult<R>
    where
        F: FnOnce(&redb::ReadTransaction) -> NetabaseResult<R>,
    {
        match &self.transaction {
            RedbTransactionType::Read(read_txn) => f(&read_txn.inner),
            RedbTransactionType::Write(_) => Err(NetabaseError::Other),
        }
    }

    /// Execute a function with the raw write transaction (limited scope)
    pub fn with_write_transaction<F, R>(&self, f: F) -> NetabaseResult<R>
    where
        F: FnOnce(&redb::WriteTransaction) -> NetabaseResult<R>,
    {
        match &self.transaction {
            RedbTransactionType::Write(write_txn) => f(&write_txn.inner),
            RedbTransactionType::Read(_) => Err(NetabaseError::Other),
        }
    }

    /// Commit the transaction, persisting all changes to the database.
    ///
    /// For write transactions, this atomically applies all changes made during
    /// the transaction. For read transactions, this is a no-op (read transactions
    /// don't need to be committed).
    ///
    /// # Errors
    ///
    /// Returns an error if the commit fails (e.g., due to I/O errors).
    ///
    /// # Examples
    ///
    /// See [tests/comprehensive_functionality.rs](../../../tests/comprehensive_functionality.rs),
    /// [tests/integration_crud.rs](../../../tests/integration_crud.rs), and
    /// [tests/readme_examples.rs](../../../tests/readme_examples.rs) for working examples.
    ///
    /// ```rust
    /// use netabase_store::prelude::*;
    /// use netabase_store::traits::database::store::NBStore;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[netabase_macros::netabase_definition(MyApp)]
    /// mod models {
    ///     use super::*;
    ///
    ///     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
    ///     pub struct User {
    ///         #[primary_key]
    ///         pub id: String,
    ///         pub name: String,
    ///     }
    /// }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use models::*;
    /// let (store, _temp) = RedbStore::<MyApp>::new_temporary()?;
    /// let txn = store.begin_write()?;
    /// txn.create(&User { id: UserID("1".into()), name: "Alice".into() })?;
    /// txn.commit()?; // Persist the changes
    /// # Ok(())
    /// # }
    /// ```
    pub fn commit(self) -> NetabaseResult<()> {
        match self.transaction {
            RedbTransactionType::Write(write_txn) => write_txn.commit(),
            RedbTransactionType::Read(_) => {
                // Read transactions don't need to be committed
                Ok(())
            }
        }
    }

    /// Check if this is a write transaction.
    pub fn is_write(&self) -> bool {
        matches!(self.transaction, RedbTransactionType::Write(_))
    }

    /// Check if this is a read-only transaction.
    pub fn is_read(&self) -> bool {
        matches!(self.transaction, RedbTransactionType::Read(_))
    }

    // ========================================================================
    // High-Level CRUD Operations
    // ========================================================================

    /// Create a new record in the database.
    ///
    /// Inserts the model into the appropriate table(s), including primary key,
    /// secondary indexes, relational links, and blob storage.
    ///
    /// # Arguments
    ///
    /// * `model` - The model instance to insert
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The transaction is read-only
    /// - A record with the same primary key already exists
    /// - The database operation fails
    ///
    /// # Examples
    ///
    /// See [tests/comprehensive_functionality.rs](../../../tests/comprehensive_functionality.rs),
    /// [tests/integration_crud.rs](../../../tests/integration_crud.rs), and
    /// [tests/readme_examples.rs](../../../tests/readme_examples.rs) for working examples.
    ///
    /// ```rust
    /// use netabase_store::prelude::*;
    /// use netabase_store::traits::database::store::NBStore;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[netabase_macros::netabase_definition(MyApp)]
    /// mod models {
    ///     use super::*;
    ///
    ///     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
    ///     pub struct User {
    ///         #[primary_key]
    ///         pub id: u64,
    ///         pub name: String,
    ///     }
    /// }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use models::*;
    /// let (store, _temp) = RedbStore::<MyApp>::new_temporary()?;
    /// let txn = store.begin_write()?;
    /// let user = User { id: UserID(1), name: "Alice".to_string() };
    /// txn.create(&user)?;
    /// txn.commit()?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn create<'data: 'db, M>(&'db self, model: &'data M) -> NetabaseResult<()>
    where
        M: RedbModelCrud<'db, D> + RedbNetbaseModel<'db, D> + Clone,
        for<'a> M::TableV: redb::Value<SelfType<'a> = M>,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: Clone,
        <<M::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: 'static,
        D: 'static,
        D::SubscriptionKeys: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as redb::Value>::SelfType<'a>>,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: std::borrow::Borrow<<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem as redb::Value>::SelfType<'a>>,
    {
        let definitions = M::table_definitions();
        let perms = ModelRelationPermissions {
            relationa_tree_access: &[RelationPermission(M::TREE_NAMES, PermissionFlag::ReadWrite)],
        };
        let mut tables = self.open_model_tables(definitions, Some(perms))?;
        model.create_entry(&mut tables)
    }

    /// Create a record with selective subscription topics.
    ///
    /// This method allows you to control which subscription topics the model is added to.
    ///
    /// # Arguments
    ///
    /// * `model` - The model instance to insert
    /// * `subscription_topics` - Optional list of subscription topics:
    ///   - `None`: Subscribe to all model-level topics (default behavior, same as `create()`)
    ///   - `Some(vec![...])`: Subscribe only to the specified topics
    ///   - `Some(vec![])`: Subscribe to no topics
    ///
    /// # Type Parameters
    ///
    /// * `M` - The model type to create
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// // Subscribe only to Topic1
    /// let topics = vec![DefinitionSubscriptions::Topic1];
    /// txn.create_with_subscriptions(&user, Some(topics))?;
    ///
    /// // Subscribe to all topics (same as txn.create(&user))
    /// txn.create_with_subscriptions(&user, None)?;
    ///
    /// // Subscribe to no topics
    /// txn.create_with_subscriptions(&user, Some(vec![]))?;
    /// ```
    #[inline]
    pub fn create_with_subscriptions<'data: 'db, M>(
        &'db self,
        model: &'data M,
        subscription_topics: Option<Vec<D::SubscriptionKeys>>,
    ) -> NetabaseResult<()>
    where
        M: RedbModelCrud<'db, D> + RedbNetbaseModel<'db, D> + Clone,
        for<'a> M::TableV: redb::Value<SelfType<'a> = M>,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: Clone,
        <<M::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: 'static,
        D: 'static,
        D::SubscriptionKeys: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as redb::Value>::SelfType<'a>>,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: std::borrow::Borrow<<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem as redb::Value>::SelfType<'a>>,
    {
        let definitions = M::table_definitions();
        let perms = ModelRelationPermissions {
            relationa_tree_access: &[RelationPermission(M::TREE_NAMES, PermissionFlag::ReadWrite)],
        };
        let mut tables = self.open_model_tables(definitions, Some(perms))?;
        model.create_entry_with_subscriptions(&mut tables, subscription_topics)
    }

    /// Create a record with a pre-calculated hash.
    ///
    /// This is useful for immutable models or when the hash is already known,
    /// avoiding re-calculation during insertion.
    ///
    /// # Arguments
    ///
    /// * `model` - The model instance to insert
    /// * `hash` - The pre-calculated hash of the model
    #[inline]
    pub fn create_with_hash<'data: 'db, M>(
        &'db self,
        model: &'data M,
        hash: &crate::subscription_hash::ModelHash,
    ) -> NetabaseResult<()>
    where
        M: RedbModelCrud<'db, D> + RedbNetbaseModel<'db, D> + Clone,
        for<'a> M::TableV: redb::Value<SelfType<'a> = M>,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: Clone,
        <<M::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: 'static,
        D: 'static,
        D::SubscriptionKeys: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as redb::Value>::SelfType<'a>>,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: std::borrow::Borrow<<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem as redb::Value>::SelfType<'a>>,
    {
        let definitions = M::table_definitions();
        let perms = ModelRelationPermissions {
            relationa_tree_access: &[RelationPermission(M::TREE_NAMES, PermissionFlag::ReadWrite)],
        };
        let mut tables = self.open_model_tables(definitions, Some(perms))?;
        model.create_entry_with_hash(hash, &mut tables)
    }

    /// Read a record by its primary key.
    ///
    /// Returns `Some(model)` if a record with the given key exists,
    /// or `None` if no such record is found.
    ///
    /// # Arguments
    ///
    /// * `key` - The primary key of the record to read
    ///
    /// # Type Parameters
    ///
    /// * `M` - The model type to read
    ///
    /// # Examples
    ///
    /// See [tests/comprehensive_functionality.rs](../../../tests/comprehensive_functionality.rs),
    /// [tests/integration_crud.rs](../../../tests/integration_crud.rs), and
    /// [tests/readme_examples.rs](../../../tests/readme_examples.rs) for working examples.
    ///
    /// ```rust
    /// use netabase_store::prelude::*;
    /// use netabase_store::traits::database::store::NBStore;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[netabase_macros::netabase_definition(MyApp)]
    /// mod models {
    ///     use super::*;
    ///
    ///     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
    ///     pub struct User {
    ///         #[primary_key]
    ///         pub id: u64,
    ///         pub name: String,
    ///     }
    /// }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use models::*;
    /// let (store, _temp) = RedbStore::<MyApp>::new_temporary()?;
    /// let txn = store.begin_read()?;
    /// let user: Option<User> = txn.read::<User>(&UserID(1u64))?;
    /// if let Some(user) = user {
    ///     println!("Found user: {}", user.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn read<'data: 'db, M>(
        &'db self,
        key: &'data <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary,
    ) -> NetabaseResult<Option<M>>
    where
        M: RedbModelCrud<'db, D> + RedbNetbaseModel<'db, D> + Clone,
        for<'a> M::TableV: redb::Value<SelfType<'a> = M>,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: Clone,
        <<M::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: 'static,
        D: 'static,
        D::SubscriptionKeys: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as redb::Value>::SelfType<'a>>,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: std::borrow::Borrow<<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem as redb::Value>::SelfType<'a>>,
    {
        let definitions = M::table_definitions();
        let tables = self.open_model_tables(definitions, None)?;
        M::read_default(key, &tables)
    }

    /// Read a model by owned primary key.
    ///
    /// This is a convenience method that takes an owned key instead of a reference,
    /// useful when the key is created locally (e.g., during hydration).
    ///
    /// # Arguments
    ///
    /// * `key` - The owned primary key value
    ///
    /// # Returns
    ///
    /// `Some(model)` if found, `None` if not found.
    #[inline]
    pub fn read_by_key<M>(
        &self,
        key: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary,
    ) -> NetabaseResult<Option<M>>
    where
        M: RedbModelCrud<'db, D> + RedbNetbaseModel<'db, D> + Clone,
        for<'a> M::TableV: redb::Value<SelfType<'a> = M>,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: Clone,
        <<M::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: 'static,
        D: 'static,
        D::SubscriptionKeys: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as redb::Value>::SelfType<'a>>,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: std::borrow::Borrow<<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem as redb::Value>::SelfType<'a>>,
    {
        let definitions = M::table_definitions();
        let tables = self.open_model_tables(definitions, None)?;
        M::read_default(&key, &tables)
    }

    /// Query records by secondary key value.
    ///
    /// Returns a list of models that have the specified secondary key value.
    /// Secondary keys create indexed lookups for fields marked with `#[secondary_key]`.
    ///
    /// # Arguments
    ///
    /// * `secondary_key` - The secondary key value to search for
    ///
    /// # Returns
    ///
    /// A vector of models matching the secondary key value. Returns an empty vector
    /// if no matches are found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use netabase_store::prelude::*;
    /// use netabase_store::traits::database::store::NBStore;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[netabase_macros::netabase_definition(MyApp)]
    /// mod models {
    ///     use super::*;
    ///
    ///     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
    ///     pub struct User {
    ///         #[primary_key]
    ///         pub id: String,
    ///         pub name: String,
    ///         #[secondary_key]
    ///         pub email: String,
    ///     }
    /// }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use models::*;
    /// let (store, _temp) = RedbStore::<MyApp>::new_temporary()?;
    ///
    /// // Create some users
    /// let txn = store.begin_write()?;
    /// txn.create(&User {
    ///     id: UserID("1".into()),
    ///     name: "Alice".into(),
    ///     email: "alice@example.com".into()
    /// })?;
    /// txn.create(&User {
    ///     id: UserID("2".into()),
    ///     name: "Bob".into(),
    ///     email: "bob@example.com".into()
    /// })?;
    /// txn.commit()?;
    ///
    /// // Query by email (secondary key)
    /// let txn = store.begin_read()?;
    /// let users = txn.query_by_secondary_key::<User>(
    ///     &UserSecondaryKeys::Email(UserEmail("alice@example.com".into()))
    /// )?;
    /// assert_eq!(users.len(), 1);
    /// assert_eq!(users[0].name, "Alice");
    /// # Ok(())
    /// # }
    /// ```
    pub fn query_by_secondary_key<'data, M>(
        &self,
        secondary_key: &'data <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary,
    ) -> NetabaseResult<Vec<M>>
    where
        M: RedbModelCrud<'db, D> + RedbNetbaseModel<'db, D> + Clone,
        for<'a> M::TableV: redb::Value<SelfType<'a> = M>,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: Clone,
        <<M::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
        for<'v> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Value<SelfType<'v> = <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary>,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary as redb::Value>::SelfType<'a>>,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: 'static,
        D: 'static,
        D::SubscriptionKeys: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as redb::Value>::SelfType<'a>>,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: std::borrow::Borrow<<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem as redb::Value>::SelfType<'a>>,
    {
        let definitions = M::table_definitions();
        let tables = self.open_model_tables(definitions, None)?;

        // Get primary keys matching the secondary key
        let primary_keys = M::query_by_secondary_key(secondary_key, &tables)?;

        // Load the full models
        let mut results = Vec::with_capacity(primary_keys.len());
        for pk in primary_keys {
            if let Some(model) = M::read_default(&pk, &tables)? {
                results.push(model);
            }
        }

        Ok(results)
    }

    /// Query records by subscription topic.
    ///
    /// Returns a list of models that are subscribed to the specified topic.
    /// Subscriptions are defined by including a subscription field in the model.
    ///
    /// # Arguments
    ///
    /// * `subscription_key` - The subscription topic to search for
    ///
    /// # Returns
    ///
    /// A vector of models subscribed to the topic. Returns an empty vector
    /// if no models are subscribed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use netabase_store::prelude::*;
    /// use netabase_store::traits::database::store::NBStore;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[netabase_macros::netabase_definition(MyApp, subscriptions(Topic1, Topic2))]
    /// mod models {
    ///     use super::*;
    ///
    ///     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
    ///     #[subscribe(Topic1)]  // Model-level subscription
    ///     pub struct User {
    ///         #[primary_key]
    ///         pub id: String,
    ///         pub name: String,
    ///     }
    /// }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use models::*;
    /// let (store, _temp) = RedbStore::<MyApp>::new_temporary()?;
    ///
    /// // Create users - they automatically subscribe to Topic1 (trait-level)
    /// let txn = store.begin_write()?;
    /// txn.create(&User {
    ///     id: UserID("1".into()),
    ///     name: "Alice".into(),
    /// })?;
    /// txn.create(&User {
    ///     id: UserID("2".into()),
    ///     name: "Bob".into(),
    /// })?;
    /// txn.commit()?;
    ///
    /// // Query by subscription - returns all Users with hashes
    /// let txn = store.begin_read()?;
    /// let results = txn.query_by_subscription::<User, _>(
    ///     &MyAppSubscriptions::Topic1
    /// )?;
    /// assert_eq!(results.len(), 2);
    /// // Hashes are present
    /// assert_eq!(results[0].as_bytes().len(), 32);
    /// # Ok(())
    /// # }
    /// ```
    /// Query models by subscription with hashes.
    ///
    /// Returns model hashes subscribed to a topic.
    /// Hashes enable efficient change detection and merkle tree construction.
    pub fn query_by_subscription<'data: 'db, M, S>(
        &'db self,
        subscription_key: &'data S,
    ) -> NetabaseResult<Vec<crate::subscription_hash::ModelHash>>
    where
        M: RedbModelCrud<'db, D> + RedbNetbaseModel<'db, D> + Clone,
        S: Into<D::SubscriptionKeys> + Clone,
        for<'a> M::TableV: redb::Value<SelfType<'a> = M>,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: Clone,
        <<M::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
        for<'v> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Value<SelfType<'v> = <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary>,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: 'static,
        D: 'static,
        D::SubscriptionKeys: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as redb::Value>::SelfType<'a>>,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: std::borrow::Borrow<<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem as redb::Value>::SelfType<'a>>,
    {
        let definitions = M::table_definitions();
        let tables = self.open_model_tables(definitions, None)?;

        // Get hashes directly
        M::query_by_subscription(subscription_key, &tables)
    }

    /// Query relations associated with a model.
    ///
    /// Returns a list of relational keys associated with the given primary key.
    ///
    /// # Arguments
    ///
    /// * `primary_key` - The primary key of the model
    ///
    /// # Returns
    ///
    /// A vector of relational keys.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use netabase_store::prelude::*;
    /// use netabase_store::traits::database::store::NBStore;
    ///
    /// // Find all categories a user belongs to
    /// let txn = store.begin_read()?;
    /// let categories = txn.query_relations::<User>(
    ///     &UserID("alice".into())
    /// )?;
    /// ```
    pub fn query_relations<'data, M>(
        &self,
        primary_key: &'data <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary,
    ) -> NetabaseResult<Vec<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational>>
    where
        M: RedbModelCrud<'db, D> + RedbNetbaseModel<'db, D> + Clone,
        for<'a> M::TableV: redb::Value<SelfType<'a> = M>,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: Clone + PartialEq,
        <<M::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
        for<'v> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Value<SelfType<'v> = <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary>,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational as redb::Value>::SelfType<'a>>,
        for<'v> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational as redb::Value>::SelfType<'v>: PartialEq<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational>,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: 'static,
        D: 'static,
        D::SubscriptionKeys: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as redb::Value>::SelfType<'a>>,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: std::borrow::Borrow<<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem as redb::Value>::SelfType<'a>>,
        // Add Value bound for Relational Key to support to_owned/value()
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Value<SelfType<'a> = <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational>,
    {
        let definitions = M::table_definitions();
        let tables = self.open_model_tables(definitions, None)?;

        // Get relational keys
        let guards = M::query_relations(primary_key, &tables)?;

        // Convert to owned
        Ok(guards.into_iter().map(|g| g.value()).collect())
    }

    /// Query relations of a specific type associated with a model.
    ///
    /// Returns a list of relational keys of the specified type associated with the given primary key.
    pub fn query_relations_by_type<'data, M>(
        &self,
        primary_key: &'data <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary,
        relation_type: <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant,
    ) -> NetabaseResult<Vec<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational>>
    where
        M: RedbModelCrud<'db, D> + RedbNetbaseModel<'db, D> + Clone,
        for<'a> M::TableV: redb::Value<SelfType<'a> = M>,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: Clone + PartialEq,
        <<M::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static + PartialEq,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
        for<'v> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Value<SelfType<'v> = <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary>,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational as redb::Value>::SelfType<'a>>,
        for<'v> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational as redb::Value>::SelfType<'v>: PartialEq<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational>,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: 'static,
        D: 'static,
        D::SubscriptionKeys: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as redb::Value>::SelfType<'a>>,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: std::borrow::Borrow<<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem as redb::Value>::SelfType<'a>>,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Value<SelfType<'a> = <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational>,
    {
        let definitions = M::table_definitions();
        let tables = self.open_model_tables(definitions, None)?;

        let guards = M::query_relations_by_type(primary_key, relation_type, &tables)?;

        Ok(guards.into_iter().map(|g| g.value()).collect::<Vec<_>>())
    }

    /// Update an existing record in the database.
    ///
    /// Replaces the record with the matching primary key with the new values.
    /// All indexes are updated accordingly.
    ///
    /// # Arguments
    ///
    /// * `model` - The model instance with updated values
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The transaction is read-only
    /// - The database operation fails
    ///
    /// # Examples
    ///
    /// See [tests/comprehensive_functionality.rs](../../../tests/comprehensive_functionality.rs),
    /// [tests/integration_crud.rs](../../../tests/integration_crud.rs), and
    /// [tests/readme_examples.rs](../../../tests/readme_examples.rs) for working examples.
    ///
    /// ```rust
    /// use netabase_store::prelude::*;
    /// use netabase_store::traits::database::store::NBStore;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[netabase_macros::netabase_definition(MyApp)]
    /// mod models {
    ///     use super::*;
    ///
    ///     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
    ///     pub struct User {
    ///         #[primary_key]
    ///         pub id: u64,
    ///         pub name: String,
    ///     }
    /// }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use models::*;
    /// let (store, _temp) = RedbStore::<MyApp>::new_temporary()?;
    /// let txn = store.begin_write()?;
    /// txn.create(&User { id: UserID(1), name: "Alice".into() })?;
    /// let mut user = txn.read::<User>(&UserID(1u64))?.expect("user exists");
    /// user.name = "Bob".to_string();
    /// txn.update(&user)?;
    /// txn.commit()?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn update<'data: 'db, M>(&'db self, model: &'data M) -> NetabaseResult<()>
    where
        M: RedbModelCrud<'db, D> + RedbNetbaseModel<'db, D> + Clone,
        for<'a> M::TableV: redb::Value<SelfType<'a> = M>,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: Clone,
        <<M::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: 'static,
        D: 'static,
        D::SubscriptionKeys: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as redb::Value>::SelfType<'a>>,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: std::borrow::Borrow<<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem as redb::Value>::SelfType<'a>>,
    {
        let definitions = M::table_definitions();
        let perms = ModelRelationPermissions {
            relationa_tree_access: &[RelationPermission(M::TREE_NAMES, PermissionFlag::ReadWrite)],
        };
        let mut tables = self.open_model_tables(definitions, Some(perms))?;
        model.update_entry(&mut tables)
    }

    /// Delete a record by its primary key.
    ///
    /// Removes the record and all associated index entries.
    ///
    /// # Arguments
    ///
    /// * `key` - The primary key of the record to delete
    ///
    /// # Type Parameters
    ///
    /// * `M` - The model type to delete
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The transaction is read-only
    /// - The database operation fails
    ///
    /// # Examples
    ///
    /// See [tests/comprehensive_functionality.rs](../../../tests/comprehensive_functionality.rs),
    /// [tests/integration_crud.rs](../../../tests/integration_crud.rs), and
    /// [tests/readme_examples.rs](../../../tests/readme_examples.rs) for working examples.
    ///
    /// ```rust
    /// use netabase_store::prelude::*;
    /// use netabase_store::traits::database::store::NBStore;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[netabase_macros::netabase_definition(MyApp)]
    /// mod models {
    ///     use super::*;
    ///
    ///     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
    ///     pub struct User {
    ///         #[primary_key]
    ///         pub id: u64,
    ///         pub name: String,
    ///     }
    /// }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use models::*;
    /// let (store, _temp) = RedbStore::<MyApp>::new_temporary()?;
    /// let txn = store.begin_write()?;
    /// txn.delete::<User>(&UserID(1u64))?;
    /// txn.commit()?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn delete<'data, M>(
        &'db self,
        key: &'data <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary,
    ) -> NetabaseResult<()>
    where
        M: RedbModelCrud<'db, D> + RedbNetbaseModel<'db, D> + Clone,
        for<'a> M::TableV: redb::Value<SelfType<'a> = M>,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: Clone,
        <<M::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: 'static,
        D: 'static,
        D::SubscriptionKeys: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as redb::Value>::SelfType<'a>>,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: std::borrow::Borrow<<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem as redb::Value>::SelfType<'a>>,
    {
        let definitions = M::table_definitions();
        let perms = ModelRelationPermissions {
            relationa_tree_access: &[RelationPermission(M::TREE_NAMES, PermissionFlag::ReadWrite)],
        };
        let mut tables = self.open_model_tables(definitions, Some(perms))?;
        M::delete_entry(key, &mut tables)
    }

    // ========================================================================
    // List and Iterator Operations
    // ========================================================================

    /// List all records of a model type.
    ///
    /// Returns a vector of all model instances. For large datasets, consider
    /// using [`iter`](Self::iter) instead for streaming access.
    ///
    /// # Type Parameters
    ///
    /// * `M` - The model type to list
    ///
    /// # Examples
    ///
    /// ```rust
    /// use netabase_store::prelude::*;
    /// use netabase_store::traits::database::store::NBStore;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[netabase_macros::netabase_definition(MyApp)]
    /// mod models {
    ///     use super::*;
    ///
    ///     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
    ///     pub struct User {
    ///         #[primary_key]
    ///         pub id: u64,
    ///         pub name: String,
    ///     }
    /// }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use models::*;
    /// let (store, _temp) = RedbStore::<MyApp>::new_temporary()?;
    ///
    /// // Create some users
    /// let txn = store.begin_write()?;
    /// txn.create(&User { id: UserID(1), name: "Alice".into() })?;
    /// txn.create(&User { id: UserID(2), name: "Bob".into() })?;
    /// txn.commit()?;
    ///
    /// // List all users
    /// let txn = store.begin_read()?;
    /// let users: Vec<User> = txn.list()?;
    /// assert_eq!(users.len(), 2);
    /// # Ok(())
    /// # }
    /// ```
    pub fn list<M>(&'db self) -> NetabaseResult<Vec<M>>
    where
        M: RedbModelCrud<'db, D> + RedbNetbaseModel<'db, D> + Clone,
        for<'a> M::TableV: redb::Value<SelfType<'a> = M>,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: Clone,
        <<M::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: 'static,
        D: 'static,
        D::SubscriptionKeys: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as redb::Value>::SelfType<'a>>,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: std::borrow::Borrow<<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem as redb::Value>::SelfType<'a>>,
    {
        let definitions = M::table_definitions();
        let tables = self.open_model_tables(definitions, None)?;
        M::list_default(&tables)
    }

    /// List records with pagination options.
    ///
    /// Returns a vector of model instances with limit and offset support.
    ///
    /// # Arguments
    ///
    /// * `options` - Pagination and filtering options
    ///
    /// # Examples
    ///
    /// ```rust
    /// use netabase_store::prelude::*;
    /// use netabase_store::traits::database::store::NBStore;
    /// use netabase_store::databases::redb::transaction::CrudOptions;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[netabase_macros::netabase_definition(MyApp)]
    /// mod models {
    ///     use super::*;
    ///
    ///     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
    ///     pub struct User {
    ///         #[primary_key]
    ///         pub id: u64,
    ///         pub name: String,
    ///     }
    /// }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use models::*;
    /// let (store, _temp) = RedbStore::<MyApp>::new_temporary()?;
    ///
    /// // Create users
    /// let txn = store.begin_write()?;
    /// for i in 0..10 {
    ///     txn.create(&User { id: UserID(i), name: format!("User {}", i) })?;
    /// }
    /// txn.commit()?;
    ///
    /// // List with pagination
    /// let txn = store.begin_read()?;
    /// let options = CrudOptions::default().with_limit(5).with_offset(2);
    /// let page: Vec<User> = txn.list_with_options(options)?;
    /// assert_eq!(page.len(), 5);
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_with_options<M>(&'db self, options: CrudOptions) -> NetabaseResult<Vec<M>>
    where
        M: RedbModelCrud<'db, D> + RedbNetbaseModel<'db, D> + Clone,
        for<'a> M::TableV: redb::Value<SelfType<'a> = M>,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: Clone,
        <<M::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: 'static,
        D: 'static,
        D::SubscriptionKeys: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as redb::Value>::SelfType<'a>>,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: std::borrow::Borrow<<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem as redb::Value>::SelfType<'a>>,
    {
        let definitions = M::table_definitions();
        let tables = self.open_model_tables(definitions, None)?;
        Ok(M::list_entries(&tables, options)?
            .into_iter()
            .map(|g| g.value())
            .collect())
    }

    /// List records within a primary key range.
    ///
    /// Efficiently scans only the specified range of keys. This is much faster
    /// than listing all records and filtering.
    ///
    /// # Arguments
    ///
    /// * `range` - The range of primary keys to include
    ///
    /// # Examples
    ///
    /// ```rust
    /// use netabase_store::prelude::*;
    /// use netabase_store::traits::database::store::NBStore;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[netabase_macros::netabase_definition(MyApp)]
    /// mod models {
    ///     use super::*;
    ///
    ///     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
    ///     pub struct User {
    ///         #[primary_key]
    ///         pub id: u64,
    ///         pub name: String,
    ///     }
    /// }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use models::*;
    /// let (store, _temp) = RedbStore::<MyApp>::new_temporary()?;
    ///
    /// // Create users with sequential IDs
    /// let txn = store.begin_write()?;
    /// for i in 0..100 {
    ///     txn.create(&User { id: UserID(i), name: format!("User {}", i) })?;
    /// }
    /// txn.commit()?;
    ///
    /// // List only users with IDs 10-19
    /// let txn = store.begin_read()?;
    /// let users: Vec<User> = txn.list_range(UserID(10)..UserID(20))?;
    /// assert_eq!(users.len(), 10);
    /// assert_eq!(users[0].id.0, 10);
    /// # Ok(())
    /// # }
    /// ```
    pub fn list_range<M, R>(&'db self, range: R) -> NetabaseResult<Vec<M>>
    where
        M: RedbModelCrud<'db, D> + RedbNetbaseModel<'db, D> + Clone,
        R: std::ops::RangeBounds<<M::Keys as NetabaseModelKeys<D, M>>::Primary> + Clone,
        for<'a> M::TableV: redb::Value<SelfType<'a> = M>,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: Clone,
        <<M::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: 'static,
        D: 'static,
        D::SubscriptionKeys: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as redb::Value>::SelfType<'a>>,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: std::borrow::Borrow<<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem as redb::Value>::SelfType<'a>>,
    {
        let definitions = M::table_definitions();
        let tables = self.open_model_tables(definitions, None)?;
        Ok(M::list_range(&tables, range, CrudOptions::default())?
            .into_iter()
            .map(|g| g.value())
            .collect())
    }

    /// Lists entries for a model using a `ModelKeyRange`, which can express
    /// intersecting constraints across primary and secondary keys.
    pub fn list_with_key_ranges<M>(
        &'db self,
        ranges: &ModelKeyRange<D, M>,
    ) -> NetabaseResult<Vec<M>>
    where
        M: RedbModelCrud<'db, D> + RedbNetbaseModel<'db, D> + Clone,
        for<'a> M::TableV: redb::Value<SelfType<'a> = M>,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: Clone + Eq + std::hash::Hash + Ord,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: Clone,
        <<M::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Value<SelfType<'a> = <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary>,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: 'static,
        D: 'static,
        D::SubscriptionKeys: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as redb::Value>::SelfType<'a>>,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: std::borrow::Borrow<<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem as redb::Value>::SelfType<'a>>,
    {
        let definitions = M::table_definitions();
        let tables = self.open_model_tables(definitions, None)?;
        Ok(M::list_with_key_ranges(&tables, ranges, CrudOptions::default())?
            .into_iter()
            .map(|g| g.value())
            .collect())
    }

    /// Get an iterator over all records of a model type.
    ///
    /// This method provides streaming access to records, which is more memory-efficient
    /// than [`list`](Self::list) for large datasets. The iterator yields `Result<M>` items.
    ///
    /// # Memory Efficiency
    ///
    /// Unlike `list()` which loads all records into memory, `iter()` processes records
    /// one at a time. This is ideal for:
    /// - Large datasets that don't fit in memory
    /// - Early termination (stop iteration when you find what you need)
    /// - Batch processing with constant memory usage
    ///
    /// # Type Parameters
    ///
    /// * `M` - The model type to iterate
    ///
    /// # Examples
    ///
    /// ```rust
    /// use netabase_store::prelude::*;
    /// use netabase_store::traits::database::store::NBStore;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[netabase_macros::netabase_definition(MyApp)]
    /// mod models {
    ///     use super::*;
    ///
    ///     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
    ///     pub struct User {
    ///         #[primary_key]
    ///         pub id: u64,
    ///         pub name: String,
    ///     }
    /// }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use models::*;
    /// let (store, _temp) = RedbStore::<MyApp>::new_temporary()?;
    ///
    /// // Create users
    /// let txn = store.begin_write()?;
    /// for i in 0..5 {
    ///     txn.create(&User { id: UserID(i), name: format!("User {}", i) })?;
    /// }
    /// txn.commit()?;
    ///
    /// // Iterate over users
    /// let txn = store.begin_read()?;
    /// let mut count = 0;
    /// for user_result in txn.iter::<User>()? {
    ///     let user = user_result?;
    ///     println!("Found user: {}", user.name);
    ///     count += 1;
    /// }
    /// assert_eq!(count, 5);
    /// # Ok(())
    /// # }
    /// ```
    pub fn iter<M>(&'db self) -> NetabaseResult<iterators::ModelIterator<'db, 'db, M>>
    where
        M: RedbModelCrud<'db, D> + RedbNetbaseModel<'db, D> + Clone,
        for<'a> M::TableV: redb::Value<SelfType<'a> = M>,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: Clone,
        <<M::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: 'static,
        D: 'static,
        D::SubscriptionKeys: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as redb::Value>::SelfType<'a>>,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: std::borrow::Borrow<<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem as redb::Value>::SelfType<'a>>,
    {
        // Note: Due to redb's lifetime constraints, we collect to Vec internally.
        // The ModelIterator wrapper provides a streaming interface for consistency.
        let models = self.list::<M>()?;
        Ok(iterators::ModelIterator::from_vec(models))
    }

    /// Count the number of records of a model type.
    ///
    /// This is more efficient than `list().len()` as it doesn't load any data.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use netabase_store::prelude::*;
    /// use netabase_store::traits::database::store::NBStore;
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[netabase_macros::netabase_definition(MyApp)]
    /// mod models {
    ///     use super::*;
    ///
    ///     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
    ///     pub struct User {
    ///         #[primary_key]
    ///         pub id: u64,
    ///         pub name: String,
    ///     }
    /// }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use models::*;
    /// let (store, _temp) = RedbStore::<MyApp>::new_temporary()?;
    ///
    /// let txn = store.begin_write()?;
    /// txn.create(&User { id: UserID(1), name: "Alice".into() })?;
    /// txn.create(&User { id: UserID(2), name: "Bob".into() })?;
    /// txn.commit()?;
    ///
    /// let txn = store.begin_read()?;
    /// let count = txn.count::<User>()?;
    /// assert_eq!(count, 2);
    /// # Ok(())
    /// # }
    /// ```
    pub fn count<M>(&'db self) -> NetabaseResult<u64>
    where
        M: RedbModelCrud<'db, D> + RedbNetbaseModel<'db, D> + Clone,
        for<'a> M::TableV: redb::Value<SelfType<'a> = M>,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: Clone,
        <<M::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: 'static,
        D: 'static,
        D::SubscriptionKeys: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as redb::Value>::SelfType<'a>>,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: std::borrow::Borrow<<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem as redb::Value>::SelfType<'a>>,
    {
        let definitions = M::table_definitions();
        let tables = self.open_model_tables(definitions, None)?;
        M::count_entries(&tables)
    }

    // ========================================================================
    // Legacy method aliases (kept for backwards compatibility)
    // ========================================================================

    /// Deprecated: Use `create` instead.
    #[inline]
    #[deprecated(since = "0.2.0", note = "Use `create` instead")]
    pub fn create_redb<'data: 'db, M>(&'db self, model: &'data M) -> NetabaseResult<()>
    where
        M: RedbModelCrud<'db, D> + RedbNetbaseModel<'db, D> + Clone,
        for<'a> M::TableV: redb::Value<SelfType<'a> = M>,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: Clone,
        <<M::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
    // Add Subscription bounds
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: 'static,
        D: 'static,
        D::SubscriptionKeys: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as redb::Value>::SelfType<'a>>,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: std::borrow::Borrow<<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem as redb::Value>::SelfType<'a>>,
    {
        let definitions = M::table_definitions();
        let perms = ModelRelationPermissions {
            relationa_tree_access: &[RelationPermission(M::TREE_NAMES, PermissionFlag::ReadWrite)],
        };
        let mut tables = self.open_model_tables(definitions, Some(perms))?;

        model.create_entry(&mut tables)
    }

    pub fn read_redb<'data: 'db, M>(&'db self, key: &'data <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary) -> NetabaseResult<Option<M>>
    where
        M: RedbModelCrud<'db, D> + RedbNetbaseModel<'db, D> + Clone,
        for<'a> M::TableV: redb::Value<SelfType<'a> = M>,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: Clone,
        <<M::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: 'static,
        D: 'static,
        D::SubscriptionKeys: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as redb::Value>::SelfType<'a>>,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: std::borrow::Borrow<<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem as redb::Value>::SelfType<'a>>,
    {
        let definitions = M::table_definitions();
        let tables = self.open_model_tables(definitions, None)?;

        M::read_default(key, &tables)
    }

    pub fn update_redb<'data: 'db, M>(&'db self, model: &'data M) -> NetabaseResult<()>
    where
        M: RedbModelCrud<'db,  D> + RedbNetbaseModel<'db, D> + Clone,
        for<'a> M::TableV: redb::Value<SelfType<'a> = M>,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: Clone,
        <<M::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: 'static,
        D: 'static,
        D::SubscriptionKeys: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as redb::Value>::SelfType<'a>>,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: std::borrow::Borrow<<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem as redb::Value>::SelfType<'a>>,
    {
        let definitions = M::table_definitions();
        let perms = ModelRelationPermissions {
            relationa_tree_access: &[RelationPermission(M::TREE_NAMES, PermissionFlag::ReadWrite)],
        };
        let mut tables = self.open_model_tables(definitions, Some(perms))?;

        model.update_entry(&mut tables)
    }

    pub fn delete_redb<'data, M>(&'db self, key: &'data <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary) -> NetabaseResult<()>
    where
        M: RedbModelCrud<'db, D> + RedbNetbaseModel<'db, D> + Clone,
        for<'a> M::TableV: redb::Value<SelfType<'a> = M>,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: Clone,
        <<M::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: 'static,
        D: 'static,
        D::SubscriptionKeys: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as redb::Value>::SelfType<'a>>,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: std::borrow::Borrow<<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem as redb::Value>::SelfType<'a>>,
    {
        let definitions = M::table_definitions();
        let perms = ModelRelationPermissions {
            relationa_tree_access: &[RelationPermission(M::TREE_NAMES, PermissionFlag::ReadWrite)],
        };
        let mut tables = self.open_model_tables(definitions, Some(perms))?;

        M::delete_entry(key, &mut tables)
    }
}
