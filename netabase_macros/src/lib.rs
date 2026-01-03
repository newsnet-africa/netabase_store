//! Netabase procedural macros for defining database models and definitions.
//!
//! This crate provides derive macros and attribute macros for the netabase_store library.
//! Updated with migration support and schema export enhancements.

// Allow dead code in macro crate - utility functions may be used in future expansions
#![allow(dead_code)]

use proc_macro::TokenStream;

mod generators;
mod macros;
mod utils;
mod visitors;

#[proc_macro_derive(
    NetabaseModel,
    attributes(primary_key, secondary_key, relation, blob, subscribe)
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
