use strum::IntoEnumIterator;

pub trait NetabaseDefinition {
    type Keys: NetabaseDefinitionKeys;
    type Discriminants: NetabaseDefinitionDiscriminants;

    fn keys(&self) -> Self::Keys;
}
pub trait NetabaseDefinitionKeys {}
pub trait NetabaseDefinitionDiscriminants: IntoEnumIterator {}
