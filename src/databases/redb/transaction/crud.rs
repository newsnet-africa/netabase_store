use redb::{self, ReadableTable};
use strum::IntoDiscriminant;
use std::borrow::Borrow;

use crate::{
    traits::registery::{
        definition::redb_definition::RedbDefinition,
        models::{
            keys::NetabaseModelKeys,
            model::{NetabaseModel, redb_model::RedbNetbaseModel},
        },
    },
    errors::{NetabaseResult, NetabaseError},
};
use super::tables::{ModelOpenTables, TablePermission, ReadWriteTableType, TableType};

/// Trait to handle automatic insertion/update of models into their respective tables
pub trait RedbModelCrud<'db, 'data, D>: RedbNetbaseModel<'data, D>
where
    D: RedbDefinition + Clone,
    <D as IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Primary<'data>: redb::Key + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Secondary<'data>: redb::Key + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Relational<'data>: redb::Key + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription<'data>: redb::Key + 'static,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Secondary<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Relational<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    // Add missing static bound
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription<'data>: 'static,
    Self: 'static
{
    fn create_entry<'txn>(
        &'data self,
        tables: &mut ModelOpenTables<'txn, 'data, D, Self>
    ) -> NetabaseResult<()>;

    fn read_entry<'txn>(
        key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'data>,
        tables: &ModelOpenTables<'txn, 'data, D, Self>
    ) -> NetabaseResult<Option<Self>>;

    fn update_entry<'txn>(
        &'data self,
        tables: &mut ModelOpenTables<'txn, 'data, D, Self>
    ) -> NetabaseResult<()>;

    fn delete_entry<'txn>(
        key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'data>,
        tables: &mut ModelOpenTables<'txn, 'data, D, Self>
    ) -> NetabaseResult<()>;
}

