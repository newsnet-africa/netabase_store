//! Basic CRUD tutorial.
//!
//! This module provides a focused, runnable example for creating,
//! reading, updating and deleting records with Netabase.
//!
//! The example mirrors the introductory tutorial in `crate::tutorial`
//! but is isolated here for newcomers who only care about persistence
//! basics.
//!
//! # Example
//!
//! ```rust,no_run
//! use netabase_store::prelude::*;
//! use netabase_store::databases::redb::transaction::RedbModelCrud;
//! use serde::{Serialize, Deserialize};
//!
//! #[netabase_macros::netabase_definition(SimpleBlog)]
//! mod blog {
//!     use super::*;
//!
//!     #[derive(
//!         netabase_macros::NetabaseModel,
//!         Debug, Clone, Serialize, Deserialize,
//!         PartialEq, Eq, Hash, PartialOrd, Ord
//!     )]
//!     pub struct Post {
//!         /// The primary key - must be unique
//!         #[primary_key]
//!         pub id: String,
//!
//!         /// Regular fields
//!         pub title: String,
//!         pub content: String,
//!         pub published: bool,
//!     }
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use blog::*;
//! use netabase_store::traits::database::store::NBStore;
//!
//! // Create a temporary database (deleted when _temp is dropped)
//! let (store, _temp) = RedbStore::<SimpleBlog>::new_temporary()?;
//!
//! // CREATE: Insert a new post
//! let write_txn = store.begin_write()?;
//! write_txn.create(&Post {
//!     id: PostID("post-1".into()),
//!     title: "Hello Netabase".into(),
//!     content: "This is my first post".into(),
//!     published: true,
//! })?;
//! write_txn.commit()?;
//!
//! // READ: Retrieve the post
//! let read_txn = store.begin_read()?;
//! let post: Option<Post> = read_txn.read(&PostID("post-1".into()))?;
//! assert_eq!(post.unwrap().title, "Hello Netabase");
//!
//! // UPDATE: Modify the post
//! let write_txn = store.begin_write()?;
//! let mut post = write_txn.read(&PostID("post-1".into()))?.unwrap();
//! post.title = "Updated Title".into();
//! write_txn.update(&post)?;
//! write_txn.commit()?;
//!
//! // DELETE: Remove the post
//! let write_txn = store.begin_write()?;
//! write_txn.delete::<Post>(&PostID("post-1".into()))?;
//! write_txn.commit()?;
//! # Ok(())
//! # }
//! ```
