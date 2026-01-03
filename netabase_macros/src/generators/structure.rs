use crate::utils::schema::{DefinitionSchema, KeyTypeSchema};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

pub struct StructureGenerator;

impl StructureGenerator {
    pub fn generate(schema: &DefinitionSchema) -> TokenStream {
        let mut tokens = TokenStream::new();

        for model in &schema.models {
            let model_name = Ident::new(&model.name, Span::call_site());

            let mut fields = TokenStream::new();
            for field in &model.fields {
                let field_name = Ident::new(&field.name, Span::call_site());
                let field_type: syn::Type = syn::parse_str(&field.type_name)
                    .unwrap_or_else(|_| panic!("Failed to parse type: {}", field.type_name));

                let attr = match &field.key_type {
                    KeyTypeSchema::Primary => quote! { #[primary_key] },
                    KeyTypeSchema::Secondary => quote! { #[secondary_key] },
                    KeyTypeSchema::Relational { definition, model } => {
                        let def_ident = Ident::new(definition, Span::call_site());
                        let mod_ident = Ident::new(model, Span::call_site());
                        quote! { #[link(#def_ident, #mod_ident)] }
                    }
                    KeyTypeSchema::Blob => quote! { #[blob] },
                    KeyTypeSchema::Regular => quote! {},
                };

                fields.extend(quote! {
                    #attr
                    pub #field_name: #field_type,
                });
            }

            let subscribe_attr = if !model.subscriptions.is_empty() {
                let topics: Vec<_> = model
                    .subscriptions
                    .iter()
                    .map(|s| Ident::new(s, Span::call_site()))
                    .collect();
                quote! { #[subscribe(#(#topics),*)] }
            } else {
                quote! {}
            };

            // Generate version attribute if this model has version info
            let version_attr = if let (Some(family), Some(version)) = (&model.family, model.version)
            {
                let is_current = model.is_current;
                if is_current {
                    quote! { #[netabase_version(family = #family, version = #version, current)] }
                } else {
                    quote! { #[netabase_version(family = #family, version = #version)] }
                }
            } else {
                quote! {}
            };

            tokens.extend(quote! {
                #[derive(
                    netabase_macros::NetabaseModel,
                    Debug,
                    Clone,
                    bincode::Encode,
                    bincode::Decode,
                    serde::Serialize,
                    serde::Deserialize,
                    PartialEq,
                    Eq,
                    Hash,
                    PartialOrd,
                    Ord,
                )]
                #subscribe_attr
                #version_attr
                pub struct #model_name {
                    #fields
                }
            });
        }

        // Generate historical model versions from model_history
        for history in &schema.model_history {
            for versioned_model in &history.versions {
                // Skip the current version if it's already generated from models
                let is_already_generated = schema.models.iter().any(|m| {
                    m.name == versioned_model.struct_name
                        && m.family.as_ref() == Some(&history.family)
                        && m.version == Some(versioned_model.version)
                });

                if is_already_generated {
                    continue;
                }

                let model_name = Ident::new(&versioned_model.struct_name, Span::call_site());

                let mut fields = TokenStream::new();
                for field in &versioned_model.fields {
                    let field_name = Ident::new(&field.name, Span::call_site());
                    let field_type: syn::Type = syn::parse_str(&field.type_name)
                        .unwrap_or_else(|_| panic!("Failed to parse type: {}", field.type_name));

                    let attr = match &field.key_type {
                        KeyTypeSchema::Primary => quote! { #[primary_key] },
                        KeyTypeSchema::Secondary => quote! { #[secondary_key] },
                        KeyTypeSchema::Relational { definition, model } => {
                            let def_ident = Ident::new(definition, Span::call_site());
                            let mod_ident = Ident::new(model, Span::call_site());
                            quote! { #[link(#def_ident, #mod_ident)] }
                        }
                        KeyTypeSchema::Blob => quote! { #[blob] },
                        KeyTypeSchema::Regular => quote! {},
                    };

                    fields.extend(quote! {
                        #attr
                        pub #field_name: #field_type,
                    });
                }

                let family = &history.family;
                let version = versioned_model.version;
                let is_current = versioned_model.version == history.current_version;

                let version_attr = if is_current {
                    quote! { #[netabase_version(family = #family, version = #version, current)] }
                } else {
                    quote! { #[netabase_version(family = #family, version = #version)] }
                };

                let supports_downgrade_attr = if versioned_model.supports_downgrade {
                    quote! { #[netabase_version(supports_downgrade)] }
                } else {
                    quote! {}
                };

                let subscribe_attr = if !versioned_model.subscriptions.is_empty() {
                    let topics: Vec<_> = versioned_model
                        .subscriptions
                        .iter()
                        .map(|s| Ident::new(s, Span::call_site()))
                        .collect();
                    quote! { #[subscribe(#(#topics),*)] }
                } else {
                    quote! {}
                };

                tokens.extend(quote! {
                    #[derive(
                        netabase_macros::NetabaseModel,
                        Debug,
                        Clone,
                        bincode::Encode,
                        bincode::Decode,
                        serde::Serialize,
                        serde::Deserialize,
                        PartialEq,
                        Eq,
                        Hash,
                        PartialOrd,
                        Ord,
                    )]
                    #subscribe_attr
                    #version_attr
                    #supports_downgrade_attr
                    pub struct #model_name {
                        #fields
                    }
                });
            }
        }

        // Note: We intentionally do NOT regenerate auxiliary structs (like blob types) here.
        // When importing a schema, these structs are expected to be in scope via `use super::*;`
        // from the calling context. The structs section is only used for schema export/documentation.
        // If the caller needs blob types, they should import them from the original definition.

        tokens
    }
}
