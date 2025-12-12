use crate::traits::{
    definition::{DiscriminantName, NetabaseDefinition},
    model::NetabaseModelTrait,
};
use std::fmt::Debug;
use strum::{IntoDiscriminant, IntoEnumIterator};

pub trait NetabaseModelKeyTrait<D: NetabaseDefinition, M: NetabaseModelTrait<D>>
where
    <D as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone,
    <Self::SecondaryEnum as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone + PartialEq,
    <Self::RelationalEnum as IntoDiscriminant>::Discriminant:
        IntoEnumIterator + std::hash::Hash + Eq + Debug + DiscriminantName + Clone + PartialEq,
{
    type PrimaryKey: bincode::Decode<()>
        + bincode::Encode
        + TryInto<Vec<u8>>
        + TryFrom<Vec<u8>>
        + 'static;
    type SecondaryEnum: IntoDiscriminant
        + IntoEnumIterator
        + Clone
        + Debug
        + Send
        + bincode::Decode<()>
        + bincode::Encode
        + TryInto<Vec<u8>>
        + TryFrom<Vec<u8>>
        + 'static;
    type RelationalEnum: IntoDiscriminant
        + IntoEnumIterator
        + Clone
        + Debug
        + Send
        + bincode::Decode<()>
        + bincode::Encode
        + TryInto<Vec<u8>>
        + TryFrom<Vec<u8>>
        + 'static;

    fn as_self_secondary(s: Self::SecondaryEnum) -> Self;
    fn as_self_relational(s: Self::RelationalEnum) -> Self;
    fn as_self_primary(s: Self::PrimaryKey) -> Self;

    fn secondary_keys(model: &M) -> Vec<Self::SecondaryEnum>;
    fn relational_keys(model: &M) -> Vec<Self::RelationalEnum>;
    fn all_keys(model: &M) -> Vec<Self>
    where
        Self: std::marker::Sized,
    {
        let mut sec = Self::secondary_keys(model)
            .iter()
            .map(|s| Self::as_self_secondary(s.clone()))
            .collect::<Vec<Self>>();

        sec.extend(
            Self::relational_keys(model)
                .iter()
                .map(|r| Self::as_self_relational(r.clone())),
        );

        sec
    }
}
