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
{
    type PrimaryKey: bincode::Decode<()> + bincode::Encode + TryInto<Vec<u8>> + TryFrom<Vec<u8>>;
    type SecondaryEnum: IntoDiscriminant
        + Clone
        + Debug
        + Send
        + bincode::Decode<()>
        + bincode::Encode
        + TryInto<Vec<u8>>
        + TryFrom<Vec<u8>>;
    type RelationalEnum: IntoDiscriminant
        + Clone
        + Debug
        + Send
        + bincode::Decode<()>
        + bincode::Encode
        + TryInto<Vec<u8>>
        + TryFrom<Vec<u8>>;

    fn secondary_keys(model: &M) -> Vec<Self::SecondaryEnum>;
    fn relational_keys(model: &M) -> Vec<Self::RelationalEnum>;
}
