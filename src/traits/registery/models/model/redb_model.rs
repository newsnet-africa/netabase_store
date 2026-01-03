use redb::{MultimapTableDefinition, TableDefinition};
use strum::IntoDiscriminant;

use crate::traits::registery::{
    definition::redb_definition::RedbDefinition,
    models::keys::{NetabaseModelKeys, blob::NetabaseModelBlobKey},
};
use super::NetabaseModel;

#[derive(Clone)]
pub struct RedbModelTableDefinitions<'db, M: RedbNetbaseModel<'db, D>, D: RedbDefinition>
where
    D::Discriminant: 'static + std::fmt::Debug,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key + 'static,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key + 'static,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational:
        redb::Key + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: redb::Key + 'static,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
    M: 'db
{
    pub main: TableDefinition<'db, <M::Keys as NetabaseModelKeys<D, M>>::Primary, M::TableV>,
    pub main_name: &'db str,
    
    pub secondary: Vec<(
        MultimapTableDefinition<'db, <M::Keys as NetabaseModelKeys<D, M>>::Secondary, <M::Keys as NetabaseModelKeys<D, M>>::Primary>,
        &'db str
    )>,

    pub blob: Vec<(
        MultimapTableDefinition<'db, <M::Keys as NetabaseModelKeys<D, M>>::Blob, <<M::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem>,
        &'db str
    )>,
    
    pub relational: Vec<(
        MultimapTableDefinition<'db, <M::Keys as NetabaseModelKeys<D, M>>::Primary, <M::Keys as NetabaseModelKeys<D, M>>::Relational>,
        &'db str
    )>,

    pub subscription: Vec<(
        MultimapTableDefinition<'db, <M::Keys as NetabaseModelKeys<D, M>>::Subscription, <M::Keys as NetabaseModelKeys<D, M>>::Primary>,
        &'db str
    )>,
}

pub trait RedbNetbaseModel<'db, D: RedbDefinition>: NetabaseModel<D> + redb::Value + redb::Key
where
    D::Discriminant: 'static + std::fmt::Debug,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Primary:
        redb::Key + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Secondary:
        redb::Key + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Relational:
        redb::Key + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription:
        redb::Key + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob:
        redb::Key + 'static,
    <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob as NetabaseModelBlobKey<D, Self>>::BlobItem:
        redb::Key + 'static,
    <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Secondary as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Relational as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    Self: 'db
{
    type RedbTables;
    type TableV: redb::Value + redb::Key + 'static;

    fn table_definitions() -> RedbModelTableDefinitions<'db, Self, D> 
    where 
        D::Discriminant: 'static,
    {
        // Access the constant table names directly from TREE_NAMES in the supertrait
        let main = TableDefinition::new(Self::TREE_NAMES.main.table_name);
        let main_name = Self::TREE_NAMES.main.table_name;
        
        // Create secondary tables from the stored names
        let secondary = Self::TREE_NAMES.secondary
            .iter()
            .map(|disc_table| {
                let table_def = MultimapTableDefinition::new(disc_table.table_name);
                (table_def, disc_table.table_name)
            })
            .collect();
            
        // Create blob tables from the stored names
        let blob = Self::TREE_NAMES.blob
            .iter()
            .map(|disc_table| {
                let table_def = MultimapTableDefinition::new(disc_table.table_name);
                (table_def, disc_table.table_name)
            })
            .collect();
        
        // Create relational tables from the stored names
        let relational = Self::TREE_NAMES.relational
            .iter()
            .map(|disc_table| {
                let table_def = MultimapTableDefinition::new(disc_table.table_name);
                (table_def, disc_table.table_name)
            })
            .collect();

        // Create subscription tables from the stored names
        let subscription = match Self::TREE_NAMES.subscription {
             Some(subs) => subs.iter()
                .map(|disc_table| {
                    let table_def = MultimapTableDefinition::new(disc_table.table_name);
                    (table_def, disc_table.table_name)
                })
                .collect(),
             None => Vec::new(),
        };
        
        RedbModelTableDefinitions {
            main,
            main_name,
            secondary,
            blob,
            relational,
            subscription,
        }
    }

    fn main_definition()
    -> TableDefinition<'db, <Self::Keys as NetabaseModelKeys<D, Self>>::Primary, Self::TableV>
    {
        Self::table_definitions().main
    }

    fn secondary_definitions() -> Vec<(
        MultimapTableDefinition<'db, <Self::Keys as NetabaseModelKeys<D, Self>>::Secondary, <Self::Keys as NetabaseModelKeys<D, Self>>::Primary>,
        &'db str
    )> {
        Self::table_definitions().secondary
    }

    fn relational_definitions() -> Vec<(
        MultimapTableDefinition<'db, <Self::Keys as NetabaseModelKeys<D, Self>>::Primary, <Self::Keys as NetabaseModelKeys<D, Self>>::Relational>,
        &'db str
    )> {
        Self::table_definitions().relational
    }
}

pub struct ModelTableNames<'a, D: RedbDefinition, M: NetabaseModel<D>> 
where
    D::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug, 
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    pub main: crate::traits::registery::models::treenames::DiscriminantTableName<D::Discriminant>,
    pub secondary: &'a [crate::traits::registery::models::treenames::DiscriminantTableName<<<M::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant>],
    pub blob: &'a [crate::traits::registery::models::treenames::DiscriminantTableName<<<M::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant>],
    pub relational: &'a [crate::traits::registery::models::treenames::DiscriminantTableName<<<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant>],
    pub subscription: Option<&'a [crate::traits::registery::models::treenames::DiscriminantTableName<<<M::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant>]>,
}
