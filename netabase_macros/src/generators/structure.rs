use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;
use crate::utils::schema::{DefinitionSchema, KeyTypeSchema};

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
                    },
                    KeyTypeSchema::Blob => quote! { #[blob] },
                    KeyTypeSchema::Regular => quote! {},
                };

                fields.extend(quote! {
                    #attr
                    pub #field_name: #field_type,
                });
            }

            let subscribe_attr = if !model.subscriptions.is_empty() {
                let topics: Vec<_> = model.subscriptions.iter()
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
                pub struct #model_name {
                    #fields
                }
            });
        }

        for s in &schema.structs {
            let struct_name = Ident::new(&s.name, Span::call_site());
            
            let struct_def = if s.is_tuple {
                let mut fields = TokenStream::new();
                for field in &s.fields {
                    let field_type: syn::Type = syn::parse_str(&field.type_name)
                        .unwrap_or_else(|_| panic!("Failed to parse type: {}", field.type_name));
                    fields.extend(quote! {
                         pub #field_type,
                    });
                }
                quote! {
                    #[derive(Debug, Clone, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
                    pub struct #struct_name(#fields);
                }
            } else {
                let mut fields = TokenStream::new();
                for field in &s.fields {
                     let field_name = Ident::new(&field.name, Span::call_site());
                     let field_type: syn::Type = syn::parse_str(&field.type_name)
                        .unwrap_or_else(|_| panic!("Failed to parse type: {}", field.type_name));
                     fields.extend(quote! {
                         pub #field_name: #field_type,
                     });
                }
                quote! {
                    #[derive(Debug, Clone, bincode::Encode, bincode::Decode, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
                    pub struct #struct_name {
                        #fields
                    }
                }
            };
            
            tokens.extend(struct_def);
        }

        tokens
    }
}
