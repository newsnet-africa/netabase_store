use crate::traits::registery::{definition::NetabaseDefinition, models::keys::NetabaseModelKeys};
use serde::{Serialize, Deserialize};
use bincode::{Encode, Decode, BorrowDecode};
use strum::IntoDiscriminant;

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
///
/// Note: This trait does not require NetabaseDefinition to avoid circular dependencies.
/// Types implementing this should also implement NetabaseDefinition separately.
pub trait GlobalDefinitionEnum: IntoDiscriminant + Sized
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

/// A relational link between models supporting four lifecycle states:
///
/// # Variants
///
/// 1. **Dehydrated**: Contains only the primary key, minimal memory footprint
///    - Used for serialization and storage
///    - Created manually or from deserialization
///    - Can be hydrated on-demand
///    - No lifetime constraints
///
/// 2. **Owned**: Fully owns the related model (Box<M>)
///    - Used when the model is constructed independently
///    - No lifetime dependencies - can be freely moved
///    - Serializes with both key and model data (variant 1)
///    - Can be extracted with into_owned()
///
/// 3. **Hydrated**: Contains key + borrowed reference to model
///    - Used when model is already in memory
///    - Reference has application-controlled lifetime
///    - Full model access without database query
///    - Requires 'data lifetime
///
/// 4. **Borrowed**: Contains key + borrowed reference from database AccessGuard
///    - Created by transaction.get() operations
///    - Lifetime tied to AccessGuard (Transaction -> Table -> AccessGuard -> Value)
///    - Automatically converts to Dehydrated on serialization
///    - Zero-copy database access
///    - Requires 'data lifetime
///
/// # Lifetime Management
///
/// The `'data` lifetime represents:
/// - For Owned: No lifetime constraints (uses 'static internally)
/// - For Hydrated: lifetime of the borrowed reference
/// - For Borrowed: lifetime chain from database transaction
///
/// # Example
///
/// ```rust,ignore
/// // Dehydrated - just the key
/// let link = RelationalLink::new_dehydrated(user_id);
///
/// // Owned - with owned model (no lifetime constraints)
/// let user = User { id: user_id.clone(), name: "Alice".to_string(), age: 30 };
/// let link = RelationalLink::new_owned(user_id, user);
///
/// // Hydrated - with model reference
/// let link = RelationalLink::new_hydrated(user_id, &user);
///
/// // Borrowed - from database (in transaction context)
/// let guard = table.get(&key)?;
/// let model = guard.value();
/// let link = RelationalLink::new_borrowed(key, model);
/// ```
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
    /// Owned: Fully owns the related model (no lifetime dependency)
    /// Used when the model is constructed independently and needs to be stored with full ownership
    Owned {
        primary_key: <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary<'static>,
        model: Box<M>,
        _source: std::marker::PhantomData<SourceD>,
    },
    /// Hydrated: Contains a borrowed reference to the model (application-controlled lifetime)
    Hydrated {
        primary_key: <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary<'static>,
        model: &'data M,
        _source: std::marker::PhantomData<SourceD>,
    },
    /// Borrowed: Contains both the primary key and a borrowed reference from AccessGuard
    /// Lifetime is tied to database transaction -> table -> AccessGuard chain
    Borrowed {
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
        // All variants compare equal if primary keys match
        let pk1 = match self {
            Self::Dehydrated { primary_key, .. } => primary_key,
            Self::Owned { primary_key, .. } => primary_key,
            Self::Hydrated { primary_key, .. } => primary_key,
            Self::Borrowed { primary_key, .. } => primary_key,
        };
        let pk2 = match other {
            Self::Dehydrated { primary_key, .. } => primary_key,
            Self::Owned { primary_key, .. } => primary_key,
            Self::Hydrated { primary_key, .. } => primary_key,
            Self::Borrowed { primary_key, .. } => primary_key,
        };
        pk1 == pk2
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
            Self::Owned { primary_key, .. } => {
                1u8.hash(state);
                primary_key.hash(state);
            }
            Self::Hydrated { primary_key, model, .. } => {
                2u8.hash(state);
                primary_key.hash(state);
                model.hash(state);
            }
            Self::Borrowed { primary_key, model, .. } => {
                3u8.hash(state);
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
            // Same variants: compare by primary key
            (Self::Dehydrated { primary_key: pk1, .. }, Self::Dehydrated { primary_key: pk2, .. }) => pk1.partial_cmp(pk2),
            (Self::Owned { primary_key: pk1, .. }, Self::Owned { primary_key: pk2, .. }) => pk1.partial_cmp(pk2),
            (Self::Hydrated { primary_key: pk1, .. }, Self::Hydrated { primary_key: pk2, .. }) => pk1.partial_cmp(pk2),
            (Self::Borrowed { primary_key: pk1, .. }, Self::Borrowed { primary_key: pk2, .. }) => pk1.partial_cmp(pk2),
            // Different variants: order by variant (Dehydrated < Owned < Hydrated < Borrowed)
            (Self::Dehydrated { .. }, Self::Owned { .. }) => Some(std::cmp::Ordering::Less),
            (Self::Dehydrated { .. }, Self::Hydrated { .. }) => Some(std::cmp::Ordering::Less),
            (Self::Dehydrated { .. }, Self::Borrowed { .. }) => Some(std::cmp::Ordering::Less),
            (Self::Owned { .. }, Self::Dehydrated { .. }) => Some(std::cmp::Ordering::Greater),
            (Self::Owned { .. }, Self::Hydrated { .. }) => Some(std::cmp::Ordering::Less),
            (Self::Owned { .. }, Self::Borrowed { .. }) => Some(std::cmp::Ordering::Less),
            (Self::Hydrated { .. }, Self::Dehydrated { .. }) => Some(std::cmp::Ordering::Greater),
            (Self::Hydrated { .. }, Self::Owned { .. }) => Some(std::cmp::Ordering::Greater),
            (Self::Hydrated { .. }, Self::Borrowed { .. }) => Some(std::cmp::Ordering::Less),
            (Self::Borrowed { .. }, Self::Dehydrated { .. }) => Some(std::cmp::Ordering::Greater),
            (Self::Borrowed { .. }, Self::Owned { .. }) => Some(std::cmp::Ordering::Greater),
            (Self::Borrowed { .. }, Self::Hydrated { .. }) => Some(std::cmp::Ordering::Greater),
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
            // Same variants: compare by primary key
            (Self::Dehydrated { primary_key: pk1, .. }, Self::Dehydrated { primary_key: pk2, .. }) => pk1.cmp(pk2),
            (Self::Owned { primary_key: pk1, .. }, Self::Owned { primary_key: pk2, .. }) => pk1.cmp(pk2),
            (Self::Hydrated { primary_key: pk1, .. }, Self::Hydrated { primary_key: pk2, .. }) => pk1.cmp(pk2),
            (Self::Borrowed { primary_key: pk1, .. }, Self::Borrowed { primary_key: pk2, .. }) => pk1.cmp(pk2),
            // Different variants: order by variant (Dehydrated < Owned < Hydrated < Borrowed)
            (Self::Dehydrated { .. }, Self::Owned { .. }) => std::cmp::Ordering::Less,
            (Self::Dehydrated { .. }, Self::Hydrated { .. }) => std::cmp::Ordering::Less,
            (Self::Dehydrated { .. }, Self::Borrowed { .. }) => std::cmp::Ordering::Less,
            (Self::Owned { .. }, Self::Dehydrated { .. }) => std::cmp::Ordering::Greater,
            (Self::Owned { .. }, Self::Hydrated { .. }) => std::cmp::Ordering::Less,
            (Self::Owned { .. }, Self::Borrowed { .. }) => std::cmp::Ordering::Less,
            (Self::Hydrated { .. }, Self::Dehydrated { .. }) => std::cmp::Ordering::Greater,
            (Self::Hydrated { .. }, Self::Owned { .. }) => std::cmp::Ordering::Greater,
            (Self::Hydrated { .. }, Self::Borrowed { .. }) => std::cmp::Ordering::Less,
            (Self::Borrowed { .. }, Self::Dehydrated { .. }) => std::cmp::Ordering::Greater,
            (Self::Borrowed { .. }, Self::Owned { .. }) => std::cmp::Ordering::Greater,
            (Self::Borrowed { .. }, Self::Hydrated { .. }) => std::cmp::Ordering::Greater,
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

    /// Create a new owned relational link with a Box-owned model
    /// This variant owns the model completely and has no lifetime dependencies
    pub fn new_owned(
        primary_key: <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary<'static>,
        model: M,
    ) -> Self {
        Self::Owned {
            primary_key,
            model: Box::new(model),
            _source: std::marker::PhantomData,
        }
    }

    /// Create a new borrowed relational link from an AccessGuard-backed reference
    /// This variant is used when loading models from the database
    /// The lifetime 'data is tied to the AccessGuard lifetime chain:
    /// Transaction<'txn> -> Table<'txn> -> AccessGuard<'txn> -> Value<'txn>
    pub fn new_borrowed(
        primary_key: <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary<'static>,
        model: &'data M,
    ) -> Self {
        Self::Borrowed {
            primary_key,
            model,
            _source: std::marker::PhantomData,
        }
    }

    /// Get the primary key from the relation
    pub fn get_primary_key(&self) -> &<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary<'static> {
        match self {
            Self::Dehydrated { primary_key, .. } => primary_key,
            Self::Owned { primary_key, .. } => primary_key,
            Self::Hydrated { primary_key, .. } => primary_key,
            Self::Borrowed { primary_key, .. } => primary_key,
        }
    }

    /// Check if this relation is currently hydrated (contains model data)
    /// Returns true for Owned, Hydrated, and Borrowed variants
    pub fn is_hydrated(&self) -> bool {
        matches!(self, Self::Owned { .. } | Self::Hydrated { .. } | Self::Borrowed { .. })
    }

    /// Check if this relation is dehydrated (contains only primary key)
    pub fn is_dehydrated(&self) -> bool {
        matches!(self, Self::Dehydrated { .. })
    }

    /// Check if this relation is owned (fully owns the model)
    pub fn is_owned(&self) -> bool {
        matches!(self, Self::Owned { .. })
    }

    /// Check if this relation is borrowed (from AccessGuard)
    pub fn is_borrowed(&self) -> bool {
        matches!(self, Self::Borrowed { .. })
    }

    /// Consume the link and extract the owned model if it's an Owned variant
    /// Returns None for other variants
    pub fn into_owned(self) -> Option<M> {
        match self {
            Self::Owned { model, .. } => Some(*model),
            _ => None,
        }
    }

    /// Get the hydrated model if available, otherwise None
    /// Works for Owned, Hydrated, and Borrowed variants
    pub fn get_model(&self) -> Option<&M> {
        match self {
            Self::Owned { model, .. } => Some(model.as_ref()),
            Self::Hydrated { model, .. } => Some(model),
            Self::Borrowed { model, .. } => Some(model),
            Self::Dehydrated { .. } => None,
        }
    }

    /// Get borrowed model reference if available
    /// This is an alias for get_model() but with a more explicit name
    /// Works for Owned (derefs Box), Hydrated, and Borrowed variants
    pub fn as_borrowed(&self) -> Option<&M> {
        self.get_model()
    }

    /// Convert to owned/dehydrated - useful when you need to persist
    /// Extracts the primary key and discards the model reference
    pub fn to_owned_key(self) -> Self {
        let primary_key = match self {
            Self::Dehydrated { primary_key, .. } => primary_key,
            Self::Owned { primary_key, .. } => primary_key,
            Self::Hydrated { primary_key, .. } => primary_key,
            Self::Borrowed { primary_key, .. } => primary_key,
        };
        Self::Dehydrated {
            primary_key,
            _source: std::marker::PhantomData,
        }
    }

    /// Convert a dehydrated relation to hydrated by providing the model data
    pub fn hydrate_with_model(self, model: &'data M) -> Self {
        let primary_key = match self {
            Self::Dehydrated { primary_key, .. } => primary_key,
            Self::Owned { primary_key, .. } => primary_key,
            Self::Hydrated { primary_key, .. } => primary_key,
            Self::Borrowed { primary_key, .. } => primary_key,
        };
        Self::Hydrated {
            primary_key,
            model,
            _source: std::marker::PhantomData,
        }
    }

    /// Convert a hydrated or borrowed relation back to dehydrated
    pub fn dehydrate(self) -> Self {
        let primary_key = match self {
            Self::Dehydrated { primary_key, .. } => primary_key,
            Self::Owned { primary_key, .. } => primary_key,
            Self::Hydrated { primary_key, .. } => primary_key,
            Self::Borrowed { primary_key, .. } => primary_key,
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
            Self::Owned { primary_key, model, .. } => {
                // Owned serializes as variant 1 with both key and model data
                use serde::ser::SerializeTupleVariant;
                let mut tv = serializer.serialize_tuple_variant("RelationalLink", 1, "Owned", 2)?;
                tv.serialize_field(primary_key)?;
                tv.serialize_field(model.as_ref())?;
                tv.end()
            }
            Self::Hydrated { primary_key, model, .. } => {
                // Hydrated also serializes as variant 1 with both key and model data
                use serde::ser::SerializeTupleVariant;
                let mut tv = serializer.serialize_tuple_variant("RelationalLink", 1, "Hydrated", 2)?;
                tv.serialize_field(primary_key)?;
                tv.serialize_field(model)?;
                tv.end()
            }
            Self::Borrowed { primary_key, .. } => {
                // Borrowed converts to Dehydrated on serialization (can't serialize DB references)
                serializer.serialize_newtype_variant("RelationalLink", 0, "Dehydrated", primary_key)
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
            Self::Owned { primary_key, model, .. } => {
                // Owned encodes as variant 1 with both key and model data
                1u32.encode(encoder)?;
                primary_key.encode(encoder)?;
                model.as_ref().encode(encoder)?;
            }
            Self::Hydrated { primary_key, model, .. } => {
                // Hydrated also encodes as variant 1 with both key and model data
                1u32.encode(encoder)?;
                primary_key.encode(encoder)?;
                model.encode(encoder)?;
            }
            Self::Borrowed { primary_key, .. } => {
                // Borrowed encodes as Dehydrated (variant 0) - can't serialize DB references
                0u32.encode(encoder)?;
                primary_key.encode(encoder)?;
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
                // Variant 0: Dehydrated (only key)
                let primary_key = <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary<'static> as Decode<C>>::decode(decoder)?;
                Ok(Self::Dehydrated {
                    primary_key,
                    _source: std::marker::PhantomData,
                })
            }
            1 => {
                // Variant 1: Key + Model data - decode into Owned variant
                let primary_key = <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary<'static> as Decode<C>>::decode(decoder)?;
                let model = M::decode(decoder)?;
                Ok(Self::Owned {
                    primary_key,
                    model: Box::new(model),
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
        let variant = u32::decode(decoder)?;
        match variant {
            0 => {
                // Variant 0: Dehydrated (only key)
                let primary_key = <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary<'static> as BorrowDecode<'de, C>>::borrow_decode(decoder)?;
                Ok(Self::Dehydrated {
                    primary_key,
                    _source: std::marker::PhantomData,
                })
            }
            1 => {
                // Variant 1: Key + Model data - decode into Owned variant
                let primary_key = <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary<'static> as BorrowDecode<'de, C>>::borrow_decode(decoder)?;
                let model = M::borrow_decode(decoder)?;
                Ok(Self::Owned {
                    primary_key,
                    model: Box::new(model),
                    _source: std::marker::PhantomData,
                })
            }
            _ => Err(bincode::error::DecodeError::Other("Invalid RelationalLink variant")),
        }
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
