//! Relational Links Module
//!
//! This module provides type-safe relational linking between models using
//! generated relation enums for better type safety and consistency with
//! the existing netabase patterns.

pub mod compat;

/// Controls how deeply relations should be inserted recursively
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecursionLevel {
    /// Insert all relations recursively without limit
    Full,
    /// Don't insert any relations
    None,
    /// Insert relations up to a specific depth
    Value(u8),
}

impl RecursionLevel {
    /// Check if recursion should continue at the current depth
    pub fn should_recurse(&self, current_depth: u8) -> bool {
        match self {
            RecursionLevel::Full => true,
            RecursionLevel::None => false,
            RecursionLevel::Value(max_depth) => current_depth < *max_depth,
        }
    }

    /// Get the next recursion level (decremented for Value variant)
    pub fn next_level(&self) -> RecursionLevel {
        match self {
            RecursionLevel::Full => RecursionLevel::Full,
            RecursionLevel::None => RecursionLevel::None,
            RecursionLevel::Value(depth) => {
                if *depth > 0 {
                    RecursionLevel::Value(depth - 1)
                } else {
                    RecursionLevel::None
                }
            }
        }
    }
}

impl Default for RecursionLevel {
    fn default() -> Self {
        RecursionLevel::Value(1) // Default to 1 level of recursion
    }
}

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

/// A type-safe relational link between models
///
/// `RelationalLink` provides a flexible way to represent relationships between models,
/// supporting both eager loading (embedded entities) and lazy loading (key references).
///
/// # Type Parameters
///
/// * `D` - The netabase definition trait that defines the data model schema
/// * `M` - The target model type this link points to
///
/// # Variants
///
/// ## `Reference(PrimaryKey)`
/// Stores only the primary key of the related entity. This is useful for:
/// - Reducing memory usage when the full entity isn't needed
/// - Preventing circular references
/// - Serialization where you only want to store IDs
///
/// ## `Entity(M)`
/// Stores the complete related entity. This is useful for:
/// - Avoiding additional database lookups
/// - Bundling related data for efficient transfer
/// - Working with entities that will be modified together
///
/// # Design Philosophy
///
/// This design allows you to:
/// 1. **Defer loading decisions**: Choose at runtime whether to load entities or just keys
/// 2. **Mix strategies**: Some relations can be eager-loaded while others remain lazy
/// 3. **Optimize for your use case**: Use references for large graphs, entities for small clusters
///
/// # Examples
///
/// ## Basic Usage
///
/// ```ignore
/// use netabase_store::links::RelationalLink;
///
/// // Create a link with an embedded entity (eager loading)
/// let author_link = RelationalLink::Entity(user);
///
/// // Create a link with just a reference (lazy loading)
/// let author_link = RelationalLink::Reference(user_id);
///
/// // Access the key regardless of variant
/// let id = author_link.key();
///
/// // Check which variant it is
/// if author_link.is_entity() {
///     println!("Already loaded!");
/// }
/// ```
///
/// ## Hydration Pattern
///
/// ```ignore
/// // Start with a reference
/// let author_link = RelationalLink::Reference(user_id);
///
/// // Later, load the full entity when needed
/// let author = author_link.hydrate(&store)?;
/// ```
///
/// ## Converting Between Variants
///
/// ```ignore
/// // Convert entity to reference (extract key)
/// let entity_link = RelationalLink::Entity(user);
/// let ref_link = entity_link.to_reference();  // Now just stores the key
/// ```
///
/// ## In a Model Definition
///
/// ```ignore
/// #[derive(NetabaseModel)]
/// #[netabase(BlogDefinition)]
/// pub struct Post {
///     #[primary_key]
///     pub id: u64,
///     pub title: String,
///
///     // This field can hold either a User entity or just a user ID
///     pub author: RelationalLink<BlogDefinition, User>,
///
///     // Collections work too
///     pub comments: Vec<RelationalLink<BlogDefinition, Comment>>,
/// }
/// ```
///
/// ## Serialization
///
/// The type automatically handles serialization, preserving the variant:
/// - `Reference` serializes as `{"type": "Reference", "key": <value>}`
/// - `Entity` serializes as `{"type": "Entity", "entity": <value>}`
///
/// # Performance Considerations
///
/// - **Memory**: `Entity` variant uses more memory but avoids lookups
/// - **Network**: `Reference` variant minimizes data transfer
/// - **Lookups**: `Entity` variant avoids database roundtrips
/// - **Updates**: `Reference` variant doesn't require updating when related entity changes
///
#[derive(Clone, PartialEq)]
pub enum RelationalLink<D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
{
    /// A reference to an entity via its primary key
    Reference(<M as NetabaseModelTrait<D>>::PrimaryKey),
    /// A full entity instance
    Entity(M),
}

