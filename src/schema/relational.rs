//! Relational linking system for type-safe foreign key relationships.
//!
//! This module provides `RelationalLink<R, SourceD, TargetD, M>` which represents
//! a relationship between models while enforcing repository isolation at compile time.
//!
//! # Four Link Variants
//!
//! 1. **Dehydrated**: Only stores the primary key (minimal memory)
//! 2. **Owned**: Owns the full model in a Box (independent lifetime)
//! 3. **Hydrated**: Borrowed reference with application-controlled lifetime
//! 4. **Borrowed**: Borrowed reference tied to database AccessGuard
//!
//! # Repository Isolation
//!
//! The `R` type parameter enforces that both source and target models belong
//! to the same repository. This prevents unauthorized cross-repository references.
//!
//! Repository isolation is enforced at compile time - incompatible repositories
//! will cause type errors.
//!
//! # Common Patterns
//!
//! ## Creating Links
//!
//! ```rust,no_run
//! use netabase_store::doc_example::*;
//! use netabase_store::relational::RelationalLink;
//! use netabase_store::traits::registry::repository::Standalone;
//!
//! // Dehydrated link (for storage) - just the key
//! let author_id = AuthorID("author123".into());
//! let link: RelationalLink<Standalone, ExampleDef, ExampleDef, Author> =
//!     RelationalLink::new_dehydrated(author_id);
//! ```
//!
//! ## Hydration (Loading Related Data)
//!
//! Hydration loads the full model data for a relational link. This typically happens
//! through transaction methods that can fetch the related model from the database.
//!
//! # Serialization Behavior
//!
//! When serializing, all variants convert to the dehydrated form (key only).
//! This ensures:
//! - Compact wire format
//! - No accidental data duplication
//! - Consistent serialization regardless of hydration state
//!
//! # Use Cases
//!
//! - **User -> Posts**: One-to-many relationships
//! - **Post -> Author**: Many-to-one relationships
//! - **Team -> Members**: Many-to-many (via intermediate model)
//! - **Document -> Attachments**: Hierarchical data
//!
//! # Limitations
//!
//! - Cannot cross repository boundaries (enforced at compile time)
//! - Target model must have a primary key type
//! - No automatic cascade delete (must be handled manually)
//! - Circular references must be carefully managed

use crate::traits::registry::{
    definition::NetabaseDefinition,
    models::{keys::NetabaseModelKeys, model::NetabaseModel, treenames::ModelTreeNames},
    repository::{InRepository, NetabaseRepository, RepositoryPermissions},
};
use serde::{Serialize, Deserialize};
use strum::IntoDiscriminant;

/// Permission flag for relational access control.
///
/// Determines whether related data can be read-only or modified.
pub enum PermissionFlag {
    /// Read-only access to related data
    ReadOnly,
    /// Full read-write access to related data  
    ReadWrite
}

pub struct RelationPermission<'tree_name, D: NetabaseDefinition, M: NetabaseModel<D>>(pub ModelTreeNames<'tree_name, D, M>, pub PermissionFlag)
where
    D::Discriminant: 'static + std::fmt::Debug,
    M: NetabaseModel<D>,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: IntoDiscriminant,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: IntoDiscriminant,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: IntoDiscriminant,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug;

pub struct ModelRelationPermissions<'source, 'tree_name, D: NetabaseDefinition, M: NetabaseModel<D>>
where
    D::Discriminant: 'static + std::fmt::Debug,
    M: NetabaseModel<D>,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: IntoDiscriminant,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: IntoDiscriminant,
    <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: IntoDiscriminant,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Libp2p as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
{
    pub relationa_tree_access: & 'source [RelationPermission<'tree_name, D, M>],
}

