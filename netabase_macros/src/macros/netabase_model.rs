//! NetabaseModel derive macro.
//!
//! This derive macro is used to mark structs as database models within a
//! `#[netabase_definition]` module. It is now primarily a marker - the actual
//! code generation is done by the `#[netabase_definition]` macro.
//!
//! # Usage
//!
//! ```rust,ignore
//! #[derive(NetabaseModel)]
//! pub struct User {
//!     #[primary_key]
//!     pub id: String,
//!     pub name: String,
//! }
//! ```
//!
//! # Field Attributes
//!
//! - `#[primary_key]`: Mark field as the unique identifier
//! - `#[secondary_key]`: Create an index on this field
//! - `#[relational(TargetModel)]`: Foreign key reference
//! - `#[blob]`: Mark field for blob storage (auto-detected for large types)
//! - `#[subscription(Topic)]`: Subscribe this model to a topic
//!
//! # Auto-Detection
//!
//! The macro automatically detects:
//! - Blob fields: Vec<u8> larger than 60KB
//! - Relational fields: `RelationalLink<T>` types
//! - Subscription fields: Marked with `#[subscription]`
//!
//! # Generated Methods
//!
//! For each model, the following are generated:
//! - `get_primary_key()`: Extract the primary key
//! - `get_secondary_keys()`: Extract all secondary index values
//! - `get_relational_keys()`: Extract all foreign key references
//! - `get_blob_entries()`: Extract all blob data
//! - Serialization/deserialization with postcard
//!
//! # Rules and Limitations
//!
//! - Must have exactly one `#[primary_key]` field
//! - Primary key type must implement `StoreKey`
//! - Must be declared within a `#[netabase_definition]` module
//! - Struct name must be unique within the definition
//! - Generic types are not supported

use proc_macro2::TokenStream;
use syn::Result;

/// Implementation of the NetabaseModel derive macro.
///
/// This is now a no-op because the netabase_definition attribute macro
/// handles all the code generation and struct mutation.
/// This derive macro mainly serves as a marker for the visitor to identify
/// which structs should be processed as models.
pub fn netabase_model_derive(_input: TokenStream) -> Result<TokenStream> {
    Ok(TokenStream::new())
}