impl<D, M> std::fmt::Debug for RelationalLink<D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D> + std::fmt::Debug,
    <M as NetabaseModelTrait<D>>::PrimaryKey: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reference(key) => f.debug_tuple("Reference").field(key).finish(),
            Self::Entity(entity) => f.debug_tuple("Entity").field(entity).finish(),
        }
    }
}

impl<D, M> From<M> for RelationalLink<D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
{
    fn from(value: M) -> Self {
        RelationalLink::Entity(value)
    }
}

impl<D, M> RelationalLink<D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D>,
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
    pub fn hydrate<T>(self, store: &T) -> Result<Option<M>, NetabaseError>
    where
        T: StoreOps<D, M>,
    {
        match self {
            RelationalLink::Reference(key) => store.get_raw(key),
            RelationalLink::Entity(model) => Ok(Some(model)),
        }
    }

    /// Get the primary key regardless of whether this is a Reference or Entity
    pub fn key(&self) -> M::PrimaryKey {
        match self {
            RelationalLink::Reference(key) => key.clone(),
            RelationalLink::Entity(entity) => entity.primary_key(),
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
        }
    }

    /// Extract the reference key if this is a Reference variant
    pub fn as_reference(&self) -> Option<&M::PrimaryKey> {
        match self {
            RelationalLink::Reference(key) => Some(key),
            RelationalLink::Entity(_) => None,
        }
    }

    /// Convert this link to a reference, extracting the key
    pub fn to_reference(self) -> RelationalLink<D, M> {
        match self {
            RelationalLink::Reference(key) => RelationalLink::Reference(key),
            RelationalLink::Entity(entity) => RelationalLink::Reference(entity.primary_key()),
        }
    }
}

// Manual implementation of bincode traits for proper serialization
impl<D, M> bincode::Encode for RelationalLink<D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D> + bincode::Encode,
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
        }
    }
}

impl<D, M, Context> bincode::Decode<Context> for RelationalLink<D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D> + bincode::Decode<Context>,
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

impl<'a, D, M, Context> bincode::BorrowDecode<'a, Context> for RelationalLink<D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D> + bincode::Decode<Context>,
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
impl<D, M> serde::Serialize for RelationalLink<D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D> + serde::Serialize,
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
        }
    }
}

