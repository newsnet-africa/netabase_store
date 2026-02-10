//! # Netabase Macros
//!
//! Procedural macros for defining type-safe database models, definitions, and repositories.
//!
//! This crate provides the compile-time code generation that powers `netabase_store`'s
//! type-safe, schema-verified database system. All macros work together to generate
//! efficient, zero-overhead abstractions.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use netabase_macros::{NetabaseModel, netabase_definition};
//! use serde::{Serialize, Deserialize};
//!
//! #[netabase_definition(MyApp)]
//! mod my_app {
//!     use super::*;
//!
//!     #[derive(NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
//!     pub struct User {
//!         #[primary_key]
//!         pub id: String,
//!         pub name: String,
//!         #[secondary_key]
//!         pub email: String,
//!     }
//! }
//! ```
//!
//! # Macros Overview
//!
//! ## `#[derive(NetabaseModel)]` - Model Derive Macro
//!
//! Marks a struct as a database model within a definition module. This derive is
//! primarily a marker - the actual code generation is done by `#[netabase_definition]`.
//!
//! ### Field Attributes
//!
//! | Attribute | Description | Example |
//! |-----------|-------------|---------|
//! | `#[primary_key]` | Unique identifier (exactly one required) | `#[primary_key] pub id: String` |
//! | `#[secondary_key]` | Creates an index for fast lookups | `#[secondary_key] pub email: String` |
//! | `#[link(Def, Model)]` | Foreign key reference to another model | `#[link(MyDef, Author)] pub author_id: AuthorId` |
//! | `#[blob]` | Large data stored separately with chunking | `#[blob] pub image: Vec<u8>` |
//! | `#[subscribe]` | Subscribe model to a pub/sub topic | `#[subscribe] pub topic: TopicId` |
//! | `#[netabase_version(...)]` | Version info for migrations | `#[netabase_version(family = "User", version = 2)]` |
//!
//! ### Generated Code
//!
//! For each model, the macros generate:
//!
//! ```rust,ignore
//! // Primary key wrapper type
//! pub struct UserID(pub String);
//!
//! // Key enums for indexing
//! pub enum UserSecondaryKeys { Email(String) }
//! pub enum UserRelationalKeys { /* if any links */ }
//! pub enum UserBlobKeys { /* if any blobs */ }
//!
//! // Trait implementations
//! impl NetabaseModel<MyApp> for User { ... }
//! impl redb::Value for User { ... }
//! impl redb::Key for UserID { ... }
//! ```
//!
//! ## `#[netabase_definition(Name)]` - Definition Attribute Macro
//!
//! Creates a definition module containing related models. This is the primary macro
//! that orchestrates code generation.
//!
//! ### Syntax
//!
//! ```rust,ignore
//! #[netabase_definition(
//!     DefinitionName,
//!     subscriptions(Topic1, Topic2),  // Optional: pub/sub topics
//!     repos(RepoName),                 // Optional: repository membership
//!     from_file = "schema.toml"       // Optional: import from TOML
//! )]
//! mod my_definition {
//!     // Models go here
//! }
//! ```
//!
//! ### Generated Code
//!
//! ```rust,ignore
//! // Definition enum wrapping all models
//! pub enum MyApp {
//!     User(User),
//!     Product(Product),
//! }
//!
//! // Discriminant for efficient matching
//! pub enum MyAppDiscriminant {
//!     User,
//!     Product,
//! }
//!
//! // All table names
//! pub enum MyAppTreeNames {
//!     Users,
//!     UsersSecondaryEmail,
//!     Products,
//!     // ...
//! }
//!
//! // Key union for transactions
//! pub enum MyAppKeys {
//!     User(UserID),
//!     Product(ProductID),
//! }
//!
//! // Schema export
//! impl MyApp {
//!     pub fn schema_toml() -> String { ... }
//!     pub fn schema_hash<H: HashAlgorithm>() -> u64 { ... }
//! }
//! ```
//!
//! ## `#[netabase_repository(Name)]` - Repository Attribute Macro
//!
//! Groups multiple definitions for cross-definition communication with compile-time
//! access control.
//!
//! ```rust,ignore
//! // Using external definitions (recommended)
//! #[netabase_repository(MyRepo, definitions(UserDef, ProductDef))]
//! mod my_repository {}
//!
//! // Using nested definitions
//! #[netabase_repository(MyRepo)]
//! mod my_repository {
//!     #[netabase_definition(UserDef, repos(MyRepo))]
//!     mod users { ... }
//!
//!     #[netabase_definition(ProductDef, repos(MyRepo))]
//!     mod products { ... }
//! }
//! ```
//!
//! ### Generated Code
//!
//! ```rust,ignore
//! // Repository marker
//! pub struct MyRepo;
//!
//! // Definition discriminant
//! pub enum MyRepoDefinitions {
//!     UserDef,
//!     ProductDef,
//! }
//!
//! // Model discriminant across all definitions
//! pub enum MyRepoModels {
//!     UserDef_User,
//!     UserDef_Profile,
//!     ProductDef_Product,
//! }
//!
//! // Repository store with multi-definition support
//! impl MyRepo {
//!     pub fn new<P: AsRef<Path>>(repo_path: P) -> Result<Self> { ... }
//!     pub fn store_names() -> &'static [&'static str] { ... }
//! }
//! ```
//!
//! ## `#[derive(NetabaseBlobItem)]` - Blob Item Derive Macro
//!
//! Enables automatic chunking for large data types used in `#[blob]` fields.
//!
//! ```rust,ignore
//! use netabase_macros::NetabaseBlobItem;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(NetabaseBlobItem, Serialize, Deserialize, Clone)]
//! pub struct LargeDocument {
//!     pub content: Vec<u8>,
//!     pub metadata: DocumentMetadata,
//! }
//!
//! // Now use in a model:
//! #[derive(NetabaseModel, ...)]
//! pub struct Article {
//!     #[primary_key]
//!     pub id: ArticleId,
//!     #[blob]
//!     pub document: LargeDocument,  // Automatically chunked at 60KB
//! }
//! ```
//!
//! ## `#[netabase]` - Convenience Macro
//!
//! Combines definition and model setup in a single attribute for simple cases.
//!
//! ## Additional Macros
//!
//! | Macro | Purpose |
//! |-------|---------|
//! | `#[netabase_libp2p]` | Add libp2p PeerId fields for P2P sync |
//! | `#[netabase_networking]` | Enable networking capabilities |
//! | `#[netabase_content_addressed]` | Use content-based addressing (CID) |
//! | `infer_netabase_definition!` | Import definition from TOML file |
//! | `generate_cli!` | Generate CLI commands for a definition |
//!
//! # Model Versioning
//!
//! For schema migrations, use version families:
//!
//! ```rust,ignore
//! #[netabase_definition(CRM)]
//! mod crm {
//!     // Version 1 (old)
//!     #[derive(NetabaseModel, ...)]
//!     #[netabase_version(family = "Customer", version = 1)]
//!     pub struct CustomerV1 {
//!         #[primary_key]
//!         pub id: String,
//!         pub name: String,
//!     }
//!
//!     // Version 2 (current)
//!     #[derive(NetabaseModel, ...)]
//!     #[netabase_version(family = "Customer", version = 2)]
//!     pub struct Customer {
//!         #[primary_key]
//!         pub id: String,
//!         pub name: String,
//!         pub email: String,  // New field!
//!     }
//!
//!     // Migration logic
//!     impl netabase_store::traits::migration::MigrateFrom<CustomerV1> for Customer {
//!         fn migrate_from(old: CustomerV1) -> Self {
//!             Customer {
//!                 id: old.id,
//!                 name: old.name,
//!                 email: String::new(),
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! # TOML Schema Import/Export
//!
//! Definitions can be exported to and imported from TOML:
//!
//! ```toml
//! # schema.toml
//! name = "UserDef"
//! version = 1
//! hash = 12345678
//!
//! [[models]]
//! name = "User"
//!
//! [[models.fields]]
//! name = "id"
//! type_name = "String"
//! kind = "primary_key"
//!
//! [[models.fields]]
//! name = "email"
//! type_name = "String"
//! kind = "secondary_key"
//! ```
//!
//! # Compile-Time Verification
//!
//! The macros enforce:
//! - ✅ Exactly one `#[primary_key]` per model
//! - ✅ Valid field types for keys
//! - ✅ Repository isolation for relational links
//! - ✅ Proper attribute usage
//! - ✅ No duplicate model names within a definition
//! - ✅ Valid version numbers and family names
//!
//! # Error Messages
//!
//! The macros provide helpful compile-time error messages:
//!
//! ```text
//! error: Model must have exactly one #[primary_key] field
//!   --> src/models.rs:5:1
//!    |
//! 5  | pub struct User {
//!    | ^^^^^^^^^^^^^^^^
//!
//! error: Cannot link to model outside repository boundary
//!   --> src/models.rs:12:5
//!    |
//! 12 |     #[link(OtherDef, OtherModel)]
//!    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//!    = help: Add OtherDef to the same repository, or use a non-relational reference
//! ```
//!
//! # Internal Architecture
//!
//! The macro crate is organized into:
//!
//! - `generators/` - Code generation modules
//!   - `model/` - Per-model code (keys, serialization, constructors)
//!   - `definition/` - Definition-level code (enums, traits)
//!   - `repository/` - Repository code (stores, discriminants)
//! - `macros/` - Macro entry points
//! - `visitors/` - AST visitors for extracting metadata
//! - `utils/` - Helper functions (attributes, naming, schema)
//!
//! Build timestamp: 2026-02-01

