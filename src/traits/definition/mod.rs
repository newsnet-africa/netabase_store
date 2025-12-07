use crate::traits::store::tree_manager::TreeManager;
use std::fmt::Debug;
use strum::{AsRefStr, IntoDiscriminant, IntoEnumIterator};

pub mod key;

/// Trait for converting discriminants to string names safely
/// Uses strum's AsRefStr for prefix matching support
pub trait DiscriminantName: AsRef<str> {
    fn name(&self) -> &'static str {
        self.as_ref()
    }
}

pub trait NetabaseDefinitionTrait: IntoDiscriminant
where
    <Self as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName,
{
    type Keys;
}

pub trait NetabaseDefinition: NetabaseDefinitionTrait + TreeManager<Self> + Sized
where
    <Self as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName,
{
}

impl<T: NetabaseDefinitionTrait + TreeManager<T>> NetabaseDefinition for T where
    <T as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName
{
}
