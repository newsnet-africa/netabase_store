use redb::ReadableDatabase;
use strum::IntoDiscriminant;

use crate::{
    databases::redb::{NetabasePermissions, RedbStorePermissions, ModelOperationPermission},
    relational::{CrossDefinitionPermissions, RelationalLink, GlobalDefinitionEnum},
    traits::{
        database::transaction::NBTransaction,
        registery::{
            definition::NetabaseDefinition,
            models::{
                keys::NetabaseModelKeys,
                model::{NetabaseModel, redb_model::{RedbModelTableDefinitions, RedbNetbaseModel}},
            },
        },
    },
    errors::{NetabaseResult, NetabaseError},
};

pub struct RedbTransactionInner<D: NetabaseDefinition>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    transaction: RedbTransactionType,
    permissions: NetabasePermissions<D>,
}

pub enum RedbTransactionType {
    Read(redb::ReadTransaction),
    Write(redb::WriteTransaction),
}

pub type RedbTransaction<D> = RedbTransactionInner<D>;

// --- Copied types from redb_transaction.rs ---

pub struct ModelOpenTables<'txn, 'db, D: NetabaseDefinition, M: RedbNetbaseModel<'db, D> + redb::Key> 
where
    D::Discriminant: 'static + std::fmt::Debug,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: redb::Key + 'static,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: redb::Key + 'static,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: redb::Key + 'static,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    M: 'static,
{
    pub main: TablePermission<'txn, <M::Keys as NetabaseModelKeys<D, M>>::Primary<'db>, M>,

    pub secondary: Vec<(
        TablePermission<'txn, <M::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>, <M::Keys as NetabaseModelKeys<D, M>>::Primary<'db>>,
        &'db str,
    )>,

    pub relational: Vec<(
        TablePermission<'txn, <M::Keys as NetabaseModelKeys<D, M>>::Relational<'db>, <M::Keys as NetabaseModelKeys<D, M>>::Primary<'db>>,
        &'db str,
    )>,
}

pub enum TableType<K, V>
where
    K: redb::Key + 'static,
    V: redb::Value + redb::Key + 'static,
{
    Table(redb::ReadOnlyTable<K, V>),
    MultimapTable(redb::ReadOnlyMultimapTable<K, V>),
}

