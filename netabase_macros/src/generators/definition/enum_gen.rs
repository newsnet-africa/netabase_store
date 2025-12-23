use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;
use crate::visitors::definition::DefinitionVisitor;
use crate::utils::naming::*;

/// Generator for Definition enum and DefinitionSubscriptions enum
pub struct DefinitionEnumGenerator<'a> {
    visitor: &'a DefinitionVisitor,
}

impl<'a> DefinitionEnumGenerator<'a> {
    pub fn new(visitor: &'a DefinitionVisitor) -> Self {
        Self { visitor }
    }

    /// Generate the Definition enum that wraps all models and nested definitions
    pub fn generate_definition_enum(&self) -> TokenStream {
        let definition_name = &self.visitor.definition_name;
        let tree_name = definition_tree_name_type(definition_name);

        let mut variants = Vec::new();

        // Models
        for model in &self.visitor.models {
            let model_name = &model.name;
            variants.push(quote! { #model_name(#model_name) });
        }

        // Nested Definitions
        for nested in &self.visitor.nested_definitions {
            let nested_name = &nested.definition_name;
            variants.push(quote! { #nested_name(#nested_name) });
        }

        quote! {
            // Main definition enum
            #[derive(
                Clone, Debug,
                bincode::Encode, bincode::Decode,
                serde::Serialize, serde::Deserialize,
                PartialEq, Eq, PartialOrd, Ord, Hash,
                derive_more::From, derive_more::TryInto,
                strum::EnumDiscriminants
            )]
            #[strum_discriminants(name(#tree_name))]
            #[strum_discriminants(derive(
                strum::AsRefStr,
                bincode::Encode, bincode::Decode,
                serde::Serialize, serde::Deserialize
            ))]
            pub enum #definition_name {
                #(#variants),*
            }
        }
    }

    /// Generate the DefinitionKeys enum
    pub fn generate_definition_keys_enum(&self) -> TokenStream {
        let definition_name = &self.visitor.definition_name;
        let enum_name = definition_keys_enum_name(definition_name);

        let mut variants = Vec::new();

        // Models
        for model in &self.visitor.models {
            let model_name = &model.name;
            let keys_enum = unified_keys_enum_name(model_name);
            variants.push(quote! { #model_name(#keys_enum) });
        }

        // Nested Definitions
        for nested in &self.visitor.nested_definitions {
            let nested_name = &nested.definition_name;
            let nested_keys_enum = definition_keys_enum_name(nested_name);
            variants.push(quote! { #nested_name(#nested_keys_enum) });
        }

        quote! {
            #[derive(
                Clone, Debug,
                bincode::Encode, bincode::Decode,
                serde::Serialize, serde::Deserialize,
                PartialEq, Eq, PartialOrd, Ord, Hash
            )]
            pub enum #enum_name {
                #(#variants),*
            }
        }
    }

    /// Generate the DefinitionSubscriptions enum
    pub fn generate_subscriptions_enum(&self) -> TokenStream {
        let definition_name = &self.visitor.definition_name;
        let enum_name = definition_subscriptions_enum_name(definition_name);
        
        // Define the discriminant name (e.g. DefinitionSubscriptionsDiscriminants)
        let discriminant_name = Ident::new(
            &format!("{}Discriminants", enum_name),
            enum_name.span()
        );

        if self.visitor.subscriptions.topics.is_empty() {
            // Generate an empty enum
            // For empty enum, strum generates empty discriminant enum
            return quote! {
                #[derive(
                    Clone, Eq, PartialEq, PartialOrd, Ord, Debug,
                    bincode::Encode, bincode::Decode,
                    serde::Serialize, serde::Deserialize,
                    Hash,
                    strum::EnumDiscriminants
                )]
                #[strum_discriminants(name(#discriminant_name))]
                #[strum_discriminants(derive(strum::AsRefStr))]
                pub enum #enum_name {}
            };
        }

        let variants: Vec<_> = self.visitor.subscriptions.topics
            .iter()
            .map(|topic| {
                let topic_ident = path_last_segment(topic)
                    .expect("Invalid subscription topic");

                quote! { #topic_ident }
            })
            .collect();

        quote! {
            #[derive(
                Clone, Eq, PartialEq, PartialOrd, Ord, Debug,
                bincode::Encode, bincode::Decode,
                serde::Serialize, serde::Deserialize,
                Hash,
                strum::EnumDiscriminants
            )]
            #[strum_discriminants(name(#discriminant_name))]
            #[strum_discriminants(derive(strum::AsRefStr))]
            pub enum #enum_name {
                #(#variants),*
            }
            
            // Generate helper to implement Value/Key for owned types
            impl redb::Value for #enum_name {
                type SelfType<'a> = Self;
                type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

                fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
                where
                    Self: 'a,
                {
                    bincode::decode_from_slice(data, bincode::config::standard())
                        .unwrap()
                        .0
                }

                fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
                where
                    Self: 'a,
                {
                    std::borrow::Cow::Owned(
                        bincode::encode_to_vec(value, bincode::config::standard()).unwrap()
                    )
                }

                fn fixed_width() -> Option<usize> {
                    None
                }
                fn type_name() -> redb::TypeName {
                     redb::TypeName::new(&format!("{}::{}", module_path!(), stringify!(#enum_name)))
                }
            }

            impl redb::Key for #enum_name {
                fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
                    data1.cmp(data2)
                }
            }
        }
    }

    /// Generate the DefinitionTreeNames complex enum
    pub fn generate_definition_tree_names_enum(&self) -> TokenStream {
        let definition_name = &self.visitor.definition_name;
        let enum_name = definition_tree_names_enum_name(definition_name); // Complex enum
        let discriminant_name = definition_tree_name_type(definition_name); // Simple discriminant enum

        let mut variants = Vec::new();
        let mut get_tree_names_arms = Vec::new();

        // Models
        for model in &self.visitor.models {
            let model_name = &model.name;
            variants.push(quote! { 
                #model_name(netabase_store::traits::registery::models::treenames::ModelTreeNames<'static, #definition_name, #model_name>) 
            });
            get_tree_names_arms.push(quote! {
                #discriminant_name::#model_name => vec![#enum_name::#model_name(#model_name::TREE_NAMES)]
            });
        }

        // Nested Definitions
        for nested in &self.visitor.nested_definitions {
            let nested_name = &nested.definition_name;
            let nested_tree_names = definition_tree_names_enum_name(nested_name);
            
            variants.push(quote! { 
                #nested_name(#nested_tree_names)
            });
            
            // For nested definitions, we return the default tree names for that definition wrapped in the variant
            get_tree_names_arms.push(quote! {
                #discriminant_name::#nested_name => vec![#enum_name::#nested_name(#nested_tree_names::default())]
            });
        }

        // Default implementation (use first model or nested def)
        let default_variant = if !self.visitor.models.is_empty() {
             let first_model = &self.visitor.models[0].name;
             quote! { #enum_name::#first_model(#first_model::TREE_NAMES) }
        } else if !self.visitor.nested_definitions.is_empty() {
             let first_nested = &self.visitor.nested_definitions[0].definition_name;
             let nested_tree_names = definition_tree_names_enum_name(first_nested);
             quote! { #enum_name::#first_nested(#nested_tree_names::default()) }
        } else {
             // Empty definition?
             quote! { panic!("Empty definition") }
        };

        let default_impl = quote! {
            impl Default for #enum_name {
                fn default() -> Self {
                    #default_variant
                }
            }
        };

        // TryInto implementation (returns Err(()))
        let try_into_impl = quote! {
            impl TryInto<netabase_store::traits::registery::models::treenames::DiscriminantTableName<#definition_name>> for #enum_name {
                type Error = ();

                fn try_into(self) -> Result<netabase_store::traits::registery::models::treenames::DiscriminantTableName<#definition_name>, Self::Error> {
                    Err(())
                }
            }
        };

        // NetabaseDefinitionTreeNames trait implementation
        let netabase_definition_tree_names_impl = quote! {
            impl netabase_store::traits::registery::definition::NetabaseDefinitionTreeNames<#definition_name> for #enum_name {
                fn get_tree_names(discriminant: #discriminant_name) -> Vec<Self> {
                    match discriminant {
                        #(#get_tree_names_arms),*
                    }
                }

                fn get_model_tree<M: netabase_store::traits::registery::models::model::NetabaseModel<#definition_name>>(&self) -> Option<M>
                where
                    for<'a> Self: From<netabase_store::traits::registery::models::treenames::ModelTreeNames<'a, Self, M>>,
                    for<'a> <<M as netabase_store::traits::registery::models::model::NetabaseModel<#definition_name>>::Keys as netabase_store::traits::registery::models::keys::NetabaseModelKeys<#definition_name, M>>::Secondary<'a>:
                        strum::IntoDiscriminant,
                    for<'a> <<M as netabase_store::traits::registery::models::model::NetabaseModel<#definition_name>>::Keys as netabase_store::traits::registery::models::keys::NetabaseModelKeys<#definition_name, M>>::Relational<'a>:
                        strum::IntoDiscriminant,
                    for<'a> <<M as netabase_store::traits::registery::models::model::NetabaseModel<#definition_name>>::Keys as netabase_store::traits::registery::models::keys::NetabaseModelKeys<#definition_name, M>>::Subscription<'a>:
                        strum::IntoDiscriminant,
                    for<'a> <<<M as netabase_store::traits::registery::models::model::NetabaseModel<#definition_name>>::Keys as netabase_store::traits::registery::models::keys::NetabaseModelKeys<#definition_name, M>>::Secondary<'a> as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
                    for<'a> <<<M as netabase_store::traits::registery::models::model::NetabaseModel<#definition_name>>::Keys as netabase_store::traits::registery::models::keys::NetabaseModelKeys<#definition_name, M>>::Relational<'a> as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
                    for<'a> <<<M as netabase_store::traits::registery::models::model::NetabaseModel<#definition_name>>::Keys as netabase_store::traits::registery::models::keys::NetabaseModelKeys<#definition_name, M>>::Subscription<'a> as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
                    for<'a> <<<M as netabase_store::traits::registery::models::model::NetabaseModel<#definition_name>>::Keys as netabase_store::traits::registery::models::keys::NetabaseModelKeys<#definition_name, M>>::Blob<'a> as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
                    <<M as netabase_store::traits::registery::models::model::NetabaseModel<#definition_name>>::Keys as netabase_store::traits::registery::models::keys::NetabaseModelKeys<#definition_name, M>>::Subscription<'static>: 'static
                {
                    None
                }
            }
        };

        quote! {
            #[derive(Clone, Debug, PartialEq)]
            pub enum #enum_name {
                #(#variants),*
            }

            #default_impl
            #try_into_impl
            #netabase_definition_tree_names_impl
        }
    }
}
