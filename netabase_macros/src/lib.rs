//! Netabase Procedural Macros
//!
//! This crate provides procedural macros to generate boilerplate code for netabase definitions.
//! It eliminates thousands of lines of repetitive code per definition (~94% reduction).
//!
//! # Usage
//!
//! ```
//! use netabase_macros::netabase_definition_module;
//!
//! #[netabase_definition_module(MyDefinition, MyDefinitionKeys)]
//! pub mod my_definition {
//!     use netabase_macros::NetabaseModel;
//!
//!     #[derive(NetabaseModel)]
//!     pub struct MyModel {
//!         #[primary_key]
//!         pub id: u64,
//!         #[secondary_key]
//!         pub email: String,
//!     }
//! }
//!
//! fn main() {}
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod parse;
mod generate;
mod utils;

/// Attribute macro for defining a netabase definition module
///
/// This macro processes a module containing NetabaseModel structs and generates:
/// - Key wrapper types for each model
/// - Secondary, relational, and subscription key enums
/// - Definition-level enums and trait implementations
/// - Backend-specific implementations (Redb and Sled)
///
/// # Syntax
///
/// ```
/// use netabase_macros::netabase_definition_module;
/// use netabase_macros::NetabaseModel;
///
/// #[netabase_definition_module(DefinitionName, DefinitionKeys, subscriptions(Topic1, Topic2))]
/// pub mod definition_name {
///     use super::*;
///
///     #[derive(NetabaseModel)]
///     #[subscribe(Topic1)]
///     pub struct MyModel {
///         #[primary_key]
///         pub id: u64,
///     }
/// }
///
/// fn main() {}
/// ```
///
/// # Arguments
///
/// - `DefinitionName`: The name for the definition enum
/// - `DefinitionKeys`: The name for the keys enum
/// - `subscriptions(...)`: Optional list of subscription topics available in this definition
#[proc_macro_attribute]
pub fn netabase_definition_module(attr: TokenStream, item: TokenStream) -> TokenStream {
    let module = parse_macro_input!(item as syn::ItemMod);

    // Convert attribute TokenStream to proc_macro2::TokenStream and create an attribute
    let attr_tokens = proc_macro2::TokenStream::from(attr);
    let attr: syn::Attribute = syn::parse_quote!(#[netabase_definition_module(#attr_tokens)]);

    // Parse the module and all its models
    let module_metadata = match parse::ModuleVisitor::parse_module(&attr, &module) {
        Ok(m) => m,
        Err(e) => return TokenStream::from(e.to_compile_error()),
    };

    // Generate all boilerplate code for the entire definition
    let generated = match generate::generate_complete_definition(&module_metadata) {
        Ok(code) => code,
        Err(e) => return TokenStream::from(e.to_compile_error()),
    };

    // Return the original module plus generated code
    TokenStream::from(quote! {
        #module
        #generated
    })
}

/// Derive macro for NetabaseModel
///
/// This macro generates all the boilerplate code for a model, including:
/// - Primary key wrapper type
/// - Secondary key wrappers and enums
/// - Relational key enums
/// - Subscription enums
/// - All required trait implementations
///
/// # Attributes
///
/// Field attributes:
/// - `#[primary_key]`: Marks the primary key field (required, exactly one per model)
/// - `#[secondary_key]`: Marks a secondary index field (optional, multiple allowed)
/// - `#[relation]`: Marks a relational link field (optional, multiple allowed)
/// - `#[cross_definition_link(path)]`: Links to a model in another definition
///
/// Model attributes:
/// - `#[subscribe(Topic1, Topic2)]`: Subscribes this model to topics (optional)
///
/// # Example
///
/// ```
/// use netabase_macros::NetabaseModel;
/// use netabase_macros::netabase_definition_module;
///
/// // Note: NetabaseModel must be used within a netabase_definition_module to fully work,
/// // as it relies on the module to generate the definition enum.
/// #[netabase_definition_module(UserDefinition, UserKeys, subscriptions(Updates))]
/// pub mod user_def {
///     use super::*;
///
///     #[derive(NetabaseModel)]
///     #[subscribe(Updates)]
///     pub struct User {
///         #[primary_key]
///         pub id: u64,
///         #[secondary_key]
///         pub email: String,
///         #[secondary_key]
///         pub username: String,
///         pub name: String,
///         pub age: u32,
///     }
/// }
///
/// fn main() {}
/// ```
#[proc_macro_derive(NetabaseModel, attributes(primary_key, secondary_key, relation, cross_definition_link, subscribe))]
pub fn derive_netabase_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Parse the model using our visitor
    let model = match parse::ModelVisitor::parse_model(&input) {
        Ok(m) => m,
        Err(e) => return TokenStream::from(e.to_compile_error()),
    };

    // For now, use a placeholder definition name
    // The real definition name will come from the module attribute
    let definition_name = syn::Ident::new(
        &format!("{}Definition", model.name),
        proc_macro2::Span::call_site()
    );

    // Generate all boilerplate code for this model
    let generated = match generate::generate_complete_model(&model, &definition_name) {
        Ok(code) => code,
        Err(e) => return TokenStream::from(e.to_compile_error()),
    };

    TokenStream::from(generated)
}
