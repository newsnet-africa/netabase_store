//! Repository and inter-definition links tutorial.
//!
//! This module explains how to group definitions into repositories
//! using `#[netabase_repository]` and how that enables safe
//! cross-definition links.
//!
//! # Repositories as Data Graph Boundaries
//!
//! A repository is declared with `#[netabase_repository(Name, definitions(...))]`
//! and acts as the boundary for which definitions can link to each other.
//!
//! ```rust,ignore
//! use netabase_store::prelude::*;
//! use netabase_store::databases::redb::RedbStore;
//! use netabase_store::traits::database::store::NBStore;
//! use serde::{Serialize, Deserialize};
//!
//! #[netabase_repository(MainRepo)]
//! mod repo {
//!     use super::*;
//!
//!     #[netabase_definition(UserDef, repos(MainRepo))]
//!     pub mod users {
//!         use super::*;
//!
//!         #[derive(NetabaseModel, Debug, Clone, Serialize, Deserialize,
//!                  PartialEq, Eq, Hash, PartialOrd, Ord)]
//!         pub struct User {
//!             #[primary_key]
//!             pub id: String,
//!             pub name: String,
//!         }
//!     }
//!
//!     #[netabase_definition(PostDef, repos(MainRepo))]
//!     pub mod posts {
//!         use super::*;
//!         use netabase_store::relational::RelationalLink;
//!
//!         #[derive(NetabaseModel, Debug, Clone, Serialize, Deserialize,
//!                  PartialEq, Eq, Hash, PartialOrd, Ord)]
//!         pub struct Post {
//!             #[primary_key]
//!             pub id: String,
//!
//!             /// Author in another definition but same repository.
//!             #[link(UserDef, User)]
//!             pub author: String,
//!         }
//!     }
//! }
//! ```
//!
//! The macros generate a `MainRepo` marker type, definition and model
//! discriminants, and repository metadata used by `RedbRepositoryStore`.
//! They also emit repository and definition entries that show up in
//! `schema.toml` and `repository.toml`, mapping syntax like
//! `repos(MainRepo)` and `definitions(UserDef, PostDef)` to concrete
//! folders and files on disk.
//!
//! See `crate::traits::registry::repository` and `src/databases/redb/repository.rs`
//! for the underlying traits and store implementation, and
//! `tests/repository_comprehensive.rs` for a full repository stress test.
