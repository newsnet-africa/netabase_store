//! Type registry system for models, definitions, and repositories.
//!
//! This module provides the compile-time type system that enforces schema correctness
//! and manages relationships between models, definitions, and repositories.
//!
//! # Architecture
//!
//! The registry system has three layers:
//!
//! 1. **Models** ([`models`]) - Individual data structures with primary keys, secondary keys, and links
//! 2. **Definitions** ([`definition`]) - Collections of related models forming a schema unit
//! 3. **Repositories** ([`repository`]) - Access boundaries that group definitions
//!
//! # Type Safety Guarantees
//!
//! - Models must belong to exactly one definition
//! - Links between models must respect repository boundaries
//! - Primary keys are unique and strongly typed
//! - Secondary keys create indexed lookups
//! - Blob fields are automatically chunked
//!
//! # Example
//!
//! ```rust
//! use netabase_store::prelude::*;
//! use serde::{Serialize, Deserialize};
//!
//! // A definition contains related models
//! #[netabase_macros::netabase_definition(MyApp)]
//! mod my_app {
//!     use super::*;
//!
//!     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
//!     pub struct User {
//!         #[primary_key]
//!         pub id: String,
//!         #[secondary_key]
//!         pub email: String,
//!     }
//!
//!     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
//!     pub struct Post {
//!         #[primary_key]
//!         pub id: String,
//!         #[link(MyApp, User)]
//!         pub author: String,
//!     }
//! }
//! ```

pub mod definition;
pub mod models;
pub mod repository;