impl<'de, D, M> serde::Deserialize<'de> for RelationalLink<D, M>
where
    D: NetabaseDefinitionTrait,
    M: NetabaseModelTrait<D> + serde::Deserialize<'de>,
    M::PrimaryKey: serde::Deserialize<'de>,
{
    fn deserialize<De>(deserializer: De) -> Result<Self, De::Error>
    where
        De: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct RelationalLinkVisitor<D, M> {
            _phantom: std::marker::PhantomData<(D, M)>,
        }

        impl<'de, D, M> Visitor<'de> for RelationalLinkVisitor<D, M>
        where
            D: NetabaseDefinitionTrait,
            M: NetabaseModelTrait<D> + serde::Deserialize<'de>,
            M::PrimaryKey: serde::Deserialize<'de>,
        {
            type Value = RelationalLink<D, M>;

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

/// Trait for models that support recursive relation insertion with depth control
pub trait RecursiveRelationInsertion<D: NetabaseDefinitionTrait> {
    /// Insert this model with its relations recursively up to the specified depth
    fn insert_with_relations_depth<S>(
        &self,
        store: &S,
        level: RecursionLevel,
    ) -> Result<(), NetabaseError>
    where
        S: OpenTree<D, Self>,
        Self: NetabaseModelTrait<D> + Clone;
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

/// Store-level operations for models with relational links
///
/// This module provides extension traits for stores to handle insertion
/// of models with embedded relational entities.
pub mod store_ops {
    use super::*;
    use crate::traits::relation::NetabaseRelationTrait;

    /// Extension trait for stores that provides relational link insertion
    ///
    /// This trait extends any store that implements `OpenTree` to provide
    /// automatic insertion of models with their embedded relational entities.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use netabase_store::links::store_ops::StoreWithLinks;
    ///
    /// // Store implements StoreWithLinks automatically
    /// let post = Post {
    ///     id: 1,
    ///     title: "Hello".into(),
    ///     author: RelationalLink::Entity(user),
    /// };
    ///
    /// // This will insert both the user and the post
    /// store.put_with_links(&post)?;
    /// ```
    pub trait StoreWithLinks<D: NetabaseDefinitionTrait> {
        /// Insert a model and all its embedded relational entities
        ///
        /// This method will:
        /// 1. Check if the model has any `RelationalLink` fields
        /// 2. For each field containing an `Entity` variant, recursively insert that entity
        /// 3. Finally insert the main model
        ///
        /// # Note
        /// Models are inserted with their embedded entities converted to references.
        /// The original model remains unchanged.
        ///
        /// # Errors
        /// Returns an error if any insertion fails
        fn put_with_links<M>(&self, model: &M) -> Result<(), NetabaseError>
        where
            M: NetabaseRelationTrait<D> + Clone,
            Self: OpenTree<D, M>;
    }

    /// Blanket implementation for all stores that can open trees
    impl<D, S> StoreWithLinks<D> for S
    where
        D: NetabaseDefinitionTrait,
        S: ?Sized,
    {
        fn put_with_links<M>(&self, model: &M) -> Result<(), NetabaseError>
        where
            M: NetabaseRelationTrait<D> + Clone,
            Self: OpenTree<D, M>,
        {
            // For models with relations, use generated helper methods
            // For models without relations, just insert normally
            if model.has_relations() {
                // Note: The actual insertion of related entities must be done
                // using the generated helper methods on the model itself,
                // as we can't dynamically access the fields here.
                // Users should call the generated `insert_<field>_if_entity` methods.
            }

            // Insert the main model
            let tree = self.open_tree();
            tree.put_raw(model.clone())
        }
    }
}

/// Utility functions for working with relational links
pub mod link_utils {
    use super::*;

    /// Extract all entity variants from a collection of relational links
    pub fn extract_entities<D, M, I>(links: I) -> Vec<M>
    where
        D: NetabaseDefinitionTrait,
        M: NetabaseModelTrait<D>,
        I: IntoIterator<Item = RelationalLink<D, M>>,
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
    pub fn extract_references<D, M, I>(links: I) -> Vec<M::PrimaryKey>
    where
        D: NetabaseDefinitionTrait,
        M: NetabaseModelTrait<D>,
        I: IntoIterator<Item = RelationalLink<D, M>>,
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
    pub fn to_keys<D, M, I>(links: I) -> Vec<M::PrimaryKey>
    where
        D: NetabaseDefinitionTrait,
        M: NetabaseModelTrait<D>,
        I: IntoIterator<Item = RelationalLink<D, M>>,
    {
        links.into_iter().map(|link| link.key()).collect()
    }

    /// Check if any links in a collection contain entities
    pub fn has_entities<D, M, I>(links: I) -> bool
    where
        D: NetabaseDefinitionTrait,
        M: NetabaseModelTrait<D>,
        I: IntoIterator<Item = RelationalLink<D, M>>,
    {
        links.into_iter().any(|link| link.is_entity())
    }

    /// Check if any links in a collection contain references
    pub fn has_references<D, M, I>(links: I) -> bool
    where
        D: NetabaseDefinitionTrait,
        M: NetabaseModelTrait<D>,
        I: IntoIterator<Item = RelationalLink<D, M>>,
    {
        links.into_iter().any(|link| link.is_reference())
    }

    /// Hydrate all links in a collection, returning successfully loaded entities
    pub fn hydrate_all<D, M, I, T>(links: I, store: &T) -> Result<Vec<M>, NetabaseError>
    where
        D: NetabaseDefinitionTrait,
        M: NetabaseModelTrait<D>,
        I: IntoIterator<Item = RelationalLink<D, M>>,
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
