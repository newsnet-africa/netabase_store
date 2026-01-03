//! Migration traits for versioned model evolution.
//!
//! This module provides the trait infrastructure for migrating models between versions.
//! The trait system enables graceful migration for P2P nodes that may be an arbitrary
//! number of versions behind the current schema.
//!
//! # Architecture
//!
//! Models are grouped by "family name" and tagged with version numbers. Each version
//! implements `MigrateFrom<PreviousVersion>` to define the upgrade path. The compiler
//! will inline and optimize chained conversions through monomorphization.
//!
//! # Example
//!
//! ```rust
//! use netabase_store::prelude::*;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq)]
//! #[netabase_version(family = "User", version = 1)]
//! pub struct UserV1 {
//!     #[primary_key]
//!     pub id: String,
//!     pub name: String,
//! }
//!
//! #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq)]
//! #[netabase_version(family = "User", version = 2)]
//! pub struct UserV2 {
//!     #[primary_key]
//!     pub id: String,
//!     pub first_name: String,
//!     pub last_name: String,
//! }
//!
//! impl MigrateFrom<UserV1> for UserV2 {
//!     fn migrate_from(old: UserV1) -> Self {
//!         let parts: Vec<&str> = old.name.split_whitespace().collect();
//!         UserV2 {
//!             id: old.id,
//!             first_name: parts.first().map(|s| s.to_string()).unwrap_or_default(),
//!             last_name: parts.get(1).map(|s| s.to_string()).unwrap_or_default(),
//!         }
//!     }
//! }
//!
//! // For P2P downgrade (optional - implement when sending to older nodes)
//! impl MigrateTo<UserV1> for UserV2 {
//!     fn migrate_to(&self) -> UserV1 {
//!         UserV1 {
//!             id: self.id.clone(),
//!             name: format!("{} {}", self.first_name, self.last_name),
//!         }
//!     }
//! }
//! ```

mod chain;
mod context;
mod traits;

pub use chain::*;
pub use context::*;
pub use traits::*;
