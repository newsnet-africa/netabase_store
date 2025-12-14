use strum::IntoDiscriminant;

pub trait NetabaseDefinition: IntoDiscriminant
where
    Self::Discriminant: 'static,
{
    type TreeNames: NetabaseDefinitionTreeNames;
}
pub trait NetabaseDefinitionTreeNames {}
