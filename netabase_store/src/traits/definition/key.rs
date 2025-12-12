use crate::traits::{definition::{NetabaseDefinition, DiscriminantName}, model::NetabaseModelTrait};
use std::fmt::Debug;

pub trait NetabaseDefinitionKeyTrait<D: NetabaseDefinition>
where
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
    fn inner<M: NetabaseModelTrait<D>>(&self) -> M::Keys
    where
        D: NetabaseDefinition,
        Self: TryInto<M::Keys>,
        <Self as TryInto<M::Keys>>::Error: std::fmt::Debug;
}
