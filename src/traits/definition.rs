use strum::{IntoDiscriminant, IntoEnumIterator};

pub trait NetabaseDefinition: IntoDiscriminant
where
    <Self as IntoDiscriminant>::Discriminant: NetabaseDefinitionDiscriminant
        + PartialEq
        + Eq
        + std::hash::Hash
        + Send
        + Sync
        + Clone
        + Copy
        + 'static,
{
    type Keys: NetabaseDefinitionKeys<Discriminants = <Self as IntoDiscriminant>::Discriminant>;

    fn keys(&self) -> Self::Keys;
}
pub trait NetabaseDefinitionKeys {
    type Discriminants: NetabaseDefinitionDiscriminant;

    /// Get the discriminant for this definition key
    fn definition_discriminant(&self) -> Self::Discriminants;
}
pub trait NetabaseDefinitionDiscriminant:
    IntoEnumIterator + bincode::Decode<()> + bincode::Encode + Send + Sync + Clone + Copy + 'static
{
}
