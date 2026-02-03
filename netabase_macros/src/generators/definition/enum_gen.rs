use crate::utils::naming::*;
use crate::visitors::definition::DefinitionVisitor;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

// TODO: Add support for more complex relational links: Vec<RelationalLink>, Option<RelationalLink> etc.

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
            if model.is_content_addressed() {
                let envelope_name = format_ident!("{}Envelope", model_name);
                variants.push(quote! { #model_name(#envelope_name) });
            } else {
                variants.push(quote! { #model_name(#model_name) });
            }
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
                serde::Serialize, serde::Deserialize,
                PartialEq, Eq, PartialOrd, Ord, Hash,
                derive_more::From, derive_more::TryInto,
                strum::EnumDiscriminants
            )]
            #[strum_discriminants(name(#tree_name))]
            #[strum_discriminants(derive(
                strum::AsRefStr,
                serde::Serialize, serde::Deserialize,
                Hash
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
        let discriminant_name =
            Ident::new(&format!("{}Discriminants", enum_name), enum_name.span());

        if self.visitor.subscriptions.topics.is_empty() {
            // Generate an empty enum with necessary trait implementations
            // When empty, NetabaseDefinition trait uses () as SubscriptionKeysDiscriminant
            return quote! {
                #[derive(
                    Clone, Eq, PartialEq, PartialOrd, Ord, Debug,
                    serde::Serialize, serde::Deserialize,
                    Hash
                )]
                pub enum #enum_name {}

                // Implement IntoDiscriminant for empty enum - discriminant is ()
                impl strum::IntoDiscriminant for #enum_name {
                    type Discriminant = ();

                    fn discriminant(&self) -> Self::Discriminant {
                        match *self {}
                    }
                }

                // Implement redb::Value for empty enum
                impl redb::Value for #enum_name {
                    type SelfType<'a> = Self;
                    type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

                    fn from_bytes<'a>(_data: &'a [u8]) -> Self::SelfType<'a>
                    where
                        Self: 'a,
                    {
                        panic!("Cannot deserialize empty subscription enum")
                    }

                    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
                    where
                        Self: 'a,
                    {
                        match *value {}
                    }

                    fn fixed_width() -> Option<usize> {
                        Some(0)
                    }

                    fn type_name() -> redb::TypeName {
                        redb::TypeName::new(&format!("{}::{}", module_path!(), stringify!(#enum_name)))
                    }
                }

                // Implement redb::Key for empty enum
                impl redb::Key for #enum_name {
                    fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
                        data1.cmp(data2)
                    }
                }
            };
        }

        let variants: Vec<_> = self
            .visitor
            .subscriptions
            .topics
            .iter()
            .map(|topic| {
                let topic_ident = path_last_segment(topic).expect("Invalid subscription topic");

                quote! { #topic_ident }
            })
            .collect();

        // Manual generation of Discriminant Enum to avoid conflicts
        quote! {
            #[derive(
                Clone, Eq, PartialEq, PartialOrd, Ord, Debug,
                serde::Serialize, serde::Deserialize,
                Hash
            )]
            pub enum #enum_name {
                #(#variants),*
            }

            #[derive(
                Clone, Copy, Debug, PartialEq, Eq, Hash,
                serde::Serialize, serde::Deserialize,
                strum::AsRefStr
            )]
            pub enum #discriminant_name {
                #(#variants),*
            }

            impl strum::IntoDiscriminant for #enum_name {
                type Discriminant = #discriminant_name;

                fn discriminant(&self) -> Self::Discriminant {
                    match self {
                        #(
                            #enum_name::#variants => #discriminant_name::#variants
                        ),*
                    }
                }
            }

            // Generate helper to implement Value/Key for owned types
            impl redb::Value for #enum_name {
                type SelfType<'a> = Self;
                type AsBytes<'a> = std::borrow::Cow<'a, [u8]>;

                fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
                where
                    Self: 'a,
                {
                    postcard::from_bytes(data).unwrap()
                }

                fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
                where
                    Self: 'a,
                {
                    std::borrow::Cow::Owned(
                        postcard::to_allocvec(value).unwrap()
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

    pub fn generate_iter(&self) -> TokenStream {
        let def_name = &self.visitor.definition_name;
        let tables_name = format_ident!("{}ReadOnlyTables", def_name);
        let iter_name = format_ident!("{}Iter", def_name);

        let mut table_field_defs = Vec::new();
        let mut table_inits = Vec::new();
        let mut table_field_names = Vec::new();

        let mut iter_field_defs = Vec::new();
        let mut iter_inits = Vec::new();
        let mut next_arms = Vec::new();

        for (idx, model) in self.visitor.models.iter().enumerate() {
            let model_name = &model.name;
            let pk_type = primary_key_type_name_for_model(&model.visitor);

            let table_value_type = if model.is_content_addressed() {
                format_ident!("{}Envelope", model_name)
            } else {
                model_name.clone()
            };

            // Field names
            let table_field_ident = format_ident!("table_{}", model_name);
            let iter_field_ident = format_ident!("iter_{}", model_name);

            // Table Field Definition
            table_field_defs.push(quote! {
                pub #table_field_ident: redb::ReadOnlyTable<#pk_type, #table_value_type>
            });
            table_field_names.push(table_field_ident.clone());

            // Table Init Logic
            let def_str = def_name.to_string();
            let model_str = model_name.to_string();
            let table_name_str = table_name(&def_str, &model_str, "Primary", "Main");

            table_inits.push(quote! {
                  let #table_field_ident = txn.open_table(redb::TableDefinition::new(#table_name_str))?;
             });

            // Iter Field Definition
            iter_field_defs.push(quote! {
                pub #iter_field_ident: Option<redb::Range<'a, #pk_type, #table_value_type>>
            });

            // Iter Init Logic
            iter_inits.push(quote! {
                #iter_field_ident: Some(self.#table_field_ident.range::<#pk_type>(..)?)
            });

            // Next Arm
            next_arms.push(quote! {
                 #idx => {
                     if let Some(range) = &mut self.#iter_field_ident {
                         match range.next() {
                             Some(Ok((_k, v))) => return Some(Ok(#def_name::#model_name(v.value()))),
                             Some(Err(e)) => return Some(Err(netabase_store::errors::NetabaseError::RedbStorageError(e))),
                             None => {
                                 self.state += 1;
                                 continue;
                             }
                         }
                     }
                     self.state += 1;
                     continue;
                 }
             });
        }

        let iter_record_name = format_ident!("{}RecordIter", def_name);
        let record_wrapper_name = format_ident!("{}Record", def_name);

        let mut record_match_arms = Vec::new();
        for model in &self.visitor.models {
            let model_name = &model.name;
            record_match_arms.push(quote! {
                #def_name::#model_name(m) => {
                    let wrapper: #record_wrapper_name = m.into();
                    wrapper.into()
                }
            });
        }

        quote! {
            /// Helper struct to hold open read-only tables for definition iteration
            pub struct #tables_name {
                #(#table_field_defs),*
            }

            impl #tables_name {
                pub fn new(txn: &redb::ReadTransaction) -> Result<Self, redb::Error> {
                     #(#table_inits)*
                     Ok(Self {
                         #(#table_field_names),*
                     })
                }

                pub fn iter<'a>(&'a self) -> Result<#iter_name<'a>, redb::Error> {
                    Ok(#iter_name {
                        #(#iter_inits),*,
                        state: 0
                    })
                }

                pub fn iter_records<'a>(&'a self) -> Result<#iter_record_name<'a>, redb::Error> {
                    Ok(#iter_record_name {
                        inner: self.iter()?
                    })
                }
            }

            /// Iterator over all models in the definition
            pub struct #iter_name<'a> {
                #(#iter_field_defs),*,
                state: usize,
            }

            impl<'a> Iterator for #iter_name<'a> {
                type Item = netabase_store::errors::NetabaseResult<#def_name>;

                fn next(&mut self) -> Option<Self::Item> {
                    loop {
                        match self.state {
                            #(#next_arms)*
                            _ => return None,
                        }
                    }
                }
            }

            /// Iterator over all records in the definition
            pub struct #iter_record_name<'a> {
                inner: #iter_name<'a>
            }

            impl<'a> Iterator for #iter_record_name<'a> {
                type Item = netabase_store::errors::NetabaseResult<netabase_store::libp2p::kad::Record>;

                fn next(&mut self) -> Option<Self::Item> {
                    match self.inner.next() {
                        Some(Ok(def)) => {
                            let record = match def {
                                #(#record_match_arms)*
                                _ => unreachable!("Iterator only yields models"),
                            };
                            Some(Ok(record))
                        }
                        Some(Err(e)) => Some(Err(e)),
                        None => None,
                    }
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
            let target_type = if model.is_content_addressed() {
                format_ident!("{}Envelope", model_name)
            } else {
                model_name.clone()
            };

            variants.push(quote! { 
                #model_name(netabase_store::traits::registry::models::treenames::ModelTreeNames<'static, #definition_name, #target_type>) 
            });
            get_tree_names_arms.push(quote! {
                #discriminant_name::#model_name => vec![#enum_name::#model_name(<#target_type as netabase_store::traits::registry::models::model::NetabaseModel<#definition_name>>::TREE_NAMES)]
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
            let first_model = &self.visitor.models[0];
            let first_model_name = &first_model.name;
            let target_type = if first_model.is_content_addressed() {
                format_ident!("{}Envelope", first_model_name)
            } else {
                first_model_name.clone()
            };

            quote! { #enum_name::#first_model_name(<#target_type as netabase_store::traits::registry::models::model::NetabaseModel<#definition_name>>::TREE_NAMES) }
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
            impl TryInto<netabase_store::traits::registry::models::treenames::DiscriminantTableName<#definition_name>> for #enum_name {
                type Error = ();

                fn try_into(self) -> Result<netabase_store::traits::registry::models::treenames::DiscriminantTableName<#definition_name>, Self::Error> {
                    Err(())
                }
            }
        };

        // NetabaseDefinitionTreeNames trait implementation
        let netabase_definition_tree_names_impl = quote! {
            impl netabase_store::traits::registry::definition::NetabaseDefinitionTreeNames<#definition_name> for #enum_name {
                #[inline]
                fn get_tree_names(discriminant: #discriminant_name) -> Vec<Self> {
                    match discriminant {
                        #(#get_tree_names_arms),*
                    }
                }

                fn get_model_tree_names<M: netabase_store::traits::registry::models::model::NetabaseModel<#definition_name>>(&self) -> Option<&'static netabase_store::traits::registry::models::treenames::ModelTreeNames<'static, #definition_name, M>>
                where
                    for<'a> Self: From<netabase_store::traits::registry::models::treenames::ModelTreeNames<'a, Self, M>>,
                    <<M as netabase_store::traits::registry::models::model::NetabaseModel<#definition_name>>::Keys as netabase_store::traits::registry::models::keys::NetabaseModelKeys<#definition_name, M>>::Secondary:
                        strum::IntoDiscriminant,
                    <<M as netabase_store::traits::registry::models::model::NetabaseModel<#definition_name>>::Keys as netabase_store::traits::registry::models::keys::NetabaseModelKeys<#definition_name, M>>::Relational:
                        strum::IntoDiscriminant,
                    <<M as netabase_store::traits::registry::models::model::NetabaseModel<#definition_name>>::Keys as netabase_store::traits::registry::models::keys::NetabaseModelKeys<#definition_name, M>>::Subscription:
                        strum::IntoDiscriminant,
                    <<<M as netabase_store::traits::registry::models::model::NetabaseModel<#definition_name>>::Keys as netabase_store::traits::registry::models::keys::NetabaseModelKeys<#definition_name, M>>::Secondary as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
                    <<<M as netabase_store::traits::registry::models::model::NetabaseModel<#definition_name>>::Keys as netabase_store::traits::registry::models::keys::NetabaseModelKeys<#definition_name, M>>::Relational as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
                    <<<M as netabase_store::traits::registry::models::model::NetabaseModel<#definition_name>>::Keys as netabase_store::traits::registry::models::keys::NetabaseModelKeys<#definition_name, M>>::Subscription as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
                    <<<M as netabase_store::traits::registry::models::model::NetabaseModel<#definition_name>>::Keys as netabase_store::traits::registry::models::keys::NetabaseModelKeys<#definition_name, M>>::Blob as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug,
                    <<M as netabase_store::traits::registry::models::model::NetabaseModel<#definition_name>>::Keys as netabase_store::traits::registry::models::keys::NetabaseModelKeys<#definition_name, M>>::Subscription: 'static,
                    <<<M as netabase_store::traits::registry::models::model::NetabaseModel<#definition_name>>::Keys as netabase_store::traits::registry::models::keys::NetabaseModelKeys<#definition_name, M>>::Libp2p as strum::IntoDiscriminant>::Discriminant: 'static + std::fmt::Debug
                {
                    // Return the static TREE_NAMES if this variant matches the model
                    // This is a placeholder - proper implementation would check discriminant match
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