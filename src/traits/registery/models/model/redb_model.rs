use redb::{MultimapTableDefinition, TableDefinition};
use strum::IntoDiscriminant;

use crate::traits::registery::{
    definition::{NetabaseDefinition, redb_definition::RedbDefinition},
    models::keys::NetabaseModelKeys,
};
use super::NetabaseModel;

#[derive(Clone)]
pub struct RedbModelTableDefinitions<'db, M: RedbNetbaseModel<'db, D>, D: RedbDefinition>
where
    D::Discriminant: 'static + std::fmt::Debug,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: redb::Key + 'static,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: redb::Key + 'static,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>:
        redb::Key + 'static,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'db>: redb::Key + 'static,
    M: 'db + 'static
{
    pub main: TableDefinition<'db, <M::Keys as NetabaseModelKeys<D, M>>::Primary<'db>, M>,
    pub main_name: &'db str,
    
    pub secondary: Vec<(
        MultimapTableDefinition<'db, <M::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>, <M::Keys as NetabaseModelKeys<D, M>>::Primary<'db>>,
        &'db str
    )>,
    
    pub relational: Vec<(
        MultimapTableDefinition<'db, <M::Keys as NetabaseModelKeys<D, M>>::Relational<'db>, <M::Keys as NetabaseModelKeys<D, M>>::Primary<'db>>,
        &'db str
    )>,

    pub subscription: Vec<(
        MultimapTableDefinition<'db, <M::Keys as NetabaseModelKeys<D, M>>::Subscription<'db>, <M::Keys as NetabaseModelKeys<D, M>>::Primary<'db>>,
        &'db str
    )>,
}

pub trait RedbNetbaseModel<'db, D: RedbDefinition>: NetabaseModel<D> + redb::Value + redb::Key
where
    D::Discriminant: 'static + std::fmt::Debug,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Primary<'db>:
        redb::Key + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Secondary<'db>:
        redb::Key + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Relational<'db>:
        redb::Key + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription<'db>:
        redb::Key + 'static,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Secondary<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Relational<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription<'a> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    Self: 'db + 'static
{
    type RedbTables;

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
            relational,
            subscription,
        }
    }

    fn main_definition()
    -> TableDefinition<'db, <Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'db>, Self>
    {
        Self::table_definitions().main
    }

    fn secondary_definitions() -> Vec<(
        MultimapTableDefinition<'db, <Self::Keys as NetabaseModelKeys<D, Self>>::Secondary<'db>, <Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'db>>,
        &'db str
    )> {
        Self::table_definitions().secondary
    }

    fn relational_definitions() -> Vec<(
        MultimapTableDefinition<'db, <Self::Keys as NetabaseModelKeys<D, Self>>::Relational<'db>, <Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'db>>,
        &'db str
    )> {
        Self::table_definitions().relational
    }
}

pub struct ModelOpenTables<'a, D: RedbDefinition, M: NetabaseModel<D>> 
where
    D::Discriminant: 'static + std::fmt::Debug,
    for<'b> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'b> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'b> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'b> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug, 
    for<'b> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'b> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    pub main: crate::traits::registery::models::treenames::DiscriminantTableName<D::Discriminant>,
    pub secondary: &'a [crate::traits::registery::models::treenames::DiscriminantTableName<<<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant>],
    pub relational: &'a [crate::traits::registery::models::treenames::DiscriminantTableName<<<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant>],
    pub subscription: Option<&'a [crate::traits::registery::models::treenames::DiscriminantTableName<<<M::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant>]>,
}
