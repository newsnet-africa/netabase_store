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
//! Primary keys are generated automatically from the `#[primary_key]` attribute.
//! Here's how to work with them:
//!
//! ```rust
//! use netabase_store::doc_example::*;
//!
//! // UserID is generated from User struct's #[primary_key] field
//! let user_id = UserID("alice".into());
//! let cloned_id = user_id.clone();
//! assert_eq!(user_id, cloned_id);
//! ```
//!
//! See [`doc_example`](crate::doc_example) for the model definitions.
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
/// See [`doc_example`](crate::doc_example) for pre-built examples.
pub trait NetabaseModelPrimaryKey<D: NetabaseDefinition, M: NetabaseModelMarker<D>>:
    StoreValueMarker<D> + StoreKeyMarker<D> + Clone
where
    D::Discriminant: 'static + std::fmt::Debug,
{
}
