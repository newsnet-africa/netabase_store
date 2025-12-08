use crate::databases::redb_store::RedbStore;
use crate::error::{NetabaseError, NetabaseResult};
use crate::traits::definition::{DiscriminantName, NetabaseDefinition, NetabaseDefinitionTrait, ModelAssociatedTypesExt};
use crate::traits::model::{
    NetabaseModelTrait, RedbNetabaseModelTrait, key::NetabaseModelKeyTrait,
};
use crate::traits::store::transaction::{ReadTransaction, WriteTransaction};
use redb::{Key, ReadableTable, TableDefinition, Value};
use std::borrow::Borrow;
use std::fmt::Debug;
use strum::{IntoDiscriminant, IntoEnumIterator};

pub struct RedbReadTransaction<'db, D: NetabaseDefinition>
where
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator,
    <D as strum::IntoDiscriminant>::Discriminant: std::hash::Hash,
    <D as strum::IntoDiscriminant>::Discriminant: std::cmp::Eq,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: DiscriminantName + Clone,
{
    pub(crate) txn: redb::ReadTransaction,
    pub(crate) redb_store: &'db RedbStore<D>,
}

impl<'db, D: NetabaseDefinition> ReadTransaction<D> for RedbReadTransaction<'db, D>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    fn get<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        key: M::PrimaryKey,
    ) -> NetabaseResult<Option<M>>
    where
        M::PrimaryKey: Key + 'static,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        M: for<'a> Value<SelfType<'a> = M>,
        Vec<u8>: TryFrom<
            <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::PrimaryKey,
        >,
    {
        let definition: TableDefinition<M::PrimaryKey, M> = M::definition(&self.redb_store);
        let table = self.txn.open_table(definition)?;
        let result = table.get(key)?;
        Ok(result.map(|v| v.value()))
    }

    fn get_pk_by_secondary_key<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        secondary_key: <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum,
    ) -> NetabaseResult<Option<M::PrimaryKey>>
    where
        M::PrimaryKey: Key + 'static + for<'a> Value<SelfType<'a> = M::PrimaryKey>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: Key + 'static,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: for<'a> Borrow<<<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as Value>::SelfType<'a>>,
        M: for<'a> Value<SelfType<'a> = M>,
    {
        use strum::IntoDiscriminant;

        // Get the discriminant to determine which secondary key table to use
        let discriminant = secondary_key.discriminant();
        let table_name = M::secondary_key_table_name(discriminant);

        // Open the secondary key table
        let table_def: TableDefinition<<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum, M::PrimaryKey> =
            TableDefinition::new(&table_name);
        let table = self.txn.open_table(table_def)?;

        // Look up the primary key
        let result = table.get(secondary_key)?;
        Ok(result.map(|v| v.value()))
    }
}

/// Queue operation for managing transaction operations using generics
/// Similar to redb's Value trait pattern - generic over all operation types
pub enum QueueOperation<PK, M, SK, RK>
where
    PK: Key + Send + Clone + 'static,
    M: Value + Send + Clone + 'static,
    SK: Key + Send + Clone + 'static,
    RK: Key + Send + Clone + 'static,
{
    MainTreeInsert {
        table_name: String,
        primary_key: PK,
        model_data: M,
        table_def: redb::TableDefinition<'static, PK, M>,
    },
    SecondaryKeyInsert {
        tree_name: String,
        key_data: SK,
        primary_key_ref: PK,
    },
    RelationalKeyInsert {
        tree_name: String,
        key_data: RK,
        primary_key_ref: PK,
    },
    HashTreeInsert {
        tree_name: String,
        key_data: SK,
        value_data: Vec<u8>, // Hash values as bytes for now
    },
    Delete {
        table_name: String,
        primary_key: PK,
        table_def: redb::TableDefinition<'static, PK, M>,
    },
}

