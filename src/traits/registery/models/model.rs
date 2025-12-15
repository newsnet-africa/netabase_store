use redb::{MultimapTableDefinition, TableDefinition};
use strum::IntoDiscriminant;

use crate::{
    traits::registery::{
        definition::NetabaseDefinition,
        models::{StoreKey, StoreValue, StoreValueMarker, keys::NetabaseModelKeys},
    },
    relational::{RelationalLink, RelationalLinkError},
};

pub trait NetabaseModelMarker: StoreValueMarker {}

pub trait NetabaseModel<D: NetabaseDefinition>:
    NetabaseModelMarker
    + Sized
    + for<'a> StoreValue<D, <Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'a>>
where
    D::Discriminant: 'static,
    for<'a> <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Primary<'a>:
        StoreKey<D, Self>,
    for<'a> <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Secondary<'a>:
        IntoDiscriminant,
    for<'a> <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Relational<'a>:
        IntoDiscriminant,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Secondary<'a> as IntoDiscriminant>::Discriminant: 'static,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Relational<'a> as IntoDiscriminant>::Discriminant: 'static,
{
    type Keys: NetabaseModelKeys<D, Self>;

    fn get_primary_key<'a>(&'a self) -> <Self::Keys as NetabaseModelKeys<D, Self>>::Primary<'a>;
    fn get_secondary_keys<'a>(
        &'a self,
    ) -> Vec<<Self::Keys as NetabaseModelKeys<D, Self>>::Secondary<'a>>;
    fn get_relational_keys<'a>(
        &'a self,
    ) -> Vec<<Self::Keys as NetabaseModelKeys<D, Self>>::Relational<'a>>;

    /// Get all relational links from this model
    fn get_relational_links(&self) -> Vec<Box<dyn std::any::Any>> {
        Vec::new() // Default implementation returns empty
    }
    
    /// Update a relational link by type (default implementation does nothing)
    fn update_relational_link<OD: NetabaseDefinition, M: NetabaseModel<OD>, FK>(
        &mut self, 
        _link: RelationalLink<OD, M, FK>
    ) -> Result<(), RelationalLinkError> 
    where
        FK: Clone + Send + Sync + PartialEq + 'static,
        <OD as strum::IntoDiscriminant>::Discriminant: 'static,
        for<'a> <<<M as NetabaseModel<OD>>::Keys as NetabaseModelKeys<OD, M>>::Secondary<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
        for<'a> <<<M as NetabaseModel<OD>>::Keys as NetabaseModelKeys<OD, M>>::Relational<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
    {
        Ok(()) // Default implementation does nothing
    }

    /// Check if this model has any relational links
    fn has_relational_links(&self) -> bool {
        !self.get_relational_links().is_empty()
    }

    /// Get the number of relational links in this model
    fn relational_link_count(&self) -> usize {
        self.get_relational_links().len()
    }
}

pub struct RedbModelTableDefinitions<'db, M: RedbNetbaseModel<'db, D>, D: NetabaseDefinition>
where
    D::Discriminant: 'static,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary<'db>: redb::Key + 'static,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'db>: redb::Key + 'static,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'db>:
        redb::Key + 'static,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant: 'static,
    for<'a> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant: 'static,
    M: 'db + 'static,
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
}

pub trait RedbNetbaseModel<'db, D: NetabaseDefinition>: NetabaseModel<D> + redb::Value + redb::Key
where
    D::Discriminant: 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Primary<'db>:
        redb::Key + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Secondary<'db>:
        redb::Key + 'static,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Relational<'db>:
        redb::Key + 'static,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Secondary<'a> as IntoDiscriminant>::Discriminant: 'static,
    for<'a> <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Relational<'a> as IntoDiscriminant>::Discriminant: 'static,
    Self: 'db + 'static,
{
    const TREE_NAMES: ModelTreeNames<'db, D, Self>;

    type RedbTables;

    fn table_definitions() -> RedbModelTableDefinitions<'db, Self, D> 
    where 
        D::Discriminant: 'static,
    {
        // Access the constant table names directly from TREE_NAMES
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
        
        RedbModelTableDefinitions {
            main,
            main_name,
            secondary,
            relational,
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

/// A tuple that stores a discriminant alongside its formatted table name
/// Format: "{Definition}:{Model}:{KeyType}:{TableName}" in PascalCase
#[derive(Debug, Clone)]
pub struct DiscriminantTableName<D> {
    pub discriminant: D,
    pub table_name: &'static str, // Use &'static str for const contexts
}

impl<D> DiscriminantTableName<D> {
    pub const fn new(discriminant: D, table_name: &'static str) -> Self {
        Self {
            discriminant,
            table_name,
        }
    }
}

pub struct ModelTreeNames<'a, D: NetabaseDefinition, M>
where
    D::Discriminant: 'static,
    M: NetabaseModel<D>,
    for<'b> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'b>: IntoDiscriminant,
    for<'b> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'b>: IntoDiscriminant,
    for<'b> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'b> as IntoDiscriminant>::Discriminant: 'static,
    for<'b> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'b> as IntoDiscriminant>::Discriminant: 'static,
{
    pub main: DiscriminantTableName<D::Discriminant>,
    pub secondary: &'a [DiscriminantTableName<<<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant>],
    pub relational: &'a [DiscriminantTableName<<<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant>],
}

pub struct ModelOpenTables<'a, D: NetabaseDefinition, M: NetabaseModel<D>> 
where
    D::Discriminant: 'static,
    for<'b> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'b> as IntoDiscriminant>::Discriminant: 'static,
    for<'b> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'b> as IntoDiscriminant>::Discriminant: 'static,
{
    pub main: DiscriminantTableName<D::Discriminant>,
    pub secondary: &'a [DiscriminantTableName<<<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant>],
    pub relational: &'a [DiscriminantTableName<<<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant>],
}
