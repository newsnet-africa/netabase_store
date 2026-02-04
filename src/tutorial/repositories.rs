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
//! ```rust
//! use netabase_store::{NetabaseModel, netabase_definition, netabase_repository};
//! use serde::{Serialize, Deserialize};
//!
//! #[netabase_definition(UserDef, repos(MainRepo))]
//! mod users {
//!     use super::*;
//!
//!     #[derive(NetabaseModel, Debug, Clone, Serialize, Deserialize,
//!              PartialEq, Eq, Hash, PartialOrd, Ord)]
//!     pub struct User {
//!         #[primary_key]
//!         pub id: String,
//!         pub name: String,
//!     }
//! }
//!
//! #[netabase_definition(PostDef, repos(MainRepo))]
//! mod posts {
//!     use super::*;
//!
//!     #[derive(NetabaseModel, Debug, Clone, Serialize, Deserialize,
//!              PartialEq, Eq, Hash, PartialOrd, Ord)]
//!     pub struct Post {
//!         #[primary_key]
//!         pub id: String,
//!
//!         /// Author in another definition but same repository.
//!         #[link(UserDef, User)]
//!         pub author: String,
//!     }
//! }
//!
//! #[netabase_repository(MainRepo, definitions(UserDef, PostDef))]
//! mod repo {}
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
