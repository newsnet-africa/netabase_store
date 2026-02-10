//! Complete tutorial and examples for using Netabase.
//!
//! This module provides comprehensive, runnable examples demonstrating
//! all features of Netabase from basic CRUD to advanced patterns.
//! For feature-specific deep dives, see the submodules of `tutorial`.
//!
//! # Table of Contents
//!
//! 1. [Basic CRUD Operations](#basic-crud-operations)
//! 2. [Secondary Index Queries](#secondary-index-queries)
//! 3. [Relational Links](#relational-links)
//! 4. [Blob Storage](#blob-storage)
//! 5. [Subscriptions](#subscriptions)
//! 6. [Migrations](#migrations)
//! 7. [Transactions](#transactions)
//! 8. [Repository Isolation](#repository-isolation)
//! 9. [Common Patterns](crate::tutorial::patterns)
//!
//! For deeper, focused guides, also see the submodules:
//! - [`tutorial::basic_crud`]
//! - [`tutorial::blobs`]
//! - [`tutorial::subscriptions`]
//! - [`tutorial::repositories`]
//!
//! # Basic CRUD Operations
//!
//! ## Creating Your First Model
//!
//! ```rust
//! use netabase_store::prelude::*;
//! use netabase_store::databases::redb::RedbStore;
//! use netabase_store::traits::database::store::NBStore;
//! use netabase_store::databases::redb::RedbStore;
//! use netabase_store::traits::database::store::NBStore;
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
//! let mut post: Post = write_txn.read(&PostID("post-1".into()))?.unwrap();
//! post.title = "Updated Title".into();
//! write_txn.update(&post)?;
//! write_txn.commit()?;
//!
//! // DELETE: Remove the post
//! let write_txn = store.begin_write()?;
//! write_txn.delete::<Post, _>(&PostID("post-1".into()))?;
//! write_txn.commit()?;
//! # Ok(())
//! # }
//! ```
//!
//! # Secondary Index Queries
//!
//! Secondary indexes enable fast lookups on non-primary key fields.
//!
//! ```rust,no_run
//! use netabase_store::prelude::*;
//! use netabase_store::databases::redb::RedbStore;
//! use netabase_store::traits::database::store::NBStore;
//! use netabase_store::databases::redb::transaction::RedbModelCrud;
//! use netabase_store::relational::RelationalLink;
//! 
//! use serde::{Serialize, Deserialize};
//!
//! #[netabase_macros::netabase_definition(UserApp)]
//! mod users {
//!     use super::*;
//!
//!     #[derive(
//!         netabase_macros::NetabaseModel,
//!         Debug, Clone, Serialize, Deserialize,
//!         PartialEq, Eq, Hash, PartialOrd, Ord
//!     )]
//!     pub struct User {
//!         #[primary_key]
//!         pub id: String,
//!
//!         /// Secondary index on email for fast email lookups
//!         #[secondary_key]
//!         pub email: String,
//!
//!         /// Secondary index on username
//!         #[secondary_key]
//!         pub username: String,
//!
//!         pub name: String,
//!     }
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use users::*;
//! use netabase_store::traits::database::store::NBStore;
//!
//! let (store, _temp) = RedbStore::<UserApp>::new_temporary()?;
//!
//! // Insert users
//! let txn = store.begin_write()?;
//! txn.create(&User {
//!     id: UserID("u1".into()),
//!     email: "alice@example.com".into(),
//!     username: "alice".into(),
//!     name: "Alice".into(),
//! })?;
//! txn.create(&User {
//!     id: UserID("u2".into()),
//!     email: "bob@example.com".into(),
//!     username: "bob".into(),
//!     name: "Bob".into(),
//! })?;
//! txn.commit()?;
//!
//! // Query by secondary index
//! let txn = store.begin_read()?;
//!
//! // Lookup by email
//! let secondary_key = UserSecondaryKeys::Email("alice@example.com".into());
//! let users: Vec<User> = txn.read_by_secondary_key(&secondary_key)?;
//! assert_eq!(users.len(), 1);
//! assert_eq!(users[0].name, "Alice");
//!
//! // Lookup by username
//! let secondary_key = UserSecondaryKeys::Username("bob".into());
//! let users: Vec<User> = txn.read_by_secondary_key(&secondary_key)?;
//! assert_eq!(users[0].name, "Bob");
//! # Ok(())
//! # }
//! ```
//!
//! ## How secondary indexes are stored
//!
//! Each `#[secondary_key]` on a field causes the definition to gain an auxiliary
//! index table. For `User` the logical layout is:
//!
//! | Logical Table          | Purpose                         | Key columns                     | Value columns          |
//! |------------------------|---------------------------------|---------------------------------|------------------------|
//! | `User`                 | Main model rows                 | `primary_key` (`UserID`)        | all non-key fields     |
//! | `UserByEmail`          | Email → primary key index       | `email`                         | `UserID`               |
//! | `UserByUsername`       | Username → primary key index    | `username`                      | `UserID`               |
//!
//! The generated `UserSecondaryKeys` enum is the key type for these index tables
//! and is also represented in `export_toml()` so schema tools know which
//! secondary lookups are available.
//!
//! # Relational Links
//!
//! Link models together with type-safe foreign keys.
//!
//! ```rust,no_run
//! use netabase_store::prelude::*;
//! use netabase_store::databases::redb::RedbStore;
//! use netabase_store::traits::database::store::NBStore;
//! use netabase_store::databases::redb::transaction::RedbModelCrud;
//! use netabase_store::relational::RelationalLink;
//! 
//! use serde::{Serialize, Deserialize};
//!
//! #[netabase_macros::netabase_definition(BlogApp)]
//! mod blog {
//!     use super::*;
//!
//!     #[derive(
//!         netabase_macros::NetabaseModel,
//!         Debug, Clone, Serialize, Deserialize,
//!         PartialEq, Eq, Hash, PartialOrd, Ord
//!     )]
//!     pub struct Author {
//!         #[primary_key]
//!         pub id: String,
//!         pub name: String,
//!     }
//!
//!     #[derive(
//!         netabase_macros::NetabaseModel,
//!         Debug, Clone, Serialize, Deserialize,
//!         PartialEq, Eq, Hash, PartialOrd, Ord
//!     )]
//!     pub struct Post {
//!         #[primary_key]
//!         pub id: String,
//!
//!         /// Link to Author model
//!         #[link(BlogApp, Author)]
//!         pub author_id: String,
//!
//!         pub title: String,
//!     }
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use blog::*;
//! use netabase_store::traits::database::store::NBStore;
//!
//! let (store, _temp) = RedbStore::<BlogApp>::new_temporary()?;
//!
//! // Create an author
//! let txn = store.begin_write()?;
//! txn.create(&Author {
//!     id: AuthorID("a1".into()),
//!     name: "Alice".into(),
//! })?;
//! txn.commit()?;
//!
//! // Create posts linked to the author
//! let txn = store.begin_write()?;
//! txn.create(&Post {
//!     id: PostID("p1".into()),
//!     author_id: AuthorID("a1".into()),
//!     title: "First Post".into(),
//! })?;
//! txn.commit()?;
//!
//! // Query posts by author using relational link
//! let txn = store.begin_read()?;
//! let relational_key = PostRelationalKeys::AuthorId(
//!     RelationalLink::new(AuthorID("a1".into()))
//! );
//! let posts: Vec<Post> = txn.read_related(&relational_key)?;
//! assert_eq!(posts.len(), 1);
//! assert_eq!(posts[0].title, "First Post");
//! # Ok(())
//! # }
//! ```
//!
//! ## How relational links are stored
//!
//! For each `#[link(Definition, Model)]` field, the definition gains a
//! relational index table. In the `BlogApp` example:
//!
//! | Logical Table              | Purpose                          | Key columns                      | Value columns        |
//! |---------------------------|----------------------------------|----------------------------------|----------------------|
//! | `Author`                  | Authors                          | `primary_key` (`AuthorID`)       | author fields        |
//! | `Post`                    | Posts                            | `primary_key` (`PostID`)         | post fields          |
//! | `PostByAuthorId`          | AuthorID → post index            | `AuthorID`                       | `PostID`             |
//!
//! The `PostRelationalKeys` enum is the key type for these relational tables
//! and appears in the exported schema so query tooling can discover available
//! relationship traversals.
//!
//! # Blob Storage
//!
//! Store large binary data with automatic chunking.
//!
//! ```rust,no_run
//! use netabase_store::prelude::*;
//! use netabase_store::databases::redb::RedbStore;
//! use netabase_store::traits::database::store::NBStore;
//! use netabase_store::databases::redb::transaction::RedbModelCrud;
//! use netabase_store::relational::RelationalLink;
//! 
//! use serde::{Serialize, Deserialize};
//!
//! #[netabase_macros::netabase_definition(MediaApp)]
//! mod media {
//!     use super::*;
//!
//!     #[derive(
//!         netabase_macros::NetabaseModel,
//!         Debug, Clone, Serialize, Deserialize,
//!         PartialEq, Eq, Hash, PartialOrd, Ord
//!     )]
//!     pub struct Image {
//!         #[primary_key]
//!         pub id: String,
//!         
//!         pub title: String,
//!         
//!         /// Large binary data - automatically chunked
//!         #[blob]
//!         pub data: Vec<u8>,
//!     }
//! }
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use media::*;
//! use netabase_store::traits::database::store::NBStore;
//!
//! let (store, _temp) = RedbStore::<MediaApp>::new_temporary()?;
//!
//! // Create a large image (> 60KB will be chunked automatically)
//! let large_data = vec![0u8; 100_000]; // 100KB
//!
//! let txn = store.begin_write()?;
//! txn.create(&Image {
//!     id: ImageID("img1".into()),
//!     title: "Large Image".into(),
//!     data: large_data.clone(),
//! })?;
//! txn.commit()?;
//!
//! // Read it back - automatically reassembled from chunks
//! let txn = store.begin_read()?;
//! let image: Option<Image> = txn.read(&ImageID("img1".into()))?;
//! assert_eq!(image.unwrap().data, large_data);
//! # Ok(())
//! # }
//! ```
//!
//! # Subscriptions
//!
//! Implement pub/sub patterns with subscription topics.
//!
//! ```rust,no_run
//! use netabase_store::prelude::*;
//! use netabase_store::databases::redb::RedbStore;
//! use netabase_store::traits::database::store::NBStore;
//! use serde::{Serialize, Deserialize};
//!
//! // Define a topic
//! pub struct NewPostTopic;
//!
//! #[netabase_macros::netabase_definition(BlogWithSubs, subscriptions(NewPostTopic))]
//! mod blog {
//!     use super::*;
//!
//!     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
//!     pub struct Post {
//!         #[primary_key]
//!         pub id: String,
//!         pub title: String,
//!     }
//!
//!     // This model subscribes to new posts
//!     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
//!     #[subscribe(NewPostTopic)]
//!     pub struct Notification {
//!         #[primary_key]
//!         pub id: String,
//!         pub message: String,
//!     }
//! }
//!
//! // When a Post is created, Notifications can be automatically linked via the topic
//! ```
//! # Migrations
//!
//! Evolve your schema over time with version migrations.
//!
//! ```rust,no_run
//! use netabase_store::prelude::*;
//! use netabase_store::databases::redb::RedbStore;
//! use netabase_store::traits::database::store::NBStore;
//! use serde::{Serialize, Deserialize};
//!
//! // Version 1 of User
//! #[derive(NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
//! #[netabase_version(family = "User", version = 1)]
//! pub struct UserV1 {
//!     #[primary_key]
//!     pub id: String,
//!     pub name: String,
//! }
//!
//! // Version 2 adds email field
//! #[derive(NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
//! #[netabase_version(family = "User", version = 2)]
//! pub struct UserV2 {
//!     #[primary_key]
//!     pub id: String,
//!     pub name: String,
//!     pub email: String, // New field
//! }
//!
//! // Implement migration
//! impl MigrateFrom<UserV1> for UserV2 {
//!     fn migrate_from(old: UserV1) -> Result<Self, MigrationError> {
//!         Ok(UserV2 {
//!             id: old.id,
//!             name: old.name,
//!             email: "unknown@example.com".into(), // Default value
//!         })
//!     }
//! }
//! ```
//!
//! # Transactions
//!
//! ## Read Transactions
//!
//! Read transactions provide snapshot isolation:
//!
//! ```rust,no_run
//! use netabase_store::prelude::*;
//! use netabase_store::databases::redb::RedbStore;
//! use netabase_store::traits::database::store::NBStore;
//! use netabase_store::traits::database::store::NBStore;
//! use serde::{Serialize, Deserialize};
//!
//! # #[netabase_macros::netabase_definition(TxnApp)]
//! # mod app {
//! #     use super::*;
//! #     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
//! #     pub struct Item { #[primary_key] pub id: String }
//! # }
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # use app::*;
//! let (store, _temp) = RedbStore::<TxnApp>::new_temporary()?;
//!
//! // Read transactions are cheap and can be held open
//! let txn = store.begin_read()?;
//! let item: Option<Item> = txn.read(&ItemID("item1".into()))?;
//! // txn is automatically dropped, no explicit commit needed
//! # Ok(())
//! # }
//! ```
//!
//! ## Write Transactions
//!
//! Write transactions must be explicitly committed:
//!
//! ```rust,no_run
//! use netabase_store::prelude::*;
//! use netabase_store::databases::redb::RedbStore;
//! use netabase_store::traits::database::store::NBStore;
//! use netabase_store::traits::database::store::NBStore;
//! use serde::{Serialize, Deserialize};
//!
//! # #[netabase_macros::netabase_definition(WriteApp)]
//! # mod app {
//! #     use super::*;
//! #     #[derive(netabase_macros::NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
//! #     pub struct Item { #[primary_key] pub id: String, pub value: u32 }
//! # }
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # use app::*;
//! let (store, _temp) = RedbStore::<WriteApp>::new_temporary()?;
//!
//! // Write transaction
//! let txn = store.begin_write()?;
//! txn.create(&Item { id: ItemID("item1".into()), value: 42 })?;
//! txn.commit()?; // Must explicitly commit
//!
//! // If txn is dropped without commit, changes are rolled back
//! let txn = store.begin_write()?;
//! txn.create(&Item { id: ItemID("item2".into()), value: 99 })?;
//! drop(txn); // Rolled back!
//!
//! let txn = store.begin_read()?;
//! let item_result: Option<Item> = txn.read(&ItemID("item2".into()))?;
//! assert!(item_result.is_none());
//! # Ok(())
//! # }
//! ```
//!
//! # Repository Isolation
//!
//! Group definitions into repositories for access control. See also
//! the more detailed [`tutorial::repositories`] module.
//!
//! ```rust,no_run
//! use netabase_store::prelude::*;
//! use netabase_store::databases::redb::RedbStore;
//! use netabase_store::traits::database::store::NBStore;
//! use serde::{Serialize, Deserialize};
//!
//! #[netabase_repository(MyRepo)]
//! mod my_repo {
//!     #[netabase_definition(UserDef, repos(MyRepo))]
//!     mod users {
//!         #[derive(NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
//!         pub struct User {
//!             #[primary_key]
//!             pub id: String,
//!         }
//!     }
//!
//!     #[netabase_definition(PostDef, repos(MyRepo))]
//!     mod posts {
//!         #[derive(NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
//!         pub struct Post {
//!             #[primary_key]
//!             pub id: String,
//!             
//!             // Can link across definitions in the same repository
//!             #[link(UserDef, User)]
//!             pub author: String,
//!         }
//!     }
//! }
//! ```
//!
//! # Best Practices
//!
//! ## 1. Always Use Type Aliases
//!
//! The macros generate type aliases for primary keys:
//!
//! ```rust,no_run
//! // Generated by macro (simplified for illustration):
//! pub type UserID = String;
//!
//! // Use the alias, not the raw type:
//! let user_id = UserID("user-123".into()); // Good
//! let user_id = "user-123".to_string();     // Bad - loses type safety
//! ```
//!
//! ## 2. Commit Write Transactions
//!
//! Always explicitly commit write transactions:
//!
//! ```rust,no_run
//! // Note: Actual usage depends on your definition type
//! let txn = store.begin_write()?;
//! txn.commit()?; // Don't forget this!
//! ```
//!
//! ## 3. Use Secondary Indexes Wisely
//!
//! Only add secondary indexes on fields you'll query:
//!
//! ```rust,no_run
//! // Good - query by email often
//! #[secondary_key]
//! pub email: String,
//!
//! // Bad - rarely need to query by middle_name
//! #[secondary_key]
//! pub middle_name: Option<String>,
//! ```
//!
//! ## 4. Design for Immutability
//!
//! Where possible, design models to be append-only:
//!
//! ```rust,no_run
//! // Instead of updating:
//! pub struct Post {
//!     pub content: String,
//!     pub edit_count: u32,
//! }
//!
//! // Consider versioning:
//! pub struct PostVersion {
//!     pub post_id: String,
//!     pub version: u32,
//!     pub content: String,
//! }
//! ```
//!
//! # Stress Testing and Common Patterns
//!
//! For a complete, end-to-end example combining CRUD, secondary indexes,
//! relational links, blob storage, subscriptions, query configuration and
//! repositories, see the `patterns` submodule below. The heavy I/O parts of
//! these examples are exercised in the `tests/macro_attributes.rs` integration
//! test to keep doctests fast while still fully validating behavior.
//!
//! In practice you will typically:
//! - Define one or more `#[netabase_definition]` modules for your schemas
//! - Group them with `#[netabase_repository(Repo, definitions(...))]` when
//!   you need inter-definition links
//! - Use `#[blob]` fields with types deriving `NetabaseBlobItem` for large
//!   payloads
//! - Use `#[subscribe(immutable, Topic1, ..)]` when modeling append-only
//!   event streams
//! - Configure reads with `CrudOptions` and `QueryConfig` to keep queries
//!   predictable and efficient.
//!
//! The `patterns::overview` module contains a non-trivial example application
//! that pulls these patterns together.

