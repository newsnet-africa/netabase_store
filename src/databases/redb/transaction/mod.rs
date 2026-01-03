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
//! ```rust
//! use netabase_store::prelude::*;
//! use netabase_store::traits::database::store::NBStore;
//! use serde::{Serialize, Deserialize};
//!
//! #[netabase_macros::netabase_definition(MyApp)]
//! mod models {
//!     use super::*;
//!
//!     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq)]
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
//! txn.create(&MyModel { id: "1".into(), data: "test".into() })?;
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
//! ```rust
//! use netabase_store::prelude::*;
//! use netabase_store::traits::database::store::NBStore;
//! use serde::{Serialize, Deserialize};
//!
//! #[netabase_macros::netabase_definition(MyApp)]
//! mod models {
//!     use super::*;
//!
//!     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq)]
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
//!     txn.create(&Item { id: i, value: format!("item_{}", i) })?;
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
//! ```rust
//! use netabase_store::prelude::*;
//! use netabase_store::traits::database::store::NBStore;
//! use serde::{Serialize, Deserialize};
//!
//! #[netabase_macros::netabase_definition(MyApp)]
//! mod models {
//!     use super::*;
//!
//!     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq)]
//!     pub struct Product {
//!         #[primary_key]
//!         pub sku: String,
//!         pub name: String,
//!     }
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use models::*;
//!
//! let (store, _temp) = RedbStore::<MyApp>::new_temporary()?;
//!
//! let txn = store.begin_read()?;
//! let config = QueryConfig::new()
//!     .with_limit(10);
//!
//! let results = txn.list_with_config::<Product>(config)?;
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

pub mod crud;
pub mod options;
pub mod tables;
pub mod wrappers;

use redb::{ReadableDatabase, TransactionError};
use strum::IntoDiscriminant;

use crate::{
    errors::{NetabaseError, NetabaseResult},
    relational::{ModelRelationPermissions, PermissionFlag, RelationPermission},
    traits::{
        database::transaction::NBTransaction,
        registery::{
            definition::{NetabaseDefinition, redb_definition::RedbDefinition},
            models::{
                keys::{NetabaseModelKeys, blob::NetabaseModelBlobKey},
                model::{
                    NetabaseModel,
                    redb_model::{RedbModelTableDefinitions, RedbNetbaseModel},
                },
            },
        },
    },
};

pub use self::crud::RedbModelCrud;
pub use self::options::*;
pub use self::tables::{ModelOpenTables, ReadWriteTableType, TablePermission, TableType};
pub use self::wrappers::{NetabaseRedbReadTransaction, NetabaseRedbWriteTransaction};

pub struct RedbTransactionInner<'txn, D: RedbDefinition>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    transaction: RedbTransactionType<'txn, D>,
}

pub enum RedbTransactionType<'txn, D: RedbDefinition>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    Read(NetabaseRedbReadTransaction<'txn, D>),
    Write(NetabaseRedbWriteTransaction<'txn, D>),
}

pub type RedbTransaction<'db, D> = RedbTransactionInner<'db, D>;

