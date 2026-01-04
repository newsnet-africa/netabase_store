//! Pre-compiled example types for documentation and testing.
//!
//! This module provides a complete set of example models that are used throughout
//! the documentation. By defining them once here, doctests can simply import them
//! rather than re-expanding the macro in every example.
//!
//! # Available Types
//!
//! ## Models
//! - [`User`] - A user with id, name, and email (secondary key)
//! - [`Product`] - A product with sku, name, category (secondary key), and price
//! - [`Author`] - An author with id and name
//! - [`Book`] - A book with isbn, title, and a relational link to Author
//!
//! ## Generated Types
//! - `UserID`, `ProductID`, `AuthorID`, `BookID` - Primary key wrappers
//! - `ExampleDef` - The definition enum containing all models
//!
//! # Usage in Doctests
//!
//! ```rust
//! use netabase_store::doc_examples::*;
//! use netabase_store::databases::redb::RedbStore;
//! use netabase_store::databases::redb::transaction::RedbModelCrud;
//!
//! // Create a pure in-memory database (no IO operations)
//! let store = RedbStore::<ExampleDef>::new_in_memory().unwrap();
//!
//! // Write data
//! let txn = store.begin_write().unwrap();
//! txn.create(&User {
//!     id: UserID("alice".into()),
//!     name: "Alice".into(),
//!     email: "alice@example.com".into(),
//! }).unwrap();
//! txn.commit().unwrap();
//!
//! // Read data
//! let txn = store.begin_read().unwrap();
//! let user: Option<User> = txn.read(&UserID("alice".into())).unwrap();
//! assert_eq!(user.unwrap().name, "Alice");
//! ```

use serde::{Deserialize, Serialize};

/// Main example definition containing User, Product, Author, and Book models.
///
/// This definition is used throughout the documentation to demonstrate
/// various features of the netabase_store library.
#[netabase_macros::netabase_definition(ExampleDef)]
pub mod example_def {
    use super::*;

    /// A user model demonstrating primary and secondary keys.
    ///
    /// # Fields
    /// - `id` - Primary key (becomes `UserID`)
    /// - `name` - Regular field
    /// - `email` - Secondary key for email lookups
    #[derive(
        netabase_macros::NetabaseModel,
        Debug,
        Clone,
        Serialize,
        Deserialize,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
    )]
    pub struct User {
        #[primary_key]
        pub id: String,
        pub name: String,
        #[secondary_key]
        pub email: String,
    }

    /// A product model demonstrating multiple field types.
    ///
    /// # Fields
    /// - `sku` - Primary key (becomes `ProductSKU`)
    /// - `name` - Product name
    /// - `category` - Secondary key for category filtering
    /// - `price` - Price in cents (u64 for Eq/Hash compatibility)
    #[derive(
        netabase_macros::NetabaseModel,
        Debug,
        Clone,
        Serialize,
        Deserialize,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
    )]
    pub struct Product {
        #[primary_key]
        pub sku: String,
        pub name: String,
        #[secondary_key]
        pub category: String,
        pub price: u64,
    }

    /// An author model for demonstrating relational links.
    #[derive(
        netabase_macros::NetabaseModel,
        Debug,
        Clone,
        Serialize,
        Deserialize,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
    )]
    pub struct Author {
        #[primary_key]
        pub id: String,
        pub name: String,
        #[secondary_key]
        pub genre: String,
    }

    /// A book model demonstrating relational links to Author.
    #[derive(
        netabase_macros::NetabaseModel,
        Debug,
        Clone,
        Serialize,
        Deserialize,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
    )]
    pub struct Book {
        #[primary_key]
        pub isbn: String,
        pub title: String,
        #[secondary_key]
        pub genre: String,
        #[link(ExampleDef, Author)]
        pub author: String,
    }
}

// Re-export everything from the example definition for easy access
pub use example_def::*;
