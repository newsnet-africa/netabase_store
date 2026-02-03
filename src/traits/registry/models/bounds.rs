//! Trait bound helpers to reduce verbosity and separate concerns.
//!
//! This module provides helper traits that collect common bounds used across
//! the trait hierarchy. This enables:
//!
//! 1. **Backend Agnosticism**: Core traits don't require backend-specific bounds
//! 2. **Reduced Verbosity**: Single trait replaces 10+ where clause lines
//! 3. **Consistency**: Bounds are defined once and reused everywhere
//!
//! # Trait Hierarchy
//!
//! ```text
//! DiscriminantBounds          - Base: 'static + Debug
//!        ↓
//! HasDiscriminant             - IntoDiscriminant with valid Discriminant
//!        ↓
//! ModelKeyBounds              - All key types have proper discriminants
//!        ↓
//! RedbKeyBounds (in redb/)    - Adds redb::Key requirements
//! ```
//!
//! # Usage
//!
//! Instead of writing:
//! ```rust,ignore
//! where
//!     <<M::Keys as NetabaseModelKeys<D, M>>::Secondary as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
//!     <<M::Keys as NetabaseModelKeys<D, M>>::Relational as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
//!     // ... 8 more lines
//! ```
//!
//! Write:
//! ```rust,ignore
//! where
//!     M::Keys: ModelKeyBounds<D, M>,
//! ```

use strum::IntoDiscriminant;

/// Base bounds required for all discriminant types.
///
/// Discriminants must be:
/// - `'static`: No borrowed data
/// - `Debug`: Printable for error messages
///
/// This trait is automatically implemented for all qualifying types.
pub trait DiscriminantBounds: 'static + std::fmt::Debug {}

impl<T: 'static + std::fmt::Debug> DiscriminantBounds for T {}

/// Extension trait for types that implement `IntoDiscriminant` with valid bounds.
///
/// This combines `IntoDiscriminant` with the requirement that the discriminant
/// type itself satisfies `DiscriminantBounds`.
pub trait HasDiscriminant: IntoDiscriminant
where
    Self::Discriminant: DiscriminantBounds,
{
}

impl<T> HasDiscriminant for T
where
    T: IntoDiscriminant,
    T::Discriminant: DiscriminantBounds,
{
}

/// Marker trait indicating a model's keys have all required discriminant bounds.
///
/// This trait is automatically satisfied when all key types implement
/// `IntoDiscriminant` with discriminants that are `'static + Debug`.
///
/// # Usage
///
/// Use this in where clauses to require all key discriminant bounds at once:
///
/// ```rust,ignore
/// fn process_model<D, M>(model: &M)
/// where
///     D: NetabaseDefinition,
///     M: NetabaseModel<D>,
///     M::Keys: ModelKeyBounds<D, M>,
/// {
///     // All key discriminants are guaranteed valid
/// }
/// ```
pub trait ModelKeyBounds<D, M>: Sized
where
    D: crate::traits::registry::definition::NetabaseDefinition,
    D::Discriminant: DiscriminantBounds,
    M: crate::traits::registry::models::model::NetabaseModelMarker<D>,
{
}

// Blanket implementation for any Keys type that satisfies all bounds
impl<D, M, K> ModelKeyBounds<D, M> for K
where
    D: crate::traits::registry::definition::NetabaseDefinition,
    D::Discriminant: DiscriminantBounds,
    M: crate::traits::registry::models::model::NetabaseModelMarker<D>,
    K: crate::traits::registry::models::keys::NetabaseModelKeys<D, M>,
    <K as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Secondary:
        strum::IntoDiscriminant,
    <K as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Relational:
        strum::IntoDiscriminant,
    <K as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Subscription:
        strum::IntoDiscriminant + 'static,
    <K as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Blob:
        strum::IntoDiscriminant,
    <K as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Libp2p:
        strum::IntoDiscriminant,
    // All discriminants must be 'static + Debug
    <<K as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Secondary as strum::IntoDiscriminant>::Discriminant:
        DiscriminantBounds,
    <<K as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Relational as strum::IntoDiscriminant>::Discriminant:
        DiscriminantBounds,
    <<K as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Subscription as strum::IntoDiscriminant>::Discriminant:
        DiscriminantBounds,
    <<K as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Blob as strum::IntoDiscriminant>::Discriminant:
        DiscriminantBounds,
    <<K as crate::traits::registry::models::keys::NetabaseModelKeys<D, M>>::Libp2p as strum::IntoDiscriminant>::Discriminant:
        DiscriminantBounds,
{
}
