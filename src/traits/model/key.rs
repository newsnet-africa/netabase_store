use crate::traits::{
    definition::{DiscriminantName, NetabaseDefinition},
    model::NetabaseModelTrait,
};
use std::fmt::Debug;
use strum::{AsRefStr, IntoDiscriminant, IntoEnumIterator};

pub trait NetabaseModelKeyTrait<D: NetabaseDefinition, M: NetabaseModelTrait<D>>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName,
{
    type SecondaryEnum: IntoDiscriminant + Clone + Debug + Send + AsRef<str> + DiscriminantName;
    type RelationalEnum: IntoDiscriminant + Clone + Debug + Send + AsRef<str> + DiscriminantName;

    fn secondary_keys(model: &M) -> Vec<Self::SecondaryEnum>;
    fn relational_keys(model: &M) -> Vec<Self::RelationalEnum>;
}
