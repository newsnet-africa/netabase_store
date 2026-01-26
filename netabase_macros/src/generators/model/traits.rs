use crate::utils::naming::*;
use crate::visitors::model::field::ModelFieldVisitor;
use proc_macro2::TokenStream;
use quote::quote;

/// Generator for trait implementations
/// Note: This generates model-level traits only. Definition-dependent traits are generated
/// by the definition-level macro to avoid circular dependencies.
pub struct TraitGenerator<'a> {
    visitor: &'a ModelFieldVisitor,
}

impl<'a> TraitGenerator<'a> {
    pub fn new(visitor: &'a ModelFieldVisitor) -> Self {
        Self { visitor }
    }

    /// Generate NetabaseModelKeys trait implementation
    pub fn generate_model_keys_trait(&self, definition_name: &syn::Ident) -> TokenStream {
        let model_name = &self.visitor.model_name;
        let keys_enum = unified_keys_enum_name(model_name);
        let id_type = primary_key_type_name_for_model(self.visitor);

        let secondary_type = secondary_keys_enum_name(model_name);
        let relational_type = relational_keys_enum_name(model_name);
        let subscription_type = subscriptions_enum_name(model_name);
        let blob_type = blob_keys_enum_name(model_name);
        let libp2p_type = libp2p_provider_key_enum_name(model_name);

        let target_type = if self.visitor.content_addressed_config.is_some() {
            quote::format_ident!("{}Envelope", model_name)
        } else {
            model_name.clone()
        };

        quote! {
            impl netabase_store::traits::registery::models::keys::NetabaseModelKeys<#definition_name, #target_type> for #keys_enum {
                type Primary = #id_type;
                type Secondary = #secondary_type;
                type Relational = #relational_type;
                type Subscription = #subscription_type;
                type Blob = #blob_type;
                type Libp2p = #libp2p_type;
            }
        }
    }

    /// Generate NetabaseModel trait implementation with TREE_NAMES
    pub fn generate_netabase_model_trait(&self, definition_name: &syn::Ident) -> TokenStream {
        let model_name = &self.visitor.model_name;
        let keys_enum = unified_keys_enum_name(model_name);
        let id_type = primary_key_type_name_for_model(self.visitor);

        // Determine if we are implementing for Envelope or Model
        let is_content_addressed = self.visitor.content_addressed_config.is_some();
        let target_type = if is_content_addressed {
            quote::format_ident!("{}Envelope", model_name)
        } else {
            model_name.clone()
        };

        // Generate TREE_NAMES
        let tree_names = self.generate_tree_names(definition_name);

        // Generate get_primary_key method
        let get_primary_key = if is_content_addressed {
            quote! {
                #[inline]
                fn get_primary_key<'b>(&'b self) -> #id_type {
                    self.hash.clone()
                }

                #[inline]
                fn get_primary_key_ref<'b>(&'b self) -> &#id_type {
                    &self.hash
                }
            }
        } else {
            let pk_field = self.visitor.primary_key.as_ref().unwrap();
            let pk_field_name = &pk_field.name;
            quote! {
                #[inline]
                fn get_primary_key<'b>(&'b self) -> #id_type {
                    self.#pk_field_name.clone()
                }

                #[inline]
                fn get_primary_key_ref<'b>(&'b self) -> &#id_type {
                    &self.#pk_field_name
                }
            }
        };

        // Generate get_secondary_keys method
        let get_secondary_keys = self.generate_get_secondary_keys();

        // Generate get_relational_keys method
        let get_relational_keys = self.generate_get_relational_keys();

        // Generate get_subscription_keys method
        let get_subscription_keys = self.generate_get_subscription_keys(definition_name);

        // Generate get_blob_entries method
        let get_blob_entries = self.generate_get_blob_entries(definition_name);

        quote! {
            impl netabase_store::traits::registery::models::model::NetabaseModel<#definition_name> for #target_type {
                type Keys = #keys_enum;

                #tree_names

                #get_primary_key
                #get_secondary_keys
                #get_relational_keys
                #get_subscription_keys
                #get_blob_entries
            }
        }
    }

    fn generate_tree_names(&self, definition_name: &syn::Ident) -> TokenStream {
        let model_name = &self.visitor.model_name;
        let def_str = definition_name.to_string();
        let model_str = model_name.to_string();

        // Main table
        let main_table_name = table_name(&def_str, &model_str, "Primary", "Main");
        let definition_tree_name = definition_tree_name_type(definition_name);

        // Secondary tables
        let secondary_tables: Vec<_> = self.visitor.secondary_keys
            .iter()
            .map(|field| {
                let field_str = to_pascal_case(&field.name.to_string());
                let field_ident = syn::Ident::new(&field_str, field.name.span());
                let table_name_str = table_name(&def_str, &model_str, "Secondary", &field_str);
                let tree_name = tree_name_type(&secondary_keys_enum_name(model_name));

                quote! {
                    netabase_store::traits::registery::models::treenames::DiscriminantTableName::new(
                        #tree_name::#field_ident,
                        #table_name_str
                    )
                }
            })
            .collect();

        let secondary_array = if secondary_tables.is_empty() {
            quote! { &[] }
        } else {
            quote! { &[#(#secondary_tables),*] }
        };

        // Relational tables
        let relational_tables: Vec<_> = self.visitor.relational_keys
            .iter()
            .map(|field| {
                let field_str = to_pascal_case(&field.name.to_string());
                let field_ident = syn::Ident::new(&field_str, field.name.span());
                let table_name_str = table_name(&def_str, &model_str, "Relational", &field_str);
                let tree_name = tree_name_type(&relational_keys_enum_name(model_name));

                quote! {
                    netabase_store::traits::registery::models::treenames::DiscriminantTableName::new(
                        #tree_name::#field_ident,
                        #table_name_str
                    )
                }
            })
            .collect();

        let relational_array = if relational_tables.is_empty() {
            quote! { &[] }
        } else {
            quote! { &[#(#relational_tables),*] }
        };

        // Subscription tables
        let subscription_array = if let Some(ref subs) = self.visitor.subscriptions {
            let sub_tables: Vec<_> = subs.topics
                .iter()
                .map(|topic| {
                    let topic_ident = path_last_segment(topic).unwrap();
                    let topic_str = topic_ident.to_string();
                    let table_name_str = subscription_table_name(&def_str, &model_str, &topic_str);
                    let tree_name = tree_name_type(&subscriptions_enum_name(model_name));

                    quote! {
                        netabase_store::traits::registery::models::treenames::DiscriminantTableName::new(
                            #tree_name::#topic_ident,
                            #table_name_str
                        )
                    }
                })
                .collect();

            quote! { Some(&[#(#sub_tables),*]) }
        } else {
            quote! { None }
        };

        // Blob tables
        let blob_tables: Vec<_> = self.visitor.blob_fields
            .iter()
            .map(|field| {
                let field_str = to_pascal_case(&field.name.to_string());
                let field_ident = syn::Ident::new(&field_str, field.name.span());
                let table_name_str = table_name(&def_str, &model_str, "Blob", &field_str);
                let tree_name = tree_name_type(&blob_keys_enum_name(model_name));

                quote! {
                    netabase_store::traits::registery::models::treenames::DiscriminantTableName::new(
                        #tree_name::#field_ident,
                        #table_name_str
                    )
                }
            })
            .collect();

        let blob_array = if blob_tables.is_empty() {
            quote! { &[] }
        } else {
            quote! { &[#(#blob_tables),*] }
        };

        // Providers
        let libp2p_enum_name = libp2p_provider_key_enum_name(model_name);
        let libp2p_tree_name = tree_name_type(&libp2p_enum_name);
        let libp2p_table_name_str = table_name(&def_str, &model_str, "Libp2p", "Provider");

        // We map all variants to the same table for now
        let providers_array = quote! {
            &[
                netabase_store::traits::registery::models::treenames::DiscriminantTableName::new(
                    #libp2p_tree_name::Full,
                    #libp2p_table_name_str
                ),
                netabase_store::traits::registery::models::treenames::DiscriminantTableName::new(
                    #libp2p_tree_name::Bare,
                    #libp2p_table_name_str
                ),
                netabase_store::traits::registery::models::treenames::DiscriminantTableName::new(
                    #libp2p_tree_name::WithBlobs,
                    #libp2p_table_name_str
                ),
                netabase_store::traits::registery::models::treenames::DiscriminantTableName::new(
                    #libp2p_tree_name::WithRelations,
                    #libp2p_table_name_str
                ),
            ]
        };

        quote! {
            const TREE_NAMES: netabase_store::traits::registery::models::treenames::ModelTreeNames<'static, #definition_name, Self> =
                netabase_store::traits::registery::models::treenames::ModelTreeNames {
                    main: netabase_store::traits::registery::models::treenames::DiscriminantTableName::new(
                        #definition_tree_name::#model_name,
                        #main_table_name
                    ),
                    secondary: #secondary_array,
                    relational: #relational_array,
                    subscription: #subscription_array,
                    blob: #blob_array,
                    providers: #providers_array,
                };
        }
    }

    fn generate_get_secondary_keys(&self) -> TokenStream {
        let model_name = &self.visitor.model_name;
        let enum_name = secondary_keys_enum_name(model_name);
        let is_content_addressed = self.visitor.content_addressed_config.is_some();

        let key_constructions: Vec<_> = self
            .visitor
            .secondary_keys
            .iter()
            .map(|field| {
                let field_name = &field.name;
                let variant_name = to_pascal_case(&field.name.to_string());
                let variant_ident = syn::Ident::new(&variant_name, field.name.span());
                let wrapper_type = field_wrapper_name(model_name, field_name);

                if is_content_addressed {
                    quote! {
                        #enum_name::#variant_ident(#wrapper_type(self.inner.#field_name.clone()))
                    }
                } else {
                    quote! {
                        #enum_name::#variant_ident(#wrapper_type(self.#field_name.clone()))
                    }
                }
            })
            .collect();

        quote! {
            #[inline]
            fn get_secondary_keys<'b>(&'b self) -> Vec<#enum_name> {
                vec![#(#key_constructions),*]
            }
        }
    }

    fn generate_get_relational_keys(&self) -> TokenStream {
        let model_name = &self.visitor.model_name;
        let enum_name = relational_keys_enum_name(model_name);
        let is_content_addressed = self.visitor.content_addressed_config.is_some();

        let key_constructions: Vec<_> = self.visitor.relational_keys
            .iter()
            .map(|field| {
                let field_name = &field.name;
                let variant_name = to_pascal_case(&field.name.to_string());
                let variant_ident = syn::Ident::new(&variant_name, field.name.span());
                let wrapper_type = field_wrapper_name(model_name, field_name);

                if is_content_addressed {
                    quote! {
                        #enum_name::#variant_ident(#wrapper_type(self.inner.#field_name.get_primary_key().clone()))
                    }
                } else {
                    quote! {
                        #enum_name::#variant_ident(#wrapper_type(self.#field_name.get_primary_key().clone()))
                    }
                }
            })
            .collect();

        quote! {
            #[inline]
            fn get_relational_keys<'b>(&'b self) -> Vec<#enum_name> {
                vec![#(#key_constructions),*]
            }
        }
    }

    fn generate_get_subscription_keys(&self, definition_name: &syn::Ident) -> TokenStream {
        let model_name = &self.visitor.model_name;
        let enum_name = subscriptions_enum_name(model_name);
        let def_subs_enum = definition_subscriptions_enum_name(definition_name);

        // If no subscriptions declared on the model, return empty
        let Some(subscription_info) = &self.visitor.subscriptions else {
            return quote! {
                #[inline]
                fn get_subscription_keys<'b>(&'b self) -> Vec<#enum_name> {
                    vec![]
                }
            };
        };

        // Return the static subscription topics declared on the model type
        // via #[subscribe(Topic1, Topic2, ...)] attribute
        // The topics are just identifiers, we need to fully qualify them
        // as DefinitionSubscriptions::TopicIdent and wrap in ModelSubscriptions
        let topic_constructions: Vec<_> = subscription_info
            .topics
            .iter()
            .map(|topic_path| {
                // Extract the identifier from the path
                let topic_ident = &topic_path.segments.last().expect("Empty topic path").ident;

                // Generate: UserSubscriptions::Topic1(DefinitionSubscriptions::Topic1)
                quote! {
                    #enum_name::#topic_ident(#def_subs_enum::#topic_ident)
                }
            })
            .collect();

        quote! {
            #[inline]
            fn get_subscription_keys<'b>(&'b self) -> Vec<#enum_name> {
                vec![
                    #( #topic_constructions ),*
                ]
            }
        }
    }

    fn generate_get_blob_entries(&self, _definition_name: &syn::Ident) -> TokenStream {
        let model_name = &self.visitor.model_name;
        let blob_keys_enum = blob_keys_enum_name(model_name);
        let blob_item_enum = blob_item_enum_name(model_name);
        let is_content_addressed = self.visitor.content_addressed_config.is_some();

        let blob_entries: Vec<_> = self
            .visitor
            .blob_fields
            .iter()
            .map(|field| {
                let field_name = &field.name;
                let variant_name = to_pascal_case(&field.name.to_string());
                let variant_ident = syn::Ident::new(&variant_name, field.name.span());

                if is_content_addressed {
                    quote! {
                        {
                            let mut entries = Vec::new();
                            for blob in self.inner.#field_name.split_into_blobs() {
                                entries.push((
                                    #blob_keys_enum::#variant_ident { owner: self.get_primary_key() },
                                    #blob_item_enum::#variant_ident(blob)
                                ));
                            }
                            entries
                        }
                    }
                } else {
                    quote! {
                        {
                            let mut entries = Vec::new();
                            for blob in self.#field_name.split_into_blobs() {
                                entries.push((
                                    #blob_keys_enum::#variant_ident { owner: self.get_primary_key() },
                                    #blob_item_enum::#variant_ident(blob)
                                ));
                            }
                            entries
                        }
                    }
                }
            })
            .collect();

        quote! {
            #[inline]
            fn get_blob_entries<'a>(&'a self) -> Vec<Vec<(#blob_keys_enum, #blob_item_enum)>> {
                vec![#(#blob_entries),*]
            }
        }
    }

    /// Generate ContentAddressedModel trait implementation
    pub fn generate_content_addressed_model_trait(
        &self,
        _definition_name: &syn::Ident,
    ) -> TokenStream {
        let model_name = &self.visitor.model_name;

        if let Some(config) = &self.visitor.content_addressed_config {
            let hasher = &config.hasher;
            let function = &config.function;

            // The ID type (wrapper) IS the key type
            let id_type = primary_key_type_name_for_model(self.visitor);

            quote! {
                impl ::netabase_store::traits::registery::models::content_addressed::ContentAddressedModel for #model_name {
                    type Hasher = #hasher;
                    type Key = #id_type;

                    fn compute_hash(&self) -> Self::Key {
                        #id_type(#function(self))
                    }
                }
            }
        } else {
            TokenStream::new()
        }
    }

    /// Generate Libp2pModel trait implementation
    pub fn generate_libp2p_model_trait(&self) -> TokenStream {
        let model_name = &self.visitor.model_name;
        let is_content_addressed = self.visitor.content_addressed_config.is_some();

        let target_type = if is_content_addressed {
            quote::format_ident!("{}Envelope", model_name)
        } else {
            model_name.clone()
        };

        let body = if self.visitor.is_libp2p_enabled {
            if is_content_addressed {
                quote! { self.inner.libp2p_metadata.as_ref() }
            } else {
                quote! { self.libp2p_metadata.as_ref() }
            }
        } else {
            quote! { None }
        };

        quote! {
            impl netabase_store::traits::libp2p::libp2p_model::Libp2pModel for #target_type {
                #[inline]
                fn get_libp2p_metadata(&self) -> Option<&netabase_store::traits::libp2p::libp2p_model::Libp2pMetadata> {
                    #body
                }
            }
        }
    }
}

/// Helper function to convert snake_case to PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}
