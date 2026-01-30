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

#[proc_macro_derive(NetabaseBlobItem, attributes(blob, blob_as))]
pub fn netabase_blob_item(input: TokenStream) -> TokenStream {
    macros::netabase_blob_item::netabase_blob_item_derive(input.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_attribute]
pub fn netabase_libp2p(attr: TokenStream, item: TokenStream) -> TokenStream {
    macros::netabase_libp2p::netabase_libp2p_attribute(attr.into(), item.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_attribute]
pub fn netabase_networking(attr: TokenStream, item: TokenStream) -> TokenStream {
    macros::netabase_networking::netabase_networking_attribute(attr.into(), item.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

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

    let model_commands = model_idents.iter().map(|model| {
        let model_lower = model.to_string().to_lowercase();

        quote! {
            #[command(name = #model_lower, subcommand)]
            #model(#model::Commands)
        }
    });

    let model_modules = model_idents.iter().map(|model| {
        quote! {
            pub mod #model {
                use clap::{Args, Subcommand};

                #[derive(Subcommand, Debug, Clone)]
                pub enum Commands {
                    /// Create a new record
                    Create(CreateArgs),
                    /// Read a record by ID
                    Read(ReadArgs),
                    /// Update a record
                    Update(UpdateArgs),
                    /// Delete a record
                    Delete(DeleteArgs),
                    /// List all records
                    List,
                }

                #[derive(Args, Debug, Clone)]
                pub struct CreateArgs {
                    /// JSON string of the record to create
                    #[arg(short, long)]
                    pub json: String,
                }

                #[derive(Args, Debug, Clone)]
                pub struct ReadArgs {
                    /// Primary key of the record to read
                    #[arg(short, long)]
                    pub id: String,
                }

                #[derive(Args, Debug, Clone)]
                pub struct UpdateArgs {
                    /// Primary key of the record to update
                    #[arg(short, long)]
                    pub id: String,
                    /// JSON string of the updated record
                    #[arg(short, long)]
                    pub json: String,
                }

                #[derive(Args, Debug, Clone)]
                pub struct DeleteArgs {
                    /// Primary key of the record to delete
                    #[arg(short, long)]
                    pub id: String,
                }
            }
        }
    });

    let cli_name = format_ident!("{}Cli", def_name);
    let commands_name = format_ident!("{}Commands", def_name);

    let output = quote! {
        use clap::{Parser, Subcommand, Args};

        #[derive(Parser, Debug)]
        #[command(name = stringify!(#def_name))]
        #[command(about = "CLI for interacting with the database store", long_about = None)]
        pub struct #cli_name {
            /// Database path
            #[arg(short, long, default_value = "./database")]
            pub db_path: String,

            #[command(subcommand)]
            pub command: #commands_name,
        }

        #[derive(Subcommand, Debug, Clone)]
        pub enum #commands_name {
            #(#model_commands,)*
        }

        #(#model_modules)*
    };

    output.into()
}
