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
//! use netabase_store::prelude::*;
//! use netabase_store::databases::redb::RedbStore;
//! use netabase_store::traits::database::store::NBStore;
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
//!     struct Notification {
//!         #[primary_key]
//!         id: String,
//!         message: String,
//!     }
//! }
//! ```
//!
//! The `immutable` flag indicates that subscriber records are append-only
//! (suitable for event streams and audit logs).
//!
//! Under the hood the macros generate:
//! - A `SubscriptionKeys` enum on the definition bundling all topics
//! - Auxiliary subscription key types and tables used by backends
//! - Schema entries so `export_toml()` can describe which models
//!   participate in which topics.
//!
//! ## Table Layout
//!
//! Using the `BlogWithSubs` snippet above as a guide, a backend typically has:
//!
//! | Logical Table                 | Purpose                                   | Key columns                               | Value columns        |
//! |------------------------------|-------------------------------------------|-------------------------------------------|----------------------|
//! | `Post`                       | Main posts table                          | `primary_key` (`PostID`)                  | all post fields      |
//! | `Notification`               | Subscriber rows                           | `primary_key` (`NotificationID`)          | all notification     |
//! | `BlogWithSubsSubscriptions`  | Topic → subscriber index (per definition) | `topic_discriminant`, `subscriber_pk`     | backlink / metadata  |
//!
//! - The `SubscriptionKeys` enum is the key type for `BlogWithSubsSubscriptions`.
//! - `topic_discriminant` is a compact representation of `NewPostTopic`, etc.
//! - `subscriber_pk` is the primary key of the subscribing model (`NotificationID`).
//!
//! ## Schema Export
//!
//! In `export_toml()`, subscriptions appear in a separate section, for example:
//!
//! ```toml
//! [[subscriptions]]
//! topic = "NewPostTopic"
//! model = "Notification"
//! immutable = true
//! ```
//!
//! This information is mirrored into repository-level `repository.toml`, so tools
//! can answer questions like "which definitions publish to this topic?".
//!
//! See `crate::tutorial::patterns` and `tests/macro_attributes.rs` for
//! integrated examples that create stores and exercise the subscription tables.
