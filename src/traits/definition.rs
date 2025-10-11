use strum::IntoEnumIterator;

pub trait NetabaseDefinition {
    type Keys: NetabaseDefinitionKeys;
    type Discriminants: NetabaseDefinitionDiscriminants + PartialEq + Eq + std::hash::Hash;

    fn keys(&self) -> Self::Keys;
}
pub trait NetabaseDefinitionKeys {}
pub trait NetabaseDefinitionDiscriminants:
    IntoEnumIterator + bincode::Decode<()> + bincode::Encode
{
}
