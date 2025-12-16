use crate::traits::registery::{definition::{NetabaseDefinition, NetabaseDefinitionKeys}, models::keys::NetabaseModelKeys};
use serde::{Serialize, Deserialize};
use bincode::{Encode, Decode, BorrowDecode};

/// Trait for types that can be converted to/from a global definition enum
pub trait IntoGlobalDefinition {
    type GlobalEnum: Clone + PartialEq + Eq + std::hash::Hash + std::fmt::Debug;

    fn into_global(self) -> Self::GlobalEnum;
    fn from_global(global: Self::GlobalEnum) -> Option<Self>
    where
        Self: Sized;
}

/// Trait for managing global definition collections
pub trait GlobalDefinitionCollection {
    type DefinitionType;
    type GlobalEnum;

    fn add_definition(&mut self, def: Self::DefinitionType);
    fn get_definition(&self, global: &Self::GlobalEnum) -> Option<&Self::DefinitionType>;
    fn remove_definition(&mut self, global: &Self::GlobalEnum) -> Option<Self::DefinitionType>;
}

/// Trait that enables any definition to be part of a global enum system
/// This should be implemented by macro for all NetabaseDefinition types
///
/// The hierarchy is:
/// - GlobalDefinition: enum holding all definition instances (Definition1 | Definition2 | ...)
/// - GlobalDefinitionKeys: enum holding all definition discriminants (Definition1, Definition2, ...)
/// - GlobalKeys: enum holding model discriminants across definitions (Definition1::User, Definition2::Article, ...)
pub trait GlobalDefinitionEnum: NetabaseDefinition
where
    <Self as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    /// Enum that holds all definition instances
    type GlobalDefinition: Clone + PartialEq + Eq + std::hash::Hash + std::fmt::Debug;

    /// Enum that holds definition-level discriminants (which definition, not which model)
    type GlobalDefinitionKeys: Clone + PartialEq + Eq + std::hash::Hash + std::fmt::Debug + 'static;

    /// Enum that holds model discriminants across definitions (which definition + which model)
    type GlobalKeys: Clone + PartialEq + Eq + std::hash::Hash + std::fmt::Debug + 'static;

    fn into_global_definition(self) -> Self::GlobalDefinition;
    fn from_global_definition(global: Self::GlobalDefinition) -> Option<Self>
    where
        Self: Sized;

    fn discriminant_into_global(discriminant: Self::Discriminant) -> Self::GlobalKeys;
    fn discriminant_from_global(global: Self::GlobalKeys) -> Option<Self::Discriminant>;

    fn definition_discriminant_to_global() -> Self::GlobalDefinitionKeys;
    fn global_to_definition_discriminant(global: Self::GlobalDefinitionKeys) -> bool;
}

/// A relational link between models, supporting both same-definition and cross-definition relations
/// Generic over source definition, target definition, and target model for full type safety
#[derive(Debug, Clone)]
pub enum RelationalLink<'data, SourceD, TargetD, M>
where
    SourceD: NetabaseDefinition + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registery::models::model::NetabaseModel<TargetD>,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Relational<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
{
    /// Dehydrated: Contains only the primary key
    Dehydrated {
        primary_key: <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary<'static>,
        _source: std::marker::PhantomData<SourceD>,
    },
    /// Hydrated: Contains both the primary key and the loaded model
    Hydrated {
        primary_key: <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary<'static>,
        model: &'data M,
        _source: std::marker::PhantomData<SourceD>,
    },
}

// PartialEq implementation for new RelationalLink
impl<'data, SourceD, TargetD, M> PartialEq for RelationalLink<'data, SourceD, TargetD, M>
where
    SourceD: NetabaseDefinition + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registery::models::model::NetabaseModel<TargetD>,
    <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary<'static>: PartialEq,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Relational<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Dehydrated { primary_key: pk1, .. }, Self::Dehydrated { primary_key: pk2, .. }) => pk1 == pk2,
            (Self::Hydrated { primary_key: pk1, .. }, Self::Hydrated { primary_key: pk2, .. }) => pk1 == pk2,
            (Self::Dehydrated { primary_key: pk1, .. }, Self::Hydrated { primary_key: pk2, .. }) => pk1 == pk2,
            (Self::Hydrated { primary_key: pk1, .. }, Self::Dehydrated { primary_key: pk2, .. }) => pk1 == pk2,
        }
    }
}