impl<PK, M, SK, RK> QueueOperation<PK, M, SK, RK>
where
    PK: Key + Send + Clone + 'static,
    PK: for<'a> std::borrow::Borrow<<PK as Value>::SelfType<'a>>,
    M: Value + Send + Clone + 'static,
    M: for<'a> std::borrow::Borrow<<M as Value>::SelfType<'a>>,
    SK: Key + Send + Clone + 'static,
    RK: Key + Send + Clone + 'static,
{
    pub fn priority(&self) -> u8 {
        match self {
            QueueOperation::MainTreeInsert { .. } => 0,
            QueueOperation::SecondaryKeyInsert { .. } => 1,
            QueueOperation::RelationalKeyInsert { .. } => 2,
            QueueOperation::HashTreeInsert { .. } => 3,
            QueueOperation::Delete { .. } => 4,
        }
    }

    pub fn execute<D: NetabaseDefinition>(
        self,
        wrapper: &mut RedbWriteTransaction<D>,
    ) -> NetabaseResult<()>
    where
        <D as IntoDiscriminant>::Discriminant:
            IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    {
        match self {
            QueueOperation::MainTreeInsert {
                primary_key,
                model_data,
                table_def,
                ..
            } => {
                let mut table = wrapper.txn.open_table(table_def)?;
                table.insert(primary_key, model_data)?;
            }
            QueueOperation::SecondaryKeyInsert {
                tree_name,
                key_data: _,
                primary_key_ref: _,
                ..
            } => {
                // TODO: Implement secondary key table insertion when tables are defined
                let _ = tree_name;
            }
            QueueOperation::RelationalKeyInsert {
                tree_name,
                key_data: _,
                primary_key_ref: _,
                ..
            } => {
                // TODO: Implement relational key table insertion when tables are defined
                let _ = tree_name;
            }
            QueueOperation::HashTreeInsert {
                tree_name,
                key_data: _,
                value_data: _,
                ..
            } => {
                // TODO: Implement hash tree insertion when tables are defined
                let _ = tree_name;
            }
            QueueOperation::Delete {
                primary_key,
                table_def,
                ..
            } => {
                let mut table = wrapper.txn.open_table(table_def)?;
                table.remove(primary_key)?;
            }
        }
        Ok(())
    }
}

/// Concrete operation executor enum - replaces Box<dyn OperationExecutor> for better performance
/// Uses enum discriminants and ModelAssociatedTypes instead of opaque types
pub enum ConcreteOperationExecutor<D: NetabaseDefinition>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <D as NetabaseDefinitionTrait>::Keys: strum::IntoDiscriminant,
{
    MainTree {
        table_discriminant: D::Discriminant,            // Model discriminant for tree identification
        primary_key: D::ModelAssociatedTypes,          // Typed primary key
        model_data: D::ModelAssociatedTypes,           // Typed model data
        table_name: String,                             // Table name for creating TableDefinition
        priority: u8,
    },
    SecondaryKey {
        model_discriminant: D::Discriminant,            // Model discriminant
        key_discriminant: D::ModelAssociatedTypes,     // Typed key discriminant (wrapped secondary key discriminant)
        key_data: D::ModelAssociatedTypes,             // Typed key data
        primary_key_ref: D::ModelAssociatedTypes,      // Typed primary key reference
        table_name: String,                             // Table name for creating TableDefinition
        priority: u8,
    },
    RelationalKey {
        model_discriminant: D::Discriminant,            // Model discriminant
        key_discriminant: D::ModelAssociatedTypes,     // Typed relational key discriminant
        key_data: D::ModelAssociatedTypes,             // Typed key data
        primary_key_ref: D::ModelAssociatedTypes,      // Typed primary key reference
        table_name: String,                             // Table name for creating TableDefinition
        priority: u8,
    },
    HashTree {
        model_discriminant: D::Discriminant,            // Model discriminant
        hash: [u8; 32],                                 // Blake3 hash
        primary_key_ref: D::ModelAssociatedTypes,      // Typed primary key reference
        table_name: String,                             // Table name for creating TableDefinition
        priority: u8,
    },
    Delete {
        table_discriminant: D::Discriminant,            // Model discriminant for tree identification
        primary_key: D::ModelAssociatedTypes,          // Typed primary key
        table_name: String,                             // Main table name for creating TableDefinition
        priority: u8,
    },
}

