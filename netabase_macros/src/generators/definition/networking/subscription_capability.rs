use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::Ident;

use crate::visitors::definition::{DefinitionVisitor, ModelInfo};

pub struct SubscriptionCapabilityGenerator<'a> {
    visitor: &'a DefinitionVisitor,
}

impl<'a> SubscriptionCapabilityGenerator<'a> {
    pub fn new(visitor: &'a DefinitionVisitor) -> Self {
        Self { visitor }
    }

    pub fn generate_all_subscription_capabilities(&self) -> TokenStream {
        let models = &self.visitor.models;
        let generated = self
            .visitor
            .subscriptions
            .topics
            .iter()
            .map(|s| Self::generate_single_subscription_capability(s, models));
        let def_name = format_ident!(
            "gen_{}_capabilities",
            heck::AsSnakeCase(self.visitor.definition_name.to_string()).to_string()
        );
        
        let def_caps = self.generate_definition_capabilities_struct();
        let network_def_impl = self.generate_network_definition_impl();

        quote! {
            pub mod #def_name {
                use super::*;
                #(#generated)*
                
                #def_caps
                #network_def_impl
            }
        }
    }

    pub fn generate_single_subscription_capability(
        subscriptions: &syn::Path,
        models: &[ModelInfo],
    ) -> TokenStream {
        let mut relevant_models = Vec::new();
        for m in models {
            if let Some(sub_info) = &m.visitor.subscriptions
                && sub_info.topics.iter().any(|t| t.eq(subscriptions)) {
                    relevant_models.push(m);
                }
        }

        let model_fields = relevant_models.iter().map(|m| {
            let field_name = Ident::new(
                &heck::AsSnakeCase(format!("{}_capability", m.name)).to_string(),
                Span::mixed_site(),
            );
            let model_name = &m.name;
            let model_type = if m.is_content_addressed() {
                let envelope = format_ident!("{}Envelope", model_name);
                quote! { #envelope }
            } else {
                quote! { #model_name }
            };
            quote! {
                pub #field_name: netabase::capabilities::Capability<D, #model_type>
            }
        });

        let where_clauses = relevant_models.iter().map(|m| {
            let model_name = &m.name;
            let model_type = if m.is_content_addressed() {
                let envelope = format_ident!("{}Envelope", model_name);
                quote! { #envelope }
            } else {
                quote! { #model_name }
            };
            quote! {
                #model_type: netabase_store::prelude::NetabaseModel<D>,
                <#model_type as netabase_store::prelude::NetabaseModel<D>>::Keys: std::fmt::Debug + Clone + std::cmp::Eq
            }
        });

        let subscription_name = if let Some(subs) = subscriptions.segments.last() {
            format_ident!("{}Capabilities", subs.ident)
        } else {
            panic!("Subscription name could not be found")
        };

        quote! {
            pub struct #subscription_name<D: netabase::data::store::network::NetworkDefinition + 'static>
            where
                D::Discriminant: std::fmt::Debug,
                D::SubscriptionKeysDiscriminant: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug + Clone + PartialEq + Eq,
                #(#where_clauses),*
            {
                #(#model_fields),*
            }
        }
    }

    fn generate_definition_capabilities_struct(&self) -> TokenStream {
        let def_name = &self.visitor.definition_name;
        let struct_name = format_ident!("{}Capabilities", def_name);
        
        let fields = self.visitor.models.iter().map(|m| {
            let model_name = &m.name;
            let field_name = format_ident!("{}_capabilities", heck::AsSnakeCase(model_name.to_string()).to_string());
            let model_type = if m.is_content_addressed() {
                let envelope = format_ident!("{}Envelope", model_name);
                quote! { #envelope }
            } else {
                quote! { #model_name }
            };
            // Capability<D, M>
            quote! {
                pub #field_name: Vec<netabase::capabilities::Capability<#def_name, #model_type>>
            }
        });

        quote! {
            #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
            pub struct #struct_name {
                #(#fields),*
            }
        }
    }

    fn generate_network_definition_impl(&self) -> TokenStream {
        let def_name = &self.visitor.definition_name;
        let cap_struct_name = format_ident!("{}Capabilities", def_name);
        
        quote! {
            impl netabase::data::store::network::NetworkDefinition for #def_name {
                type DefinitionCapabilities = #cap_struct_name;
            }
        }
    }
}