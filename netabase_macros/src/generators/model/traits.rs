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

        quote! {
            impl netabase_store::traits::registery::models::keys::NetabaseModelKeys<#definition_name, #model_name> for #keys_enum {
                type Primary = #id_type;
                type Secondary = #secondary_type;
                type Relational = #relational_type;
                type Subscription = #subscription_type;
                type Blob = #blob_type;
            }
        }
    }

    /// Generate NetabaseModel trait implementation with TREE_NAMES
    pub fn generate_netabase_model_trait(&self, definition_name: &syn::Ident) -> TokenStream {
        let model_name = &self.visitor.model_name;
        let keys_enum = unified_keys_enum_name(model_name);
        let id_type = primary_key_type_name_for_model(self.visitor);

        // Generate TREE_NAMES
        let tree_names = self.generate_tree_names(definition_name);

        // Generate get_primary_key method
        let pk_field = self.visitor.primary_key.as_ref().unwrap();
        let pk_field_name = &pk_field.name;
        let get_primary_key = quote! {
            fn get_primary_key<'b>(&'b self) -> #id_type {
                self.#pk_field_name.clone()
            }
        };

        // Generate get_secondary_keys method
        let get_secondary_keys = self.generate_get_secondary_keys();

        // Generate get_relational_keys method
        let get_relational_keys = self.generate_get_relational_keys();

        // Generate get_subscription_keys method
        let get_subscription_keys = self.generate_get_subscription_keys();

        // Generate get_blob_entries method
        let get_blob_entries = self.generate_get_blob_entries(definition_name);

        quote! {
            impl netabase_store::traits::registery::models::model::NetabaseModel<#definition_name> for #model_name {
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
                };
        }
    }

    fn generate_get_secondary_keys(&self) -> TokenStream {
        let model_name = &self.visitor.model_name;
        let enum_name = secondary_keys_enum_name(model_name);

        let key_constructions: Vec<_> = self
            .visitor
            .secondary_keys
            .iter()
            .map(|field| {
                let field_name = &field.name;
                let variant_name = to_pascal_case(&field.name.to_string());
                let variant_ident = syn::Ident::new(&variant_name, field.name.span());
                let wrapper_type = field_wrapper_name(model_name, field_name);

                quote! {
                    #enum_name::#variant_ident(#wrapper_type(self.#field_name.clone()))
                }
            })
            .collect();

        quote! {
            fn get_secondary_keys<'b>(&'b self) -> Vec<#enum_name> {
                vec![#(#key_constructions),*]
            }
        }
    }

    fn generate_get_relational_keys(&self) -> TokenStream {
        let model_name = &self.visitor.model_name;
        let enum_name = relational_keys_enum_name(model_name);

        let key_constructions: Vec<_> = self.visitor.relational_keys
            .iter()
            .map(|field| {
                let field_name = &field.name;
                let variant_name = to_pascal_case(&field.name.to_string());
                let variant_ident = syn::Ident::new(&variant_name, field.name.span());
                let wrapper_type = field_wrapper_name(model_name, field_name);

                quote! {
                    #enum_name::#variant_ident(#wrapper_type(self.#field_name.get_primary_key().clone()))
                }
            })
            .collect();

        quote! {
            fn get_relational_keys<'b>(&'b self) -> Vec<#enum_name> {
                vec![#(#key_constructions),*]
            }
        }
    }

    fn generate_get_subscription_keys(&self) -> TokenStream {
        let model_name = &self.visitor.model_name;
        let enum_name = subscriptions_enum_name(model_name);

        if self.visitor.subscriptions.is_none() {
            return quote! {
                fn get_subscription_keys<'b>(&'b self) -> Vec<#enum_name> {
                    vec![]
                }
            };
        }

        // For now, we return empty subscriptions - the actual subscription logic
        // would need to be implemented based on the model's state
        quote! {
            fn get_subscription_keys<'b>(&'b self) -> Vec<#enum_name> {
                vec![]
            }
        }
    }

    fn generate_get_blob_entries(&self, _definition_name: &syn::Ident) -> TokenStream {
        let model_name = &self.visitor.model_name;
        let blob_keys_enum = blob_keys_enum_name(model_name);
        let blob_item_enum = blob_item_enum_name(model_name);

        let blob_entries: Vec<_> = self
            .visitor
            .blob_fields
            .iter()
            .map(|field| {
                let field_name = &field.name;
                let variant_name = to_pascal_case(&field.name.to_string());
                let variant_ident = syn::Ident::new(&variant_name, field.name.span());

                quote! {
                    {
                        let mut entries = Vec::new();
                        for blob in self.#field_name.split_into_blobs() {
                            entries.push((
                                #blob_keys_enum::#variant_ident { owner: self.get_primary_key() },
                                blob
                            ));
                        }
                        entries
                    }
                }
            })
            .collect();

        quote! {
            fn get_blob_entries<'a>(&'a self) -> Vec<Vec<(#blob_keys_enum, #blob_item_enum)>> {
                vec![#(#blob_entries),*]
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