impl<D: NetabaseDefinition> ConcreteOperationExecutor<D>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <D as NetabaseDefinitionTrait>::Keys: IntoDiscriminant,
{
    pub fn execute(self, _wrapper: &mut RedbWriteTransaction<D>) -> NetabaseResult<()> {
        match self {
            ConcreteOperationExecutor::MainTree {
                table_discriminant,
                primary_key,
                model_data,
                table_name,
                ..
            } => {
                /*
                println!(
                    "Executing MainTree operation for table: {}",
                    table_discriminant.name()
                );
                */
                model_data.insert_model_into_redb(&_wrapper.txn, &table_name, &primary_key)?;
            }
            ConcreteOperationExecutor::SecondaryKey {
                model_discriminant,
                key_discriminant,
                key_data,
                primary_key_ref,
                table_name,
                ..
            } => {
                /*
                println!(
                    "Executing SecondaryKey operation for model: {}",
                    model_discriminant.name()
                );
                */
                key_data.insert_secondary_key_into_redb(&_wrapper.txn, &table_name, &primary_key_ref)?;
            }
            ConcreteOperationExecutor::RelationalKey {
                model_discriminant,
                key_discriminant,
                key_data,
                primary_key_ref,
                table_name,
                ..
            } => {
                /*
                println!(
                    "Executing RelationalKey operation for model: {}",
                    model_discriminant.name()
                );
                */
                key_data.insert_relational_key_into_redb(&_wrapper.txn, &table_name, &primary_key_ref)?;
            }
            ConcreteOperationExecutor::HashTree {
                model_discriminant,
                hash,
                primary_key_ref,
                table_name,
                ..
            } => {
                /*
                println!(
                    "Executing HashTree operation for model: {}",
                    model_discriminant.name()
                );
                */
                D::ModelAssociatedTypes::insert_hash_into_redb(&hash, &_wrapper.txn, &table_name, &primary_key_ref)?;
            }
            ConcreteOperationExecutor::Delete {
                table_discriminant,
                primary_key,
                table_name,
                ..
            } => {
                /*
                println!(
                    "Executing Delete operation for table: {}",
                    table_discriminant.name()
                );
                */
                primary_key.delete_model_from_redb(&_wrapper.txn, &table_name)?;
            }
        }
        Ok(())
    }

    pub fn priority(&self) -> u8 {
        match self {
            ConcreteOperationExecutor::MainTree { priority, .. } => *priority,
            ConcreteOperationExecutor::SecondaryKey { priority, .. } => *priority,
            ConcreteOperationExecutor::RelationalKey { priority, .. } => *priority,
            ConcreteOperationExecutor::HashTree { priority, .. } => *priority,
            ConcreteOperationExecutor::Delete { priority, .. } => *priority,
        }
    }

    /// Get the table name for this operation
    pub fn table_name(&self) -> &str {
        match self {
            ConcreteOperationExecutor::MainTree { table_name, .. } => table_name,
            ConcreteOperationExecutor::SecondaryKey { table_name, .. } => table_name,
            ConcreteOperationExecutor::RelationalKey { table_name, .. } => table_name,
            ConcreteOperationExecutor::HashTree { table_name, .. } => table_name,
            ConcreteOperationExecutor::Delete { table_name, .. } => table_name,
        }
    }

    /// Get the model discriminant for this operation
    pub fn model_discriminant(&self) -> &D::Discriminant {
        match self {
            ConcreteOperationExecutor::MainTree {
                table_discriminant, ..
            } => table_discriminant,
            ConcreteOperationExecutor::SecondaryKey {
                model_discriminant, ..
            } => model_discriminant,
            ConcreteOperationExecutor::RelationalKey {
                model_discriminant, ..
            } => model_discriminant,
            ConcreteOperationExecutor::HashTree {
                model_discriminant, ..
            } => model_discriminant,
            ConcreteOperationExecutor::Delete {
                table_discriminant, ..
            } => table_discriminant,
        }
    }
}

pub struct RedbWriteTransaction<D: NetabaseDefinition>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    pub(crate) txn: redb::WriteTransaction,
    pub(crate) operation_queue: Vec<ConcreteOperationExecutor<D>>,
    pub(crate) redb_store: *const RedbStore<D>, // Store reference for model definitions
}

impl<D: NetabaseDefinition> RedbWriteTransaction<D>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <D as NetabaseDefinitionTrait>::Keys: IntoDiscriminant,
{
    pub fn new(txn: redb::WriteTransaction, redb_store: &RedbStore<D>) -> Self {
        Self {
            txn,
            operation_queue: Vec::new(),
            redb_store: redb_store as *const RedbStore<D>,
        }
    }

    /// Add an operation to the queue using the new enum system
    pub fn add_operation(&mut self, operation: ConcreteOperationExecutor<D>) {
        self.operation_queue.push(operation);
    }

    /// Process the operation queue in order
    fn process_queue(&mut self) -> NetabaseResult<()> {
        // Sort operations by priority: Main -> Secondary -> Relational -> Hash -> Delete
        self.operation_queue.sort_by_key(|op| op.priority());

        // Process all operations by draining the queue
        let operations = std::mem::take(&mut self.operation_queue);
        for operation in operations {
            operation.execute(self)?;
        }

        Ok(())
    }
}

