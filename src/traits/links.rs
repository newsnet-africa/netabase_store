//! Traits for link-aware storage operations
//!
//! This module defines traits that extend the basic StoreOps with support
//! for automatic handling of RelationalLink fields using the new type-safe
//! relation system.

use crate::{
    error::NetabaseError,
    links::RelationalLink,
    traits::{
        definition::NetabaseDefinitionTrait,
        model::NetabaseModelTrait,
        store_ops::{OpenTree, StoreOps},
    },
};

/// Extension trait for StoreOps that provides link-aware operations
pub trait LinkedStoreOps<D, M>: StoreOps<D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
    <D as strum::IntoDiscriminant>::Discriminant: crate::traits::definition::NetabaseDiscriminant,
{
    /// Insert a model and all its linked entities recursively
    ///
    /// This method will:
    /// 1. Identify all RelationalLink fields in the model
    /// 2. For each field containing an Entity variant, recursively insert the linked entity
    /// 3. Finally insert the main model
    ///
    /// # Arguments
    /// * `model` - The model to insert with its links
    ///
    /// # Returns
    /// Result indicating success or failure of the operation
    fn put_with_links(&self, model: M) -> Result<(), NetabaseError> {
        // This is a default implementation that just inserts the model
        // The actual link handling is done by the generated code in the derive macro
        self.put_raw(model)
    }

    /// Check if a model has any linked entities that need insertion
    fn has_entity_links(&self, _model: &M) -> bool {
        // Default implementation assumes no links
        // This will be overridden by generated implementations
        false
    }
}

/// Blanket implementation for all StoreOps
impl<D, M, T> LinkedStoreOps<D, M> for T
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
    T: StoreOps<D, M>,
    <D as strum::IntoDiscriminant>::Discriminant: crate::traits::definition::NetabaseDiscriminant,
{
    // Uses default implementations from the trait
}

/// Helper trait for stores that can handle multiple model types
pub trait MultiModelStore<D: NetabaseDefinitionTrait> {
    /// Insert a model that might have links to other model types
    fn insert_with_cross_links<M>(&self, model: M) -> Result<(), NetabaseError>
    where
        M: NetabaseModelTrait<D> + Clone,
        Self: OpenTree<D, M>;
}

/// Marker trait to indicate that a model supports automatic link insertion
pub trait AutoInsertLinks<D: NetabaseDefinitionTrait>: NetabaseModelTrait<D> {
    /// Insert this model and all linked entities
    fn auto_insert<S>(&self, store: &S) -> Result<(), NetabaseError>
    where
        S: OpenTree<D, Self>,
        Self: Clone;
}

/// Helper functions for working with RelationalLink fields
pub mod link_utils {
    use super::*;

    /// Check if a RelationalLink contains an Entity variant
    pub fn is_entity<D, M>(link: &RelationalLink<D, M>) -> bool
    where
        D: NetabaseDefinitionTrait,
        M: NetabaseModelTrait<D>,
    {
        matches!(link, RelationalLink::Entity(_))
    }

    /// Extract the entity from a RelationalLink if it's an Entity variant
    pub fn extract_entity<D, M>(link: &RelationalLink<D, M>) -> Option<&M>
    where
        D: NetabaseDefinitionTrait,
        M: NetabaseModelTrait<D>,
    {
        match link {
            RelationalLink::Entity(entity) => Some(entity),
            RelationalLink::Reference(_) => None,
        }
    }

    /// Extract the key from a RelationalLink regardless of variant
    pub fn extract_key<D, M>(link: &RelationalLink<D, M>) -> M::PrimaryKey
    where
        D: NetabaseDefinitionTrait,
        M: NetabaseModelTrait<D>,
    {
        match link {
            RelationalLink::Entity(entity) => entity.primary_key(),
            RelationalLink::Reference(key) => key.clone(),
        }
    }
}
