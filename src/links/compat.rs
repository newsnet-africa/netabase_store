//! Backward compatibility layer for RelationalLink types
//!
//! This module provides a simpler compatibility approach by having the macro
//! generate appropriate type aliases in each model's scope.

/// Type alias for the old RelationalLink name - will be overridden by macro-generated aliases
/// This serves as a placeholder and will not actually be used in practice.
pub type RelationalLink<D, M> = crate::links::RelationalLink<D, M, PlaceholderRelations>;

/// Placeholder relation discriminant for the generic type alias
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlaceholderRelations;

impl crate::traits::relation::NetabaseRelationDiscriminant for PlaceholderRelations {
    fn field_name(&self) -> &'static str {
        unreachable!("PlaceholderRelations should never be instantiated")
    }

    fn target_model_name(&self) -> &'static str {
        unreachable!("PlaceholderRelations should never be instantiated")
    }

    fn all_variants() -> Vec<Self> {
        vec![]
    }
}

impl crate::netabase_deps::strum::IntoDiscriminant for PlaceholderRelations {
    type Discriminant = PlaceholderRelationsDiscriminant;

    fn discriminant(&self) -> Self::Discriminant {
        unreachable!("PlaceholderRelations should never be instantiated")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlaceholderRelationsDiscriminant;

impl crate::netabase_deps::bincode::Encode for PlaceholderRelations {
    fn encode<E: crate::netabase_deps::bincode::enc::Encoder>(
        &self,
        _encoder: &mut E,
    ) -> Result<(), crate::netabase_deps::bincode::error::EncodeError> {
        unreachable!("PlaceholderRelations should never be instantiated")
    }
}

impl crate::netabase_deps::bincode::Decode<()> for PlaceholderRelations {
    fn decode<D: crate::netabase_deps::bincode::de::Decoder<Context = ()>>(
        _decoder: &mut D,
    ) -> Result<Self, crate::netabase_deps::bincode::error::DecodeError> {
        unreachable!("PlaceholderRelations should never be instantiated")
    }
}

impl crate::netabase_deps::bincode::Encode for PlaceholderRelationsDiscriminant {
    fn encode<E: crate::netabase_deps::bincode::enc::Encoder>(
        &self,
        _encoder: &mut E,
    ) -> Result<(), crate::netabase_deps::bincode::error::EncodeError> {
        unreachable!("PlaceholderRelationsDiscriminant should never be instantiated")
    }
}

impl crate::netabase_deps::bincode::Decode<()> for PlaceholderRelationsDiscriminant {
    fn decode<D: crate::netabase_deps::bincode::de::Decoder<Context = ()>>(
        _decoder: &mut D,
    ) -> Result<Self, crate::netabase_deps::bincode::error::DecodeError> {
        unreachable!("PlaceholderRelationsDiscriminant should never be instantiated")
    }
}