// Allow dead code in macro crate - utility functions may be used in future expansions
#![allow(dead_code)]

use proc_macro::TokenStream;

// Force rebuild marker - updated 2026-01-03
const _BUILD_MARKER: &str = "v0.1.5-all-tests-fixed";

mod generators;
mod macros;
mod utils;
mod visitors;

/// Derive macro for defining database models within a Netabase definition.
///
/// This derive macro marks a struct as a database model. It must be used inside
/// a module annotated with `#[netabase_definition(...)]`. The actual code generation
/// is performed by the definition macro, but this derive enables field attribute parsing.
///
/// # Required Traits
///
/// The struct must also derive these traits:
/// - `Debug`, `Clone` - For debugging and cloning
/// - `Serialize`, `Deserialize` (from serde) - For database serialization
/// - `PartialEq`, `Eq`, `Hash` - For key comparisons
/// - `PartialOrd`, `Ord` - For range queries
///
/// # Field Attributes
///
/// | Attribute | Required | Description |
/// |-----------|----------|-------------|
/// | `#[primary_key]` | **Yes (exactly one)** | Unique identifier field |
/// | `#[secondary_key]` | No | Creates an index for fast lookups |
/// | `#[link(Definition, Model)]` | No | Foreign key to another model |
/// | `#[blob]` | No | Large data with automatic chunking |
/// | `#[subscribe]` | No | Subscribe to a pub/sub topic |
/// | `#[netabase_version(...)]` | No | Version info for migrations |
///
/// # Generated Types
///
/// For a model named `User`, the macros generate:
///
/// - `UserID` - Primary key wrapper type (newtype pattern)
/// - `UserSecondaryKeys` - Enum of all secondary key variants
/// - `UserRelationalKeys` - Enum of all relational link variants
/// - `UserBlobKeys` - Enum of all blob field variants
/// - `UserSubscriptionKeys` - Enum of subscription topics
///
/// # Example
///
/// ```rust,ignore
/// use netabase_macros::{NetabaseModel, netabase_definition};
/// use serde::{Serialize, Deserialize};
///
/// #[netabase_definition(MyApp)]
/// mod models {
///     use super::*;
///
///     #[derive(NetabaseModel, Debug, Clone, Serialize, Deserialize,
///              PartialEq, Eq, Hash, PartialOrd, Ord)]
///     pub struct User {
///         #[primary_key]
///         pub id: String,
///         
///         #[secondary_key]
///         pub email: String,
///         
///         pub name: String,
///     }
/// }
///
/// // Generated types can be used:
/// use models::{User, UserID, UserSecondaryKeys};
///
/// let user_id = UserID("user-123".into());
/// let email_key = UserSecondaryKeys::Email("test@example.com".into());
/// ```
///
/// # Compile-Time Checks
///
/// The macro enforces:
/// - Exactly one `#[primary_key]` field
/// - Valid types for key fields (for the redb backend, wrappers/enums implement `redb::Key`)
/// - Proper attribute syntax
///
/// # See Also
///
/// - [`netabase_definition`] - The definition macro that processes models
/// - [`NetabaseBlobItem`] - For custom blob types
#[proc_macro_derive(
    NetabaseModel,
    attributes(
        primary_key,
        secondary_key,
        link,
        blob,
        subscribe,
        netabase_version
    )
)]
pub fn netabase_model(input: TokenStream) -> TokenStream {
    macros::netabase_model::netabase_model_derive(input.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Attribute macro for creating a database definition module.
///
/// A definition is a collection of related database models that share a schema.
/// This macro processes all `#[derive(NetabaseModel)]` structs within the module
/// and generates the complete type-safe database interface.
///
/// # Syntax
///
/// ```rust,ignore
/// #[netabase_definition(DefinitionName)]
/// #[netabase_definition(DefinitionName, subscriptions(Topic1, Topic2))]
/// #[netabase_definition(DefinitionName, repos(RepositoryName))]
/// #[netabase_definition(DefinitionName, from_file = "path/to/schema.toml")]
/// ```
///
/// # Arguments
///
/// | Argument | Required | Description |
/// |----------|----------|-------------|
/// | `DefinitionName` | **Yes** | The name of the definition type |
/// | `subscriptions(...)` | No | Pub/sub topics available to models |
/// | `repos(...)` | No | Repository membership for access control |
/// | `from_file = "..."` | No | Import schema from TOML file |
///
/// # Generated Types
///
/// For a definition named `MyApp` with models `User` and `Post`:
///
/// ```rust,ignore
/// // Definition enum wrapping all models
/// pub enum MyApp {
///     User(User),
///     Post(Post),
/// }
///
/// // Discriminant for efficient matching
/// pub enum MyAppDiscriminant {
///     User,
///     Post,
/// }
///
/// // All table names in the database
/// pub enum MyAppTreeNames {
///     Users,
///     UsersSecondaryEmail,
///     Posts,
///     // ... secondary/relational/blob tables
/// }
///
/// // Union of all primary keys
/// pub enum MyAppKeys {
///     User(UserID),
///     Post(PostID),
/// }
///
/// // Subscription topics (if defined)
/// pub enum MyAppSubscriptions {
///     Topic1,
///     Topic2,
/// }
/// ```
///
/// # Generated Methods
///
/// ```rust,ignore
/// impl MyApp {
///     /// Export the schema to TOML format
///     pub fn schema_toml() -> String { ... }
///     
///     /// Get a hash of the schema for version comparison
///     pub fn schema_hash<H: HashAlgorithm>() -> u64 { ... }
///     
///     /// Get all table names
///     pub fn table_names() -> &'static [&'static str] { ... }
/// }
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use netabase_macros::{NetabaseModel, netabase_definition};
/// use serde::{Serialize, Deserialize};
///
/// #[netabase_definition(BlogApp, subscriptions(NewPost, PostUpdated))]
/// mod blog {
///     use super::*;
///
///     #[derive(NetabaseModel, Debug, Clone, Serialize, Deserialize,
///              PartialEq, Eq, Hash, PartialOrd, Ord)]
///     pub struct Author {
///         #[primary_key]
///         pub id: String,
///         pub name: String,
///     }
///
///     #[derive(NetabaseModel, Debug, Clone, Serialize, Deserialize,
///              PartialEq, Eq, Hash, PartialOrd, Ord)]
///     #[subscribe(NewPost)]  // Subscribe to topic
///     pub struct Post {
///         #[primary_key]
///         pub id: String,
///         
///         #[link(BlogApp, Author)]
///         pub author_id: String,
///         
///         #[secondary_key]
///         pub slug: String,
///         
///         pub title: String,
///         pub content: String,
///     }
/// }
///
/// // Use the generated types
/// use blog::*;
///
/// let store = RedbStore::<BlogApp>::new("./data")?;
/// ```
///
/// # Schema Export
///
/// Definitions can export their schema to TOML for versioning:
///
/// ```rust,ignore
/// let schema = BlogApp::schema_toml();
/// std::fs::write("schema.toml", schema)?;
/// ```
///
/// # See Also
///
/// - [`NetabaseModel`] - Model derive macro
/// - [`netabase_repository`] - Repository for grouping definitions
/// - [`infer_netabase_definition!`] - Import from TOML
#[proc_macro_attribute]
pub fn netabase_definition(attr: TokenStream, item: TokenStream) -> TokenStream {
    macros::netabase_definition::netabase_definition_attribute(attr.into(), item.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Attribute macro for creating a repository that groups multiple definitions.
///
/// A repository provides access control and cross-definition linking. Models in
/// definitions within the same repository can reference each other via
/// `#[link(...)]` attributes.
///
/// # Syntax
///
/// ```rust,ignore
/// // Repository with external definitions
/// #[netabase_repository(RepoName, definitions(Def1, Def2))]
/// mod repo {}
///
/// // Repository with nested definitions
/// #[netabase_repository(RepoName)]
/// mod repo {
///     #[netabase_definition(Def1, repos(RepoName))]
///     mod def1 { ... }
///     
///     #[netabase_definition(Def2, repos(RepoName))]
///     mod def2 { ... }
/// }
/// ```
///
/// # Arguments
///
/// | Argument | Required | Description |
/// |----------|----------|-------------|
/// | `RepoName` | **Yes** | The name of the repository struct |
/// | `definitions(...)` | No | List of external definitions to include |
///
/// # Generated Types
///
/// ```rust,ignore
/// // Repository marker struct
/// pub struct MyRepo;
///
/// // Definition discriminant
/// pub enum MyRepoDefinitions {
///     UserDef,
///     ProductDef,
/// }
///
/// // Model discriminant across all definitions
/// pub enum MyRepoModels {
///     UserDef_User,
///     UserDef_Profile,
///     ProductDef_Product,
/// }
/// ```
///
/// # Generated Methods
///
/// ```rust,ignore
/// impl MyRepo {
///     /// Create a new repository store
///     pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> { ... }
///     
///     /// Get all store file names
///     pub fn store_names() -> &'static [&'static str] { ... }
///     
///     /// Export all schemas to TOML
///     pub fn export_schemas() -> String { ... }
/// }
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use netabase_macros::{NetabaseModel, netabase_definition, netabase_repository};
/// use serde::{Serialize, Deserialize};
///
/// // Define models in separate definitions
/// #[netabase_definition(UserDef, repos(MyRepo))]
/// mod users {
///     use super::*;
///     
///     #[derive(NetabaseModel, Debug, Clone, Serialize, Deserialize,
///              PartialEq, Eq, Hash, PartialOrd, Ord)]
///     pub struct User {
///         #[primary_key]
///         pub id: String,
///         pub name: String,
///     }
/// }
///
/// #[netabase_definition(OrderDef, repos(MyRepo))]
/// mod orders {
///     use super::*;
///     
///     #[derive(NetabaseModel, Debug, Clone, Serialize, Deserialize,
///              PartialEq, Eq, Hash, PartialOrd, Ord)]
///     pub struct Order {
///         #[primary_key]
///         pub id: String,
///         
///         // Cross-definition link (allowed because same repository)
///         #[link(UserDef, User)]
///         pub user_id: String,
///         
///         pub total: u64,
///     }
/// }
///
/// // Group into repository
/// #[netabase_repository(MyRepo, definitions(UserDef, OrderDef))]
/// mod my_repo {}
///
/// // Use the repository
/// let repo = MyRepo::new("./data")?;
/// ```
///
/// # Access Control
///
/// Relations can only cross definition boundaries if both definitions
/// are in the same repository. This provides compile-time access control:
///
/// ```rust,ignore
/// // ERROR: UserDef and ExternalDef are not in the same repository
/// #[link(ExternalDef, ExternalModel)]
/// pub external_id: String,
/// ```
///
/// # See Also
///
/// - [`netabase_definition`] - Definition macro
/// - [`NetabaseModel`] - Model derive macro
#[proc_macro_attribute]
pub fn netabase_repository(attr: TokenStream, item: TokenStream) -> TokenStream {
    macros::netabase_repository::netabase_repository_attribute(attr.into(), item.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Convenience attribute macro combining definition and model setup.
///
/// This is a simplified alternative for simple use cases where you only
/// need a single model or don't need the full definition module structure.
///
/// # Example
///
/// ```rust,ignore
/// use netabase_macros::netabase;
/// use serde::{Serialize, Deserialize};
///
/// #[netabase]
/// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// pub struct SimpleModel {
///     #[primary_key]
///     pub id: String,
///     pub data: String,
/// }
/// ```
///
/// # Note
///
/// For most use cases, prefer `#[netabase_definition]` with explicit
/// `#[derive(NetabaseModel)]` for better control and clarity.
///
/// # See Also
///
/// - [`netabase_definition`] - Full-featured definition macro
/// - [`NetabaseModel`] - Model derive macro
#[proc_macro_attribute]
pub fn netabase(attr: TokenStream, item: TokenStream) -> TokenStream {
    macros::netabase::netabase_attribute(attr.into(), item.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Derive macro for custom blob item types.
///
/// Use this derive on types that will be stored in `#[blob]` fields. It enables
/// automatic chunking and reassembly of large data across multiple database entries.
///
/// # Default Chunk Size
///
/// By default, blob items are chunked at 60KB boundaries. This can be customized
/// using the `#[blob_as(...)]` attribute.
///
/// # Required Traits
///
/// The type must also derive:
/// - `Serialize`, `Deserialize` (from serde) - For serialization
/// - `Clone` - For data handling
///
/// # Example
///
/// ```rust,ignore
/// use netabase_macros::NetabaseBlobItem;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(NetabaseBlobItem, Clone, Serialize, Deserialize)]
/// pub struct LargeDocument {
///     pub content: Vec<u8>,
///     pub metadata: DocumentMetadata,
/// }
///
/// // Use in a model:
/// #[derive(NetabaseModel, Debug, Clone, Serialize, Deserialize,
///          PartialEq, Eq, Hash, PartialOrd, Ord)]
/// pub struct Article {
///     #[primary_key]
///     pub id: String,
///     
///     #[blob]
///     pub document: LargeDocument,  // Automatically chunked
/// }
/// ```
///
/// # Custom Chunk Size
///
/// ```rust,ignore
/// #[derive(NetabaseBlobItem, Clone, Serialize, Deserialize)]
/// #[blob_as(chunk_size = 128 * 1024)]  // 128KB chunks
/// pub struct VideoFrame {
///     pub data: Vec<u8>,
///     pub timestamp: u64,
/// }
/// ```
///
/// # How Chunking Works
///
/// 1. When storing: The blob is serialized, split into chunks, and each chunk
///    is stored with an index in a multimap table.
/// 2. When reading: All chunks are retrieved, sorted by index, and reassembled.
///
/// This enables efficient partial sync - peers can request only missing chunks.
///
/// # See Also
///
/// - [`NetabaseModel`] with `#[blob]` field attribute
#[proc_macro_derive(NetabaseBlobItem, attributes(blob, blob_as))]
pub fn netabase_blob_item(input: TokenStream) -> TokenStream {
    macros::netabase_blob_item::netabase_blob_item_derive(input.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Attribute macro for adding libp2p peer-to-peer networking fields.
///
/// Adds a `PeerId` field to the model for tracking which peer owns or
/// originated the data. Useful for distributed/P2P sync scenarios.
///
/// # Example
///
/// ```rust,ignore
/// use netabase_macros::{NetabaseModel, netabase_definition, netabase_libp2p};
/// use serde::{Serialize, Deserialize};
///
/// #[netabase_definition(P2PApp)]
/// mod p2p {
///     use super::*;
///     
///     #[netabase_libp2p]
///     #[derive(NetabaseModel, Debug, Clone, Serialize, Deserialize,
///              PartialEq, Eq, Hash, PartialOrd, Ord)]
///     pub struct SharedDocument {
///         #[primary_key]
///         pub id: String,
///         
///         pub content: String,
///         
///         // Added by #[netabase_libp2p]:
///         // pub peer_id: libp2p::PeerId,
///     }
/// }
/// ```
///
/// # Requirements
///
/// - The `libp2p` feature must be enabled
/// - `libp2p` crate must be available
///
/// # See Also
///
/// - [`netabase_networking`] - Full networking capability
#[proc_macro_attribute]
pub fn netabase_libp2p(attr: TokenStream, item: TokenStream) -> TokenStream {
    macros::netabase_libp2p::netabase_libp2p_attribute(attr.into(), item.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Attribute macro for enabling full networking capabilities on a model.
///
/// Adds networking-related fields and implements traits for distributed
/// synchronization, including conflict resolution metadata.
///
/// # Example
///
/// ```rust,ignore
/// use netabase_macros::{NetabaseModel, netabase_definition, netabase_networking};
/// use serde::{Serialize, Deserialize};
///
/// #[netabase_definition(SyncApp)]
/// mod sync {
///     use super::*;
///     
///     #[netabase_networking]
///     #[derive(NetabaseModel, Debug, Clone, Serialize, Deserialize,
///              PartialEq, Eq, Hash, PartialOrd, Ord)]
///     pub struct SyncedNote {
///         #[primary_key]
///         pub id: String,
///         
///         pub content: String,
///         
///         // Fields added by #[netabase_networking]:
///         // - peer_id: PeerId (origin peer)
///         // - vector_clock: VectorClock (for causality)
///         // - lamport_timestamp: u64 (logical time)
///     }
/// }
/// ```
///
/// # Requirements
///
/// - The `libp2p` feature must be enabled
///
/// # See Also
///
/// - [`netabase_libp2p`] - Just peer ID tracking
#[proc_macro_attribute]
pub fn netabase_networking(attr: TokenStream, item: TokenStream) -> TokenStream {
    macros::netabase_networking::netabase_networking_attribute(attr.into(), item.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Attribute macro for content-addressed models.
///
/// Marks a model as content-addressed, meaning its primary key is derived
/// from a hash of its content (similar to CIDs in IPFS). This makes the
/// model immutable by design - any change creates a new record with a new key.
///
/// # Example
///
/// ```rust,ignore
/// use netabase_macros::{NetabaseModel, netabase_definition, netabase_content_addressed};
/// use serde::{Serialize, Deserialize};
///
/// #[netabase_definition(ContentApp)]
/// mod content {
///     use super::*;
///     
///     #[netabase_content_addressed]
///     #[derive(NetabaseModel, Debug, Clone, Serialize, Deserialize,
///              PartialEq, Eq, Hash, PartialOrd, Ord)]
///     pub struct ImmutableBlock {
///         #[primary_key]
///         pub cid: String,  // Content ID - derived from hash
///         
///         pub data: Vec<u8>,
///     }
/// }
/// ```
///
/// # Note
///
/// Currently a marker attribute. Full CID generation is planned for a future release.
///
/// # See Also
///
/// - [`NetabaseModel`] - Standard model derive
#[proc_macro_attribute]
pub fn netabase_content_addressed(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

use quote::{format_ident, quote};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use syn::{
    Ident, LitStr, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

struct ImportInput {
    file_path: LitStr,
    module_name: Option<Ident>,
}

impl Parse for ImportInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let file_path: LitStr = input.parse()?;
        let module_name = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            Some(input.parse::<Ident>()?)
        } else {
            None
        };
        Ok(ImportInput {
            file_path,
            module_name,
        })
    }
}

#[derive(Deserialize)]
struct ModelField {
    name: String,
    type_name: String,
    kind: String,
}

#[derive(Deserialize)]
struct ModelSchema {
    name: String,
    #[serde(default)]
    fields: Vec<ModelField>,
}

#[derive(Deserialize)]
struct FullSchema {
    name: String,
    subscriptions: Vec<String>,
    #[serde(default)]
    models: Vec<ModelSchema>,
    #[serde(flatten)]
    _other: toml::Table,
}

/// Procedural macro for importing a definition from a TOML schema file.
///
/// This macro reads a TOML file at compile time and generates the corresponding
/// definition module. Useful for schema-first development or code generation
/// from external tools.
///
/// # Syntax
///
/// ```rust,ignore
/// use netabase_macros::infer_netabase_definition;
///
/// // Basic usage - module name derived from definition name in TOML
/// infer_netabase_definition!("path/to/schema.toml");
///
/// // With explicit module name
/// infer_netabase_definition!("path/to/schema.toml", my_custom_module);
/// ```
///
/// # TOML Schema Format
///
/// ```toml
/// name = "MyDefinition"
/// version = 1
/// hash = 12345678
///
/// [[subscriptions]]
/// name = "Topic1"
///
/// [[models]]
/// name = "User"
///
/// [[models.fields]]
/// name = "id"
/// type_name = "String"
/// kind = "primary_key"
///
/// [[models.fields]]
/// name = "email"
/// type_name = "String"
/// kind = "secondary_key"
///
/// [[models.fields]]
/// name = "name"
/// type_name = "String"
/// kind = "field"
/// ```
///
/// # Generated Code
///
/// The macro generates a complete definition module equivalent to:
///
/// ```rust,ignore
/// #[netabase_definition(MyDefinition, subscriptions(Topic1), from_file = "schema.toml")]
/// pub mod MyDefinitionModule {
///     // Generated model structs and implementations
/// }
/// ```
///
/// # Path Resolution
///
/// Paths are resolved relative to `CARGO_MANIFEST_DIR` (the crate root).
///
/// # Use Cases
///
/// 1. **Schema-first development**: Design your schema in TOML, generate code
/// 2. **External tooling**: Generate TOML from other systems
/// 3. **Migration validation**: Compare TOML with compiled schema
///
/// # See Also
///
/// - [`netabase_definition`] - Direct definition macro
/// - `Definition::schema_toml()` - Export to TOML
#[proc_macro]
pub fn infer_netabase_definition(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ImportInput);
    let file_path_lit = input.file_path;
    let file_path_str = file_path_lit.value();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let full_path = PathBuf::from(manifest_dir).join(&file_path_str);

    let content = match fs::read_to_string(&full_path) {
        Ok(c) => c,
        Err(e) => {
            return syn::Error::new_spanned(
                file_path_lit,
                format!("Failed to read file at {:?}: {}", full_path, e),
            )
            .to_compile_error()
            .into();
        }
    };

    let schema: FullSchema = match toml::from_str(&content) {
        Ok(s) => s,
        Err(e) => {
            return syn::Error::new_spanned(file_path_lit, format!("Failed to parse TOML: {}", e))
                .to_compile_error()
                .into();
        }
    };

    let def_name = syn::Ident::new(&schema.name, proc_macro2::Span::call_site());
    let subs: Vec<syn::Ident> = schema
        .subscriptions
        .iter()
        .map(|s| syn::Ident::new(s, proc_macro2::Span::call_site()))
        .collect();

    let module_name = input.module_name.unwrap_or_else(|| def_name.clone());
    let module_name = format_ident!("{module_name}Module");

    let output = quote! {
        #[netabase_macros::netabase_definition(
            #def_name,
            subscriptions(#(#subs),*),
            from_file = #file_path_str
        )]
        pub mod #module_name {
            use super::*;
        }
    };

    output.into()
}

/// Procedural macro for generating CLI commands from a TOML schema.
///
/// Generates a complete command-line interface for interacting with a database
/// using the `clap` crate. Each model gets CRUD subcommands.
///
/// # Syntax
///
/// ```rust,ignore
/// use netabase_macros::generate_cli;
///
/// generate_cli!("path/to/schema.toml");
/// ```
///
/// # Generated Structure
///
/// For a schema with `User` and `Post` models:
///
/// ```rust,ignore
/// use clap::{Parser, Subcommand, Args};
///
/// #[derive(Parser)]
/// pub struct MyDefinitionCli {
///     #[arg(short, long, default_value = "./database")]
///     pub db_path: String,
///     
///     #[command(subcommand)]
///     pub command: MyDefinitionCommands,
/// }
///
/// #[derive(Subcommand)]
/// pub enum MyDefinitionCommands {
///     User(user::Commands),
///     Post(post::Commands),
/// }
///
/// pub mod user {
///     #[derive(Subcommand)]
///     pub enum Commands {
///         Create(CreateArgs),
///         Read(ReadArgs),
///         Update(UpdateArgs),
///         Delete(DeleteArgs),
///         List,
///     }
///     
///     #[derive(Args)]
///     pub struct CreateArgs {
///         #[arg(short, long)]
///         pub json: String,
///     }
///     // ... other args
/// }
/// ```
///
/// # Usage Example
///
/// ```rust,ignore
/// use netabase_macros::generate_cli;
///
/// generate_cli!("schema.toml");
///
/// fn main() {
///     let cli = MyDefinitionCli::parse();
///     
///     match cli.command {
///         MyDefinitionCommands::User(cmd) => {
///             // Handle user commands
///         }
///         MyDefinitionCommands::Post(cmd) => {
///             // Handle post commands
///         }
///     }
/// }
/// ```
///
/// # Command Examples
///
/// ```bash
/// # Create a user
/// ./cli user create --json '{"id": "u1", "name": "Alice"}'
///
/// # Read a user
/// ./cli user read --id u1
///
/// # List all users
/// ./cli user list
///
/// # Delete a user
/// ./cli user delete --id u1
/// ```
///
/// # Requirements
///
/// - `clap` crate with `derive` feature
/// - TOML file with valid schema
///
/// # See Also
///
/// - [`infer_netabase_definition!`] - Import definition from TOML
/// - `StoreConfig::with_client_binary()` - Export CLI with database
#[proc_macro]
pub fn generate_cli(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ImportInput);
    let file_path_lit = input.file_path;
    let file_path_str = file_path_lit.value();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let full_path = PathBuf::from(manifest_dir).join(&file_path_str);

    let content = match fs::read_to_string(&full_path) {
        Ok(c) => c,
        Err(e) => {
            return syn::Error::new_spanned(
                file_path_lit,
                format!("Failed to read file at {:?}: {}", full_path, e),
            )
            .to_compile_error()
            .into();
        }
    };

    let schema: FullSchema = match toml::from_str(&content) {
        Ok(s) => s,
        Err(e) => {
            return syn::Error::new_spanned(file_path_lit, format!("Failed to parse TOML: {}", e))
                .to_compile_error()
                .into();
        }
    };

    let def_name = syn::Ident::new(&schema.name, proc_macro2::Span::call_site());
    let model_idents: Vec<syn::Ident> = schema
        .models
        .iter()
        .map(|m| syn::Ident::new(&m.name, proc_macro2::Span::call_site()))
        .collect();

    let cli_name = format_ident!("{}Cli", def_name);
    
    // Use the shared generator which includes the run method
    let output = crate::generators::cli::generate_single_definition_cli(
        &cli_name,
        &def_name,
        &model_idents
    );

    output.into()
}
