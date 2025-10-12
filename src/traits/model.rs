use strum::IntoDiscriminant;

use crate::traits::definition::{
    NetabaseDefinition, NetabaseDefinitionDiscriminant, NetabaseDefinitionKeys,
};
use std::hash::Hash;

pub trait NetabaseModel: bincode::Decode<()> + bincode::Encode
where
    Self::Defined: From<Self>,
    Self: TryInto<Self::Defined>,
    <<Self as NetabaseModel>::Defined as strum::IntoDiscriminant>::Discriminant:
        NetabaseDefinitionDiscriminant,
    <<Self as NetabaseModel>::Defined as strum::IntoDiscriminant>::Discriminant: Eq + Hash,
{
    type Key: NetabaseModelKey;
    type Defined: NetabaseDefinition;

    const DISCRIMINANT: <<Self as NetabaseModel>::Defined as IntoDiscriminant>::Discriminant;

    fn key(&self) -> Self::Key;
}
pub trait NetabaseModelKey: bincode::Decode<()> + bincode::Encode
where
    Self: TryFrom<Self::DefinedKeys>,
{
    type DefinedKeys: NetabaseDefinitionKeys + From<Self>;
    const DISCRIMINANT:
        <<Self as NetabaseModelKey>::DefinedKeys as NetabaseDefinitionKeys>::Discriminants;
}
