//! Relational Links Module
//!
//! This module provides type-safe relational linking between models using
//! generated relation enums for better type safety and consistency with
//! the existing netabase patterns.

pub mod compat;

use crate::{
    NetabaseDefinitionTrait, NetabaseModelTrait, error::NetabaseError, store_ops::StoreOps,
    traits::store_ops::OpenTree,
};

// Re-export the relation types for convenience
pub use crate::traits::relation::{
    MultiModelStore, NetabaseRelationDiscriminant, NetabaseRelationTrait, RelationLink,
    relation_utils,
};

// Re-export compatibility type alias for backward compatibility
pub use compat::RelationalLink as LegacyRelationalLink;

/// A type-safe relational link that uses the model's relation discriminant
///
/// This enum provides a clean way to represent relationships between models,
/// allowing either direct entity embedding or lazy loading via primary key references.
///
/// # Type Parameters
///
/// * `D` - The netabase definition trait
/// * `M` - The target model type
/// * `R` - The relation discriminant enum for type safety
///
/// # Examples
///
/// ```ignore
/// use netabase_store::links::RelationalLink;
///
/// // Direct entity embedding
/// let author_link = RelationalLink::Entity(user);
///
/// // Lazy loading via reference
/// let author_link = RelationalLink::Reference(user_id);
///
/// // Hydration (loading the entity if it's a reference)
/// let author = author_link.hydrate(store)?;
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum RelationalLink<D, M, R>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
    R: NetabaseRelationDiscriminant,
{
    /// A reference to an entity via its primary key
    Reference(M::PrimaryKey),
    /// A full entity instance
    Entity(M),
    /// Phantom marker to carry the relation type information
    _RelationMarker(std::marker::PhantomData<R>),
}

impl<D, M, R> From<M> for RelationalLink<D, M, R>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
    R: NetabaseRelationDiscriminant,
{
    fn from(value: M) -> Self {
        RelationalLink::Entity(value)
    }
}

impl<D, M, R> RelationalLink<D, M, R>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
    R: NetabaseRelationDiscriminant,
{
    /// Create a reference link from a primary key
    pub fn from_key(key: M::PrimaryKey) -> Self {
        RelationalLink::Reference(key)
    }

    /// Create an entity link from a model instance
    pub fn from_entity(entity: M) -> Self {
        RelationalLink::Entity(entity)
    }

    /// Hydrate this link, returning the entity if it's an Entity variant,
    /// or loading it from the store if it's a Reference variant
    pub fn hydrate<T: StoreOps<D, M>>(self, store: &T) -> Result<Option<M>, NetabaseError> {
        match self {
            RelationalLink::Reference(key) => store.get_raw(key),
            RelationalLink::Entity(model) => Ok(Some(model)),
            RelationalLink::_RelationMarker(_) => {
                unreachable!("RelationMarker should never be instantiated")
            }
        }
    }

    /// Get the primary key regardless of whether this is a Reference or Entity
    pub fn key(&self) -> M::PrimaryKey {
        match self {
            RelationalLink::Reference(key) => key.clone(),
            RelationalLink::Entity(entity) => entity.primary_key(),
            RelationalLink::_RelationMarker(_) => {
                unreachable!("RelationMarker should never be instantiated")
            }
        }
    }

    /// Check if this link contains an Entity variant
    pub fn is_entity(&self) -> bool {
        matches!(self, RelationalLink::Entity(_))
    }

    /// Check if this link contains a Reference variant
    pub fn is_reference(&self) -> bool {
        matches!(self, RelationalLink::Reference(_))
    }

    /// Extract the entity if this is an Entity variant
    pub fn as_entity(&self) -> Option<&M> {
        match self {
            RelationalLink::Entity(entity) => Some(entity),
            RelationalLink::Reference(_) => None,
            RelationalLink::_RelationMarker(_) => {
                unreachable!("RelationMarker should never be instantiated")
            }
        }
    }

    /// Extract the reference key if this is a Reference variant
    pub fn as_reference(&self) -> Option<&M::PrimaryKey> {
        match self {
            RelationalLink::Reference(key) => Some(key),
            RelationalLink::Entity(_) => None,
            RelationalLink::_RelationMarker(_) => {
                unreachable!("RelationMarker should never be instantiated")
            }
        }
    }

    /// Convert this link to a reference, extracting the key
    pub fn to_reference(self) -> RelationalLink<D, M, R> {
        match self {
            RelationalLink::Reference(key) => RelationalLink::Reference(key),
            RelationalLink::Entity(entity) => RelationalLink::Reference(entity.primary_key()),
            RelationalLink::_RelationMarker(_) => {
                unreachable!("RelationMarker should never be instantiated")
            }
        }
    }
}

