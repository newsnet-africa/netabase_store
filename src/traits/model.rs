use crate::traits::definition::NetabaseDefinition;

pub trait NetabaseModel
where
    Self::Defined: From<Self>,
    Self: TryInto<Self::Defined>,
{
    type Key: NetabaseModelKey;
    type Defined: NetabaseDefinition;

    const DISCRIMINANT: <<Self as NetabaseModel>::Defined as NetabaseDefinition>::Discriminants;

    fn key(&self) -> Self::Key;
}
pub trait NetabaseModelKey {
    type Model: NetabaseModel;
}
