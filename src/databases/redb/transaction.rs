use redb::ReadableDatabase;

use crate::{
    databases::redb::{NetabasePermissions, RedbStorePermissions},
    relational::{CrossDefinitionPermissions, RelationalLink, GlobalDefinitionEnum},
    traits::{
        database::transaction::redb_transaction::{
            ModelOpenTables, NBRedbReadTransaction, NBRedbTransaction, NBRedbTransactionBase,
            NBRedbWriteTransaction, ReadWriteTableType, TablePermission, TableType,
        },
        registery::{
            definition::NetabaseDefinition,
            models::{
                keys::NetabaseModelKeys,
                model::{NetabaseModel, RedbModelTableDefinitions, RedbNetbaseModel},
            },
        },
    },
};

pub struct RedbTransactionInner<D: NetabaseDefinition>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static,
{
    transaction: RedbTransactionType,
    permissions: NetabasePermissions<D>,
}

pub enum RedbTransactionType {
    Read(redb::ReadTransaction),
    Write(redb::WriteTransaction),
}

pub type RedbTransaction<D> = RedbTransactionInner<D>;

impl<'db, D: NetabaseDefinition> NBRedbTransactionBase<'db, D> for RedbTransaction<D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static,
{
    type ReadTransaction = redb::ReadTransaction;
    type WriteTransaction = redb::WriteTransaction;

    fn check_permissions(&self) -> crate::errors::NetabaseResult<()> {
        // Permission checking logic here - for now just return Ok
        Ok(())
    }

    fn new_with_permissions(
        db: RedbStorePermissions,
        permissions: NetabasePermissions<D>,
    ) -> crate::errors::NetabaseResult<Self> {
        let transaction = match db {
            RedbStorePermissions::ReadOnly(read_only_database) => {
                let read_txn = read_only_database
                    .begin_read()
                    .map_err(|e| crate::errors::NetabaseError::RedbTransactionError(e))?;
                RedbTransactionType::Read(read_txn)
            }
            RedbStorePermissions::ReadWrite(database) => {
                let write_txn = database
                    .begin_write()
                    .map_err(|e| crate::errors::NetabaseError::RedbTransactionError(e))?;
                RedbTransactionType::Write(write_txn)
            }
        };

        Ok(RedbTransactionInner {
            transaction,
            permissions,
        })
    }
}