// Manual implementation of bincode traits for proper serialization with generic context
impl<D, M, R> bincode::Encode for RelationalLink<D, M, R>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D> + bincode::Encode,
    R: NetabaseRelationDiscriminant,
    M::PrimaryKey: bincode::Encode,
{
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        match self {
            RelationalLink::Reference(key) => {
                0u8.encode(encoder)?;
                key.encode(encoder)
            }
            RelationalLink::Entity(entity) => {
                1u8.encode(encoder)?;
                entity.encode(encoder)
            }
            RelationalLink::_RelationMarker(_) => {
                unreachable!("RelationMarker should never be instantiated")
            }
        }
    }
}

impl<D, M, R, Context> bincode::Decode<Context> for RelationalLink<D, M, R>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D> + bincode::Decode<Context>,
    R: NetabaseRelationDiscriminant,
    M::PrimaryKey: bincode::Decode<Context>,
{
    fn decode<De: bincode::de::Decoder<Context = Context>>(
        decoder: &mut De,
    ) -> Result<Self, bincode::error::DecodeError> {
        let variant = u8::decode(decoder)?;
        match variant {
            0 => Ok(RelationalLink::Reference(M::PrimaryKey::decode(decoder)?)),
            1 => Ok(RelationalLink::Entity(M::decode(decoder)?)),
            _ => Err(bincode::error::DecodeError::Other(
                "Invalid RelationalLink variant".into(),
            )),
        }
    }
}

impl<'a, D, M, R, Context> bincode::BorrowDecode<'a, Context> for RelationalLink<D, M, R>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D> + bincode::Decode<Context>,
    R: NetabaseRelationDiscriminant,
    M::PrimaryKey: bincode::Decode<Context>,
{
    fn borrow_decode<De: bincode::de::BorrowDecoder<'a>>(
        decoder: &mut De,
    ) -> Result<Self, bincode::error::DecodeError>
    where
        De: bincode::de::Decoder<Context = Context>,
    {
        <Self as bincode::Decode<Context>>::decode(decoder)
    }
}

// Manual implementation of serde traits for proper serialization
impl<D, M, R> serde::Serialize for RelationalLink<D, M, R>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D> + serde::Serialize,
    R: NetabaseRelationDiscriminant,
    M::PrimaryKey: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        match self {
            RelationalLink::Reference(key) => {
                let mut state = serializer.serialize_struct("RelationalLink", 2)?;
                state.serialize_field("type", "Reference")?;
                state.serialize_field("key", key)?;
                state.end()
            }
            RelationalLink::Entity(entity) => {
                let mut state = serializer.serialize_struct("RelationalLink", 2)?;
                state.serialize_field("type", "Entity")?;
                state.serialize_field("entity", entity)?;
                state.end()
            }
            RelationalLink::_RelationMarker(_) => {
                unreachable!("RelationMarker should never be instantiated")
            }
        }
    }
}

impl<'de, D, M, R> serde::Deserialize<'de> for RelationalLink<D, M, R>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D> + serde::Deserialize<'de>,
    R: NetabaseRelationDiscriminant,
    M::PrimaryKey: serde::Deserialize<'de>,
{
    fn deserialize<De>(deserializer: De) -> Result<Self, De::Error>
    where
        De: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct RelationalLinkVisitor<D, M, R> {
            _phantom: std::marker::PhantomData<(D, M, R)>,
        }

        impl<'de, D, M, R> Visitor<'de> for RelationalLinkVisitor<D, M, R>
        where
            D: NetabaseDefinitionTrait,
            M: NetabaseModelTrait<D> + serde::Deserialize<'de>,
            R: NetabaseRelationDiscriminant,
            M::PrimaryKey: serde::Deserialize<'de>,
        {
            type Value = RelationalLink<D, M, R>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a RelationalLink")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut link_type: Option<String> = None;
                let mut key: Option<M::PrimaryKey> = None;
                let mut entity: Option<M> = None;

                while let Some(k) = map.next_key::<String>()? {
                    match k.as_str() {
                        "type" => {
                            if link_type.is_some() {
                                return Err(de::Error::duplicate_field("type"));
                            }
                            link_type = Some(map.next_value()?);
                        }
                        "key" => {
                            if key.is_some() {
                                return Err(de::Error::duplicate_field("key"));
                            }
                            key = Some(map.next_value()?);
                        }
                        "entity" => {
                            if entity.is_some() {
                                return Err(de::Error::duplicate_field("entity"));
                            }
                            entity = Some(map.next_value()?);
                        }
                        _ => {
                            let _ = map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }

                let link_type = link_type.ok_or_else(|| de::Error::missing_field("type"))?;
                match link_type.as_str() {
                    "Reference" => {
                        let key = key.ok_or_else(|| de::Error::missing_field("key"))?;
                        Ok(RelationalLink::Reference(key))
                    }
                    "Entity" => {
                        let entity = entity.ok_or_else(|| de::Error::missing_field("entity"))?;
                        Ok(RelationalLink::Entity(entity))
                    }
                    _ => Err(de::Error::unknown_variant(
                        &link_type,
                        &["Reference", "Entity"],
                    )),
                }
            }
        }

        deserializer.deserialize_struct(
            "RelationalLink",
            &["type", "key", "entity"],
            RelationalLinkVisitor {
                _phantom: std::marker::PhantomData,
            },
        )
    }
}