// Eq implementation
impl<'data, SourceD, TargetD, M> Eq for RelationalLink<'data, SourceD, TargetD, M>
where
    SourceD: NetabaseDefinition + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registery::models::model::NetabaseModel<TargetD>,
    <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary<'static>: Eq,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Relational<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
{}

// Hash implementation
impl<'data, SourceD, TargetD, M> std::hash::Hash for RelationalLink<'data, SourceD, TargetD, M>
where
    SourceD: NetabaseDefinition + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registery::models::model::NetabaseModel<TargetD> + std::hash::Hash,
    <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary<'static>: std::hash::Hash,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Relational<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Dehydrated { primary_key, .. } => {
                0u8.hash(state);
                primary_key.hash(state);
            }
            Self::Hydrated { primary_key, model, .. } => {
                1u8.hash(state);
                primary_key.hash(state);
                model.hash(state);
            }
        }
    }
}

// PartialOrd implementation
impl<'data, SourceD, TargetD, M> PartialOrd for RelationalLink<'data, SourceD, TargetD, M>
where
    SourceD: NetabaseDefinition + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registery::models::model::NetabaseModel<TargetD> + PartialOrd,
    <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary<'static>: PartialOrd,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Relational<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Dehydrated { primary_key: pk1, .. }, Self::Dehydrated { primary_key: pk2, .. }) => pk1.partial_cmp(pk2),
            (Self::Hydrated { primary_key: pk1, .. }, Self::Hydrated { primary_key: pk2, .. }) => pk1.partial_cmp(pk2),
            // Compare based on primary keys even if mixed? Or order variants?
            // Let's order variants first. Dehydrated < Hydrated.
            (Self::Dehydrated { .. }, Self::Hydrated { .. }) => Some(std::cmp::Ordering::Less),
            (Self::Hydrated { .. }, Self::Dehydrated { .. }) => Some(std::cmp::Ordering::Greater),
        }
    }
}

// Ord implementation
impl<'data, SourceD, TargetD, M> Ord for RelationalLink<'data, SourceD, TargetD, M>
where
    SourceD: NetabaseDefinition + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registery::models::model::NetabaseModel<TargetD> + Ord,
    <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary<'static>: Ord,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Relational<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Dehydrated { primary_key: pk1, .. }, Self::Dehydrated { primary_key: pk2, .. }) => pk1.cmp(pk2),
            (Self::Hydrated { primary_key: pk1, .. }, Self::Hydrated { primary_key: pk2, .. }) => pk1.cmp(pk2),
            (Self::Dehydrated { .. }, Self::Hydrated { .. }) => std::cmp::Ordering::Less,
            (Self::Hydrated { .. }, Self::Dehydrated { .. }) => std::cmp::Ordering::Greater,
        }
    }
}

