use crate::traits::definition::{DiscriminantName, NetabaseDefinition};
use crate::traits::model::NetabaseModelTrait;
use std::fmt::Debug;
use strum::{IntoDiscriminant, IntoEnumIterator};

/// Represents a link to another model that can be either just the key (Unloaded)
/// or the fully hydrated model (Loaded)
#[derive(Debug, Clone)]
pub enum RelationalLink<M, D>
where
    M: NetabaseModelTrait<D>,
    D: NetabaseDefinition,
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    M::PrimaryKey: Clone,
{
    Unloaded(M::PrimaryKey),
    Loaded(M),
}

impl<M, D> RelationalLink<M, D>
where
    M: NetabaseModelTrait<D>,
    D: NetabaseDefinition,
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    M::PrimaryKey: Clone,
{
    pub fn key(&self) -> M::PrimaryKey {
        match self {
            RelationalLink::Unloaded(k) => k.clone(),
            RelationalLink::Loaded(m) => m.primary_key(),
        }
    }

    pub fn is_loaded(&self) -> bool {
        matches!(self, RelationalLink::Loaded(_))
    }

    pub fn model(&self) -> Option<&M> {
        match self {
            RelationalLink::Unloaded(_) => None,
            RelationalLink::Loaded(m) => Some(m),
        }
    }
    
    pub fn unwrap_model(self) -> M {
        match self {
            RelationalLink::Unloaded(_) => panic!("Called unwrap_model on Unloaded link"),
            RelationalLink::Loaded(m) => m,
        }
    }
}