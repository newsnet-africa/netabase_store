use crate::traits::registery::{
    definition::NetabaseDefinition,
    models::{keys::NetabaseModelKeys, model::NetabaseModel},
};
use strum::IntoDiscriminant;

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

#[derive(Clone, Debug)]
pub struct ModelTreeNames<'a, D: NetabaseDefinition, M>
where
    D::Discriminant: 'static + std::fmt::Debug,
    M: NetabaseModel<D>,
    for<'b> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'b>: IntoDiscriminant,
    for<'b> <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'b>: IntoDiscriminant,
    for<'b> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary<'b> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'b> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational<'b> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    for<'b> <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription<'b> as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    pub main: DiscriminantTableName<D::Discriminant>,
    pub secondary: &'a [DiscriminantTableName<<<M::Keys as NetabaseModelKeys<D, M>>::Secondary<'a> as IntoDiscriminant>::Discriminant>],
    pub relational: &'a [DiscriminantTableName<<<M::Keys as NetabaseModelKeys<D, M>>::Relational<'a> as IntoDiscriminant>::Discriminant>],
    pub subscription: Option<&'a [DiscriminantTableName<<<M::Keys as NetabaseModelKeys<D, M>>::Subscription<'a> as IntoDiscriminant>::Discriminant>]>,
}
