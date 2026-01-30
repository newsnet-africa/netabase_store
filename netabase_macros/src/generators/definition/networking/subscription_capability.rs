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
        let def_name = format_ident!("gen_{}_capabilities", self.visitor.definition_name);
        quote! {
            pub mod #def_name {
                #(#generated),*
            }
        }
    }

    pub fn generate_single_subscription_capability(
        subscriptions: &syn::Path,
        models: &[ModelInfo],
    ) -> TokenStream {
        let model_fields = models.iter().filter_map(|m| {
            if let Some(sub_info) = &m.visitor.subscriptions {
                if sub_info.topics.iter().any(|t| t.eq(subscriptions)) {
                    let field_name: Ident = Ident::new(
                        &heck::AsSnakeCase(format!("{}_capability", m.name.to_string()))
                            .to_string(),
                        Span::mixed_site(),
                    );
                    let model_type = &m.name;
                    Some(quote! {
                        #field_name: netabase::node::capabilities::Capability<D, #model_type>
                    })
                } else {
                    None
                }
            } else {
                None
            }
        });

        let subscription_name = if let Some(subs) = subscriptions.segments.last() {
            format_ident!("{}Capabilities", subs.ident)
        } else {
            panic!("Subscription name could not be found")
        };

        quote! {
            pub struct #subscription_name {
                #(#model_fields),*
            }
        }
    }
}