/// Helper trait to detect if a model has custom relation insertion behavior
pub trait HasCustomRelationInsertion<D: NetabaseDefinitionTrait> {
    const HAS_RELATIONS: bool = false;
}

/// Generic helper function for inserting models with relations
pub fn insert_linked_model<D, M, S>(model: &M, store: &S) -> Result<(), NetabaseError>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D> + Clone,
    S: OpenTree<D, M>,
{
    let tree = store.open_tree();
    tree.put_raw(model.clone())
}

/// Trait for inserting models with their related entities
///
/// This trait provides a unified interface for inserting models that may contain
/// RelationalLink fields with embedded entities.
pub trait InsertWithLinks<D: NetabaseDefinitionTrait> {
    /// Insert this model and all linked entities recursively
    fn insert_with_links<S>(&self, store: &S) -> Result<(), NetabaseError>
    where
        S: MultiModelStore<D>,
        Self: Clone;

    /// Insert only the linked entities without inserting this model
    fn insert_relations_only<S>(&self, store: &S) -> Result<(), NetabaseError>
    where
        S: MultiModelStore<D>,
        Self: Clone;
}

/// Blanket implementation for models that implement NetabaseRelationTrait
impl<D, M> InsertWithLinks<D> for M
where
    D: NetabaseDefinitionTrait,
    M: crate::traits::relation::NetabaseRelationTrait<D>,
{
    fn insert_with_links<S>(&self, store: &S) -> Result<(), NetabaseError>
    where
        S: MultiModelStore<D>,
        Self: Clone,
    {
        self.insert_with_relations(store)
    }

    fn insert_relations_only<S>(&self, store: &S) -> Result<(), NetabaseError>
    where
        S: MultiModelStore<D>,
        Self: Clone,
    {
        self.insert_relations_only(store)
    }
}

/// Utility functions for working with relational links
pub mod link_utils {
    use super::*;

    /// Extract all entity variants from a collection of relational links
    pub fn extract_entities<D, M, R, I>(links: I) -> Vec<M>
    where
        D: NetabaseDefinitionTrait,
        M: NetabaseModelTrait<D>,
        R: NetabaseRelationDiscriminant,
        I: IntoIterator<Item = RelationalLink<D, M, R>>,
    {
        links
            .into_iter()
            .filter_map(|link| match link {
                RelationalLink::Entity(entity) => Some(entity),
                _ => None,
            })
            .collect()
    }

    /// Extract all reference keys from a collection of relational links
    pub fn extract_references<D, M, R, I>(links: I) -> Vec<M::PrimaryKey>
    where
        D: NetabaseDefinitionTrait,
        M: NetabaseModelTrait<D>,
        R: NetabaseRelationDiscriminant,
        I: IntoIterator<Item = RelationalLink<D, M, R>>,
    {
        links
            .into_iter()
            .filter_map(|link| match link {
                RelationalLink::Reference(key) => Some(key),
                _ => None,
            })
            .collect()
    }

    /// Convert a collection of relational links to their primary keys
    pub fn to_keys<D, M, R, I>(links: I) -> Vec<M::PrimaryKey>
    where
        D: NetabaseDefinitionTrait,
        M: NetabaseModelTrait<D>,
        R: NetabaseRelationDiscriminant,
        I: IntoIterator<Item = RelationalLink<D, M, R>>,
    {
        links.into_iter().map(|link| link.key()).collect()
    }

    /// Check if any links in a collection contain entities
    pub fn has_entities<D, M, R, I>(links: I) -> bool
    where
        D: NetabaseDefinitionTrait,
        M: NetabaseModelTrait<D>,
        R: NetabaseRelationDiscriminant,
        I: IntoIterator<Item = RelationalLink<D, M, R>>,
    {
        links.into_iter().any(|link| link.is_entity())
    }

    /// Check if any links in a collection contain references
    pub fn has_references<D, M, R, I>(links: I) -> bool
    where
        D: NetabaseDefinitionTrait,
        M: NetabaseModelTrait<D>,
        R: NetabaseRelationDiscriminant,
        I: IntoIterator<Item = RelationalLink<D, M, R>>,
    {
        links.into_iter().any(|link| link.is_reference())
    }

    /// Hydrate all links in a collection, returning successfully loaded entities
    pub fn hydrate_all<D, M, R, I, T>(links: I, store: &T) -> Result<Vec<M>, NetabaseError>
    where
        D: NetabaseDefinitionTrait,
        M: NetabaseModelTrait<D>,
        R: NetabaseRelationDiscriminant,
        I: IntoIterator<Item = RelationalLink<D, M, R>>,
        T: StoreOps<D, M>,
    {
        let mut entities = Vec::new();
        for link in links {
            if let Some(entity) = link.hydrate(store)? {
                entities.push(entity);
            }
        }
        Ok(entities)
    }
}