/// A relational link between models within a repository context.
///
/// This type enforces repository isolation at compile time through the `R` type parameter.
/// Both source and target definitions must belong to the same repository.
///
/// # Type Parameters
///
/// - `'data`: Lifetime for borrowed references
/// - `R`: Repository type providing isolation context
/// - `SourceD`: Source definition type
/// - `TargetD`: Target definition type
/// - `M`: Target model type
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
/// # Security Model
///
/// The repository type parameter `R` ensures compile-time isolation:
/// - Both `SourceD` and `TargetD` must implement `InRepository<R>`
/// - Links cannot cross repository boundaries
/// - This prevents unauthorized access between unrelated definitions
///
/// # Example
///
/// ```rust,no_run
/// use netabase_store::doc_example::*;
/// use netabase_store::relational::RelationalLink;
/// use netabase_store::traits::registry::repository::Standalone;
///
/// // Create a dehydrated link to an Author
/// let author_id = AuthorID("author1".into());
/// let link: RelationalLink<Standalone, ExampleDef, ExampleDef, Author> =
///     RelationalLink::new_dehydrated(author_id);
/// ```
#[derive(Clone)]
pub enum RelationalLink<'data, R, SourceD, TargetD, M>
where
    R: NetabaseRepository,
    SourceD: NetabaseDefinition + InRepository<R> + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + InRepository<R> + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registry::models::model::NetabaseModel<TargetD>,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Relational as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Blob as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription as strum::IntoDiscriminant>::Discriminant: 'static,
{
    /// Dehydrated: Contains only the primary key
    Dehydrated {
        primary_key: <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Primary,
        _source: SourceD::DebugName,
        _repo: std::marker::PhantomData<R>,
    },
    /// Owned: Fully owns the related model (no lifetime dependency)
    /// Used when the model is constructed independently and needs to be stored with full ownership
    Owned {
        primary_key: <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Primary,
        model: Box<M>,
        _source: SourceD::DebugName,
        _repo: std::marker::PhantomData<R>,
    },
    /// Borrowed: Contains both the primary key and a borrowed reference from AccessGuard
    /// Lifetime is tied to database transaction -> table -> AccessGuard chain
    Borrowed {
        primary_key: <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Primary,
        model: &'data M,
        _source: SourceD::DebugName,
        _repo: std::marker::PhantomData<R>,
    },
}

// PartialEq implementation for RelationalLink
impl<'data, R, SourceD, TargetD, M> PartialEq for RelationalLink<'data, R, SourceD, TargetD, M>
where
    R: NetabaseRepository,
    SourceD: NetabaseDefinition + InRepository<R> + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + InRepository<R> + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registry::models::model::NetabaseModel<TargetD>,
    <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Primary: PartialEq,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Relational as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Blob as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription as strum::IntoDiscriminant>::Discriminant: 'static,
{
    fn eq(&self, other: &Self) -> bool {
        // All variants compare equal if primary keys match
        let pk1 = match self {
            Self::Dehydrated { primary_key, .. } => primary_key,
            Self::Owned { primary_key, .. } => primary_key,
            Self::Borrowed { primary_key, .. } => primary_key,
        };
        let pk2 = match other {
            Self::Dehydrated { primary_key, .. } => primary_key,
            Self::Owned { primary_key, .. } => primary_key,
            Self::Borrowed { primary_key, .. } => primary_key,
        };
        pk1 == pk2
    }
}

// Eq implementation
impl<'data, R, SourceD, TargetD, M> Eq for RelationalLink<'data, R, SourceD, TargetD, M>
where
    R: NetabaseRepository,
    SourceD: NetabaseDefinition + InRepository<R> + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + InRepository<R> + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registry::models::model::NetabaseModel<TargetD>,
    <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Primary: Eq,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Relational as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Blob as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription as strum::IntoDiscriminant>::Discriminant: 'static,
{}

// Hash implementation
impl<'data, R, SourceD, TargetD, M> std::hash::Hash for RelationalLink<'data, R, SourceD, TargetD, M>
where
    R: NetabaseRepository,
    SourceD: NetabaseDefinition + InRepository<R> + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + InRepository<R> + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registry::models::model::NetabaseModel<TargetD> + std::hash::Hash,
    <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Primary: std::hash::Hash,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Relational as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Blob as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription as strum::IntoDiscriminant>::Discriminant: 'static,
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
            Self::Borrowed { primary_key, model, .. } => {
                3u8.hash(state);
                primary_key.hash(state);
                model.hash(state);
            }
        }
    }
}

