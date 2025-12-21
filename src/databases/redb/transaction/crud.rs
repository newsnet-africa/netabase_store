use redb::{self, ReadableTable, Value};
use strum::IntoDiscriminant;
use std::borrow::Borrow;

use crate::{
    errors::{NetabaseError, NetabaseResult}, traits::registery::{
        definition::redb_definition::RedbDefinition,
        models::{
            StoreKey, keys::{NetabaseModelKeys, blob::NetabaseModelBlobKey}, model::{NetabaseModel, redb_model::RedbNetbaseModel}
        },
    }
};
use super::tables::{ModelOpenTables, TablePermission, ReadWriteTableType, TableType};

/// Trait to handle automatic insertion/update of models into their respective tables
pub trait RedbModelCrud<'db,  D>: RedbNetbaseModel<'db, D>
where
    D: RedbDefinition + Clone,
    <D as IntoDiscriminant>::Discriminant: std::fmt::Debug + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Primary<'db>: redb::Key + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Secondary<'db>: redb::Key + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Relational<'db>: redb::Key + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob<'db>: redb::Key + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription<'db>: redb::Key + 'static,
    D::SubscriptionKeys: redb::Key + 'static,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Secondary<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Relational<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug + Copy + PartialEq<<<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription<'static> as IntoDiscriminant>::Discriminant>,
    // Add missing static bound
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription<'db>: 'static,
    for<'a> <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob<'a>: IntoDiscriminant,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob<'a> as NetabaseModelBlobKey<'a, D, Self, <Self as NetabaseModel<D>>::Keys>>::BlobTypes: redb::Key + 'static + std::borrow::Borrow<<<<<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob<'a> as NetabaseModelBlobKey<'a, D, Self, <Self as NetabaseModel<D>>::Keys>>::BlobTypes as redb::Value>::SelfType<'a>>,
    Self: 'db, <Self as NetabaseModel<D>>::Keys: 'static
{
    fn create_entry<'txn>(
        &'db self,
        tables: &mut ModelOpenTables<'txn, 'db, D, Self>
    ) -> NetabaseResult<()>;

    fn read_entry<'txn>(
        key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'db>,
        tables: &ModelOpenTables<'txn, 'db, D, Self>
    ) -> NetabaseResult<Option<Self>>;

    fn update_entry<'txn>(
        &'db self,
        tables: &mut ModelOpenTables<'txn, 'db, D, Self>
    ) -> NetabaseResult<()>;

    fn delete_entry<'txn>(
        key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'db>,
        tables: &mut ModelOpenTables<'txn, 'db, D, Self>
    ) -> NetabaseResult<()>;
}

