//! Netabase procedural macros for defining database models and definitions.
//!
//! This crate provides the procedural macros that power netabase_store's
//! type-safe, compile-time verified database schema system.
//!
//! # Macros Overview
//!
//! ## `#[netabase_model]` - Derive Macro
//!
//! Derives the `NetabaseModel` trait for a struct, making it storable in the database.
//!
//! **Attributes:**
//! - `#[primary_key]` - Marks the unique identifier field (exactly one required)
//! - `#[secondary_key]` - Creates an index on this field for fast lookups
//! - `#[relation]` or `#[link(Def, Model)]` - Marks relational link to another model
//! - `#[blob]` - Stores large data separately with automatic chunking
//! - `#[subscribe]` - Subscribes this model to a topic
//! - `#[netabase_version(family = "Name", version = N)]` - For versioned models
//!
//! **Example:**
//! ```rust,ignore
//! #[derive(NetabaseModel)]
//! pub struct User {
//!     #[primary_key]
//!     pub id: UserId,
//!     
//!     #[secondary_key]
//!     pub email: String,
//!     
//!     #[link(UserDef, Company)]
//!     pub company: CompanyId,
//!     
//!     #[blob]
//!     pub avatar: Vec<u8>,
//!     
//!     pub name: String,
//! }
//! ```
//!
//! ## `#[netabase_definition(Name)]` - Attribute Macro
//!
//! Creates a definition module containing related models. Generates:
//! - Definition enum wrapping all models
//! - Discriminant enum for pattern matching
//! - TreeNames enum for table access
//! - DefKeys enum for unified key handling
//! - Schema export functionality
//!
//! **Example:**
//! ```rust,ignore
//! #[netabase_definition(UserDef)]
//! mod user_definition {
//!     #[derive(NetabaseModel)]
//!     pub struct User { /* ... */ }
//!     
//!     #[derive(NetabaseModel)]
//!     pub struct Post { /* ... */ }
//! }
//! ```
//!
//! This generates:
//! ```rust,ignore
//! pub enum UserDef {
//!     User(User),
//!     Post(Post),
//! }
//! ```
//!
//! ## `#[netabase_repository(Name)]` - Attribute Macro
//!
//! Creates a repository grouping multiple definitions for inter-definition
//! communication. Enforces compile-time isolation.
//!
//! **Example:**
//! ```rust,ignore
//! #[netabase_repository(MyRepo)]
//! mod my_repository {
//!     #[netabase_definition(UserDef, repos(MyRepo))]
//!     mod users { /* ... */ }
//!     
//!     #[netabase_definition(PostDef, repos(MyRepo))]
//!     mod posts { /* ... */ }
//! }
//! ```
//!
//! ## `#[netabase]` - Convenience Macro
//!
//! Combines definition and model setup in a single module.
//!
//! ## `#[derive(NetabaseBlobItem)]` - Derive Macro
//!
//! Derives blob serialization for custom types used in `#[blob]` fields.
//!
//! # Code Generation
//!
//! The macros generate:
//!
//! 1. **Trait Implementations**
//!    - `NetabaseModel<D>` for models
//!    - `NetabaseDefinition` for definitions
//!    - `NetabaseRepository` for repositories
//!
//! 2. **Supporting Enums**
//!    - Discriminant enums for efficient pattern matching
//!    - Key enums for type-safe key access
//!    - TreeNames enums for table naming
//!
//! 3. **Helper Functions**
//!    - Key extraction methods
//!    - Conversion methods (Into/TryFrom)
//!    - Schema export methods
//!
//! # Compile-Time Verification
//!
//! The macros enforce:
//! - Exactly one `#[primary_key]` per model
//! - Valid field types for keys
//! - Repository isolation for relational links
//! - Proper attribute usage
//!
//! # Error Messages
//!
//! The macros provide helpful error messages:
//! - Missing primary key
//! - Invalid attribute placement
//! - Type mismatch in relational links
//! - Duplicate keys
//!
//! # Internal Modules
//!
//! - `generators/`: Code generation logic
//! - `macros/`: Macro entry points
//! - `utils/`: Helper functions and attribute parsing
//! - `visitors/`: AST visitors for extracting metadata
//!
//! # Migration Support
//!
//! Models can declare version information:
//!
//! ```rust,ignore
//! #[derive(NetabaseModel)]
//! #[netabase_version(family = "User", version = 2)]
//! pub struct UserV2 { /* ... */ }
//! ```
//!
//! The macro generates migration chain infrastructure.
//!
//! # Schema Export
//!
//! Definitions can export their schema to TOML:
//!
//! ```rust,ignore
//! let toml = UserDef::export_toml();
//! ```
//!
//! This includes:
//! - All model structures
//! - Key definitions
//! - Blob field locations
//! - Subscription topics
//!
//! Build timestamp: 2026-01-03

// Allow dead code in macro crate - utility functions may be used in future expansions
#![allow(dead_code)]

use proc_macro::TokenStream;

// Force rebuild marker - updated 2026-01-03
const _BUILD_MARKER: &str = "v0.1.5-all-tests-fixed";

mod generators;
mod macros;
mod utils;
mod visitors;

#[proc_macro_derive(
    NetabaseModel,
    attributes(
        primary_key,
        secondary_key,
        relation,
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

#[proc_macro_attribute]
pub fn netabase_definition(attr: TokenStream, item: TokenStream) -> TokenStream {
    macros::netabase_definition::netabase_definition_attribute(attr.into(), item.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_attribute]
pub fn netabase_repository(attr: TokenStream, item: TokenStream) -> TokenStream {
    macros::netabase_repository::netabase_repository_attribute(attr.into(), item.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_attribute]
pub fn netabase(attr: TokenStream, item: TokenStream) -> TokenStream {
    macros::netabase::netabase_attribute(attr.into(), item.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(NetabaseBlobItem)]
pub fn netabase_blob_item(input: TokenStream) -> TokenStream {
    macros::netabase_blob_item::netabase_blob_item_derive(input.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