impl<D: NetabaseDefinition> RedbTransaction<D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static,
{
    /// Open tables for a specific model with proper permission checking (concrete implementation)
    pub fn open_model_tables_impl<'txn, 'db, M>(
        &'txn self,
        definitions: RedbModelTableDefinitions<'db, M, D>,
    ) -> crate::errors::NetabaseResult<ModelOpenTables<'txn, 'db, D, M>>
    where
        M: RedbNetbaseModel<'db, D> + redb::Key + 'static,
        D::Discriminant: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: redb::Key + 'static,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
    {
        // Check permissions first
        <Self as NBRedbTransactionBase<D>>::check_permissions(self)?;

        match &self.transaction {
            RedbTransactionType::Read(read_txn) => {
                // For read transactions, open read-only tables
                let main_table = {
                    let table = read_txn.open_table(definitions.main)?;
                    TablePermission::ReadOnly(TableType::Table(table))
                };

                let secondary_tables: Result<Vec<_>, crate::errors::NetabaseError> = definitions
                    .secondary
                    .into_iter()
                    .map(
                        |(table_def, name)| -> Result<_, crate::errors::NetabaseError> {
                            let table = read_txn.open_multimap_table(table_def)?;
                            Ok((
                                TablePermission::ReadOnly(TableType::MultimapTable(table)),
                                name,
                            ))
                        },
                    )
                    .collect();

                let relational_tables: Result<Vec<_>, crate::errors::NetabaseError> = definitions
                    .relational
                    .into_iter()
                    .map(
                        |(table_def, name)| -> Result<_, crate::errors::NetabaseError> {
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
                    return Err(crate::errors::NetabaseError::Permission);
                }

                // For write transactions, open read-write tables
                let main_table = {
                    let table = write_txn.open_table(definitions.main)?;
                    TablePermission::ReadWrite(ReadWriteTableType::Table(table))
                };

                let secondary_tables: Result<Vec<_>, crate::errors::NetabaseError> = definitions
                    .secondary
                    .into_iter()
                    .map(
                        |(table_def, name)| -> Result<_, crate::errors::NetabaseError> {
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

                let relational_tables: Result<Vec<_>, crate::errors::NetabaseError> = definitions
                    .relational
                    .into_iter()
                    .map(
                        |(table_def, name)| -> Result<_, crate::errors::NetabaseError> {
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
    pub fn with_read_transaction<F, R>(&self, f: F) -> crate::errors::NetabaseResult<R>
    where
        F: FnOnce(&redb::ReadTransaction) -> crate::errors::NetabaseResult<R>,
    {
        match &self.transaction {
            RedbTransactionType::Read(read_txn) => f(read_txn),
            RedbTransactionType::Write(_) => return Err(crate::errors::NetabaseError::Permission),
        }
    }

    /// Execute a function with the raw write transaction (limited scope)
    pub fn with_write_transaction<F, R>(&self, f: F) -> crate::errors::NetabaseResult<R>
    where
        F: FnOnce(&redb::WriteTransaction) -> crate::errors::NetabaseResult<R>,
    {
        if !self.permissions.can_write() {
            return Err(crate::errors::NetabaseError::Permission);
        }

        match &self.transaction {
            RedbTransactionType::Write(write_txn) => f(write_txn),
            RedbTransactionType::Read(_) => Err(crate::errors::NetabaseError::Permission),
        }
    }

    /// Commit the transaction if it's a write transaction
    pub fn commit(self) -> crate::errors::NetabaseResult<()> {
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
}

impl<'db, D: NetabaseDefinition> NBRedbReadTransaction<'db, D> for RedbTransaction<D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static,
{
    fn read<M>(&self, _key: M::Keys) -> crate::errors::NetabaseResult<M>
    where
        M: NetabaseModel<D> + RedbNetbaseModel<'db, D>,
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
        // Open model tables and perform read within limited scope
        let definitions = M::table_definitions();
        let tables = self.open_model_tables_impl(definitions)?;

        // Use limited scope for table operations
        match &tables.main {
            TablePermission::ReadOnly(TableType::Table(_table)) => {
                // Perform read operation
                todo!("Implement read operation using opened table")
            }
            TablePermission::ReadWrite(ReadWriteTableType::Table(_table)) => {
                // Perform read operation on write table
                todo!("Implement read operation using opened write table")
            }
            _ => unreachable!("Main table should always be a Table, not MultimapTable"),
        }
    }

    fn read_if<M, F>(&self, _predicate: F) -> crate::errors::NetabaseResult<Vec<M>>
    where
        M: NetabaseModel<D> + RedbNetbaseModel<'db, D>,
        F: Fn(&M) -> bool,
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
        // Open model tables and perform filtered read within limited scope
        let definitions = M::table_definitions();
        let _tables = self.open_model_tables_impl(definitions)?;

        // Iterate through all entries and apply predicate
        todo!("Implement read_if operation using opened tables")
    }

    fn read_range<M, K>(&self, _range: std::ops::Range<K>) -> crate::errors::NetabaseResult<Vec<M>>
    where
        M: NetabaseModel<D> + RedbNetbaseModel<'db, D>,
        K: Into<M::Keys>,
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
        // Open model tables and perform range read within limited scope
        let definitions = M::table_definitions();
        let _tables = self.open_model_tables_impl(definitions)?;

        // Use range iterator on the table
        todo!("Implement read_range operation using opened tables")
    }

    fn read_fn<F, R>(&self, f: F) -> crate::errors::NetabaseResult<R>
    where
        F: FnOnce(&Self::ReadTransaction) -> crate::errors::NetabaseResult<R>,
    {
        self.with_read_transaction(f)
    }
}

impl<'db, D: NetabaseDefinition> NBRedbWriteTransaction<'db, D> for RedbTransaction<D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static,
{
    fn create<M>(&self, model: M) -> crate::errors::NetabaseResult<()>
    where
        M: NetabaseModel<D> + RedbNetbaseModel<'db, D> + Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: Clone,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: Clone,
        for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<D, M>>::Secondary<'a> as strum::IntoDiscriminant>::Discriminant:
            'static,
        for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<D, M>>::Relational<'a> as strum::IntoDiscriminant>::Discriminant:
        'static, <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: redb::Key, <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: redb::Key, <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: redb::Key, <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: 'static, <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: 'static, <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: 'static
    {
        // Check write permissions first
        if !self
            .permissions
            .can_perform_operation(&crate::databases::redb::ModelOperationPermission::Create)
        {
            return Err(crate::errors::NetabaseError::Permission);
        }

        // Open model tables and perform create within limited scope
        let definitions = M::table_definitions();
        let tables = self.open_model_tables_impl(definitions)?;

        // Get the primary key for this model
        let _primary_key = model.get_primary_key();

        // Insert into main table using limited scope
        match &tables.main {
            TablePermission::ReadWrite(ReadWriteTableType::Table(_table)) => {
                // TODO: Implement proper table insertion with correct Borrow traits
                // This requires ensuring all key types implement the required Borrow<T::SelfType<'_>> traits
                // table.insert(primary_key.clone(), model.clone())?;

                // For now, return success to allow compilation and focus on the architecture
                todo!("Implement table insertion with proper Borrow trait support");
            }
            _ => Err(crate::errors::NetabaseError::Permission),
        }
    }

    fn update<M>(&self, model: M) -> crate::errors::NetabaseResult<()>
    where
        M: NetabaseModel<D> + RedbNetbaseModel<'db, D>,
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
        // Check update permissions first
        if !self
            .permissions
            .can_perform_operation(&crate::databases::redb::ModelOperationPermission::Update)
        {
            return Err(crate::errors::NetabaseError::Permission);
        }

        // Open model tables and perform update within limited scope
        let definitions = M::table_definitions();
        let tables = self.open_model_tables_impl(definitions)?;

        let _primary_key = model.get_primary_key();

        // Update main table
        match &tables.main {
            TablePermission::ReadWrite(ReadWriteTableType::Table(_table)) => {
                // TODO: Implement update with proper Borrow traits
                // table.insert(&primary_key, &model)?; // insert acts as upsert
                todo!("Implement update operation with proper Borrow trait support")
            }
            _ => Err(crate::errors::NetabaseError::Permission),
        }
    }

    fn delete<M>(&self, _key: M::Keys) -> crate::errors::NetabaseResult<()>
    where
        M: NetabaseModel<D> + RedbNetbaseModel<'db, D>,
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
        // Check delete permissions first
        if !self
            .permissions
            .can_perform_operation(&crate::databases::redb::ModelOperationPermission::Delete)
        {
            return Err(crate::errors::NetabaseError::Permission);
        }

        // Open model tables and perform delete within limited scope
        let definitions = M::table_definitions();
        let _tables = self.open_model_tables_impl(definitions)?;

        // Convert key to primary key (simplified - in practice this would be more complex)
        // For now, we'll need a way to extract the primary key from M::Keys
        todo!("Delete operation needs key extraction logic")
    }

    fn create_many<M>(&self, models: Vec<M>) -> crate::errors::NetabaseResult<()>
    where
        M: NetabaseModel<D> + RedbNetbaseModel<'db, D>,
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
        // Use a single table opening for all models (efficient batch operation)
        if !self
            .permissions
            .can_perform_operation(&crate::databases::redb::ModelOperationPermission::Create)
        {
            return Err(crate::errors::NetabaseError::Permission);
        }

        // Open tables once for all models to be more efficient
        let definitions = M::table_definitions();
        let tables = self.open_model_tables_impl(definitions)?;

        match &tables.main {
            TablePermission::ReadWrite(ReadWriteTableType::Table(_table)) => {
                for _model in models {
                    // TODO: Implement bulk create with proper Borrow traits
                    // let primary_key = model.get_primary_key();
                    // table.insert(&primary_key, &model)?;

                    // TODO: Insert secondary and relational keys
                }

                // For now, return success to allow compilation
                todo!("Implement bulk create with proper Borrow trait support")
            }
            _ => Err(crate::errors::NetabaseError::Permission),
        }
    }

    fn update_range<M, K, F>(&self, _range: std::ops::Range<K>, _updater: F) -> crate::errors::NetabaseResult<()>
    where
        M: NetabaseModel<D> + RedbNetbaseModel<'db, D>,
        K: Into<M::Keys>,
        F: Fn(&mut M),
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
        todo!("Implement update_range operation")
    }

    fn update_if<M, P, U>(&self, _predicate: P, _updater: U) -> crate::errors::NetabaseResult<()>
    where
        M: NetabaseModel<D> + RedbNetbaseModel<'db, D>,
        P: Fn(&M) -> bool,
        U: Fn(&mut M),
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
        todo!("Implement update_if operation")
    }

    fn delete_many<M>(&self, _keys: Vec<M::Keys>) -> crate::errors::NetabaseResult<()>
    where
        M: NetabaseModel<D> + RedbNetbaseModel<'db, D>,
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
        todo!("Implement delete_many operation")
    }

