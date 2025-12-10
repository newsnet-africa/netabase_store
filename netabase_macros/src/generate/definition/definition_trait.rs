//! NetabaseDefinitionTrait implementation generation
//!
//! Generates the implementation of NetabaseDefinitionTrait for the definition enum.

use proc_macro2::TokenStream;
use quote::quote;
use crate::parse::metadata::ModuleMetadata;

/// Generate NetabaseDefinitionTrait implementation
pub fn generate_definition_trait(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    let keys_name = &module.keys_name;
    let associated_types_name = quote::format_ident!("{}ModelAssociatedTypes", definition_name);
    let permissions_name = quote::format_ident!("{}Permissions", definition_name);

    quote! {
        impl netabase_store::traits::definition::NetabaseDefinitionTrait for #definition_name {
            type Keys = #keys_name;
            type ModelAssociatedTypes = #associated_types_name;
            type Permissions = #permissions_name;
        }
    }
}
