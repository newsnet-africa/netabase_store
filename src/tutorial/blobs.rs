//! Blob storage and `NetabaseBlobItem` tutorial.
//!
//! This module explains how blob fields work, when to use the
//! `NetabaseBlobItem` derive macro, and how data is chunked and
//! reconstructed.
//!
//! # Blob Field Basics
//!
//! Blob fields are declared with `#[blob]` on a model field whose
//! type implements [`netabase_store::schema::blob::NetabaseBlobItem`].
//! For most cases you should define a dedicated blob payload type and
//! derive `NetabaseBlobItem` on it.
//!
//! ```rust
//! use netabase_store::{NetabaseBlobItem, NetabaseModel, netabase_definition};
//! use netabase_store::prelude::*;
//! use serde::{Serialize, Deserialize};
//!
//! // 1. Define the blob payload
//! #[derive(NetabaseBlobItem, Serialize, Deserialize, Clone)]
//! pub struct ImageBlob {
//!     pub bytes: Vec<u8>,
//!     pub content_type: String,
//! }
//!
//! // 2. Use it in a model as a #[blob] field
//! #[netabase_definition(MediaApp)]
//! mod media {
//!     use super::*;
//!
//!     #[derive(NetabaseModel, Debug, Clone, Serialize, Deserialize,
//!              PartialEq, Eq, Hash, PartialOrd, Ord)]
//!     pub struct Image {
//!         #[primary_key]
//!         pub id: String,
//!
//!         pub title: String,
//!
//!         /// Large binary data stored as chunks in a separate blob table.
//!         #[blob]
//!         pub data: ImageBlob,
//!     }
//! }
//! ```
//!
//! Under the hood, the `NetabaseBlobItem` derive generates a `{Type}Blobs`
//! wrapper that implements chunking into ~60KB segments and knows how to
//! reconstruct the original value.
//!
//! See also: `tests/macro_attributes.rs` for an executable end-to-end
//! blob round-trip using `#[blob]` and `NetabaseBlobItem`.
