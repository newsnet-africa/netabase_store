//! Subscriptions tutorial.
//!
//! This module dives into the `#[subscribe(...)]` attribute and how
//! subscription topics interact with definitions and models.
//!
//! # Topics and Definition-Level Subscriptions
//!
//! Topics are plain marker types used to tag subscription relationships.
//! Definitions declare which topics they participate in via the
//! `subscriptions(Topic1, Topic2, ..)` argument to `#[netabase_definition]`.
//!
//! ```rust,ignore
//! use netabase_store::{NetabaseModel, netabase_definition};
//! use serde::{Serialize, Deserialize};
//!
//! // Topic markers
//! pub struct NewPostTopic;
//! pub struct NewCommentTopic;
//!
//! #[netabase_definition(BlogWithSubs, subscriptions(NewPostTopic, NewCommentTopic))]
//! mod blog {
//!     use super::*;
//!
//!     #[derive(NetabaseModel, Debug, Clone, Serialize, Deserialize,
//!              PartialEq, Eq, Hash, PartialOrd, Ord)]
//!     pub struct Post {
//!         #[primary_key]
//!         pub id: String,
//!         pub title: String,
//!     }
//!
//!     /// Immutable subscriber model that listens to new posts.
//!     #[derive(NetabaseModel, Debug, Clone, Serialize, Deserialize,
//!              PartialEq, Eq, Hash, PartialOrd, Ord)]
//!     #[subscribe(immutable, NewPostTopic)]
//!     pub struct Notification {
//!         #[primary_key]
//!         pub id: String,
//!         pub message: String,
//!     }
//! }
//! ```
//!
//! The `immutable` flag indicates that subscriber records are append-only
//! (suitable for event streams and audit logs). See `crate::tutorial::patterns`
//! and `tests/macro_attributes.rs` for integrated examples that create
//! stores and exercise the subscription tables.
