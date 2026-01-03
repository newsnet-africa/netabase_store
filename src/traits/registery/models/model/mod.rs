//! Model trait hierarchy for database entities.
//!
//! A **Model** represents a single entity type within a Definition. Models define:
//! - The structure of stored data (fields)
//! - How data is keyed and indexed (primary/secondary keys)
//! - How data relates to other models (relational links)
//! - How data is partitioned (blob storage)
//! - How data subscribes to topics (subscriptions)
//!
//! # Core Concepts
//!
//! - **NetabaseModel**: Main trait for database entities
//! - **NetabaseModelMarker**: Marker trait linking models to definitions
//! - **Keys**: The key structure (primary, secondary, relational, blob, subscription)
//! - **TreeNames**: The table names for this model's data
//!
//! # Key System
//!
//! Models define multiple key types for different access patterns:
//!
//! 1. **Primary Key**: Unique identifier for the model instance
//! 2. **Secondary Keys**: Additional indexes for efficient queries
//! 3. **Relational Keys**: Foreign key references to other models
//! 4. **Blob Keys**: Keys for large binary data storage
//! 5. **Subscription Keys**: Topic keys for pub/sub patterns
//!
//! # Blob Storage
//!
//! Large fields (> 60KB) are automatically stored separately:
//! - Fields marked with `#[blob]` are chunked and stored in blob tables
//! - Chunks are keyed by `(primary_key, field_index, chunk_index)`
//! - Automatic reassembly on read
//!
//! See [`BLOB_QUERY_METHODS.md`](../../../../../BLOB_QUERY_METHODS.md) for details.
//!
//! # Example
//!
//! ```rust,ignore
//! #[netabase_model]
//! pub struct User {
//!     #[primary_key]
//!     pub id: UserId,
//!     
//!     #[secondary_key]
//!     pub email: String,
//!     
//!     #[relational]
//!     pub company: RelationalLink<Company>,
//!     
//!     #[blob]
//!     pub avatar: Vec<u8>,
//!     
//!     #[subscription]
//!     pub topics: Vec<TopicId>,
//! }
//! ```
//!
//! # Trait Bounds
//!
//! The trait bounds ensure:
//! - Models can be converted to/from their definition enum
//! - Keys implement proper discriminant traits for efficient matching
//! - All key types have static lifetimes for database storage
//!
//! # Relational Links
//!
//! Models can link to other models in the same definition or repository:
//! - `RelationalLink<T>` for required relationships
//! - `Option<RelationalLink<T>>` for optional relationships
//! - `Vec<RelationalLink<T>>` for one-to-many relationships
//!
//! Links can be hydrated (full data) or dehydrated (just the key).

//! Model trait system for data structures stored in the database.
//!
//! A **Model** is a struct type that can be stored, queried, and indexed in
//! the database. Models are the fundamental unit of data in netabase_store.
//!
//! # What is a Model?
//!
//! A model is a Rust struct that derives `NetabaseModel` and is declared within
//! a `#[netabase_definition]` module. Each model gets:
//! - Primary key-based storage (like a primary table)
//! - Secondary indexes (for fast lookups on other fields)
//! - Relational links (typed foreign keys to other models)
//! - Blob storage (for large binary data)
//! - Subscription support (pub/sub on model changes)
//!
//! # Key Extraction
//!
//! The trait provides methods to extract all keys from an instance:
//! - `get_primary_key()`: The unique identifier
//! - `get_secondary_keys()`: All secondary index values
//! - `get_relational_keys()`: All foreign key references
//! - `get_blob_entries()`: All large binary data chunks
//! - `get_subscription_keys()`: All pub/sub topic subscriptions
//!
//! # Storage Layout
//!
//! Each model creates multiple redb tables:
//! ```text
//! User:
//!   ├── User_primary          (id -> User)
//!   ├── User_secondary_email  (email -> id)
//!   ├── User_relational_team  (id -> team_id)
//!   ├── User_blob_0_avatar    (blob_key -> chunk)
//!   └── User_subscription_*   (topic -> id)
//! ```
//!
//! # Rules and Limitations
//!
//! - Primary key must implement `StoreKey` trait
//! - Primary key must be unique across all instances
//! - Blob fields are automatically chunked (max 60KB per chunk)
//! - RelationalLinks must reference models in the same repository
//! - Subscription topics must be declared in the definition