pub enum ReadWriteTableType<'txn, K, V>
where
    K: redb::Key + 'static,
    V: redb::Value + redb::Key + 'static,
{
    Table(redb::Table<'txn, K, V>),
    MultimapTable(redb::MultimapTable<'txn, K, V>),
}

pub enum TablePermission<'txn, K, V>
where
    K: redb::Key + 'static,
    V: redb::Value + redb::Key + 'static,
{
    ReadOnly(TableType<K, V>),
    ReadWrite(ReadWriteTableType<'txn, K, V>),
}

pub enum ModelTableType<'txn, 'db, D: NetabaseDefinition, M: RedbNetbaseModel<'db, D>>
where
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: redb::Key + 'static,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: redb::Key + 'static,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: redb::Key + 'static,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    D::Discriminant: 'static + std::fmt::Debug,
{
    Main(TablePermission<'txn, <M::Keys as NetabaseModelKeys<D, M>>::Primary<'db>, M>),
    Secondary(
        Vec<(
            TablePermission<'txn, <M::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>, <M::Keys as NetabaseModelKeys<D, M>>::Primary<'db>>,
            &'db str,
        )>
    ),
    Relational(
        Vec<(
            TablePermission<'txn, <M::Keys as NetabaseModelKeys<D, M>>::Relational<'db>, <M::Keys as NetabaseModelKeys<D, M>>::Primary<'db>>,
            &'db str,
        )>
    ),
}


impl<D: NetabaseDefinition> RedbTransaction<D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    pub fn new(
        db: RedbStorePermissions,
        permissions: NetabasePermissions<D>,
    ) -> NetabaseResult<Self> {
        let transaction = match db {
            RedbStorePermissions::ReadOnly(read_only_database) => {
                let read_txn = read_only_database
                    .begin_read()
                    .map_err(|e| NetabaseError::RedbTransactionError(e))?;
                RedbTransactionType::Read(read_txn)
            }
            RedbStorePermissions::ReadWrite(database) => {
                let write_txn = database
                    .begin_write()
                    .map_err(|e| NetabaseError::RedbTransactionError(e))?;
                RedbTransactionType::Write(write_txn)
            }
        };

        let txn = RedbTransactionInner {
            transaction,
            permissions,
        };
        txn.check_permissions()?;
        Ok(txn)
    }

    fn check_permissions(&self) -> NetabaseResult<()> {
        // Permission checking logic here - for now just return Ok
        Ok(())
    }

    /// Open tables for a specific model with proper permission checking (concrete implementation)
    pub fn open_model_tables<'txn, 'db, M>(
        &'txn self,
        definitions: RedbModelTableDefinitions<'db, M, D>,
    ) -> NetabaseResult<ModelOpenTables<'txn, 'db, D, M>>
    where
        M: RedbNetbaseModel<'db, D> + redb::Key + 'static,
        D::Discriminant: 'static + std::fmt::Debug,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: redb::Key + 'static,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    {
        // Check permissions first
        self.check_permissions()?;

        match &self.transaction {
            RedbTransactionType::Read(read_txn) => {
                // For read transactions, open read-only tables
                let main_table = {
                    let table = read_txn.open_table(definitions.main)?;
                    TablePermission::ReadOnly(TableType::Table(table))
                };

                let secondary_tables: Result<Vec<_>, NetabaseError> = definitions
                    .secondary
                    .into_iter()
                    .map(
                        |(table_def, name)| -> Result<_, NetabaseError> {
                            let table = read_txn.open_multimap_table(table_def)?;
                            Ok((
                                TablePermission::ReadOnly(TableType::MultimapTable(table)),
                                name,
                            ))
                        },
                    )
                    .collect();

                let relational_tables: Result<Vec<_>, NetabaseError> = definitions
                    .relational
                    .into_iter()
                    .map(
                        |(table_def, name)| -> Result<_, NetabaseError> {
                            let table = read_txn.open_multimap_table(table_def)?;
                            Ok((
                                TablePermission::ReadOnly(TableType::MultimapTable(table)),
                                name,
                            ))
                        },
                    )
                    .collect();

                Ok(ModelOpenTables {
                    main: main_table,
                    secondary: secondary_tables?,
                    relational: relational_tables?,
                })
            }
            RedbTransactionType::Write(write_txn) => {
                // Check if we have write permissions
                if !self.permissions.can_write() {
                    return Err(NetabaseError::Permission);
                }

                // For write transactions, open read-write tables
                let main_table = {
                    let table = write_txn.open_table(definitions.main)?;
                    TablePermission::ReadWrite(ReadWriteTableType::Table(table))
                };

                let secondary_tables: Result<Vec<_>, NetabaseError> = definitions
                    .secondary
                    .into_iter()
                    .map(
                        |(table_def, name)| -> Result<_, NetabaseError> {
                            let table = write_txn.open_multimap_table(table_def)?;
                            Ok((
                                TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(
                                    table,
                                )),
                                name,
                            ))
                        },
                    )
                    .collect();

                let relational_tables: Result<Vec<_>, NetabaseError> = definitions
                    .relational
                    .into_iter()
                    .map(
                        |(table_def, name)| -> Result<_, NetabaseError> {
                            let table = write_txn.open_multimap_table(table_def)?;
                            Ok((
                                TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(
                                    table,
                                )),
                                name,
                            ))
                        },
                    )
                    .collect();

                Ok(ModelOpenTables {
                    main: main_table,
                    secondary: secondary_tables?,
                    relational: relational_tables?,
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
            RedbTransactionType::Read(read_txn) => f(read_txn),
            RedbTransactionType::Write(_) => return Err(NetabaseError::Permission),
        }
    }

    /// Execute a function with the raw write transaction (limited scope)
    pub fn with_write_transaction<F, R>(&self, f: F) -> NetabaseResult<R>
    where
        F: FnOnce(&redb::WriteTransaction) -> NetabaseResult<R>,
    {
        if !self.permissions.can_write() {
            return Err(NetabaseError::Permission);
        }

        match &self.transaction {
            RedbTransactionType::Write(write_txn) => f(write_txn),
            RedbTransactionType::Read(_) => Err(NetabaseError::Permission),
        }
    }

    /// Commit the transaction if it's a write transaction
    pub fn commit(self) -> NetabaseResult<()> {
        match self.transaction {
            RedbTransactionType::Write(write_txn) => {
                write_txn.commit()?;
                Ok(())
            }
            RedbTransactionType::Read(_) => {
                // Read transactions don't need to be committed
                Ok(())
            }
        }
    }

    // --- Inherent methods for Redb models ---

    pub fn create_redb<'db, M>(&self, _model: M) -> NetabaseResult<()>
    where
        M: NetabaseModel<D> + RedbNetbaseModel<'db, D> + Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: Clone,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as strum::IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as strum::IntoDiscriminant>::Discriminant:
            'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: redb::Key,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: 'static,
    {
        if !self.permissions.can_perform_operation(&ModelOperationPermission::Create) {
            return Err(NetabaseError::Permission);
        }

        let definitions = M::table_definitions();
        let tables = self.open_model_tables(definitions)?;

        match &tables.main {
            TablePermission::ReadWrite(ReadWriteTableType::Table(_table)) => {
                // TODO: Implement proper table insertion
                todo!("Implement table insertion with proper Borrow trait support");
            }
            _ => Err(NetabaseError::Permission),
        }
    }

    // ... Other methods (update, delete, read) would follow similar pattern ...
}

impl<'db, D: NetabaseDefinition + GlobalDefinitionEnum> NBTransaction<'db, D> for RedbTransaction<D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    type ReadTransaction = redb::ReadTransaction;
    type WriteTransaction = redb::WriteTransaction;

    fn create<M>(&self, _model: M) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static
    {
         // This is where the issue is. We can't call self.create_redb(model) because M doesn't implement RedbNetbaseModel.
         // But we can implement it as todo!() for now as requested.
         todo!("NBTransaction::create: Requires M to be RedbNetbaseModel. This trait bound mismatch is expected.")
    }

    fn read<M>(&self, _key: M::Keys) -> NetabaseResult<M>
    where
        M: NetabaseModel<D>,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static
    {
        todo!("NBTransaction::read")
    }

    fn update<M>(&self, _model: M) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static
    {
        todo!("NBTransaction::update")
    }

    fn delete<M>(&self, _key: M::Keys) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static
    {
        todo!("NBTransaction::delete")
    }

    fn create_many<M>(&self, _models: Vec<M>) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static
    {
        todo!("NBTransaction::create_many")
    }

    fn read_if<M, F>(&self, _predicate: F) -> NetabaseResult<Vec<M>>
    where
        M: NetabaseModel<D>,
        F: Fn(&M) -> bool,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static
    {
        todo!("NBTransaction::read_if")
    }

    fn read_range<M, K>(&self, _range: std::ops::Range<K>) -> NetabaseResult<Vec<M>>
    where
        M: NetabaseModel<D>,
        K: Into<M::Keys>,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static
    {
        todo!("NBTransaction::read_range")
    }

    fn update_range<M, K, F>(&self, _range: std::ops::Range<K>, _updater: F) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        K: Into<M::Keys>,
        F: Fn(&mut M),
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static
    {
        todo!("NBTransaction::update_range")
    }

    fn update_if<M, P, U>(&self, _predicate: P, _updater: U) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        P: Fn(&M) -> bool,
        U: Fn(&mut M),
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static
    {
        todo!("NBTransaction::update_if")
    }

    fn delete_many<M>(&self, _keys: Vec<M::Keys>) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static
    {
        todo!("NBTransaction::delete_many")
    }

    fn delete_if<M, F>(&self, _predicate: F) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        F: Fn(&M) -> bool,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static
    {
        todo!("NBTransaction::delete_if")
    }

    fn delete_range<M, K>(&self, _range: std::ops::Range<K>) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        K: Into<M::Keys>,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static
    {
        todo!("NBTransaction::delete_range")
    }

    fn write<F, R>(&self, f: F) -> NetabaseResult<R>
    where
        F: FnOnce(&Self::WriteTransaction) -> NetabaseResult<R>
    {
        self.with_write_transaction(f)
    }

    fn read_fn<F, R>(&self, f: F) -> NetabaseResult<R>
    where
        F: FnOnce(&Self::ReadTransaction) -> NetabaseResult<R>
    {
        self.with_read_transaction(f)
    }

    fn read_related<OD, M>(&self, _key: M::Keys) -> NetabaseResult<Option<M>>
    where
        OD: NetabaseDefinition,
        M: NetabaseModel<OD>,
        <OD as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        for<'a> <<M::Keys as NetabaseModelKeys<OD, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<OD, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static
    {
        todo!("NBTransaction::read_related")
    }

    fn hydrate_relation<M>(&self, _link: RelationalLink<M>) -> NetabaseResult<RelationalLink<M>>
    where
        M: GlobalDefinitionEnum,
        <M as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    {
        todo!("NBTransaction::hydrate_relation")
    }

    fn can_access_definition<OD>(&self) -> bool
    where
        OD: NetabaseDefinition,
        <OD as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug
    {
        // Simple implementation
        true 
    }

    fn get_cross_permissions<OD>(&self) -> Option<CrossDefinitionPermissions<D>>
    where
        OD: NetabaseDefinition + GlobalDefinitionEnum,
        <OD as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug
    {
        None
    }

    fn create_with_relations<M>(&self, _model: M, _relations: Vec<RelationalLink<M>>) -> NetabaseResult<()>
    where
        M: NetabaseModel<D> + GlobalDefinitionEnum,
        <M as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static
    {
        todo!("NBTransaction::create_with_relations")
    }

    fn update_relations<M, RM>(&self, _model_key: M::Keys, _relation_updates: Vec<RelationalLink<RM>>) -> NetabaseResult<()>
    where
        M: NetabaseModel<D>,
        RM: GlobalDefinitionEnum,
        <RM as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant:
            'static
    {
        todo!("NBTransaction::update_relations")
    }
}