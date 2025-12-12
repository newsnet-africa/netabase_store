use crate::traits::definition::{DiscriminantName, NetabaseDefinition};
use crate::traits::model::NetabaseModelTrait;
use std::fmt::Debug;
use std::marker::PhantomData;
use strum::{IntoDiscriminant, IntoEnumIterator};

/// Represents a link to another model that can be either just the key (Unloaded)
/// or the fully hydrated model (Loaded)
#[derive(Debug, Clone)]
pub enum RelationalLink<D: NetabaseDefinition, M: NetabaseModelTrait<D>>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    M::PrimaryKey: Clone,
{
    Unloaded(M::PrimaryKey, D::Discriminant),
    Loaded(M),
}

impl<D: NetabaseDefinition, M: NetabaseModelTrait<D>> RelationalLink<D, M>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    M::PrimaryKey: Clone,
{
    pub fn key(&self) -> M::PrimaryKey {
        match self {
            RelationalLink::Unloaded(k, _) => k.clone(),
            RelationalLink::Loaded(m) => m.primary_key(),
        }
    }

    pub fn is_loaded(&self) -> bool {
        matches!(self, RelationalLink::Loaded(_))
    }

    pub fn model(&self) -> Option<&M> {
        match self {
            RelationalLink::Unloaded(_, _) => None,
            RelationalLink::Loaded(m) => Some(m),
        }
    }

    pub fn unwrap_model(self) -> M {
        match self {
            RelationalLink::Unloaded(_, _) => panic!("Called unwrap_model on Unloaded link"),
            RelationalLink::Loaded(m) => m,
        }
    }
}
