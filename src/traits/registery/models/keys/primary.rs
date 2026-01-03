//! Primary key trait for model identification.
//!
//! Every model has exactly one primary key that uniquely identifies instances.
//!
//! # Design
//!
//! Primary keys must:
//! - Implement `Clone` for duplication
//! - Implement `StoreKeyMarker` and `StoreValueMarker` for database storage
//! - Be unique within the model's table
//!
//! # Examples
//!
//! Primary keys are typically newtypes around string or numeric types:
//!
//! ```rust,ignore
//! #[derive(Clone, Debug, PartialEq, Eq, Hash)]
//! pub struct UserID(pub String);
//!
//! #[derive(NetabaseModel)]
//! pub struct User {
//!     #[primary]
//!     pub id: UserID,
//!     // ... other fields
//! }
//! ```
//!
//! # Rules
//!
//! 1. Primary keys must be stable - changing a primary key creates a new entity
//! 2. Primary keys should be unique across all instances
//! 3. Prefer opaque types (UUIDs, ULIDs) over business data for primary keys
//! 4. Keep primary keys small for index efficiency

use crate::traits::registery::definition::NetabaseDefinition;
use crate::traits::registery::models::model::NetabaseModelMarker;
use crate::traits::registery::models::{StoreKeyMarker, StoreValueMarker};

/// Marker trait for primary key types.
///
/// Implemented automatically by the `#[derive(NetabaseModel)]` macro for types
/// marked with `#[primary]` attribute.
///
/// This is a simple marker trait without the K parameter to avoid
/// early/late-bound lifetime issues with GATs.
///
/// # Automatic Implementation
///
/// You don't implement this trait manually. The macro generates the implementation:
///
/// ```rust,ignore
/// impl NetabaseModelPrimaryKey<MyDefinition, User> for UserID {}
/// ```
pub trait NetabaseModelPrimaryKey<D: NetabaseDefinition, M: NetabaseModelMarker<D>>:
    StoreValueMarker<D> + StoreKeyMarker<D> + Clone
where
    D::Discriminant: 'static + std::fmt::Debug,
{
}
