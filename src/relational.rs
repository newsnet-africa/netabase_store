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
//! to the same repository. This prevents unauthorized cross-repository references:
//!
//! ```rust,ignore
//! // OK: Both in EmployeeRepo
//! RelationalLink<EmployeeRepo, Employee, Inventory, Item>
//!
//! // Compile error: Different repositories
//! RelationalLink<EmployeeRepo, Employee, ReportsRepo, Report>
//! ```
//!
//! # Common Patterns
//!
//! ## Creating Links
//!
//! ```rust,ignore
//! // Dehydrated (for storage)
//! let link = RelationalLink::new_dehydrated(user_id);
//!
//! // Owned (when you have the model)
//! let link = RelationalLink::new_owned(user_id, Box::new(user));
//!
//! // Hydrated (with app-controlled reference)
//! let link = RelationalLink::new_hydrated(user_id, &user);
//! ```
//!
//! ## Hydration (Loading Related Data)
//!
//! ```rust,ignore
//! // Manual hydration
//! let dehydrated: RelationalLink<R, _, _, User> = ...;
//! if let Some(key) = dehydrated.key() {
//!     let user = txn.read(key)?;
//!     let hydrated = dehydrated.clone().with_model(user.as_ref());
//! }
//!
//! // Automatic hydration via transaction
//! let hydrated = txn.hydrate_link(link)?;
//! ```
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

