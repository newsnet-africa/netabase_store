//! Redb-specific trait bounds for models.
//!
//! This module provides helper traits that document and collect common redb-specific
//! bounds. These traits serve as documentation and can be used as convenience bounds
//! when all the individual requirements are met.
//!
//! # Purpose
//!
//! The redb backend requires many bounds (`redb::Key`, `redb::Value`, discriminant
//! bounds, etc.). These helper traits collect common bound patterns for reference.
//!
//! # Available Traits
//!
//! - `RedbModelBounds`: Complete bounds for a model in the redb backend
//!
//! # Usage
//!
//! Since Rust's trait system requires all bounds to be explicitly stated at each
//! usage site, these traits are primarily for documentation. When you need full
//! redb compatibility, use `RedbNetbaseModel` directly with all its required bounds.
//!
//! ```rust,ignore
//! use netabase_store::traits::registry::models::model::redb_model::RedbNetbaseModel;
//!
//! fn process_model<'db, D, M>(model: &M)
//! where
//!     D: RedbDefinition,
//!     M: RedbNetbaseModel<'db, D> + Clone,
//!     // Plus all the key bounds required by RedbNetbaseModel...
//! {
//!     // All redb operations are available
//! }
//! ```

use crate::traits::registry::{
    definition::redb_definition::RedbDefinition,
    models::bounds::DiscriminantBounds,
};

/// Complete bounds marker for a model in the redb backend.
///
/// This trait documents all requirements for using a model with the redb backend:
/// - Implements `RedbNetbaseModel<'db, D>` (requires `NetabaseModel<D> + redb::Value + redb::Key`)
/// - All key types implement `redb::Key + 'static`
/// - All discriminants are `'static + Debug`  
/// - Model is `Clone + 'db`
///
/// # Note
///
/// Due to Rust's trait system limitations, the bounds on `RedbNetbaseModel` are
/// requirements for **implementing** the trait, not for **using** it as a bound.
/// This means code that uses `M: RedbNetbaseModel<'db, D>` must still explicitly
/// list all the key type bounds.
///
/// # Usage Pattern
///
/// For new generic code that needs redb support, follow this pattern:
///
/// ```rust,ignore
/// use netabase_store::traits::registry::definition::redb_definition::RedbDefinition;
/// use netabase_store::traits::registry::models::model::{NetabaseModel, redb_model::RedbNetbaseModel};
/// use netabase_store::traits::registry::models::keys::{NetabaseModelKeys, blob::NetabaseModelBlobKey};
///
/// fn my_function<'db, D, M>(model: &M)
/// where
///     D: RedbDefinition,
///     D::Discriminant: 'static + std::fmt::Debug,
///     M: RedbNetbaseModel<'db, D> + Clone,
///     // Key type bounds (required because Rust doesn't infer them from RedbNetbaseModel):
///     <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Primary: redb::Key + 'static,
///     <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Secondary: redb::Key + 'static,
///     <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Relational: redb::Key + 'static,
///     <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Subscription: redb::Key + 'static,
///     <<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob: redb::Key + 'static,
///     <<<M as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, M>>::Blob as NetabaseModelBlobKey<D, M>>::BlobItem: redb::Key + 'static,
///     // Discriminant bounds...
///     D::SubscriptionKeys: redb::Key + 'static,
/// {
///     // Implementation
/// }
/// ```
pub trait RedbModelBounds<'db, D>: Clone
where
    D: RedbDefinition,
    D::Discriminant: DiscriminantBounds,
    Self: 'db,
{
}

// NOTE: We intentionally do not provide a blanket implementation here.
// The bounds required by RedbNetbaseModel cannot be automatically inferred
// by the Rust compiler when using it as a trait bound. Any code that needs
// these bounds must list them explicitly.
//
// This trait serves as documentation of what bounds are needed for full
// redb compatibility.
