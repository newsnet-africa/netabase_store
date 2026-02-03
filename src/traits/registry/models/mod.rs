//! Model traits and key system for Netabase.
//!
//! This module defines the core model abstraction and its supporting types.
//! Models are the fundamental data structures stored in Netabase databases.
//!
//! # Module Structure
//!
//! - [`model`]: Core `NetabaseModel` trait and redb implementations
//! - [`keys`]: Key types (primary, secondary, relational, blob, subscription)
//! - [`treenames`]: Table name management for models
//! - [`content_addressed`]: Content-addressed (immutable) model support
//!
//! # Model Key System
//!
//! Each model has associated key types:
//!
//! - **Primary Key**: Unique identifier (required)
//! - **Secondary Keys**: Indexed lookup fields (optional)
//! - **Relational Keys**: Links to other models (optional)
//! - **Blob Keys**: Large data field identifiers (optional)
//! - **Subscription Keys**: Topic membership (optional)
//!
//! # Example
//!
//! ```rust,ignore
//! #[derive(NetabaseModel)]
//! pub struct User {
//!     #[primary_key]
//!     pub id: UserId,
//!     
//!     #[secondary_key]
//!     pub email: String,
//!     
//!     #[link(MyDef, Company)]
//!     pub company: CompanyId,
//! }
//! ```
//!
//! # Store Marker Traits
//!
//! The `StoreKeyMarker` and `StoreValueMarker` traits are used to avoid
//! cyclical dependencies in the trait system. They mark types that can
//! be used as keys or values in the storage layer.

use crate::traits::registry::definition::NetabaseDefinition;
use serde::{Deserialize, Serialize};

pub mod keys;
pub mod model;
pub mod treenames;
pub mod content_addressed;
pub mod bounds;

pub use keys::NetabaseModelKeys;
pub use model::NetabaseModel;
pub use treenames::DiscriminantTableName;
pub use bounds::{DiscriminantBounds, HasDiscriminant, ModelKeyBounds};
// NetabaseDefinitionTreeNames is in definition module, not models::treenames
pub use crate::traits::registry::definition::NetabaseDefinitionTreeNames;
pub use content_addressed::ContentAddressedModel;

/// Marker trait for types usable as storage keys.
///
/// This trait marks types that satisfy the requirements for use as keys
/// in the Netabase storage system. It requires serialization, equality,
/// hashing, and ordering.
pub trait StoreKeyMarker<D: NetabaseDefinition>:
    Serialize + for<'de> Deserialize<'de> + Eq + std::hash::Hash + PartialOrd + Ord
where
    D::Discriminant: 'static + std::fmt::Debug,
{
}

/// Marker trait for types usable as storage values.
///
/// This trait marks types that satisfy the requirements for use as values
/// in the Netabase storage system.
pub trait StoreValueMarker<D: NetabaseDefinition>:
    Serialize + for<'de> Deserialize<'de> + Eq + std::hash::Hash + PartialOrd + Ord
where
    D::Discriminant: 'static + std::fmt::Debug,
{
}

/// Associates a key type with its value type.
pub trait StoreKey<D: NetabaseDefinition, V: StoreValueMarker<D> + ?Sized>:
    StoreKeyMarker<D>
where
    D::Discriminant: 'static + std::fmt::Debug,
{
}

/// Associates a value type with its key type.
pub trait StoreValue<D: NetabaseDefinition, K: StoreKeyMarker<D>>: StoreValueMarker<D>
where
    D::Discriminant: 'static + std::fmt::Debug,
{
}
