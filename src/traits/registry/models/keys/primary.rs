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
//! Primary keys are typically newtypes around string or numeric types.
//! The `#[primary_key]` attribute on a model field generates the ID type:
//!
//! ```rust,ignore
//! // In your model definition:
//! #[derive(NetabaseModel)]
//! pub struct User {
//!     #[primary_key]  // Generates UserID(String) newtype
//!     pub id: String,
//!     // ... other fields
//! }
//! ```
//!
//! See [`doc_examples`](crate::doc_examples) for working examples.
//!
//! # Rules
//!
//! 1. Primary keys must be stable - changing a primary key creates a new entity
//! 2. Primary keys should be unique across all instances
//! 3. Prefer opaque types (UUIDs, ULIDs) over business data for primary keys
//! 4. Keep primary keys small for index efficiency

use crate::traits::registry::definition::NetabaseDefinition;
use crate::traits::registry::models::model::NetabaseModelMarker;
use crate::traits::registry::models::{StoreKeyMarker, StoreValueMarker};

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
/// You don't implement this trait manually. The macro generates the implementation
/// when you use `#[primary_key]` on a field.
///
/// See [`doc_examples`](crate::doc_examples) for pre-built examples.
pub trait NetabaseModelPrimaryKey<D: NetabaseDefinition, M: NetabaseModelMarker<D>>:
    StoreValueMarker<D> + StoreKeyMarker<D> + Clone
where
    D::Discriminant: 'static + std::fmt::Debug,
{
}