impl<'db, D, M> RedbModelCrud<'db, D> for M
where
    D: RedbDefinition + Clone,
    M: RedbNetbaseModel<'db, D> + Clone,
    D::Discriminant: 'static + std::fmt::Debug,
    <D as IntoDiscriminant>::Discriminant: std::fmt::Debug,
    for<'a> M::TableV: redb::Value<SelfType<'a> = M>,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: redb::Key + Clone + 'static,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db> as redb::Value>::SelfType<'a>>,
    
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: redb::Key + Clone + 'static + PartialEq,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db> as redb::Value>::SelfType<'a>>,
    // Add Into bound to allow bridging lifetimes
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'a>: Into<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>>,
    
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: redb::Key + Clone + 'static + PartialEq,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db> as redb::Value>::SelfType<'a>>,
    // Add Into bound to allow bridging lifetimes
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'a>: Into<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>>,
    
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'db>: redb::Key + Clone + 'static + PartialEq,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'db>: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'db> as redb::Value>::SelfType<'a>>,
    // Add Into bound to allow bridging lifetimes
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'a>: Into<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'db>>,

    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'db>: redb::Key + Clone + 'static + PartialEq,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'db>: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'db> as redb::Value>::SelfType<'a>>,
    // Add Into bound to allow bridging lifetimes
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'a>: Into<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'db>>,

    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug + Copy + PartialEq<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'static> as IntoDiscriminant>::Discriminant>,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'db>: 'static,
    for<'a> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'a>: IntoDiscriminant,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'a> as NetabaseModelBlobKey<'a, D, M, <M as NetabaseModel<D>>::Keys>>::BlobTypes: redb::Key + 'static + std::borrow::Borrow<<<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'a> as NetabaseModelBlobKey<'a, D, M, <M as NetabaseModel<D>>::Keys>>::BlobTypes as redb::Value>::SelfType<'a>>,
    D::SubscriptionKeys: redb::Key + Clone + 'static + PartialEq,
    for<'a> D::SubscriptionKeys: std::borrow::Borrow<<D::SubscriptionKeys as redb::Value>::SelfType<'a>>,
    M: 'db,
    for<'a> &'a <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: std::borrow::Borrow<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db> as redb::Value>::SelfType<'a>>,
    <M as NetabaseModel<D>>::Keys:'static,
    // Fix complex borrow bounds for BlobTypes
    // for<'a> &'a <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'db> as NetabaseModelBlobKey<'db, D, M, <M as NetabaseModel<D>>::Keys>>::BlobTypes:
    // Borrow<<<<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'db> as NetabaseModelBlobKey<'db, D, M, <M as NetabaseModel<D>>::Keys>>::BlobTypes as Value>::SelfType<'db>>,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>: StoreKey<D, <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'db> as NetabaseModelBlobKey<'db, D, M, <M as NetabaseModel<D>>::Keys>>::BlobTypes>
{
        fn create_entry<'txn>(
            &'db self,
            tables: &mut ModelOpenTables<'txn, 'db, D, Self>
        ) -> NetabaseResult<()> 
        {        // 1. Insert into Main Table
        match &mut tables.main {
            TablePermission::ReadWrite(ReadWriteTableType::Table(table)) => {
                table.insert(self.get_primary_key().borrow(), self.borrow())
                    .map_err(|e| NetabaseError::RedbError(e.into()))?;
            }
            _ => return Err(NetabaseError::Other),
        }

        // 2. Insert into Secondary Tables
        let secondary_keys = self.get_secondary_keys();
        for ((table_perm, _name), key) in tables.secondary.iter_mut().zip(secondary_keys.into_iter()) {
             match table_perm {
                 TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                     let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db> = key.into();
                     table.insert(k.borrow(), self.get_primary_key().borrow())
                         .map_err(|e| NetabaseError::RedbError(e.into()))?;
                 }
                 _ => return Err(NetabaseError::Other),
             }
        }

        // 3. Insert into Blob Tables
        let blobs = self.get_blobs();
        for (key, value) in blobs {
             let key_discriminant = <_ as IntoDiscriminant>::discriminant(&key);
             if let Some(index) = M::TREE_NAMES.blob.iter().position(|d| d.discriminant == key_discriminant) {
                 if let Some((table_perm, _name)) = tables.blob.get_mut(index) {
                     match table_perm {
                         TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                             let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'db> = key.into();
                             table.insert(k.borrow(), value.borrow())
                                 .map_err(|e| NetabaseError::RedbError(e.into()))?;
                         }
                         _ => return Err(NetabaseError::Other),
                     }
                 }
             }
        }

        // 4. Insert into Relational Tables
        let relational_keys = self.get_relational_keys();
        let primary_key = self.get_primary_key();
        for ((table_perm, _name), key) in tables.relational.iter_mut().zip(relational_keys.into_iter()) {
             match table_perm {
                 TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                     let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db> = key.into();
                     table.insert(primary_key.borrow(), k.borrow())
                         .map_err(|e| NetabaseError::RedbError(e.into()))?;
                 }
                 _ => return Err(NetabaseError::Other),
             }
        }

        // 5. Insert into Subscription Tables
        let subscription_keys = self.get_subscription_keys();
        for key in subscription_keys {
             let model_key: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'db> = key.into();
             let key_discriminant = <_ as IntoDiscriminant>::discriminant(&model_key);
             
             if let Some(definitions) = M::TREE_NAMES.subscription {
                 if let Some(index) = definitions.iter().position(|d| key_discriminant == d.discriminant) {
                     if let Some((table_perm, _name)) = tables.subscription.get_mut(index) {
                         match table_perm {
                             TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                                 let def_key: D::SubscriptionKeys = model_key.try_into().map_err(|_| NetabaseError::Other)?;
                                 table.insert(def_key.borrow(), self.get_primary_key().borrow())
                                     .map_err(|e| NetabaseError::RedbError(e.into()))?;
                             }
                             _ => return Err(NetabaseError::Other),
                         }
                     }
                 }
             }
        }


        Ok(())
    }

    fn read_entry<'txn>(
        key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'db>,
        tables: &ModelOpenTables<'txn, 'db, D, Self>
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
            _ => Err(NetabaseError::Other),
        }
    }

    fn update_entry<'txn>(
        &'db self,
        tables: &mut ModelOpenTables<'txn, 'db, D, Self>
    ) -> NetabaseResult<()>
    {
        // 1. Update Main Table and get old model in one operation
        let old_model = match &mut tables.main {
            TablePermission::ReadWrite(ReadWriteTableType::Table(table)) => {
                table.insert(self.get_primary_key().borrow(), self)
                    .map_err(|e| NetabaseError::RedbError(e.into()))?
                    .map(|access_guard| access_guard.value())
            }
            _ => return Err(NetabaseError::Other),
        };

        let primary_key = self.get_primary_key();

        if let Some(old_model) = old_model {
            // Model existed, update secondary/relational/subscription tables

            // 2. Update Secondary Tables
            let old_secondary = old_model.get_secondary_keys();
            let new_secondary = self.get_secondary_keys();

            for (((table_perm, _name), old_key), new_key) in tables.secondary.iter_mut()
                .zip(old_secondary.into_iter())
                .zip(new_secondary.into_iter())
            {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        let old_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db> = old_key.into();
                        let new_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db> = new_key.into();

                        if old_k != new_k {
                            table.remove(old_k.borrow(), primary_key.borrow())
                                .map_err(|e| NetabaseError::RedbError(e.into()))?;
                            table.insert(new_k.borrow(), primary_key.borrow())
                                .map_err(|e| NetabaseError::RedbError(e.into()))?;
                        }
                    }
                    _ => return Err(NetabaseError::Other),
                }
            }

            // 3. Update Blob Tables
            let old_blobs = old_model.get_blobs();
            for (key, _val) in old_blobs {
                 let key_discriminant = <_ as IntoDiscriminant>::discriminant(&key);
                 if let Some(index) = M::TREE_NAMES.blob.iter().position(|d| d.discriminant == key_discriminant) {
                     if let Some((table_perm, _name)) = tables.blob.get_mut(index) {
                         match table_perm {
                             TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                                 let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'db> = key.into();
                                 table.remove_all(k.borrow())
                                     .map_err(|e| NetabaseError::RedbError(e.into()))?;
                             }
                             _ => return Err(NetabaseError::Other),
                         }
                     }
                 }
            }
            
            let new_blobs = self.get_blobs();
            for (key, value) in new_blobs {
                 let key_discriminant = <_ as IntoDiscriminant>::discriminant(&key);
                 if let Some(index) = M::TREE_NAMES.blob.iter().position(|d| d.discriminant == key_discriminant) {
                     if let Some((table_perm, _name)) = tables.blob.get_mut(index) {
                         match table_perm {
                             TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                                 let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'db> = key.into();
                                 table.insert(k.borrow(), value.borrow())
                                     .map_err(|e| NetabaseError::RedbError(e.into()))?;
                             }
                             _ => return Err(NetabaseError::Other),
                         }
                     }
                 }
            }

            // 4. Update Relational Tables
            let old_relational = old_model.get_relational_keys();
            let new_relational = self.get_relational_keys();

            for (((table_perm, _name), old_key), new_key) in tables.relational.iter_mut()
                .zip(old_relational.into_iter())
                .zip(new_relational.into_iter())
            {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        let old_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db> = old_key.into();
                        let new_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db> = new_key.into();

                        if old_k != new_k {
                            table.remove(primary_key.borrow(), old_k.borrow())
                                .map_err(|e| NetabaseError::RedbError(e.into()))?;
                            table.insert(primary_key.borrow(), new_k.borrow())
                                .map_err(|e| NetabaseError::RedbError(e.into()))?;
                        }
                    }
                    _ => return Err(NetabaseError::Other),
                }
            }

            // 5. Update Subscription Tables
            let old_subscription = old_model.get_subscription_keys();
            for key in old_subscription {
                 let model_key: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'db> = key.into();
                 let key_discriminant = <_ as IntoDiscriminant>::discriminant(&model_key);
                 
                 if let Some(definitions) = M::TREE_NAMES.subscription {
                     if let Some(index) = definitions.iter().position(|d| key_discriminant == d.discriminant) {
                         if let Some((table_perm, _name)) = tables.subscription.get_mut(index) {
                             match table_perm {
                                 TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                                     let def_key: D::SubscriptionKeys = model_key.try_into().map_err(|_| NetabaseError::Other)?;
                                     table.remove(def_key.borrow(), primary_key.borrow())
                                         .map_err(|e| NetabaseError::RedbError(e.into()))?;
                                 }
                                 _ => return Err(NetabaseError::Other),
                             }
                         }
                     }
                 }
            }
            
            let new_subscription = self.get_subscription_keys();
            for key in new_subscription {
                 let model_key: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'db> = key.into();
                 let key_discriminant = <_ as IntoDiscriminant>::discriminant(&model_key);
                 
                 if let Some(definitions) = M::TREE_NAMES.subscription {
                     if let Some(index) = definitions.iter().position(|d| key_discriminant == d.discriminant) {
                         if let Some((table_perm, _name)) = tables.subscription.get_mut(index) {
                             match table_perm {
                                 TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                                     let def_key: D::SubscriptionKeys = model_key.try_into().map_err(|_| NetabaseError::Other)?;
                                     table.insert(def_key.borrow(), primary_key.borrow())
                                         .map_err(|e| NetabaseError::RedbError(e.into()))?;
                                 }
                                 _ => return Err(NetabaseError::Other),
                             }
                         }
                     }
                 }
            }

        } else {
            // Model didn't exist before, insert all
            // 1. Secondary
            let secondary_keys = self.get_secondary_keys();
            for ((table_perm, _name), key) in tables.secondary.iter_mut().zip(secondary_keys.into_iter()) {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db> = key.into();
                        table.insert(k.borrow(), primary_key.borrow())
                            .map_err(|e| NetabaseError::RedbError(e.into()))?;
                    }
                    _ => return Err(NetabaseError::Other),
                }
            }

            // 2. Blobs
            let blobs = self.get_blobs();
            for (key, value) in blobs {
                 let key_discriminant = <_ as IntoDiscriminant>::discriminant(&key);
                 if let Some(index) = M::TREE_NAMES.blob.iter().position(|d| d.discriminant == key_discriminant) {
                     if let Some((table_perm, _name)) = tables.blob.get_mut(index) {
                         match table_perm {
                             TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                                 let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'db> = key.into();
                                 table.insert(k.borrow(), value.borrow())
                                     .map_err(|e| NetabaseError::RedbError(e.into()))?;
                             }
                             _ => return Err(NetabaseError::Other),
                         }
                     }
                 }
            }

            // 3. Relational
            let relational_keys = self.get_relational_keys();
            for ((table_perm, _name), key) in tables.relational.iter_mut().zip(relational_keys.into_iter()) {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db> = key.into();
                        table.insert(primary_key.borrow(), k.borrow())
                            .map_err(|e| NetabaseError::RedbError(e.into()))?;
                    }
                    _ => return Err(NetabaseError::Other),
                }
            }

            // 4. Subscription
            let subscription_keys = self.get_subscription_keys();
            for key in subscription_keys {
                 let model_key: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'db> = key.into();
                 let key_discriminant = <_ as IntoDiscriminant>::discriminant(&model_key);
                 
                 if let Some(definitions) = M::TREE_NAMES.subscription {
                     if let Some(index) = definitions.iter().position(|d| key_discriminant == d.discriminant) {
                         if let Some((table_perm, _name)) = tables.subscription.get_mut(index) {
                             match table_perm {
                                 TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                                     let def_key: D::SubscriptionKeys = model_key.try_into().map_err(|_| NetabaseError::Other)?;
                                     table.insert(def_key.borrow(), primary_key.borrow())
                                         .map_err(|e| NetabaseError::RedbError(e.into()))?;
                                 }
                                 _ => return Err(NetabaseError::Other),
                             }
                         }
                     }
                 }
            }
        }

        Ok(())
    }

    fn delete_entry<'txn>(
        key: &<Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'db>,
        tables: &mut ModelOpenTables<'txn, 'db, D, Self>
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
                _ => return Err(NetabaseError::Other),
            }

            // 3. Remove from Secondary Tables
            let secondary_keys = model.get_secondary_keys();
            for ((table_perm, _name), secondary_key) in tables.secondary.iter_mut().zip(secondary_keys.into_iter()) {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db> = secondary_key.into();
                        table.remove(k.borrow(), key.borrow())
                            .map_err(|e| NetabaseError::RedbError(e.into()))?;
                    }
                    _ => return Err(NetabaseError::Other),
                }
            }

            // 4. Remove from Blob Tables
            let blobs = model.get_blobs();
            for (key, _val) in blobs {
                 let key_discriminant = <_ as IntoDiscriminant>::discriminant(&key);
                 if let Some(index) = M::TREE_NAMES.blob.iter().position(|d| d.discriminant == key_discriminant) {
                     if let Some((table_perm, _name)) = tables.blob.get_mut(index) {
                         match table_perm {
                             TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                                 let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob<'db> = key.into();
                                 table.remove_all(k.borrow())
                                     .map_err(|e| NetabaseError::RedbError(e.into()))?;
                             }
                             _ => return Err(NetabaseError::Other),
                         }
                     }
                 }
            }

            // 5. Remove from Relational Tables
            let relational_keys = model.get_relational_keys();
            for ((table_perm, _name), relational_key) in tables.relational.iter_mut().zip(relational_keys.into_iter()) {
                match table_perm {
                    TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                        let k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db> = relational_key.into();
                        table.remove(key.borrow(), k.borrow())
                            .map_err(|e| NetabaseError::RedbError(e.into()))?;
                    }
                    _ => return Err(NetabaseError::Other),
                }
            }

            // 6. Remove from Subscription Tables
            let subscription_keys = model.get_subscription_keys();
            for subscription_key in subscription_keys {
                 let model_k: <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'db> = subscription_key.into();
                 let key_discriminant = <_ as IntoDiscriminant>::discriminant(&model_k);
                 
                 if let Some(definitions) = M::TREE_NAMES.subscription {
                     if let Some(index) = definitions.iter().position(|d| key_discriminant == d.discriminant) {
                         if let Some((table_perm, _name)) = tables.subscription.get_mut(index) {
                             match table_perm {
                                 TablePermission::ReadWrite(ReadWriteTableType::MultimapTable(table)) => {
                                     let def_k: D::SubscriptionKeys = model_k.try_into().map_err(|_| NetabaseError::Other)?;
                                     table.remove(def_k.borrow(), key.borrow())
                                         .map_err(|e| NetabaseError::RedbError(e.into()))?;
                                 }
                                 _ => return Err(NetabaseError::Other),
                             }
                         }
                     }
                 }
            }
        }

        Ok(())
    }
}