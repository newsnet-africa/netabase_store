use crate::traits::definition::NetabaseDefinition;

pub trait NetabaseModel: bincode::Decode<()> + bincode::Encode
where
    Self::Defined: From<Self>,
    Self: TryInto<Self::Defined>,
{
    type Key: NetabaseModelKey;
    type Defined: NetabaseDefinition;

    const DISCRIMINANT: <<Self as NetabaseModel>::Defined as NetabaseDefinition>::Discriminants;

    fn key(&self) -> Self::Key;
}
pub trait NetabaseModelKey: bincode::Decode<()> + bincode::Encode {
    type Model: NetabaseModel;
}
