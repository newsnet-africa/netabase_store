//! Batch operations for Redb backend
//!
//! This module provides efficient batch operations for bulk inserts and removes.

use crate::error::NetabaseError;
use crate::traits::convert::ToIVec;
use crate::traits::definition::NetabaseDefinitionTrait;
use crate::traits::model::{NetabaseModelTrait, NetabaseModelTraitKey};

use redb::{Database, MultimapTableDefinition, ReadableTable, TableDefinition};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;
use strum::IntoDiscriminant;

use super::types::BincodeWrapper;

/// Batch operation type for Redb
pub(crate) enum RedbBatchOp<D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
    <D as IntoDiscriminant>::Discriminant: crate::DiscriminantBounds,
{
    Put(M),
    Remove(<M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey),
}

/// BatchBuilder implementation for Redb
///
/// Collects operations to be executed in a single transaction for better performance.
pub struct RedbBatchBuilder<D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
    <D as IntoDiscriminant>::Discriminant: crate::DiscriminantBounds,
{
    pub(crate) db: Arc<Database>,
    pub(crate) table_name: &'static str,
    pub(crate) secondary_table_name: &'static str,
    pub(crate) operations: Vec<RedbBatchOp<D, M>>,
    pub(crate) _phantom_d: PhantomData<D>,
}

impl<D, M> RedbBatchBuilder<D, M>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + ToIVec + From<M>,
    M: NetabaseModelTrait<D>
        + TryFrom<D>
        + Into<D>
        + Clone
        + Debug
        + bincode::Encode
        + bincode::Decode<()>,
    M::Keys: Debug + bincode::Decode<()> + Ord + Clone + bincode::Encode + PartialEq,
    <D as IntoDiscriminant>::Discriminant: crate::DiscriminantBounds,
{
    /// Create a new batch builder
    pub(crate) fn new(
        db: Arc<Database>,
        table_name: &'static str,
        secondary_table_name: &'static str,
    ) -> Self {
        Self {
            db,
            table_name,
            secondary_table_name,
            operations: Vec::new(),
            _phantom_d: PhantomData,
        }
    }

    fn table_def(&self) -> TableDefinition<'static, BincodeWrapper<M::Keys>, BincodeWrapper<M>> {
        TableDefinition::new(self.table_name)
    }

    fn secondary_table_def(
        &self,
    ) -> MultimapTableDefinition<
        'static,
        <M::Keys as NetabaseModelTraitKey<D>>::SecondaryKey,
        <M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey,
    > {
        MultimapTableDefinition::new(self.secondary_table_name)
    }
}

impl<D, M> crate::traits::batch::BatchBuilder<D, M> for RedbBatchBuilder<D, M>
where
    D: NetabaseDefinitionTrait + TryFrom<M> + ToIVec + From<M>,
    M: NetabaseModelTrait<D>
        + TryFrom<D>
        + Into<D>
        + Clone
        + Debug
        + bincode::Encode
        + bincode::Decode<()>,
    M::Keys: Debug + bincode::Decode<()> + Ord + Clone + bincode::Encode + PartialEq,
    <D as IntoDiscriminant>::Discriminant: crate::DiscriminantBounds,
{
    fn put(&mut self, model: M) -> Result<(), NetabaseError> {
        self.operations.push(RedbBatchOp::Put(model));
        Ok(())
    }

    fn remove(
        &mut self,
        key: <M::Keys as NetabaseModelTraitKey<D>>::PrimaryKey,
    ) -> Result<(), NetabaseError> {
        self.operations.push(RedbBatchOp::Remove(key));
        Ok(())
    }

    fn commit(self) -> Result<(), NetabaseError> {
        if self.operations.is_empty() {
            return Ok(());
        }

        let table_def = self.table_def();
        let sec_table_def = self.secondary_table_def();

        // Begin write transaction
        let write_txn = self.db.as_ref().begin_write()?;

        {
            let mut table = write_txn.open_table(table_def)?;
            let mut sec_table = write_txn.open_multimap_table(sec_table_def)?;

            for op in self.operations {
                match op {
                    RedbBatchOp::Put(model) => {
                        let primary_key = model.primary_key();
                        let secondary_keys = model.secondary_keys();
                        let wrapped_key = M::Keys::from(primary_key.clone());

                        // Insert model into primary table
                        table.insert(&wrapped_key, &model)?;

                        // Insert secondary key entries: SecondaryKey -> PrimaryKey
                        if !secondary_keys.is_empty() {
                            for sec_key in secondary_keys.values() {
                                sec_table.insert(sec_key.clone(), primary_key.clone())?;
                            }
                        }
                    }
                    RedbBatchOp::Remove(key) => {
                        // Wrap key in M::Keys enum for redb operations
                        let wrapped_key = M::Keys::from(key.clone());

                        // First get the model to extract secondary keys
                        let secondary_keys = if let Some(model_guard) = table.get(&wrapped_key)? {
                            let model: M = model_guard.value();
                            model.secondary_keys()
                        } else {
                            std::collections::HashMap::new()
                        };

                        // Remove from primary table
                        table.remove(&wrapped_key)?;

                        // Remove secondary key entries: SecondaryKey -> PrimaryKey
                        if !secondary_keys.is_empty() {
                            for sec_key in secondary_keys.values() {
                                sec_table.remove(sec_key.clone(), key.clone())?;
                            }
                        }
                    }
                }
            }
        }

        write_txn.commit()?;
        Ok(())
    }
}
