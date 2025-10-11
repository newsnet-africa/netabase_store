use strum::IntoEnumIterator;

pub trait NetabaseDefinition {
    type Keys: NetabaseDefinitionKeys<Discriminants = Self::Discriminants>;
    type Discriminants: NetabaseDefinitionDiscriminants
        + PartialEq
        + Eq
        + std::hash::Hash
        + Send
        + Sync
        + Clone
        + Copy
        + 'static;

    fn keys(&self) -> Self::Keys;
}
pub trait NetabaseDefinitionKeys {
    type Discriminants: NetabaseDefinitionDiscriminants;

    /// Get the discriminant for this definition key
    fn discriminant(&self) -> Self::Discriminants;
}
pub trait NetabaseDefinitionDiscriminants:
    IntoEnumIterator + bincode::Decode<()> + bincode::Encode + Send + Sync + Clone + Copy + 'static
{
}
