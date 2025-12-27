use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;
use crate::visitors::definition::{DefinitionVisitor, ModelInfo};
use crate::generators::model::TraitGenerator;
use crate::utils::naming::*;

/// Generator for definition-level trait implementations
/// These are traits that need to know both the Definition and Model types
pub struct DefinitionTraitGenerator<'a> {
    visitor: &'a DefinitionVisitor,
}

impl<'a> DefinitionTraitGenerator<'a> {
    pub fn new(visitor: &'a DefinitionVisitor) -> Self {
        Self { visitor }
    }

    /// Generate all trait implementations for all models in the definition
    pub fn generate(&self) -> TokenStream {
        let mut output = TokenStream::new();

        let definition_name = &self.visitor.definition_name;

        // Generate NetabaseDefinition trait implementation for the definition
        let definition_trait = self.generate_netabase_definition_trait();
        output.extend(definition_trait);

        // Generate NetabaseDefinitionKeys trait implementation
        let def_keys_trait = self.generate_definition_keys_trait();
        output.extend(def_keys_trait);

        // Generate NetabaseDefinitionSubscriptionKeys trait implementation
        let def_subs_trait = self.generate_definition_subscription_keys_trait();
        output.extend(def_subs_trait);

        // Generate RedbDefinition trait implementation
        let redb_def_trait = self.generate_redb_definition_trait();
        output.extend(redb_def_trait);

        for model_info in &self.visitor.models {
            // First generate subscription enum for this model (if it has subscriptions)
            let sub_enum = self.generate_subscription_enum(definition_name, model_info);
            output.extend(sub_enum);

            // Then generate trait implementations
            let traits = self.generate_model_traits(definition_name, model_info);
            output.extend(traits);
        }

        output
    }

    fn generate_definition_keys_trait(&self) -> TokenStream {
        let definition_name = &self.visitor.definition_name;
        let keys_enum = definition_keys_enum_name(definition_name);

        quote! {
            impl netabase_store::traits::registery::definition::NetabaseDefinitionKeys<#definition_name> for #keys_enum {}
        }
    }

    fn generate_definition_subscription_keys_trait(&self) -> TokenStream {
        let definition_name = &self.visitor.definition_name;
        let subs_enum = definition_subscriptions_enum_name(definition_name);

        quote! {
            impl netabase_store::traits::registery::definition::subscription::NetabaseDefinitionSubscriptionKeys for #subs_enum {}
        }
    }