impl<'db, D: RedbDefinition> RedbTransaction<'db, D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    /// Create a new write transaction.
    pub fn new_write(db: &redb::Database) -> NetabaseResult<Self> {
        let write_txn = db
            .begin_write()
            .map_err(|e: TransactionError| NetabaseError::RedbTransactionError(e.into()))?;
        let transaction = RedbTransactionType::Write(NetabaseRedbWriteTransaction::new(write_txn));

        Ok(RedbTransactionInner { transaction })
    }

    /// Create a new read-only transaction.
    pub fn new_read(db: &redb::Database) -> NetabaseResult<Self> {
        let read_txn = db
            .begin_read()
            .map_err(|e: TransactionError| NetabaseError::RedbTransactionError(e.into()))?;
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

                let subscription_tables: Result<Vec<_>, NetabaseError> =
                    match M::TREE_NAMES.subscription {
                        Some(subs) => subs
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

                let subscription_tables: Result<Vec<_>, NetabaseError> =
                    match M::TREE_NAMES.subscription {
                        Some(subs) => subs
                            .iter()
                            .map(|disc_table| -> Result<_, NetabaseError> {
                                let def = redb::MultimapTableDefinition::new(disc_table.table_name);
                                write_txn.open_multimap_table(def).map(|table| {
                                    (
                                        TablePermission::ReadWrite(
                                            ReadWriteTableType::MultimapTable(table),
                                        ),
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
            RedbTransactionType::Write(_) => return Err(NetabaseError::Other),
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
    /// #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq)]
    /// struct User {
    ///     #[primary_key]
    ///     id: String,
    ///     name: String,
    /// }
    ///
    /// #[netabase_macros::netabase_definition(MyApp)]
    /// mod models { use super::*; }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let (store, _temp) = RedbStore::<MyApp>::new_temporary()?;
    /// let txn = store.begin_write()?;
    /// txn.create(&User { id: "1".into(), name: "Alice".into() })?;
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
    /// #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq)]
    /// struct User {
    ///     #[primary_key]
    ///     id: u64,
    ///     name: String,
    /// }
    ///
    /// #[netabase_macros::netabase_definition(MyApp)]
    /// mod models { use super::*; }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let (store, _temp) = RedbStore::<MyApp>::new_temporary()?;
    /// let txn = store.begin_write()?;
    /// let user = User { id: 1, name: "Alice".to_string() };
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
            'static,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
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
    /// #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq)]
    /// struct User {
    ///     #[primary_key]
    ///     id: u64,
    ///     name: String,
    /// }
    ///
    /// #[netabase_macros::netabase_definition(MyApp)]
    /// mod models { use super::*; }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
            'static,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
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
    /// #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq)]
    /// struct User {
    ///     #[primary_key]
    ///     id: u64,
    ///     name: String,
    /// }
    ///
    /// #[netabase_macros::netabase_definition(MyApp)]
    /// mod models { use super::*; }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let (store, _temp) = RedbStore::<MyApp>::new_temporary()?;
    /// let txn = store.begin_write()?;
    /// txn.create(&User { id: 1, name: "Alice".into() })?;
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
            'static,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
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
    /// #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq)]
    /// struct User {
    ///     #[primary_key]
    ///     id: u64,
    ///     name: String,
    /// }
    ///
    /// #[netabase_macros::netabase_definition(MyApp)]
    /// mod models { use super::*; }
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
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
            'static,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
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
            'static,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
    // Add Subscription bounds
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
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
            'static,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
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
            'static,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
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
            'static,
        <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant:
            'static,
        <<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant:
            'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: 'static,
        for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
        <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
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

impl<'db, D: RedbDefinition> NBTransaction<'db, D> for RedbTransaction<'db, D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
    D::SubscriptionKeys: redb::Key + 'static,
{
    type ReadTransaction = NetabaseRedbReadTransaction<'db, D>;
    type WriteTransaction = NetabaseRedbWriteTransaction<'db, D>;

    fn create(&self, _definition: &D) -> NetabaseResult<()> {
        todo!("NBTransaction::create - convert D to specific model M, call create_redb")
    }

    fn read(&self, _key: &D::DefKeys) -> NetabaseResult<Option<D>> {
        todo!(
            "NBTransaction::read - extract primary key from DefKeys, call read_redb, convert back to D"
        )
    }

    fn update(&self, _definition: &D) -> NetabaseResult<()> {
        todo!("NBTransaction::update - convert D to specific model M, call update_redb")
    }

    fn delete(&self, _key: &D::DefKeys) -> NetabaseResult<()> {
        todo!("NBTransaction::delete - extract primary key from DefKeys, call delete_redb")
    }

    fn create_many(&self, _definitions: &[D]) -> NetabaseResult<()> {
        todo!("NBTransaction::create_many - requires NBTransaction::create to be implemented first")
    }

    fn read_if<F>(&self, _predicate: F) -> NetabaseResult<Vec<D>>
    where
        F: Fn(&D) -> bool,
    {
        todo!("NBTransaction::read_if")
    }

    fn read_range(&self, _range: std::ops::Range<D::DefKeys>) -> NetabaseResult<Vec<D>> {
        todo!("NBTransaction::read_range")
    }

    fn update_range<F>(
        &self,
        _range: std::ops::Range<D::DefKeys>,
        _updater: F,
    ) -> NetabaseResult<()>
    where
        F: Fn(&mut D),
    {
        todo!("NBTransaction::update_range")
    }

    fn update_if<P, U>(&self, _predicate: P, _updater: U) -> NetabaseResult<()>
    where
        P: Fn(&D) -> bool,
        U: Fn(&mut D),
    {
        todo!("NBTransaction::update_if")
    }

    fn delete_many(&self, _keys: &[D::DefKeys]) -> NetabaseResult<()> {
        todo!("NBTransaction::delete_many - requires NBTransaction::delete to be implemented first")
    }

    fn delete_if<F>(&self, _predicate: F) -> NetabaseResult<()>
    where
        F: Fn(&D) -> bool,
    {
        todo!("NBTransaction::delete_if")
    }

    fn delete_range(&self, _range: std::ops::Range<D::DefKeys>) -> NetabaseResult<()> {
        todo!("NBTransaction::delete_range")
    }

    fn write<F, R>(&self, f: F) -> NetabaseResult<R>
    where
        F: FnOnce(&Self::WriteTransaction) -> NetabaseResult<R>,
    {
        match &self.transaction {
            RedbTransactionType::Write(write_txn) => f(write_txn),
            RedbTransactionType::Read(_) => Err(NetabaseError::Other),
        }
    }

    fn read_fn<F, R>(&self, f: F) -> NetabaseResult<R>
    where
        F: FnOnce(&Self::ReadTransaction) -> NetabaseResult<R>,
    {
        match &self.transaction {
            RedbTransactionType::Read(read_txn) => f(read_txn),
            RedbTransactionType::Write(_) => Err(NetabaseError::Other),
        }
    }

    fn read_related<OD>(&self, _key: &OD::DefKeys) -> NetabaseResult<Option<OD>>
    where
        OD: NetabaseDefinition,
        <OD as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    {
        todo!("NBTransaction::read_related")
    }

    fn can_access_definition<OD>(&self) -> bool
    where
        OD: NetabaseDefinition,
        <OD as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    {
        true
    }
}