//! Core model trait for database-backed structs.
//!
//! This module defines the `NetabaseModel` trait, which is the foundation of
//! netabase_store's type system. Models are Rust structs that get persisted
//! to the database with full relational support.
//!
//! # Architecture
//!
//! A model is a Rust struct that:
//! - Belongs to exactly one definition
//! - Has exactly one primary key field
//! - May have zero or more secondary indexes
//! - May have zero or more relational links to other models
//! - May have zero or more blob (large data) fields
//! - May subscribe to zero or more topics
//!
//! # The `NetabaseModel` Trait
//!
//! This trait is automatically implemented by the `#[netabase_model]` macro.
//! It provides:
//!
//! - **Key Extraction**: Methods to extract all keys from the model
//! - **Table Names**: Const access to all database table names
//! - **Conversion**: Bidirectional conversion to/from the definition enum
//! - **Blob Handling**: Access to large binary data fields
//! - **Relational Links**: Discovery of all relational connections
//!
//! # Example Model Definition
//!
//! ```rust,ignore
//! #[netabase_model]
//! pub struct User {
//!     #[primary_key]
//!     pub id: UserId,
//!     
//!     #[secondary_key]
//!     pub email: String,
//!     
//!     #[link(UserDef, Category)]
//!     pub category: CategoryId,
//!     
//!     #[blob]
//!     pub avatar: Vec<u8>,
//!     
//!     pub name: String,
//!     pub age: u8,
//! }
//! ```
//!
//! This generates:
//! - Primary key table: `User_primary`
//! - Secondary index: `User_secondary_email`
//! - Relational index: `User_relational_category`
//! - Blob storage: `User_blob_avatar`
//!
//! # Key Types
//!
//! Each model has a `Keys` associated type that contains:
//! - `Primary`: The primary key type
//! - `Secondary`: Enum of all secondary key types
//! - `Relational`: Enum of all relational key types
//! - `Blob`: Enum of all blob key types
//! - `Subscription`: Enum of all subscription topics
//!
//! # Trait Bounds
//!
//! Models must:
//! - Be convertible `Into<Definition>` and `TryFrom<Definition>`
//! - Implement `StoreValue` for serialization
//! - Have keys that implement `IntoDiscriminant` for efficient matching
//!
//! See [tests/comprehensive_functionality.rs] for complete usage examples.

pub mod redb_model;
pub use redb_model::*;

use strum::IntoDiscriminant;

use crate::traits::registery::{
    definition::NetabaseDefinition,
    models::{
        StoreKeyMarker, StoreValue, StoreValueMarker,
        keys::{NetabaseModelKeys, blob::NetabaseModelBlobKey},
        treenames::ModelTreeNames,
    },
};

/// Marker trait for types that can be used as models.
///
/// This is a sealed trait implemented automatically by the macro system.
/// Marker trait for types that can be used as models.
///
/// This is a supertrait of `NetabaseModel` that provides the basic
/// requirement for a type to be stored in the database. It ensures
/// the type can be serialized and associated with a definition.
///
/// # Purpose
///
/// This marker trait exists to:
/// - Separate storage concerns from full model functionality
/// - Allow future extension of model capabilities
/// - Provide a base trait for generic code that only needs storage
///
/// # Automatic Implementation
///
/// This trait is automatically implemented by the `#[netabase_model]` macro
/// and should not be manually implemented.
///
/// # Example
///
/// ```rust,ignore
/// // Generated by #[netabase_model]
/// impl<D: NetabaseDefinition> NetabaseModelMarker<D> for User
/// where D::Discriminant: 'static { }
/// ```
pub trait NetabaseModelMarker<D: NetabaseDefinition>: StoreValueMarker<D>
where
    D::Discriminant: 'static,
    <D as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
{
}