// Implementation for new RelationalLink
impl<'data, SourceD, TargetD, M> RelationalLink<'data, SourceD, TargetD, M>
where
    SourceD: NetabaseDefinition + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registery::models::model::NetabaseModel<TargetD>,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Relational<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
{
    /// Create a new dehydrated relational link with just the primary key
    pub fn new_dehydrated(
        primary_key: <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary<'static>,
    ) -> Self {
        Self::Dehydrated {
            primary_key,
            _source: std::marker::PhantomData,
        }
    }

    /// Create a new hydrated relational link with the model and primary key
    pub fn new_hydrated(
        primary_key: <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary<'static>,
        model: &'data M,
    ) -> Self {
        Self::Hydrated {
            primary_key,
            model,
            _source: std::marker::PhantomData,
        }
    }

    /// Get the primary key from the relation
    pub fn get_primary_key(&self) -> &<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary<'static> {
        match self {
            Self::Dehydrated { primary_key, .. } => primary_key,
            Self::Hydrated { primary_key, .. } => primary_key,
        }
    }

    /// Check if this relation is currently hydrated (contains model data)
    pub fn is_hydrated(&self) -> bool {
        matches!(self, Self::Hydrated { .. })
    }

    /// Check if this relation is dehydrated (contains only primary key)
    pub fn is_dehydrated(&self) -> bool {
        matches!(self, Self::Dehydrated { .. })
    }

    /// Get the hydrated model if available, otherwise None
    pub fn get_model(&self) -> Option<&M> {
        match self {
            Self::Hydrated { model, .. } => Some(model),
            Self::Dehydrated { .. } => None,
        }
    }

    /// Convert a dehydrated relation to hydrated by providing the model data
    pub fn hydrate_with_model(self, model: &'data M) -> Self {
        let primary_key = match self {
            Self::Dehydrated { primary_key, .. } => primary_key,
            Self::Hydrated { primary_key, .. } => primary_key,
        };
        Self::Hydrated {
            primary_key,
            model,
            _source: std::marker::PhantomData,
        }
    }

    /// Convert a hydrated relation back to dehydrated
    pub fn dehydrate(self) -> Self {
        let primary_key = match self {
            Self::Dehydrated { primary_key, .. } => primary_key,
            Self::Hydrated { primary_key, .. } => primary_key,
        };
        Self::Dehydrated {
            primary_key,
            _source: std::marker::PhantomData,
        }
    }

    /// Check if this is a same-definition relation (SourceD == TargetD)
    pub fn is_same_definition() -> bool {
        std::any::TypeId::of::<SourceD>() == std::any::TypeId::of::<TargetD>()
    }

    /// Check if this is a cross-definition relation (SourceD != TargetD)
    pub fn is_cross_definition() -> bool {
        !Self::is_same_definition()
    }

    // TODO: Add hydrate() method after Phase 3-4
    // This method will load the model from the database with conditional permission checks
    // Signature: pub fn hydrate<Trans>(self, transaction: &Trans, conditionally: bool) -> NetabaseResult<Self>
}

impl<'data, SourceD, TargetD, M> Serialize for RelationalLink<'data, SourceD, TargetD, M>
where
    SourceD: NetabaseDefinition + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registery::models::model::NetabaseModel<TargetD> + Serialize,
    <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary<'static>: Serialize,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Relational<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Dehydrated { primary_key, .. } => {
                // Serialize as variant 0
                serializer.serialize_newtype_variant("RelationalLink", 0, "Dehydrated", primary_key)
            }
            Self::Hydrated { primary_key, model, .. } => {
                // Serialize as variant 1. We need to serialize a tuple or struct.
                // Let's use a tuple (primary_key, model)
                use serde::ser::SerializeTupleVariant;
                let mut tv = serializer.serialize_tuple_variant("RelationalLink", 1, "Hydrated", 2)?;
                tv.serialize_field(primary_key)?;
                tv.serialize_field(model)?;
                tv.end()
            }
        }
    }
}

impl< 'de, SourceD, TargetD, M> Deserialize<'de> for RelationalLink<'de, SourceD, TargetD, M>
where
    SourceD: NetabaseDefinition + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registery::models::model::NetabaseModel<TargetD>,
    <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary<'static>: Deserialize<'de>,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Relational<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        
    let proxy = <M::Keys as NetabaseModelKeys<TargetD, M>>::Primary::deserialize(deserializer)?;
        Ok(RelationalLink::Dehydrated{primary_key: proxy, _source: std::marker::PhantomData})
    }
}

// Bincode Encode
impl<'data, SourceD, TargetD, M> Encode for RelationalLink<'data, SourceD, TargetD, M>
where
    SourceD: NetabaseDefinition + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registery::models::model::NetabaseModel<TargetD> + Encode,
    <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary<'static>: Encode,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Relational<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
{
    fn encode<E: bincode::enc::Encoder>(&self, encoder: &mut E) -> Result<(), bincode::error::EncodeError> {
        match self {
            Self::Dehydrated { primary_key, .. } => {
                0u32.encode(encoder)?;
                primary_key.encode(encoder)?;
            }
            Self::Hydrated { primary_key, model, .. } => {
                1u32.encode(encoder)?;
                primary_key.encode(encoder)?;
                model.encode(encoder)?;
            }
        }
        Ok(())
    }
}