    fn generate_redb_definition_trait(&self) -> TokenStream {
        let definition_name = &self.visitor.definition_name;

        // Use the first model as representative (following the boilerplate pattern)
        if let Some(first_model) = self.visitor.models.first() {
            let model_name = &first_model.visitor.model_name;

            quote! {
                impl ::netabase_store::traits::registery::definition::redb_definition::RedbDefinition for #definition_name {
                    type ModelTableDefinition<'db> = ::netabase_store::traits::registery::models::model::redb_model::RedbModelTableDefinitions<'db, #model_name, Self>;
                }
            }
        } else {
            // If no models, generate a placeholder (shouldn't happen in practice)
            TokenStream::new()
        }
    }

    fn generate_subscription_enum(&self, definition_name: &syn::Ident, model_info: &ModelInfo) -> TokenStream {
        let model_name = &model_info.name;
        let visitor = &model_info.visitor;

        // If no subscriptions, treat as empty topics list
        let empty_topics = Vec::new();
        let topics = visitor.subscriptions.as_ref().map(|s| &s.topics).unwrap_or(&empty_topics);

        let enum_name = subscriptions_enum_name(model_name);
        let tree_name = tree_name_type(&enum_name);
        let def_subscription_enum = definition_subscriptions_enum_name(definition_name);

        let variants: Vec<_> = topics
            .iter()
            .map(|topic| {
                let topic_ident = path_last_segment(topic).expect("Invalid subscription topic");
                quote! { #topic_ident(#def_subscription_enum) }
            })
            .collect();

        let tree_name_variants: Vec<_> = topics
            .iter()
            .map(|topic| {
                path_last_segment(topic).expect("Invalid subscription topic").clone()
            })
            .collect();

        quote! {
            // TreeName discriminant enum
            #[derive(
                Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
                bincode::Encode, bincode::Decode,
                serde::Serialize, serde::Deserialize,
                strum::AsRefStr
            )]
            pub enum #tree_name {
                #(#tree_name_variants),*
            }

            // Main subscription enum
            #[derive(
                Clone, Eq, PartialEq, PartialOrd, Ord, Debug,
                bincode::Encode, bincode::Decode,
                serde::Serialize, serde::Deserialize,
                Hash
            )]
            pub enum #enum_name {
                #(#variants),*
            }
            
            // Implement IntoDiscriminant manually for empty/non-empty enums
            impl strum::IntoDiscriminant for #enum_name {
                type Discriminant = #tree_name;

                fn discriminant(&self) -> Self::Discriminant {
                    match self {
                        #(#enum_name::#tree_name_variants(_) => #tree_name::#tree_name_variants),*
                    }
                }
            }
        }
    }

    fn generate_model_traits(&self, definition_name: &syn::Ident, model_info: &ModelInfo) -> TokenStream {
        let model_name = &model_info.name;
        let visitor = &model_info.visitor;

        // Generate marker traits (StoreKeyMarker, StoreValueMarker, etc.)
        let marker_traits = self.generate_marker_traits(definition_name, model_name, visitor);

        // Generate Store traits (StoreKey, StoreValue)
        let store_traits = self.generate_store_traits(definition_name, model_name, visitor);

        // Generate key type traits (NetabaseModelKeys, PrimaryKey, SecondaryKey, etc.)
        let trait_gen = TraitGenerator::new(visitor);
        let model_keys_trait = trait_gen.generate_model_keys_trait(definition_name);
        let key_traits = self.generate_key_type_traits(definition_name, model_name, visitor);

        // Generate NetabaseModel trait
        let netabase_model_trait = trait_gen.generate_netabase_model_trait(definition_name);

        // Generate RedbNetabaseModel trait
        let redb_trait = self.generate_redb_netabase_model_trait(definition_name, model_name);

        // Generate subscription conversion traits
        let subscription_traits = self.generate_subscription_traits(definition_name, model_name, visitor);

        quote! {
            #marker_traits
            #store_traits
            #model_keys_trait
            #key_traits
            #netabase_model_trait
            #redb_trait
            #subscription_traits
        }
    }

    fn generate_marker_traits(
        &self,
        definition_name: &syn::Ident,
        model_name: &syn::Ident,
        visitor: &crate::visitors::model::field::ModelFieldVisitor,
    ) -> TokenStream {
        let id_type = primary_key_type_name(model_name);
        let keys_enum = unified_keys_enum_name(model_name);

        let mut impls = vec![];

        // StoreKeyMarker and StoreValueMarker for ID
        impls.push(quote! {
            impl netabase_store::traits::registery::models::StoreKeyMarker<#definition_name> for #id_type {}
            impl netabase_store::traits::registery::models::StoreValueMarker<#definition_name> for #id_type {}
        });

        // StoreValueMarker for model
        impls.push(quote! {
            impl netabase_store::traits::registery::models::StoreValueMarker<#definition_name> for #model_name {}
        });

        // NetabaseModelMarker
        impls.push(quote! {
            impl netabase_store::traits::registery::models::model::NetabaseModelMarker<#definition_name> for #model_name {}
        });

        // Secondary keys
        let secondary_enum = secondary_keys_enum_name(model_name);
        impls.push(quote! {
            impl netabase_store::traits::registery::models::StoreKeyMarker<#definition_name> for #secondary_enum {}
        });

        // Relational keys
        let relational_enum = relational_keys_enum_name(model_name);
        impls.push(quote! {
            impl netabase_store::traits::registery::models::StoreKeyMarker<#definition_name> for #relational_enum {}
        });

        // Subscriptions
        let subscription_enum = subscriptions_enum_name(model_name);
        impls.push(quote! {
            impl netabase_store::traits::registery::models::StoreKeyMarker<#definition_name> for #subscription_enum {}
        });

        // Blob keys
        // For blob keys, if empty, we still generate enums?
        // KeyEnumGenerator generates blob enums ONLY if !blob_fields.is_empty().
        // I changed generate_unified_keys_enum to remove check, but generate_blob_keys_enum logic?
        // Wait, I only modified generate_unified_keys_enum and generate in key_enums.rs.
        // generate_blob_keys_enum in key_enums.rs loops over blob_fields. If empty, it generates empty enum.
        // So they ARE generated.
        let blob_keys = blob_keys_enum_name(model_name);
        let blob_item = blob_item_enum_name(model_name);
        impls.push(quote! {
            impl netabase_store::traits::registery::models::StoreKeyMarker<#definition_name> for #blob_keys {}
            impl netabase_store::traits::registery::models::StoreKeyMarker<#definition_name> for #blob_item {}
        });

        quote! { #(#impls)* }
    }

    fn generate_store_traits(
        &self,
        definition_name: &syn::Ident,
        model_name: &syn::Ident,
        visitor: &crate::visitors::model::field::ModelFieldVisitor,
    ) -> TokenStream {
        let id_type = primary_key_type_name(model_name);

        let mut impls = vec![];

        // StoreKey<Definition, Model> for ID
        // StoreValue<Definition, ID> for Model
        impls.push(quote! {
            impl netabase_store::traits::registery::models::StoreKey<#definition_name, #model_name> for #id_type {}
            impl netabase_store::traits::registery::models::StoreValue<#definition_name, #id_type> for #model_name {}
        });

        // Secondary keys
        let secondary_enum = secondary_keys_enum_name(model_name);
        impls.push(quote! {
            impl netabase_store::traits::registery::models::StoreKey<#definition_name, #id_type> for #secondary_enum {}
            impl netabase_store::traits::registery::models::StoreValue<#definition_name, #secondary_enum> for #id_type {}
        });

        // Relational keys
        let relational_enum = relational_keys_enum_name(model_name);
        impls.push(quote! {
            impl netabase_store::traits::registery::models::StoreKey<#definition_name, #id_type> for #relational_enum {}
            impl netabase_store::traits::registery::models::StoreValue<#definition_name, #relational_enum> for #id_type {}
        });

        // Subscriptions
        let subscription_enum = subscriptions_enum_name(model_name);
        impls.push(quote! {
            impl netabase_store::traits::registery::models::StoreKey<#definition_name, #id_type> for #subscription_enum {}
            impl netabase_store::traits::registery::models::StoreValue<#definition_name, #subscription_enum> for #id_type {}
        });

        quote! { #(#impls)* }
    }

    fn generate_key_type_traits(
        &self,
        definition_name: &syn::Ident,
        model_name: &syn::Ident,
        visitor: &crate::visitors::model::field::ModelFieldVisitor,
    ) -> TokenStream {
        let id_type = primary_key_type_name(model_name);
        let keys_enum = unified_keys_enum_name(model_name);

        let mut impls = vec![];

        // NetabaseModelPrimaryKey
        impls.push(quote! {
            impl<'a> netabase_store::traits::registery::models::keys::NetabaseModelPrimaryKey<'a, #definition_name, #model_name, #keys_enum> for #id_type {}
        });

        // NetabaseModelSecondaryKey
        let secondary_enum = secondary_keys_enum_name(model_name);
        impls.push(quote! {
            impl<'a> netabase_store::traits::registery::models::keys::NetabaseModelSecondaryKey<'a, #definition_name, #model_name, #keys_enum> for #secondary_enum {
                type PrimaryKey = #id_type;
            }
        });

        // NetabaseModelRelationalKey
        let relational_enum = relational_keys_enum_name(model_name);
        impls.push(quote! {
            impl<'a> netabase_store::traits::registery::models::keys::NetabaseModelRelationalKey<'a, #definition_name, #model_name, #keys_enum> for #relational_enum {}
        });

        // NetabaseModelBlobKey
        let blob_keys = blob_keys_enum_name(model_name);
        let blob_item = blob_item_enum_name(model_name);
        impls.push(quote! {
            impl<'a> netabase_store::traits::registery::models::keys::blob::NetabaseModelBlobKey<'a, #definition_name, #model_name, #keys_enum> for #blob_keys {
                type PrimaryKey = #id_type;
                type BlobItem = #blob_item;
            }
        });

        // NetabaseModelSubscriptionKey
        let subscription_enum = subscriptions_enum_name(model_name);
        impls.push(quote! {
            impl netabase_store::traits::registery::models::keys::NetabaseModelSubscriptionKey<#definition_name, #model_name, #keys_enum> for #subscription_enum {}
        });

        quote! { #(#impls)* }
    }

    fn generate_redb_netabase_model_trait(
        &self,
        definition_name: &syn::Ident,
        model_name: &syn::Ident,
    ) -> TokenStream {
        quote! {
            impl<'db> ::netabase_store::traits::registery::models::model::RedbNetbaseModel<'db, #definition_name> for #model_name {
                type RedbTables = ::netabase_store::databases::redb::transaction::ModelOpenTables<'db, 'db, #definition_name, Self>;
                type TableV = #model_name;
            }
        }
    }

    fn generate_subscription_traits(
        &self,
        definition_name: &syn::Ident,
        model_name: &syn::Ident,
        visitor: &crate::visitors::model::field::ModelFieldVisitor,
    ) -> TokenStream {
        // If no subscriptions, treat as empty
        let empty_topics = Vec::new();
        let topics = visitor.subscriptions.as_ref().map(|s| &s.topics).unwrap_or(&empty_topics);

        let subscription_enum = subscriptions_enum_name(model_name);
        let def_subscription_enum = definition_subscriptions_enum_name(definition_name);

        // Generate From impl
        let from_arms: Vec<_> = topics
            .iter()
            .map(|topic| {
                let topic_ident = path_last_segment(topic).unwrap();
                quote! {
                    #def_subscription_enum::#topic_ident => #subscription_enum::#topic_ident(value)
                }
            })
            .collect();

        // Generate TryInto impl
        let try_into_arms: Vec<_> = topics
            .iter()
            .map(|topic| {
                let topic_ident = path_last_segment(topic).unwrap();
                quote! {
                    #subscription_enum::#topic_ident(v) => Ok(v)
                }
            })
            .collect();

        quote! {
            impl From<#def_subscription_enum> for #subscription_enum {
                fn from(value: #def_subscription_enum) -> Self {
                    match value {
                        #(#from_arms,)*
                        _ => panic!("Unsupported subscription topic for {} model", stringify!(#model_name)),
                    }
                }
            }

            impl TryInto<#def_subscription_enum> for #subscription_enum {
                type Error = ();

                fn try_into(self) -> Result<#def_subscription_enum, Self::Error> {
                    match self {
                        #(#try_into_arms,)*
                    }
                }
            }
        }
    }

    fn generate_netabase_definition_trait(&self) -> TokenStream {
        let definition_name = &self.visitor.definition_name;
        let tree_names_enum = definition_tree_names_enum_name(definition_name); // Complex enum
        let def_keys_enum = definition_keys_enum_name(definition_name);
        let subscription_enum = definition_subscriptions_enum_name(definition_name);
        let discriminant_enum = definition_tree_name_type(definition_name); // Simple discriminant enum (e.g. DefinitionTreeName)

        // Debug name
        let debug_name_str = definition_name.to_string();

        // Subscription Discriminant
        let subscription_discriminant_type = if self.visitor.subscriptions.topics.is_empty() {
            quote! { () }
        } else {
            let disc_name = Ident::new(
                &format!("{}Discriminants", subscription_enum),
                subscription_enum.span()
            );
            quote! { #disc_name }
        };

        // Subscription Registry
        let registry_entries: Vec<_> = self.visitor.subscriptions.topics
            .iter()
            .map(|topic| {
                let topic_ident = path_last_segment(topic).expect("Invalid topic path");
                let topic_str = topic_ident.to_string();

                // Find all models that subscribe to this topic
                let subscribers: Vec<_> = self.visitor.models
                    .iter()
                    .filter(|m| {
                        if let Some(subs) = &m.visitor.subscriptions {
                            subs.topics.iter().any(|t| path_last_segment(t).map_or(false, |i| i == topic_ident))
                        } else {
                            false
                        }
                    })
                    .map(|m| {
                        let model_name = &m.name;
                        // Use the discriminant enum for subscribers
                        quote! { #discriminant_enum::#model_name }
                    })
                    .collect();

                quote! {
                    netabase_store::traits::registery::definition::subscription::SubscriptionEntry {
                        topic: #topic_str,
                        subscribers: &[#(#subscribers),*],
                    }
                }
            })
            .collect();

        // Schema generation
        let schema_impl = self.generate_schema_impl();

        quote! {
            impl netabase_store::traits::registery::definition::NetabaseDefinition for #definition_name {
                type TreeNames = #tree_names_enum;
                type DefKeys = #def_keys_enum;
                type DebugName = &'static str;

                fn debug_name() -> Self::DebugName {
                    #debug_name_str
                }

                fn schema() -> netabase_store::traits::registery::definition::schema::DefinitionSchema {
                    #schema_impl
                }

                type SubscriptionKeys = #subscription_enum;
                type SubscriptionKeysDiscriminant = #subscription_discriminant_type;

                const SUBSCRIPTION_REGISTRY: netabase_store::traits::registery::definition::subscription::DefinitionSubscriptionRegistry<'static, Self> =
                    netabase_store::traits::registery::definition::subscription::DefinitionSubscriptionRegistry::new(&[
                        #(#registry_entries),*
                    ]);
            }
        }
    }

    fn generate_schema_impl(&self) -> TokenStream {
        let def_name_str = self.visitor.definition_name.to_string();
        
        let sub_strs: Vec<_> = self.visitor.subscriptions.topics.iter()
            .map(|t| {
                let s = path_last_segment(t).unwrap().to_string();
                quote! { #s.to_string() }
            })
            .collect();

        let model_schemas: Vec<_> = self.visitor.models.iter().map(|model_info| {
            let model_name_str = model_info.name.to_string();
            let visitor = &model_info.visitor;

            let mut field_schemas = Vec::new();

            // Helper to add field
            let mut add_field = |info: &crate::visitors::model::field::FieldInfo, key_type_expr: TokenStream| {
                let f_name = info.name.to_string();
                let ty = &info.ty;
                let type_name = quote! { #ty }.to_string();
                field_schemas.push(quote! {
                    netabase_store::traits::registery::definition::schema::FieldSchema {
                        name: #f_name.to_string(),
                        type_name: #type_name.to_string(),
                        key_type: #key_type_expr,
                    }
                });
            };

            // Primary
            if let Some(pk) = &visitor.primary_key {
                add_field(pk, quote! { netabase_store::traits::registery::definition::schema::KeyTypeSchema::Primary });
            }

            // Secondary
            for sk in &visitor.secondary_keys {
                add_field(sk, quote! { netabase_store::traits::registery::definition::schema::KeyTypeSchema::Secondary });
            }

            // Relational
            for rk in &visitor.relational_keys {
                match &rk.key_type {
                    crate::visitors::model::field::FieldKeyType::Relational { definition, model } => {
                         let def_s = path_last_segment(definition).unwrap().to_string();
                         let mod_s = path_last_segment(model).unwrap().to_string();
                         add_field(rk, quote! {
                             netabase_store::traits::registery::definition::schema::KeyTypeSchema::Relational {
                                 definition: #def_s.to_string(),
                                 model: #mod_s.to_string(),
                             }
                         });
                    },
                    _ => panic!("Expected Relational key type"),
                }
            }

            // Blob
            for bk in &visitor.blob_fields {
                 add_field(bk, quote! { netabase_store::traits::registery::definition::schema::KeyTypeSchema::Blob });
            }

            // Regular
            for rk in &visitor.regular_fields {
                 add_field(rk, quote! { netabase_store::traits::registery::definition::schema::KeyTypeSchema::Regular });
            }

            let model_subs: Vec<_> = visitor.subscriptions.as_ref().map(|s| &s.topics).unwrap_or(&Vec::new())
                .iter()
                .map(|t| {
                    let s = path_last_segment(t).unwrap().to_string();
                    quote! { #s.to_string() }
                })
                .collect();

            quote! {
                netabase_store::traits::registery::definition::schema::ModelSchema {
                    name: #model_name_str.to_string(),
                    fields: vec![
                        #(#field_schemas),*
                    ],
                    subscriptions: vec![
                        #(#model_subs),*
                    ],
                }
            }
        }).collect();

        quote! {
            netabase_store::traits::registery::definition::schema::DefinitionSchema {
                name: #def_name_str.to_string(),
                models: vec![
                    #(#model_schemas),*
                ],
                subscriptions: vec![
                    #(#sub_strs),*
                ],
            }
        }
    }
}
