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
//! use netabase_store::doc_example::*;
//! use netabase_store::databases::redb::RedbStore;
//! use netabase_store::databases::redb::transaction::RedbModelCrud;
//! use netabase_store::traits::database::store::NBStore;
//!
//! let store = RedbStore::<ExampleDef>::new_in_memory().unwrap();
//! let txn = store.begin_write().unwrap();
//!
//! // Create a user with blob data (email is a blob field in the example)
//! let user = User {
//!     id: UserID("alice".into()),
//!     name: "Alice".into(),
//!     email: "alice@example.com".into(),
//! };
//!
//! txn.create(&user).unwrap();
//! txn.commit().unwrap();
//!
//! // Read it back - blobs are automatically reconstructed
//! let read_txn = store.begin_read().unwrap();
//! let retrieved: Option<User> = read_txn.read(&UserID("alice".into())).unwrap();
//! assert_eq!(retrieved.unwrap().email, "alice@example.com");
//! ```
//!
//! Under the hood, the `NetabaseBlobItem` derive generates a `{Type}Blobs`
//! helper that knows how to:
//! - Split your payload into ~60KB chunks for storage
//! - Reconstruct the original value from those chunks
//! - Serialize/deserialize efficiently using postcard
//!
//! When you add `#[blob]` to a field in a `NetabaseModel`:
//! - The definition gains an extra "blob" tree in its `TreeNames` enum
//! - The backend creates a dedicated blob table keyed by blob ID + chunk index
//! - Schema export (`export_toml`) records this as a separate table so tools
//!   can reason about blob storage independently from the main row
//!
//! ## Table Layout
//!
//! For the `Image` example above, a Redb-backed definition has two logical tables:
//!
//! | Logical Table         | Purpose                          | Key columns                    | Value columns             |
//! |-----------------------|----------------------------------|--------------------------------|---------------------------|
//! | `Image`               | Main model rows                  | `primary_key` (`ImageID`)      | all non-blob fields       |
//! | `ImageBlobChunks`     | Blob chunks for `Image::data`    | `blob_id`, `chunk_index`       | `chunk_bytes`             |
//!
//! - `blob_id` is a synthetic identifier stored alongside the main row.
//! - `chunk_index` orders the chunks so they can be reassembled.
//! - The `{Type}Blobs` helper knows how to map your `ImageBlob` to/from these rows.
//!
//! ## Schema Export
//!
//! In `ExampleDef::export_toml()` and `schema.toml` you will see blob metadata
//! alongside models, for example (simplified):
//!
//! ```toml
//! [[models]]
//! name = "Image"
//! primary_key = "ImageID"
//!
//!   [[models.blobs]]
//!   field = "data"
//!   payload_type = "ImageBlob"
//!   table = "ImageBlobChunks"
//! ```
//!
//! Tools can use this section to reason about storage costs, decide how to back up
//! blob tables separately, or migrate blob payload types safely.
//!
//! See also: `tests/macro_attributes.rs` for an executable end-to-end
//! blob round-trip using `#[blob]` and `NetabaseBlobItem`.