impl<'db, 'data, D, M> RedbModelCrud<'db, 'data, D> for M
where
    D: RedbDefinition + Clone,
    M: RedbNetbaseModel<'data, D> + Clone,
    D::Discriminant: 'static + std::fmt::Debug,
    <D as IntoDiscriminant>::Discriminant: std::fmt::Debug,
    M: std::borrow::Borrow<<M as redb::Value>::SelfType<'data>>,
    for<'a> M: redb::Value<SelfType<'a> = M>,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'data>: redb::Key + Clone + 'static,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'data>: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'data> as redb::Value>::SelfType<'a>>,
    
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'data>: redb::Key + Clone + 'static + PartialEq,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'data>: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'data> as redb::Value>::SelfType<'a>>,
    // Add Into bound to allow bridging lifetimes
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'a>: Into<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'data>>,
    
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'data>: redb::Key + Clone + 'static + PartialEq,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'data>: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'data> as redb::Value>::SelfType<'a>>,
    // Add Into bound to allow bridging lifetimes
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'a>: Into<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'data>>,
    
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'data>: redb::Key + Clone + 'static + PartialEq,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'data>: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'data> as redb::Value>::SelfType<'a>>,
    // Add Into bound to allow bridging lifetimes
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'a>: Into<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'data>>,

    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'data>: 'static,
    M: 'static,
    for<'a> &'a <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'data>: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'data> as redb::Value>::SelfType<'a>>
{
    fn create_entry<'txn>(
        &'data self,
        tables: &mut ModelOpenTables<'txn, 'data, D, Self>
    ) -> NetabaseResult<()> 
    {
        // 1. Insert into Main Table
        match &mut tables.main {
            TablePermission::ReadWrite(ReadWriteTableType::Table(table)) => {
                table.insert(self.get_primary_key().borrow(), self)
                    .map_err(|e| NetabaseError::RedbError(e.into()))?;
            }
            _ => return Err(NetabaseError::Permission),
        }

        // 2. Insert into Secondary Tables
        let secondary_keys = self.get_secondary_keys();
        for ((table_perm, _name), key) in tables.secondary.iter_mut().zip(secondary_keys.into_iter()) {
             match table_perm {
                 TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                     // key is Secondary<'local> (from self via get_secondary_keys)
                     // self is 'data, so key is Secondary<'data>
                     // so key.into() works trivially
                     let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'data> = key.into();
                     table.insert(k.borrow(), self.get_primary_key().borrow())
                         .map_err(|e| NetabaseError::RedbError(e.into()))?;
                 }
                 _ => return Err(NetabaseError::Permission),
             }
        }

        // 3. Insert into Relational Tables
        // TODO: Add permission checks before inserting relational keys
        // For each relational key, we should:
        // 1. Determine the target model discriminant from the relational key
        // 2. Check Self::PERMISSIONS.can_access_model(target_discriminant, AccessType::Create)
        // 3. Return PermissionDenied error if not allowed
        // This requires a mapping from relational key discriminants to target model discriminants
        let relational_keys = self.get_relational_keys();
        for ((table_perm, _name), key) in tables.relational.iter_mut().zip(relational_keys.into_iter()) {
             match table_perm {
                 TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                     let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'data> = key.into();
                     table.insert(k.borrow(), self.get_primary_key().borrow())
                         .map_err(|e| NetabaseError::RedbError(e.into()))?;
                 }
                 _ => return Err(NetabaseError::Permission),
             }
        }

        // 4. Insert into Subscription Tables
        let subscription_keys = self.get_subscription_keys();
        for ((table_perm, _name), key) in tables.subscription.iter_mut().zip(subscription_keys.into_iter()) {
             match table_perm {
                 TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                     let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'data> = key.into();
                     table.insert(k.borrow(), self.get_primary_key().borrow())
                         .map_err(|e| NetabaseError::RedbError(e.into()))?;
                 }
                 _ => return Err(NetabaseError::Permission),
             }
        }

        Ok(())
    }

    fn read_entry<'txn>(
        key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'data>,
        tables: &ModelOpenTables<'txn, 'data, D, Self>
    ) -> NetabaseResult<Option<Self>>
    {
        match &tables.main {
            TablePermission::ReadOnly(TableType::Table(table)) => {
                let result = table.get(key.borrow()).map_err(|e| NetabaseError::RedbError(e.into()))?;
                Ok(result.map(|access_guard| access_guard.value()))
            },
            TablePermission::ReadWrite(ReadWriteTableType::Table(table)) => {
                let result = table.get(key.borrow()).map_err(|e| NetabaseError::RedbError(e.into()))?;
                Ok(result.map(|access_guard| access_guard.value()))
            },
            _ => Err(NetabaseError::Permission),
        }
    }

    fn update_entry<'txn>(
        &'data self,
        tables: &mut ModelOpenTables<'txn, 'data, D, Self>
    ) -> NetabaseResult<()>
    {
        // 1. Get the old model
        let old_model = Self::read_entry(&self.get_primary_key(), tables)?;
        
        // 2. Update Main Table
        match &mut tables.main {
            TablePermission::ReadWrite(ReadWriteTableType::Table(table)) => {
                table.insert(self.get_primary_key().borrow(), self)
                    .map_err(|e| NetabaseError::RedbError(e.into()))?;
            }
            _ => return Err(NetabaseError::Permission),
        }

        if let Some(old_model) = old_model {
            let primary_key = self.get_primary_key();

            // 3. Update Secondary Tables
            let old_secondary = old_model.get_secondary_keys();
            let new_secondary = self.get_secondary_keys();
            
            for (((table_perm, _name), old_key), new_key) in tables.secondary.iter_mut()
                .zip(old_secondary.into_iter())
                .zip(new_secondary.into_iter()) 
            {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        // Convert keys to 'data lifetime to satisfy Borrow bound and PartialEq
                        let old_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'data> = old_key.into();
                        let new_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'data> = new_key.into();
                        
                        if old_k != new_k {
                            table.remove(old_k.borrow(), primary_key.borrow())
                                .map_err(|e| NetabaseError::RedbError(e.into()))?;
                            table.insert(new_k.borrow(), primary_key.borrow())
                                .map_err(|e| NetabaseError::RedbError(e.into()))?;
                        }
                    }
                    _ => return Err(NetabaseError::Permission),
                }
            }

            // 4. Update Relational Tables
            // TODO: Add permission checks before updating relational keys
            // Should check permissions for both removing old relations and creating new ones
            // Similar to create_entry, requires mapping relational key discriminants to target models
            let old_relational = old_model.get_relational_keys();
            let new_relational = self.get_relational_keys();

            for (((table_perm, _name), old_key), new_key) in tables.relational.iter_mut()
                .zip(old_relational.into_iter())
                .zip(new_relational.into_iter())
            {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        let old_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'data> = old_key.into();
                        let new_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'data> = new_key.into();

                        if old_k != new_k {
                            table.remove(old_k.borrow(), primary_key.borrow())
                                .map_err(|e| NetabaseError::RedbError(e.into()))?;
                            table.insert(new_k.borrow(), primary_key.borrow())
                                .map_err(|e| NetabaseError::RedbError(e.into()))?;
                        }
                    }
                    _ => return Err(NetabaseError::Permission),
                }
            }

            // 5. Update Subscription Tables
            let old_subscription = old_model.get_subscription_keys();
            let new_subscription = self.get_subscription_keys();

            for (((table_perm, _name), old_key), new_key) in tables.subscription.iter_mut()
                .zip(old_subscription.into_iter())
                .zip(new_subscription.into_iter())
            {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        let old_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'data> = old_key.into();
                        let new_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'data> = new_key.into();

                        if old_k != new_k {
                            table.remove(old_k.borrow(), primary_key.borrow())
                                .map_err(|e| NetabaseError::RedbError(e.into()))?;
                            table.insert(new_k.borrow(), primary_key.borrow())
                                .map_err(|e| NetabaseError::RedbError(e.into()))?;
                        }
                    }
                    _ => return Err(NetabaseError::Permission),
                }
            }
        } else {
            self.create_entry(tables)?;
        }

        Ok(())
    }

    fn delete_entry<'txn>(
        key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'data>,
        tables: &mut ModelOpenTables<'txn, 'data, D, Self>
    ) -> NetabaseResult<()>
    {
        let model = Self::read_entry(key, tables)?;

        if let Some(model) = model {
            // 2. Remove from Main Table
            match &mut tables.main {
                TablePermission::ReadWrite(ReadWriteTableType::Table(table)) => {
                    table.remove(key.borrow())
                        .map_err(|e| NetabaseError::RedbError(e.into()))?;
                }
                _ => return Err(NetabaseError::Permission),
            }

            // 3. Remove from Secondary Tables
            let secondary_keys = model.get_secondary_keys();
            for ((table_perm, _name), secondary_key) in tables.secondary.iter_mut().zip(secondary_keys.into_iter()) {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'data> = secondary_key.into();
                        table.remove(k.borrow(), key.borrow())
                            .map_err(|e| NetabaseError::RedbError(e.into()))?;
                    }
                    _ => return Err(NetabaseError::Permission),
                }
            }

            // 4. Remove from Relational Tables
            // TODO: Add permission checks before deleting relational keys
            // Should verify model has permission to modify relations with target models
            // Requires mapping relational key discriminants to target model discriminants
            let relational_keys = model.get_relational_keys();
            for ((table_perm, _name), relational_key) in tables.relational.iter_mut().zip(relational_keys.into_iter()) {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'data> = relational_key.into();
                        table.remove(k.borrow(), key.borrow())
                            .map_err(|e| NetabaseError::RedbError(e.into()))?;
                    }
                    _ => return Err(NetabaseError::Permission),
                }
            }

            // 5. Remove from Subscription Tables
            let subscription_keys = model.get_subscription_keys();
            for ((table_perm, _name), subscription_key) in tables.subscription.iter_mut().zip(subscription_keys.into_iter()) {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'data> = subscription_key.into();
                        table.remove(k.borrow(), key.borrow())
                            .map_err(|e| NetabaseError::RedbError(e.into()))?;
                    }
                    _ => return Err(NetabaseError::Permission),
                }
            }
        }

        Ok(())
    }
}