/// Core trait for types that can be stored as models in the database.
///
/// This trait is implemented by structs that derive `NetabaseModel` within
/// a `#[netabase_definition]` module. It provides all the metadata and
/// operations needed for CRUD, indexing, and relationships.
///
/// # Type Parameters
///
/// - `D`: The definition this model belongs to
///
/// # Associated Types
///
/// - `Keys`: The key types used for indexing this model
///
/// # Constants
///
/// - `TREE_NAMES`: All database table names for this model
///
/// # Example
///
/// See [tests/comprehensive_functionality.rs](../../../../tests/comprehensive_functionality.rs)
/// for complete examples.
/// Core trait for database-backed model structs.
///
/// This trait is the heart of netabase_store's type system. It provides
/// all the functionality needed to persist a Rust struct to a database
/// with full relational support, indexing, and blob storage.
///
/// # Automatic Implementation
///
/// This trait is automatically implemented by the `#[netabase_model]` macro.
/// You should never manually implement this trait.
///
/// # Associated Types
///
/// - **Keys**: Contains all key types (primary, secondary, relational, blob, subscription)
///
/// # Associated Constants
///
/// - **TREE_NAMES**: Static table name information for this model
///
/// # Key Extraction Methods
///
/// These methods extract keys from the model instance:
///
/// - `get_primary_key()`: Returns the primary key value
/// - `get_secondary_keys()`: Returns all secondary index values
/// - `get_relational_keys()`: Returns all relational link keys
/// - `get_subscription_keys()`: Returns all subscription topic keys
/// - `get_blob_entries()`: Returns all blob field data
///
/// # Relational Link Methods
///
/// - `get_relational_links()`: Returns all `RelationalLink` instances
/// - `has_relational_links()`: Check if model has any links
/// - `relational_link_count()`: Count number of links
///
/// # Usage
///
/// Models are typically used through transactions:
///
/// ```rust,ignore
/// let user = User {
///     id: UserId("alice".to_string()),
///     email: "alice@example.com".to_string(),
///     name: "Alice".to_string(),
///     age: 30,
/// };
///
/// // Extract primary key
/// let pk = user.get_primary_key();
/// assert_eq!(pk.0, "alice");
///
/// // Extract secondary keys
/// let secondary = user.get_secondary_keys();
/// // Contains email index
///
/// // Check for relational links
/// if user.has_relational_links() {
///     // Handle links
/// }
/// ```
///
/// # Trait Bounds
///
/// The trait has extensive bounds to ensure:
/// - Type-safe key extraction
/// - Efficient discriminant matching
/// - Proper serialization/deserialization
/// - Conversion to/from definition enum
///
/// See [tests/comprehensive_functionality.rs] for complete examples.
pub trait NetabaseModel<D: NetabaseDefinition>:
    NetabaseModelMarker<D>
    + Sized
    + Into<D>
    + TryFrom<D>
    + StoreValue<D, <Self::Keys as NetabaseModelKeys<D, Self>>::Primary>
where
    D::Discriminant: 'static + std::fmt::Debug,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Primary:
        StoreKeyMarker<D>,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Secondary:
        IntoDiscriminant,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Relational:
        IntoDiscriminant,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription:
        IntoDiscriminant,
    <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob:
        IntoDiscriminant,
    <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Secondary as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Relational as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
    <<<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Blob as IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
     <<Self as NetabaseModel<D>>::Keys as NetabaseModelKeys<D, Self>>::Subscription: 'static
{
    type Keys: NetabaseModelKeys<D, Self>;
    const TREE_NAMES: ModelTreeNames<'static, D, Self>;


    fn get_primary_key(&self) -> <Self::Keys as NetabaseModelKeys<D, Self>>::Primary;
    fn get_secondary_keys(
        &self,
    ) -> Vec<<Self::Keys as NetabaseModelKeys<D, Self>>::Secondary>;
    fn get_relational_keys(
        &self,
    ) -> Vec<<Self::Keys as NetabaseModelKeys<D, Self>>::Relational>;
    fn get_subscription_keys(
        &self,
    ) -> Vec<<Self::Keys as NetabaseModelKeys<D, Self>>::Subscription>;
    fn get_blob_entries(
        &self,
    ) -> Vec<Vec<(
        <Self::Keys as NetabaseModelKeys<D, Self>>::Blob,
        <<Self::Keys as NetabaseModelKeys<D, Self>>::Blob as NetabaseModelBlobKey<D, Self>>::BlobItem,
    )>>;

    /// Get all relational links from this model
    fn get_relational_links(&self) -> Vec<D::DefKeys> {
        Vec::new() // Default implementation returns empty
    }

    /// Check if this model has any relational links
    fn has_relational_links(&self) -> bool {
        !self.get_relational_links().is_empty()
    }

    /// Get the number of relational links in this model
    fn relational_link_count(&self) -> usize {
        self.get_relational_links().len()
    }
}
