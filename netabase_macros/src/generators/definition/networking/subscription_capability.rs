use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::Ident;

use crate::visitors::definition::{DefinitionSubscriptions, DefinitionVisitor, ModelInfo};

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
        quote! {
            pub mod #def_name {
                use super::*;
                #(#generated)*
            }
        }
    }

    pub fn generate_single_subscription_capability(
        subscriptions: &syn::Path,
        models: &[ModelInfo],
    ) -> TokenStream {
        let mut relevant_models = Vec::new();
        for m in models {
            if let Some(sub_info) = &m.visitor.subscriptions {
                if sub_info.topics.iter().any(|t| t.eq(subscriptions)) {
                    relevant_models.push(m);
                }
            }
        }

        let model_fields = relevant_models.iter().map(|m| {
            let field_name = Ident::new(
                &heck::AsSnakeCase(format!("{}_capability", m.name.to_string())).to_string(),
                Span::mixed_site(),
            );
            let model_type = &m.name;
            quote! {
                pub #field_name: netabase::node::capabilities::Capability<D, #model_type>
            }
        });

        let where_clauses = relevant_models.iter().map(|m| {
            let model_type = &m.name;
            quote! {
                #model_type: netabase_store::prelude::NetabaseModel<D>,
                <#model_type as netabase_store::prelude::NetabaseModel<D>>::Keys: std::cmp::Eq + std::cmp::PartialOrd
            }
        });

        let subscription_name = if let Some(subs) = subscriptions.segments.last() {
            format_ident!("{}Capabilities", subs.ident)
        } else {
            panic!("Subscription name could not be found")
        };

        quote! {
            pub struct #subscription_name<D: netabase::store::definition::NetworkDefinition + 'static>
            where
                D::Discriminant: std::fmt::Debug,
                #(#where_clauses),*
            {
                #(#model_fields),*
            }
        }
    }
}