// PartialOrd implementation
#[allow(clippy::non_canonical_partial_ord_impl)]
impl<'data, R, SourceD, TargetD, M> PartialOrd for RelationalLink<'data, R, SourceD, TargetD, M>
where
    R: NetabaseRepository,
    SourceD: NetabaseDefinition + InRepository<R> + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + InRepository<R> + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registry::models::model::NetabaseModel<TargetD> + PartialOrd,
    <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Primary: PartialOrd,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Relational as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Blob as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription as strum::IntoDiscriminant>::Discriminant: 'static,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            // Same variants: compare by primary key
            (Self::Dehydrated { primary_key: pk1, .. }, Self::Dehydrated { primary_key: pk2, .. }) => pk1.partial_cmp(pk2),
            (Self::Owned { primary_key: pk1, .. }, Self::Owned { primary_key: pk2, .. }) => pk1.partial_cmp(pk2),
            (Self::Borrowed { primary_key: pk1, .. }, Self::Borrowed { primary_key: pk2, .. }) => pk1.partial_cmp(pk2),
            // Different variants: order by variant (Dehydrated < Owned < Hydrated < Borrowed)
            (Self::Dehydrated { .. }, Self::Owned { .. }) => Some(std::cmp::Ordering::Less),
            (Self::Dehydrated { .. }, Self::Borrowed { .. }) => Some(std::cmp::Ordering::Less),
            (Self::Owned { .. }, Self::Dehydrated { .. }) => Some(std::cmp::Ordering::Greater),
            (Self::Owned { .. }, Self::Borrowed { .. }) => Some(std::cmp::Ordering::Less),
            (Self::Borrowed { .. }, Self::Dehydrated { .. }) => Some(std::cmp::Ordering::Greater),
            (Self::Borrowed { .. }, Self::Owned { .. }) => Some(std::cmp::Ordering::Greater),
        }
    }
}

// Ord implementation
impl<'data, R, SourceD, TargetD, M> Ord for RelationalLink<'data, R, SourceD, TargetD, M>
where
    R: NetabaseRepository,
    SourceD: NetabaseDefinition + InRepository<R> + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + InRepository<R> + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registry::models::model::NetabaseModel<TargetD> + Ord,
    <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Primary: Ord,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Relational as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Blob as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription as strum::IntoDiscriminant>::Discriminant: 'static,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            // Same variants: compare by primary key
            (Self::Dehydrated { primary_key: pk1, .. }, Self::Dehydrated { primary_key: pk2, .. }) => pk1.cmp(pk2),
            (Self::Owned { primary_key: pk1, .. }, Self::Owned { primary_key: pk2, .. }) => pk1.cmp(pk2),
            (Self::Borrowed { primary_key: pk1, .. }, Self::Borrowed { primary_key: pk2, .. }) => pk1.cmp(pk2),
            // Different variants: order by variant (Dehydrated < Owned < Hydrated < Borrowed)
            (Self::Dehydrated { .. }, Self::Owned { .. }) => std::cmp::Ordering::Less,
            (Self::Dehydrated { .. }, Self::Borrowed { .. }) => std::cmp::Ordering::Less,
            (Self::Owned { .. }, Self::Dehydrated { .. }) => std::cmp::Ordering::Greater,
            (Self::Owned { .. }, Self::Borrowed { .. }) => std::cmp::Ordering::Less,
            (Self::Borrowed { .. }, Self::Dehydrated { .. }) => std::cmp::Ordering::Greater,
            (Self::Borrowed { .. }, Self::Owned { .. }) => std::cmp::Ordering::Greater,
        }
    }
}

// Debug implementation
impl<'data, R, SourceD, TargetD, M> std::fmt::Debug for RelationalLink<'data, R, SourceD, TargetD, M>
where
    R: NetabaseRepository,
    SourceD: NetabaseDefinition + InRepository<R> + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + InRepository<R> + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registry::models::model::NetabaseModel<TargetD> + std::fmt::Debug,
    <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Primary: std::fmt::Debug,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Relational as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Blob as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription as strum::IntoDiscriminant>::Discriminant: 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dehydrated { primary_key, _source, .. } => {
                f.debug_struct("RelationalLink::Dehydrated")
                    .field("primary_key", primary_key)
                    .field("source", _source)
                    .finish()
            }
            Self::Owned { primary_key, model, _source, .. } => {
                f.debug_struct("RelationalLink::Owned")
                    .field("primary_key", primary_key)
                    .field("model", model)
                    .field("source", _source)
                    .finish()
            }
            Self::Borrowed { primary_key, model, _source, .. } => {
                f.debug_struct("RelationalLink::Borrowed")
                    .field("primary_key", primary_key)
                    .field("model", model)
                    .field("source", _source)
                    .finish()
            }
        }
    }
}