impl<D: NetabaseDefinition> ReadTransaction<D> for RedbWriteTransaction<D>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <D as NetabaseDefinitionTrait>::Keys: IntoDiscriminant,
{
    fn get<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        key: M::PrimaryKey,
    ) -> NetabaseResult<Option<M>>
    where
        M::PrimaryKey: Key + 'static,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        M: for<'a> Value<SelfType<'a> = M>,
        Vec<u8>: TryFrom<
            <<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::PrimaryKey,
        >,
    {
        let definition: TableDefinition<M::PrimaryKey, M> =
            M::definition(unsafe { &*self.redb_store });
        let table = self.txn.open_table(definition)?;
        let result = table.get(key)?;
        Ok(result.map(|v| v.value()))
    }

    fn get_pk_by_secondary_key<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D>>(
        &self,
        secondary_key: <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum,
    ) -> NetabaseResult<Option<M::PrimaryKey>>
    where
        M::PrimaryKey: Key + 'static + for<'a> Value<SelfType<'a> = M::PrimaryKey>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: Key + 'static,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum: for<'a> Borrow<<<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as Value>::SelfType<'a>>,
        M: for<'a> Value<SelfType<'a> = M>,
    {
        use strum::IntoDiscriminant;

        // Get the discriminant to determine which secondary key table to use
        let discriminant = secondary_key.discriminant();
        let table_name = M::secondary_key_table_name(discriminant);

        // Open the secondary key table
        let table_def: TableDefinition<<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum, M::PrimaryKey> =
            TableDefinition::new(&table_name);
        let table = self.txn.open_table(table_def)?;

        // Look up the primary key
        let result = table.get(secondary_key)?;
        Ok(result.map(|v| v.value()))
    }
}

impl<D: NetabaseDefinition> WriteTransaction<D> for RedbWriteTransaction<D>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <D as NetabaseDefinitionTrait>::Keys: IntoDiscriminant,
{
    fn put<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D> + Send + Clone>(
        &mut self,
        model: M,
    ) -> NetabaseResult<()>
    where
        M::PrimaryKey: Key + 'static + Send + Clone + TryInto<Vec<u8>>,
        Vec<u8>: TryFrom<<<M as NetabaseModelTrait<D>>::Keys as NetabaseModelKeyTrait<D, M>>::PrimaryKey>,
        Vec<u8>: TryFrom<<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum>,
        Vec<u8>: TryFrom<<M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        M: for<'a> Borrow<<M as Value>::SelfType<'a>>,
    M::Hash: Clone + Into<[u8; 32]>, // Ensure Hash can convert to [u8; 32]
        M::SecondaryKeys: Iterator<Item = <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum>,
        <<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant: IntoEnumIterator + bincode::Encode,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum:
            IntoDiscriminant + Clone + Debug + Send + Key + TryInto<Vec<u8>>,
        <<M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum as IntoDiscriminant>::Discriminant:
            Debug,
        M::RelationalKeys:
            Iterator<Item = <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum>,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum:
            IntoDiscriminant + Clone + Debug + Send + Key + TryInto<Vec<u8>>,
        <<M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum as IntoDiscriminant>::Discriminant:
            Debug + bincode::Encode,
        <D as NetabaseDefinitionTrait>::Keys: IntoDiscriminant,
    {
        // Get tree configuration from TreeManager
        let _all_trees = D::all_trees();

        // Extract data before creating operations
        let pk = model.primary_key();
        let secondary_keys: Vec<_> =
            <M::Keys as NetabaseModelKeyTrait<D, M>>::secondary_keys(&model);
        let relational_keys: Vec<_> =
            <M::Keys as NetabaseModelKeyTrait<D, M>>::relational_keys(&model);

        // 1. Create Main Tree Operation directly using discriminant
        let table_discriminant = M::MODEL_TREE_NAME;
        let table_name = table_discriminant.name().to_string();

        // Use the ModelAssociatedTypes trait to wrap types properly
        let pk_wrapped = D::ModelAssociatedTypes::from_primary_key::<M>(pk.clone());
        let model_wrapped = D::ModelAssociatedTypes::from_model::<M>(model.clone());

        let main_operation = ConcreteOperationExecutor::MainTree {
            table_discriminant,
            primary_key: pk_wrapped,
            model_data: model_wrapped,
            table_name,
            priority: 0,
        };

        self.add_operation(main_operation);

        // 2. Queue Secondary Key Operations using discriminants
        for sk in secondary_keys {
            let sk_discriminant = sk.discriminant(); // Get discriminant before moving
            let sk_discriminant_for_wrap = sk.discriminant(); // Get another copy for wrapping

            // Use ModelAssociatedTypes to wrap the discriminant and data
            let sk_discriminant_wrapped = D::ModelAssociatedTypes::from_secondary_key::<M>(sk_discriminant_for_wrap);
            let sk_data_wrapped = D::ModelAssociatedTypes::from_secondary_key_data::<M>(sk);
            let pk_ref_wrapped = D::ModelAssociatedTypes::from_primary_key::<M>(pk.clone());

            // Generate table name using the new trait method
            let table_name = M::secondary_key_table_name(sk_discriminant);

            let secondary_operation = ConcreteOperationExecutor::SecondaryKey {
                model_discriminant: M::MODEL_TREE_NAME,
                key_discriminant: sk_discriminant_wrapped,
                key_data: sk_data_wrapped,
                primary_key_ref: pk_ref_wrapped,
                table_name,
                priority: 1,
            };

            self.add_operation(secondary_operation);
        }

        // 3. Queue Relational Key Operations using discriminants
        for rk in relational_keys {
            let rk_discriminant = rk.discriminant(); // Get discriminant before moving
            let rk_discriminant_for_wrap = rk.discriminant(); // Get another copy for wrapping

            // Use ModelAssociatedTypes to wrap the discriminant and data
            let rk_discriminant_wrapped = D::ModelAssociatedTypes::from_relational_key_discriminant::<M>(rk_discriminant_for_wrap);
            let rk_data_wrapped = D::ModelAssociatedTypes::from_relational_key_data::<M>(rk);
            let pk_ref_wrapped = D::ModelAssociatedTypes::from_primary_key::<M>(pk.clone());

            // Generate table name using the new trait method
            let table_name = M::relational_key_table_name(rk_discriminant);

            let relational_operation = ConcreteOperationExecutor::RelationalKey {
                model_discriminant: M::MODEL_TREE_NAME,
                key_discriminant: rk_discriminant_wrapped,
                key_data: rk_data_wrapped,
                primary_key_ref: pk_ref_wrapped,
                table_name,
                priority: 2,
            };

            self.add_operation(relational_operation);
        }

        // 4. Queue Hash Tree Operation using Blake3 hash
        let hash: [u8; 32] = model.compute_hash().into(); // Get Blake3 hash from model and convert to [u8; 32]
        let pk_ref_wrapped = D::ModelAssociatedTypes::from_primary_key::<M>(pk.clone());
        let table_name = M::hash_tree_table_name();

        let hash_operation = ConcreteOperationExecutor::HashTree {
            model_discriminant: M::MODEL_TREE_NAME,
            hash,
            primary_key_ref: pk_ref_wrapped,
            table_name,
            priority: 3,
        };

        self.add_operation(hash_operation);

        Ok(())
    }