    fn delete_if<M, F>(&self, _predicate: F) -> crate::errors::NetabaseResult<()>
    where
        M: NetabaseModel<D> + RedbNetbaseModel<'db, D>,
        F: Fn(&M) -> bool,
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
        todo!("Implement delete_if operation")
    }

    fn delete_range<M, K>(&self, _range: std::ops::Range<K>) -> crate::errors::NetabaseResult<()>
    where
        M: NetabaseModel<D> + RedbNetbaseModel<'db, D>,
        K: Into<M::Keys>,
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
        todo!("Implement delete_range operation")
    }

    fn write<F, R>(&self, f: F) -> crate::errors::NetabaseResult<R>
    where
        F: FnOnce(&Self::WriteTransaction) -> crate::errors::NetabaseResult<R>,
    {
        self.with_write_transaction(f)
    }
}

impl<'db, D: NetabaseDefinition + GlobalDefinitionEnum> NBRedbTransaction<'db, D> for RedbTransaction<D>
where
    <D as strum::IntoDiscriminant>::Discriminant: 'static,
{
    /// Open model tables with proper permission checking
    fn open_model_tables<'txn, M>(
        &'txn self,
        definitions: RedbModelTableDefinitions<'db, M, D>,
    ) -> crate::errors::NetabaseResult<ModelOpenTables<'txn, 'db, D, M>>
    where
        M: RedbNetbaseModel<'db, D> + redb::Key + 'static,
        D::Discriminant: 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: redb::Key + 'static,
        <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: redb::Key + 'static,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
        for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
        M: 'static,
    {
        // Delegate to the concrete implementation method
        self.open_model_tables_impl(definitions)
    }

    fn load_related<OM>(
        &self,
        link: RelationalLink<OM>,
        _cross_permissions: CrossDefinitionPermissions<D>,
    ) -> crate::errors::NetabaseResult<RelationalLink<OM>>
    where
        OM: GlobalDefinitionEnum,
    {
        // TODO: Implement proper cross-definition loading
        // For now, return the link as-is
        Ok(link)
    }

    fn save_related<OM>(
        &self,
        link: RelationalLink<OM>,
        _cross_permissions: CrossDefinitionPermissions<D>,
    ) -> crate::errors::NetabaseResult<RelationalLink<OM>>
    where
        OM: GlobalDefinitionEnum,
    {
        // TODO: Implement proper cross-definition saving
        // For now, return the link as-is
        Ok(link)
    }

    fn delete_related<OM>(
        &self,
        _link: RelationalLink<OM>,
        _cross_permissions: CrossDefinitionPermissions<D>,
    ) -> crate::errors::NetabaseResult<()>
    where
        OM: GlobalDefinitionEnum,
    {
        // TODO: Implement proper cross-definition deletion
        // For now, just return success
        Ok(())
    }
}