// Implementation for RelationalLink
impl<'data, R, SourceD, TargetD, M> RelationalLink<'data, R, SourceD, TargetD, M>
where
    R: NetabaseRepository,
    SourceD: NetabaseDefinition + InRepository<R> + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + InRepository<R> + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registry::models::model::NetabaseModel<TargetD>,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Relational as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Blob as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription as strum::IntoDiscriminant>::Discriminant: 'static,
{
    /// Create a new dehydrated relational link with just the primary key
    #[inline]
    pub fn new_dehydrated(
        primary_key: <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Primary,
    ) -> Self {
        Self::Dehydrated {
            primary_key,
            _source: SourceD::debug_name(),
            _repo: std::marker::PhantomData,
        }
    }


    /// Create a new owned relational link with a Box-owned model
    /// This variant owns the model completely and has no lifetime dependencies
    #[inline]
    pub fn new_owned(
        primary_key: <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Primary,
        model: M,
    ) -> Self {
        Self::Owned {
            primary_key,
            model: Box::new(model),
            _source: SourceD::debug_name(),
            _repo: std::marker::PhantomData,
        }
    }

    /// Create a new borrowed relational link from an AccessGuard-backed reference
    /// This variant is used when loading models from the database
    /// The lifetime 'data is tied to the AccessGuard lifetime chain:
    /// Transaction<'txn> -> Table<'txn> -> AccessGuard<'txn> -> Value<'txn>
    #[inline]
    pub fn new_borrowed(
        primary_key: <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Primary,
        model: &'data M,
    ) -> Self {
        Self::Borrowed {
            primary_key,
            model,
            _source: SourceD::debug_name(),
            _repo: std::marker::PhantomData,
        }
    }

    /// Get the primary key from the relation
    #[inline]
    pub fn get_primary_key(&self) -> &<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Primary {
        match self {
            Self::Dehydrated { primary_key, .. } => primary_key,
            Self::Owned { primary_key, .. } => primary_key,
            Self::Borrowed { primary_key, .. } => primary_key,
        }
    }


    /// Check if this relation is dehydrated (contains only primary key)
    #[inline]
    pub fn is_dehydrated(&self) -> bool {
        matches!(self, Self::Dehydrated { .. })
    }

    /// Check if this relation is owned (fully owns the model)
    #[inline]
    pub fn is_owned(&self) -> bool {
        matches!(self, Self::Owned { .. })
    }

    /// Check if this relation is borrowed (from AccessGuard)
    #[inline]
    pub fn is_borrowed(&self) -> bool {
        matches!(self, Self::Borrowed { .. })
    }

    /// Consume the link and extract the owned model if it's an Owned variant
    /// Returns None for other variants
    #[inline]
    pub fn into_owned(self) -> Option<M> {
        match self {
            Self::Owned { model, .. } => Some(*model),
            _ => None,
        }
    }

    /// Get the hydrated model if available, otherwise None
    /// Works for Owned, Hydrated, and Borrowed variants
    #[inline]
    pub fn get_model(&self) -> Option<&M> {
        match self {
            Self::Owned { model, .. } => Some(model.as_ref()),
            Self::Borrowed { model, .. } => Some(model),
            Self::Dehydrated { .. } => None,
        }
    }

    /// Get borrowed model reference if available
    /// This is an alias for get_model() but with a more explicit name
    /// Works for Owned (derefs Box), Hydrated, and Borrowed variants
    #[inline]
    pub fn as_borrowed(&self) -> Option<&M> {
        self.get_model()
    }

    /// Convert to owned/dehydrated - useful when you need to persist
    /// Extracts the primary key and discards the model reference
    #[inline]
    pub fn to_owned_key(self) -> Self {
        let primary_key = match self {
            Self::Dehydrated { primary_key, .. } => primary_key,
            Self::Owned { primary_key, .. } => primary_key,
            Self::Borrowed { primary_key, .. } => primary_key,
        };
        Self::Dehydrated {
            primary_key,
            _source: SourceD::debug_name(),
            _repo: std::marker::PhantomData,
        }
    }


    /// Convert a hydrated or borrowed relation back to dehydrated
    #[inline]
    pub fn dehydrate(self) -> Self {
        let primary_key = match self {
            Self::Dehydrated { primary_key, .. } => primary_key,
            Self::Owned { primary_key, .. } => primary_key,
            Self::Borrowed { primary_key, .. } => primary_key,
        };
        Self::Dehydrated {
            primary_key,
            _source: SourceD::debug_name(),
            _repo: std::marker::PhantomData,
        }
    }

    /// Check if this is a same-definition relation (SourceD == TargetD)
    #[inline]
    pub fn is_same_definition() -> bool {
        std::any::TypeId::of::<SourceD>() == std::any::TypeId::of::<TargetD>()
    }

    /// Check if this is a cross-definition relation (SourceD != TargetD)
    #[inline]
    pub fn is_cross_definition() -> bool {
        !Self::is_same_definition()
    }

    /// Validate that this link can be accessed within the given repository permissions.
    #[inline]
    pub fn validate_repository_access(&self, _perms: &RepositoryPermissions<R>) -> Result<(), RelationalLinkError> {
        // For now, always allow - permissions will be checked more thoroughly
        // when actual hydration from database occurs
        Ok(())
    }
}