// Bincode Decode
impl<'data, SourceD, TargetD, M, C> Decode<C> for RelationalLink<'data, SourceD, TargetD, M>
where
    SourceD: NetabaseDefinition + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registery::models::model::NetabaseModel<TargetD> + Decode<C>,
    <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary<'static>: Decode<C>,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Relational<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
{
    fn decode<D: bincode::de::Decoder<Context = C>>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
        let variant = u32::decode(decoder)?;
        match variant {
            0 => {
                let primary_key = <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary<'static> as Decode<C>>::decode(decoder)?;
                Ok(Self::Dehydrated {
                    primary_key,
                    _source: std::marker::PhantomData,
                })
            }
            1 => {
                let primary_key = <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary<'static> as Decode<C>>::decode(decoder)?;
                let model = M::decode(decoder)?;
                Ok(Self::Dehydrated  {
                    primary_key,
                    _source: std::marker::PhantomData,
                })
            }
            _ => Err(bincode::error::DecodeError::Other("Invalid RelationalLink variant")),
        }
    }
}

// Bincode BorrowDecode
impl<'de, SourceD, TargetD, M, C> BorrowDecode<'de, C> for RelationalLink<'de, SourceD, TargetD, M>
where
    SourceD: NetabaseDefinition + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registery::models::model::NetabaseModel<TargetD> + BorrowDecode<'de, C>,
    <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary<'static>: BorrowDecode<'de, C>,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Relational<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
    for<'a> <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription<'a> as strum::IntoDiscriminant>::Discriminant: 'static,
{
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de, Context = C>>(decoder: &mut D) -> Result<Self, bincode::error::DecodeError> {
        let primary_key = <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary<'static> as BorrowDecode<'de, C>>::borrow_decode(decoder)?;
        Ok(Self::Dehydrated {
            primary_key,
            _source: std::marker::PhantomData,
        })
    }
}
#[derive(Debug, thiserror::Error)]
pub enum RelationalLinkError {
    #[error("Key mismatch: the provided model's primary key doesn't match the stored foreign key")]
    KeyMismatch,

    #[error("Permission denied: insufficient permissions to access related definition")]
    PermissionDenied,

    #[error("Not found: the related model could not be found")]
    NotFound,

    #[error("Cross-definition access error")]
    CrossDefinitionError,
}

/// Cross-definition permissions for relational access
/// Uses strongly typed table definitions instead of strings
#[derive(Debug, Clone)]
pub struct CrossDefinitionPermissions<D: GlobalDefinitionEnum>
where
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: 'static,
{
    /// List of accessible table definitions (strongly typed)
    pub accessible_tables: Vec<D::GlobalKeys>,
    /// Whether read access is allowed
    pub read_allowed: bool,
    /// Whether write access is allowed
    pub write_allowed: bool,
    /// Whether hydration (loading related data) is allowed
    pub hydration_allowed: bool,
}

impl<D: GlobalDefinitionEnum> CrossDefinitionPermissions<D>
where
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <D as strum::IntoDiscriminant>::Discriminant: 'static,
{
    /// Create new cross-definition permissions
    pub fn new(
        accessible_tables: Vec<D::GlobalKeys>,
        read_allowed: bool,
        write_allowed: bool,
        hydration_allowed: bool,
    ) -> Self {
        Self {
            accessible_tables,
            read_allowed,
            write_allowed,
            hydration_allowed,
        }
    }

    /// Create read-only permissions with specified tables
    pub fn read_only(accessible_tables: Vec<D::GlobalKeys>) -> Self {
        Self::new(accessible_tables, true, false, true)
    }

    /// Create full permissions with specified tables
    pub fn full_access(accessible_tables: Vec<D::GlobalKeys>) -> Self {
        Self::new(accessible_tables, true, true, true)
    }

    /// Create no permissions
    pub fn no_access() -> Self
    where
        <D as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    {
        Self::new(Vec::new(), false, false, false)
    }

    /// Check if the given operation is allowed
    pub fn can_read(&self) -> bool {
        self.read_allowed
    }

    pub fn can_write(&self) -> bool {
        self.write_allowed
    }

    pub fn can_hydrate(&self) -> bool {
        self.hydration_allowed
    }

    /// Check if a specific table key is accessible
    pub fn can_access_table(&self, key: &D::GlobalKeys) -> bool {
        self.accessible_tables.contains(key)
    }
}