    fn delete<M: NetabaseModelTrait<D> + RedbNetabaseModelTrait<D> + Clone + Send>(
        &mut self,
        key: M::PrimaryKey,
    ) -> NetabaseResult<()>
    where
        M::PrimaryKey: Key + 'static + Send + Clone + TryInto<Vec<u8>>,
        M::PrimaryKey: for<'a> Borrow<<M::PrimaryKey as Value>::SelfType<'a>>,
        M: for<'a> Borrow<<M as Value>::SelfType<'a>>,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::SecondaryEnum:
            IntoDiscriminant + Clone + Debug + Send + Key,
        <M::Keys as NetabaseModelKeyTrait<D, M>>::RelationalEnum:
            IntoDiscriminant + Clone + Debug + Send + Key,
        <D as NetabaseDefinitionTrait>::Keys: IntoDiscriminant,
    {
        let table_discriminant = M::MODEL_TREE_NAME;
        let table_name = table_discriminant.name().to_string();
        let pk_wrapped = D::ModelAssociatedTypes::from_primary_key::<M>(key);

        let delete_operation = ConcreteOperationExecutor::Delete {
            table_discriminant,
            primary_key: pk_wrapped,
            table_name,
            priority: 4,
        };

        self.add_operation(delete_operation);
        Ok(())
    }

    fn commit(mut self) -> NetabaseResult<()> {
        // Process the entire queue before committing
        self.process_queue()?;

        // Only commit when the queue is empty
        if self.operation_queue.is_empty() {
            self.txn.commit()?;
        } else {
            return Err(crate::error::NetabaseError::Other(
                "Operation queue not empty before commit".into(),
            ));
        }

        Ok(())
    }
}