// ============================================================================
// Hydration Support
// ============================================================================

/// Trait for hydrating relational links.
/// 
/// This trait is implemented for `RelationalLink` with different bounds
/// depending on whether hydration is within the same definition or across
/// definitions.
pub trait Hydratable<'data, 'db> {
    type Hydrated;
    type Error;
    type Transaction;
    
    /// Hydrate this link using the provided transaction.
    fn hydrate(self, txn: &'db Self::Transaction) -> Result<Self::Hydrated, Self::Error>;
}

impl<'data, R, SourceD, TargetD, M> RelationalLink<'data, R, SourceD, TargetD, M>
where
    R: NetabaseRepository,
    SourceD: NetabaseDefinition + InRepository<R> + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + InRepository<R> + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registry::models::model::NetabaseModel<TargetD>,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Relational as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Blob as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription as strum::IntoDiscriminant>::Discriminant: 'static,
{
    /// Create an owned (hydrated) link from a model that was loaded separately.
    ///
    /// This is useful when you've already loaded the model and want to create
    /// a hydrated link from it.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// // Load the model separately
    /// let author: Author = txn.read(&author_id)?.unwrap();
    /// 
    /// // Create a hydrated link
    /// let link = RelationalLink::from_loaded(author_id, author);
    /// ```
    pub fn from_loaded(
        primary_key: <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Primary,
        model: M,
    ) -> Self {
        Self::Owned {
            primary_key,
            model: Box::new(model),
            _source: SourceD::debug_name(),
            _repo: std::marker::PhantomData,
        }
    }
}

// ============================================================================
// Same-Definition Hydration (SourceD == TargetD)
// ============================================================================

/// Implementation for hydrating links within the same definition.
/// 
/// When `SourceD` and `TargetD` are the same, we can use a single definition
/// transaction to load the related model.
impl<'data, 'db, R, D, M> RelationalLink<'data, R, D, D, M>
where
    R: NetabaseRepository,
    D: crate::traits::registry::definition::redb_definition::RedbDefinition 
        + InRepository<R> 
        + Clone 
        + 'static,
    D::Discriminant: std::fmt::Debug + 'static,
    M: crate::traits::registry::models::model::NetabaseModel<D>
       + crate::traits::registry::models::model::redb_model::RedbNetbaseModel<'db, D>
       + crate::databases::redb::transaction::crud::RedbModelCrud<'db, D>
       + Clone,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Secondary as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Relational as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Blob as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Subscription as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Libp2p as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Primary: redb::Key + Clone + 'static,
    <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Secondary: redb::Key + Clone + 'static,
    <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Relational: redb::Key + Clone + 'static,
    <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
    <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Subscription: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Blob as crate::traits::registry::models::keys::blob::NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
    for<'a> <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
    for<'a> <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Blob: std::borrow::Borrow<<<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Blob as redb::Value>::SelfType<'a>>,
    for<'a> <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Blob as crate::traits::registry::models::keys::blob::NetabaseModelBlobKey<D, M>>::BlobItem: std::borrow::Borrow<<<<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Blob as crate::traits::registry::models::keys::blob::NetabaseModelBlobKey<D, M>>::BlobItem as redb::Value>::SelfType<'a>>,
    for<'a> <M as crate::traits::registry::models::model::redb_model::RedbNetbaseModel<'db, D>>::TableV: redb::Value<SelfType<'a> = M>,
    D::SubscriptionKeys: redb::Key + 'static,
{
    /// Hydrate this link using a transaction from the same definition.
    ///
    /// Since both source and target are in the same definition `D`, we can
    /// use a single `RedbTransaction<D>` to load the related model.
    ///
    /// # Arguments
    ///
    /// * `txn` - A read transaction for definition `D`
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` with the `Owned` variant containing the loaded model
    /// * `Err(RelationalLinkError::NotFound)` if the model doesn't exist
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// // Within a definition where Post has a link to Author
    /// let post: Post = txn.read(&post_id)?.unwrap();
    /// 
    /// // Hydrate the author link (both Post and Author are in same definition)
    /// let hydrated_link = post.author.hydrate(&txn, &author_id)?;
    /// 
    /// // Access the author
    /// let author = hydrated_link.get_model().unwrap();
    /// ```
    pub fn hydrate(
        self,
        txn: &crate::databases::redb::transaction::RedbTransaction<'db, D>,
        // Pass the key by reference - will be cloned internally
        key: &<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Primary,
    ) -> Result<Self, RelationalLinkError>
    {
        // If already hydrated, return as-is
        if self.get_model().is_some() {
            return Ok(self);
        }
        
        // Read the model from the database using read_by_key (takes owned key)
        let model = txn.read_by_key::<M>(key.clone())
            .map_err(|_| RelationalLinkError::NotFound)?
            .ok_or(RelationalLinkError::NotFound)?;
        
        // Return an owned link with the loaded model
        Ok(Self::from_loaded(key.clone(), model))
    }
    
    /// Convenience method to hydrate using the link's stored key.
    /// 
    /// This creates an owned copy of the key for the read operation.
    /// For better performance when hydrating many links, consider
    /// storing keys separately and using `hydrate` directly.
    pub fn hydrate_self(
        self,
        txn: &crate::databases::redb::transaction::RedbTransaction<'db, D>,
    ) -> Result<Self, RelationalLinkError>
    {
        // If already hydrated, return as-is
        if self.get_model().is_some() {
            return Ok(self);
        }
        
        // Clone the primary key
        let pk = self.get_primary_key().clone();
        
        // Read the model from the database using read_by_key (takes owned key)
        let model = txn.read_by_key::<M>(pk.clone())
            .map_err(|_| RelationalLinkError::NotFound)?
            .ok_or(RelationalLinkError::NotFound)?;
        
        // Return an owned link with the loaded model
        Ok(Self::from_loaded(pk, model))
    }
    
    /// Hydrate this link if it's not already hydrated.
    ///
    /// Convenience method that only performs a database read if the link
    /// is currently dehydrated.
    pub fn hydrate_if_needed(
        self,
        txn: &crate::databases::redb::transaction::RedbTransaction<'db, D>,
    ) -> Result<Self, RelationalLinkError>
    {
        if self.is_dehydrated() {
            self.hydrate_self(txn)
        } else {
            Ok(self)
        }
    }
}

// ============================================================================
// Vec<RelationalLink> Hydration Helpers
// ============================================================================

/// Extension trait for hydrating vectors of relational links.
pub trait HydrateVec<'data, 'db, R, D, M>
where
    R: NetabaseRepository,
    D: crate::traits::registry::definition::redb_definition::RedbDefinition + InRepository<R> + 'static,
    D::Discriminant: std::fmt::Debug,
    M: crate::traits::registry::models::model::NetabaseModel<D>,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Secondary as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Relational as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Blob as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Subscription as strum::IntoDiscriminant>::Discriminant: 'static,
{
    /// Hydrate all links in this vector.
    ///
    /// Fails if any link cannot be hydrated.
    fn hydrate_all(
        self,
        txn: &crate::databases::redb::transaction::RedbTransaction<'db, D>,
    ) -> Result<Vec<RelationalLink<'data, R, D, D, M>>, RelationalLinkError>;
    
    /// Hydrate as many links as possible, keeping dehydrated versions for failures.
    fn hydrate_best_effort(
        self,
        txn: &crate::databases::redb::transaction::RedbTransaction<'db, D>,
    ) -> Vec<RelationalLink<'data, R, D, D, M>>;
}

impl<'data, 'db, R, D, M> HydrateVec<'data, 'db, R, D, M> for Vec<RelationalLink<'data, R, D, D, M>>
where
    R: NetabaseRepository,
    D: crate::traits::registry::definition::redb_definition::RedbDefinition 
        + InRepository<R> 
        + Clone 
        + 'static,
    D::Discriminant: std::fmt::Debug + 'static,
    M: crate::traits::registry::models::model::NetabaseModel<D>
       + crate::traits::registry::models::model::redb_model::RedbNetbaseModel<'db, D>
       + crate::databases::redb::transaction::crud::RedbModelCrud<'db, D>
       + Clone,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Secondary as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Relational as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Blob as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Subscription as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Libp2p as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Primary: redb::Key + Clone + 'static,
    <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Secondary: redb::Key + Clone + 'static,
    <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Relational: redb::Key + Clone + 'static,
    <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
    <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Subscription: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Blob as crate::traits::registry::models::keys::blob::NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
    for<'a> <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Primary: std::borrow::Borrow<<<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Primary as redb::Value>::SelfType<'a>>,
    for<'a> <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Blob: std::borrow::Borrow<<<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Blob as redb::Value>::SelfType<'a>>,
    for<'a> <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Blob as crate::traits::registry::models::keys::blob::NetabaseModelBlobKey<D, M>>::BlobItem: std::borrow::Borrow<<<<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Blob as crate::traits::registry::models::keys::blob::NetabaseModelBlobKey<D, M>>::BlobItem as redb::Value>::SelfType<'a>>,
    for<'a> <M as crate::traits::registry::models::model::redb_model::RedbNetbaseModel<'db, D>>::TableV: redb::Value<SelfType<'a> = M>,
    D::SubscriptionKeys: redb::Key + 'static,
{
    fn hydrate_all(
        self,
        txn: &crate::databases::redb::transaction::RedbTransaction<'db, D>,
    ) -> Result<Vec<RelationalLink<'data, R, D, D, M>>, RelationalLinkError>
    {
        self.into_iter()
            .map(|link| link.hydrate_self(txn))
            .collect()
    }
    
    fn hydrate_best_effort(
        self,
        txn: &crate::databases::redb::transaction::RedbTransaction<'db, D>,
    ) -> Vec<RelationalLink<'data, R, D, D, M>>
    {
        self.into_iter()
            .map(|link| {
                let pk = link.get_primary_key().clone();
                link.hydrate_self(txn).unwrap_or_else(|_| RelationalLink::new_dehydrated(pk))
            })
            .collect()
    }
}

impl<'data, R, SourceD, TargetD, M> Serialize for RelationalLink<'data, R, SourceD, TargetD, M>
where
    R: NetabaseRepository,
    SourceD: NetabaseDefinition + InRepository<R> + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + InRepository<R> + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registry::models::model::NetabaseModel<TargetD> + Serialize,
    <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Primary: Serialize,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Relational as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Blob as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription as strum::IntoDiscriminant>::Discriminant: 'static,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // For database storage, we only serialize the primary key.
        // The model data (if present) is not persisted in the link itself.
        // When deserializing, we always get a Dehydrated variant.
        match self {
            Self::Dehydrated { primary_key, .. }
            | Self::Owned { primary_key, .. }
            | Self::Borrowed { primary_key, .. } => {
                primary_key.serialize(serializer)
            }
        }
    }
}

impl<'de, 'data, R, SourceD, TargetD, M> Deserialize<'de> for RelationalLink<'data, R, SourceD, TargetD, M>
where
    R: NetabaseRepository,
    SourceD: NetabaseDefinition + InRepository<R> + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + InRepository<R> + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registry::models::model::NetabaseModel<TargetD>,
    <M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Primary: Deserialize<'de>,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Relational as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Blob as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registry::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription as strum::IntoDiscriminant>::Discriminant: 'static,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let proxy = <M::Keys as NetabaseModelKeys<TargetD, M>>::Primary::deserialize(deserializer)?;
        Ok(RelationalLink::Dehydrated {
            primary_key: proxy,
            _source: SourceD::debug_name(),
            _repo: std::marker::PhantomData,
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

    #[error("Repository access error: cannot access definition outside of repository context")]
    RepositoryAccessError,
}