use crate::traits::registery::{
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
    <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug;

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
/// ```rust,ignore
/// // Within EmployeeRepo context
/// let link: RelationalLink<EmployeeRepo, Employee, Inventory, Item> =
///     RelationalLink::new_dehydrated(item_id);
///
/// // This would fail to compile - different repos:
/// // let bad: RelationalLink<EmployeeRepo, Employee, Reports, Report> = ...
/// ```
#[derive(Debug, Clone)]
pub enum RelationalLink<'data, R, SourceD, TargetD, M>
where
    R: NetabaseRepository,
    SourceD: NetabaseDefinition + InRepository<R> + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + InRepository<R> + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registery::models::model::NetabaseModel<TargetD>,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Relational as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Blob as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription as strum::IntoDiscriminant>::Discriminant: 'static,
{
    /// Dehydrated: Contains only the primary key
    Dehydrated {
        primary_key: <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary,
        _source: SourceD::DebugName,
        _repo: std::marker::PhantomData<R>,
    },
    /// Owned: Fully owns the related model (no lifetime dependency)
    /// Used when the model is constructed independently and needs to be stored with full ownership
    Owned {
        primary_key: <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary,
        model: Box<M>,
        _source: SourceD::DebugName,
        _repo: std::marker::PhantomData<R>,
    },
    /// Hydrated: Contains a borrowed reference to the model (application-controlled lifetime)
    Hydrated {
        primary_key: <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary,
        model: &'data M,
        _source: SourceD::DebugName,
        _repo: std::marker::PhantomData<R>,
    },
    /// Borrowed: Contains both the primary key and a borrowed reference from AccessGuard
    /// Lifetime is tied to database transaction -> table -> AccessGuard chain
    Borrowed {
        primary_key: <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary,
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
    M: crate::traits::registery::models::model::NetabaseModel<TargetD>,
    <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary: PartialEq,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Relational as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Blob as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription as strum::IntoDiscriminant>::Discriminant: 'static,
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
impl<'data, R, SourceD, TargetD, M> Eq for RelationalLink<'data, R, SourceD, TargetD, M>
where
    R: NetabaseRepository,
    SourceD: NetabaseDefinition + InRepository<R> + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + InRepository<R> + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registery::models::model::NetabaseModel<TargetD>,
    <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary: Eq,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Relational as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Blob as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription as strum::IntoDiscriminant>::Discriminant: 'static,
{}

// Hash implementation
impl<'data, R, SourceD, TargetD, M> std::hash::Hash for RelationalLink<'data, R, SourceD, TargetD, M>
where
    R: NetabaseRepository,
    SourceD: NetabaseDefinition + InRepository<R> + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + InRepository<R> + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registery::models::model::NetabaseModel<TargetD> + std::hash::Hash,
    <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary: std::hash::Hash,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Relational as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Blob as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription as strum::IntoDiscriminant>::Discriminant: 'static,
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
impl<'data, R, SourceD, TargetD, M> PartialOrd for RelationalLink<'data, R, SourceD, TargetD, M>
where
    R: NetabaseRepository,
    SourceD: NetabaseDefinition + InRepository<R> + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + InRepository<R> + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registery::models::model::NetabaseModel<TargetD> + PartialOrd,
    <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary: PartialOrd,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Relational as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Blob as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription as strum::IntoDiscriminant>::Discriminant: 'static,
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
impl<'data, R, SourceD, TargetD, M> Ord for RelationalLink<'data, R, SourceD, TargetD, M>
where
    R: NetabaseRepository,
    SourceD: NetabaseDefinition + InRepository<R> + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + InRepository<R> + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registery::models::model::NetabaseModel<TargetD> + Ord,
    <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary: Ord,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Relational as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Blob as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription as strum::IntoDiscriminant>::Discriminant: 'static,
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

// Implementation for RelationalLink
impl<'data, R, SourceD, TargetD, M> RelationalLink<'data, R, SourceD, TargetD, M>
where
    R: NetabaseRepository,
    SourceD: NetabaseDefinition + InRepository<R> + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + InRepository<R> + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registery::models::model::NetabaseModel<TargetD>,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Relational as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Blob as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription as strum::IntoDiscriminant>::Discriminant: 'static,
{
    /// Create a new dehydrated relational link with just the primary key
    #[inline]
    pub fn new_dehydrated(
        primary_key: <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary,
    ) -> Self {
        Self::Dehydrated {
            primary_key,
            _source: SourceD::debug_name(),
            _repo: std::marker::PhantomData,
        }
    }

    /// Create a new hydrated relational link with the model and primary key
    #[inline]
    pub fn new_hydrated(
        primary_key: <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary,
        model: &'data M,
    ) -> Self {
        Self::Hydrated {
            primary_key,
            model,
            _source: SourceD::debug_name(),
            _repo: std::marker::PhantomData,
        }
    }

    /// Create a new owned relational link with a Box-owned model
    /// This variant owns the model completely and has no lifetime dependencies
    #[inline]
    pub fn new_owned(
        primary_key: <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary,
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
        primary_key: <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary,
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
    pub fn get_primary_key(&self) -> &<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary {
        match self {
            Self::Dehydrated { primary_key, .. } => primary_key,
            Self::Owned { primary_key, .. } => primary_key,
            Self::Hydrated { primary_key, .. } => primary_key,
            Self::Borrowed { primary_key, .. } => primary_key,
        }
    }

    /// Check if this relation is currently hydrated (contains model data)
    /// Returns true for Owned, Hydrated, and Borrowed variants
    #[inline]
    pub fn is_hydrated(&self) -> bool {
        matches!(self, Self::Owned { .. } | Self::Hydrated { .. } | Self::Borrowed { .. })
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
            Self::Hydrated { model, .. } => Some(model),
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
            Self::Hydrated { primary_key, .. } => primary_key,
            Self::Borrowed { primary_key, .. } => primary_key,
        };
        Self::Dehydrated {
            primary_key,
            _source: SourceD::debug_name(),
            _repo: std::marker::PhantomData,
        }
    }

    /// Convert a dehydrated relation to hydrated by providing the model data
    #[inline]
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
            Self::Hydrated { primary_key, .. } => primary_key,
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

    // TODO: Add hydrate_in_repository() method after transaction implementation
    // This method will load the model from the database within repository context
    //
    // Signature: pub fn hydrate_in_repository<Trans>(self, transaction: &Trans) -> Result<Self, RelationalLinkError>
    // where Trans: NBRepositoryTransaction<'db, R>
}

impl<'data, R, SourceD, TargetD, M> Serialize for RelationalLink<'data, R, SourceD, TargetD, M>
where
    R: NetabaseRepository,
    SourceD: NetabaseDefinition + InRepository<R> + 'static,
    SourceD::Discriminant: std::fmt::Debug,
    TargetD: NetabaseDefinition + InRepository<R> + 'static,
    TargetD::Discriminant: std::fmt::Debug,
    M: crate::traits::registery::models::model::NetabaseModel<TargetD> + Serialize,
    <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary: Serialize,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Relational as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Blob as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription as strum::IntoDiscriminant>::Discriminant: 'static,
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
            | Self::Hydrated { primary_key, .. }
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
    M: crate::traits::registery::models::model::NetabaseModel<TargetD>,
    <M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Primary: Deserialize<'de>,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Secondary as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Relational as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Blob as strum::IntoDiscriminant>::Discriminant: 'static,
    <<M::Keys as crate::traits::registery::models::keys::NetabaseModelKeys<TargetD, M>>::Subscription as strum::IntoDiscriminant>::Discriminant: 'static,
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
