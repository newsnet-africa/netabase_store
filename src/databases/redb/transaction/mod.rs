pub mod crud;
pub mod tables;
pub mod wrappers;

use redb::{ReadableDatabase, TransactionError};
use strum::IntoDiscriminant;

use crate::{
    errors::{NetabaseError, NetabaseResult},
    relational::{ModelRelationPermissions, RelationPermission, PermissionFlag},
    traits::{
        database::transaction::NBTransaction,
        registery::{
            definition::{NetabaseDefinition, redb_definition::RedbDefinition},
            models::{
                keys::NetabaseModelKeys,
                model::{
                    NetabaseModel,
                    redb_model::{RedbModelTableDefinitions, RedbNetbaseModel},
                },
            },
        },
    },
};

pub use self::crud::RedbModelCrud;
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
    pub fn new(db: &redb::Database) -> NetabaseResult<Self> {
        let write_txn = db
            .begin_write()
            .map_err(|e: TransactionError| NetabaseError::RedbTransactionError(e.into()))?;
        let transaction = RedbTransactionType::Write(NetabaseRedbWriteTransaction::new(write_txn));

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
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: redb::Key + 'static,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'db>: 'static,
    {
        // For batch operations, we default to ReadWrite permissions for the model being prepared
        let perms = ModelRelationPermissions {
            relationa_tree_access: &[RelationPermission(M::TREE_NAMES, PermissionFlag::ReadWrite)]
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
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'data>: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'data>: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'data>: redb::Key + 'static,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'data>: 'static,
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

                let relational_tables: Result<Vec<_>, NetabaseError> = M::TREE_NAMES
                    .relational
                    .iter()
                    .map(|disc_table| {
                        let permission_flag = if let Some(perms) = &relational_permissions {
                             perms.relationa_tree_access.iter()
                                 .find(|p| p.0.relational.iter().any(|r| r.table_name == disc_table.table_name))
                                 .map(|p| &p.1)
                                 .unwrap_or(&PermissionFlag::ReadOnly)
                        } else {
                            &PermissionFlag::ReadOnly
                        };

                        let def = redb::MultimapTableDefinition::new(disc_table.table_name);
                        
                        write_txn.open_multimap_table(def).map(|table| {
                            let table_perm = match permission_flag {
                                PermissionFlag::ReadWrite => TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)),
                                PermissionFlag::ReadOnly => TablePermission::ReadOnlyWrite(ReadWriteTableType::MultimapTable(table)),
                            };
                            (
                                table_perm,
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

    pub fn commit(self) -> NetabaseResult<()> {
        match self.transaction {
            RedbTransactionType::Write(write_txn) => write_txn.commit(),
            RedbTransactionType::Read(_) => {
                // Read transactions don't need to be committed
                Ok(())
            }
        }
    }

    // --- Inherent methods for Redb models ---

    pub fn create_redb<'data: 'db, M>(&'db self, model: &'data M) -> NetabaseResult<()>
    where
        M: RedbModelCrud<'db, D> + RedbNetbaseModel<'db, D> + Clone,
        for<'a> M::TableV: redb::Value<SelfType<'a> = M>,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: Clone,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: 'static, <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db> as redb::Value>::SelfType<'db>>,
    // Add Subscription bounds
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'db>: 'static,
        D: 'static,
        D::SubscriptionKeys: redb::Key + 'static,
    {
        let definitions = M::table_definitions();
        let perms = ModelRelationPermissions {
            relationa_tree_access: &[RelationPermission(M::TREE_NAMES, PermissionFlag::ReadWrite)]
        };
        let mut tables = self.open_model_tables(definitions, Some(perms))?;

        model.create_entry(&mut tables)
    }

    pub fn read_redb<'data: 'db, M>(&'db self, key: &'data <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>) -> NetabaseResult<Option<M>>
    where
        M: RedbModelCrud<'db, D> + RedbNetbaseModel<'db, D> + Clone,
        for<'a> M::TableV: redb::Value<SelfType<'a> = M>,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: Clone,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: 'static, <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db> as redb::Value>::SelfType<'db>>,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'db>: 'static,
        D: 'static,
        D::SubscriptionKeys: redb::Key + 'static,
    {
        let definitions = M::table_definitions();
        let tables = self.open_model_tables(definitions, None)?;

        M::read_entry(key, &tables)
    }

    pub fn update_redb<'data: 'db, M>(&'db self, model: &'data M) -> NetabaseResult<()>
    where
        M: RedbModelCrud<'db,  D> + RedbNetbaseModel<'db, D> + Clone,
        for<'a> M::TableV: redb::Value<SelfType<'a> = M>,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: Clone,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: 'static, <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db> as redb::Value>::SelfType<'db>>,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'db>: 'static,
        D: 'static,
        D::SubscriptionKeys: redb::Key + 'static,
    {
        let definitions = M::table_definitions();
        let perms = ModelRelationPermissions {
            relationa_tree_access: &[RelationPermission(M::TREE_NAMES, PermissionFlag::ReadWrite)]
        };
        let mut tables = self.open_model_tables(definitions, Some(perms))?;

        model.update_entry(&mut tables)
    }

    pub fn delete_redb<'data, M>(&'db self, key: &'data <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>) -> NetabaseResult<()>
    where
        M: RedbModelCrud<'db, D> + RedbNetbaseModel<'db, D> + Clone,
        for<'a> M::TableV: redb::Value<SelfType<'a> = M>,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: Clone,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: 'static, <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db> as redb::Value>::SelfType<'db>>,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'db>: 'static,
        D: 'static,
        D::SubscriptionKeys: redb::Key + 'static,
    {
        let definitions = M::table_definitions();
        let perms = ModelRelationPermissions {
            relationa_tree_access: &[RelationPermission(M::TREE_NAMES, PermissionFlag::ReadWrite)]
        };
        let mut tables = self.open_model_tables(definitions, Some(perms))?;

        M::delete_entry(key, &mut tables)
    }
}

impl<'db, D: RedbDefinition> NBTransaction<'db, D> for RedbTransaction<'db, D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D: Clone,
{
    type ReadTransaction = NetabaseRedbReadTransaction<'db, D>;
    type WriteTransaction = NetabaseRedbWriteTransaction<'db, D>;

    fn create(&self, definition: &D) -> NetabaseResult<()> {
        todo!("NBTransaction::create - convert D to specific model M, call create_redb")
    }

    fn read(&self, key: &D::DefKeys) -> NetabaseResult<Option<D>> {
        todo!(
            "NBTransaction::read - extract primary key from DefKeys, call read_redb, convert back to D"
        )
    }

    fn update(&self, definition: &D) -> NetabaseResult<()> {
        todo!("NBTransaction::update - convert D to specific model M, call update_redb")
    }

    fn delete(&self, key: &D::DefKeys) -> NetabaseResult<()> {
        todo!("NBTransaction::delete - extract primary key from DefKeys, call delete_redb")
    }

    fn create_many(&self, definitions: &[D]) -> NetabaseResult<()> {
        for definition in definitions {
            self.create(definition)?;
        }
        Ok(())
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

    fn delete_many(&self, keys: &[D::DefKeys]) -> NetabaseResult<()> {
        for key in keys {
            self.delete(key)?;
        }
        Ok(())
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
