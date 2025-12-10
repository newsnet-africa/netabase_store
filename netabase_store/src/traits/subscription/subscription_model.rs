use crate::traits::{definition::{NetabaseDefinition, DiscriminantName}, model::NetabaseModelTrait};
use std::fmt::Debug;

pub trait SubscribedModel<D: NetabaseDefinition>: NetabaseModelTrait<D>
where
    <D as strum::IntoDiscriminant>::Discriminant: strum::IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
{
}