/// Common patterns and combined tutorial examples.
///
/// This inline module focuses on a single, larger example that exercises
/// most of the Netabase features together. The code is documented as
/// `rust,ignore` here for readability; an executable variant of the same
/// scenario lives in `tests/macro_attributes.rs`.
mod basic_crud;
mod blobs;
mod subscriptions;
mod repositories;
mod schema_toml;

mod patterns {
    //! # Common Patterns Overview
    //!
    //! This module sketches a small "blog + media" application that uses:
    //! - Multiple `#[netabase_definition]` blocks
    //! - A `#[netabase_repository]` for inter-definition links
    //! - `#[blob]` fields backed by a `NetabaseBlobItem` type
    //! - `#[subscribe(immutable, Topic)]` for event-style models
    //! - Query configuration via `CrudOptions` and `QueryConfig`.
    //!
    //! ```rust,no_run
    //! use netabase_store::prelude::*;
    //! use netabase_store::blob::NetabaseBlobItem;
    //! use serde::{Serialize, Deserialize};
    //!
    //! // Blob payload used by the media model
    //! #[derive(NetabaseBlobItem, Serialize, Deserialize, Clone)]
    //! pub struct MediaBlob {
    //!     pub bytes: Vec<u8>,
    //!     pub content_type: String,
    //! }
    //!
    //! // Topics for subscriptions
    //! pub struct NewPost;
    //! pub struct NewMedia;
    //!
    //! #[netabase_repository(MainRepo, definitions(BlogDef, MediaDef))]
    //! mod repo {
    //!     use super::*;
    //!
    //!     #[netabase_definition(BlogDef, repos(MainRepo), subscriptions(NewPost))]
    //!     pub mod blog_def {
    //!         use super::*;
    //!
    //!         #[derive(NetabaseModel, Debug, Clone, Serialize, Deserialize,
    //!                  PartialEq, Eq, Hash, PartialOrd, Ord)]
    //!         pub struct Post {
    //!             #[primary_key]
    //!             pub id: String,
    //!             pub title: String,
    //!         }
    //!
    //!         /// Immutable notification stream for new posts.
    //!         #[derive(NetabaseModel, Debug, Clone, Serialize, Deserialize,
    //!                  PartialEq, Eq, Hash, PartialOrd, Ord)]
    //!         #[subscribe(immutable, NewPost)]
    //!         struct Notification {
    //!             #[primary_key]
    //!             id: String,
    //!             message: String,
    //!         }
    //!     }
    //!
    //!     #[netabase_definition(MediaDef, repos(MainRepo), subscriptions(NewMedia))]
    //!     pub mod media_def {
    //!         use super::*;
    //!
    //!         #[derive(NetabaseModel, Debug, Clone, Serialize, Deserialize,
    //!                  PartialEq, Eq, Hash, PartialOrd, Ord)]
    //!         pub struct Media {
    //!             #[primary_key]
    //!             pub id: String,
    //!
    //!             /// Large blob field stored in an auxiliary blob table.
    //!             #[blob]
    //!             pub data: MediaBlob,
    //!         }
    //!     }
    //! }
    //!
    //! // This example focuses on macro wiring. For a full end-to-end scenario
    //! // with a concrete backend, see `tests/macro_attributes.rs`.
    //! ```
}



