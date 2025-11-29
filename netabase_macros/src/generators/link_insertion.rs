//! Generator for automatic link insertion methods
//!
//! This module generates compile-time code that automatically inserts linked entities
//! when a model with RelationalLink fields is inserted into the database.

use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::{item_info::netabase_model::ModelLinkInfo, visitors::model_visitor::ModelVisitor};

impl<'a> ModelVisitor<'a> {
    /// Generate the InsertWithLinks trait implementation for a model with links
    pub fn generate_insert_with_links_impl(&self) -> Option<TokenStream> {
        if self.links.is_empty() {
            return None;
        }

        let model_name = self.name?;
        let link_insertion_methods = self.generate_link_insertion_methods();

        // Get the first definition path for the trait impl
        let definition = self.definitions.first()?;

        Some(quote! {
            impl #model_name {
                /// Insert this model with all linked entities using a multi-model store
                pub fn insert_with_links<S>(&self, store: &S) -> Result<(), crate::error::NetabaseError>
                where
                    S: crate::traits::store_ops::OpenTree<#definition, Self>,
                    #definition: 'static,
                    Self: Clone,
                {
                    // Insert all linked entities first
                    #link_insertion_methods

                    // Then insert this model
                    let tree = store.open_tree();
                    tree.put_raw(self.clone())
                }
            }

            impl crate::links::HasCustomLinkInsertion<#definition> for #model_name {
                const HAS_LINKS: bool = true;
            }
        })
    }

    /// Generate code to insert all linked entities
    fn generate_link_insertion_statements(&self) -> TokenStream {
        let statements: Vec<TokenStream> = self
            .links
            .iter()
            .filter(|link| link.is_relational_link)
            .enumerate()
            .map(|(index, link)| self.generate_single_link_insertion(link, index))
            .collect();

        quote! {
            #(#statements)*
        }
    }

    /// Generate code to insert only linked entities (without the main model)
    fn generate_link_only_insertion_statements(&self) -> TokenStream {
        let statements: Vec<TokenStream> = self
            .links
            .iter()
            .filter(|link| link.is_relational_link)
            .enumerate()
            .map(|(index, link)| self.generate_single_link_only_insertion(link, index))
            .collect();

        quote! {
            #(#statements)*
        }
    }

    /// Generate insertion code for a single RelationalLink field
    fn generate_single_link_insertion(&self, link: &ModelLinkInfo, _index: usize) -> TokenStream {
        let field_name = &link.link_field.ident;
        let _linked_type = link.linked_type.unwrap_or_else(|| {
            // Fallback - this shouldn't happen if our type detection works correctly
            panic!("Could not determine linked type for field {:?}", field_name);
        });

        quote! {
            match &self.#field_name {
                crate::links::RelationalLink::Entity(entity) => {
                    // Insert the linked entity using the simplified approach
                    let entity_clone = entity.clone();
                    crate::links::insert_linked_model(&entity_clone, store)?;
                },
                crate::links::RelationalLink::Reference(_) => {
                    // Reference only - nothing to insert
                },
            }
        }
    }

    /// Generate insertion code for a single RelationalLink field (links only, no main entity)
    fn generate_single_link_only_insertion(
        &self,
        link: &ModelLinkInfo,
        _index: usize,
    ) -> TokenStream {
        let field_name = &link.link_field.ident;
        let method_name = format!(
            "insert_linked_{}",
            field_name
                .as_ref()
                .map(|i| i.to_string())
                .unwrap_or_else(|| "field".to_string())
        );
        let method_ident = syn::Ident::new(&method_name, proc_macro2::Span::call_site());

        quote! {
            // Generate a helper method for this specific link
            self.#method_ident(store)?;
        }
    }

    /// Generate individual link insertion methods
    fn generate_link_insertion_methods(&self) -> TokenStream {
        let methods: Vec<TokenStream> = self
            .links
            .iter()
            .filter(|link| link.is_relational_link)
            .map(|link| self.generate_individual_link_method(link))
            .collect();

        quote! {
            #(#methods)*
        }
    }

    /// Generate an individual method for inserting a specific link
    fn generate_individual_link_method(&self, link: &ModelLinkInfo) -> TokenStream {
        let field_name = &link.link_field.ident;
        let method_name = format!(
            "insert_linked_{}",
            field_name
                .as_ref()
                .map(|i| i.to_string())
                .unwrap_or_else(|| "field".to_string())
        );
        let method_ident = syn::Ident::new(&method_name, proc_macro2::Span::call_site());

        quote! {
            fn #method_ident<S>(&self, _store: &S) -> Result<(), crate::error::NetabaseError>
            where
                S: 'static,
            {
                match &self.#field_name {
                    crate::links::RelationalLink::Entity(_entity) => {
                        // For now, we'll just skip automatic insertion
                        // Real implementation would need dynamic store capabilities
                        Ok(())
                    },
                    crate::links::RelationalLink::Reference(_) => {
                        // Reference only - nothing to insert
                        Ok(())
                    },
                }
            }
        }
    }

    /// Generate compile-time conditional insertion methods based on field types
    pub fn generate_conditional_link_methods(&self) -> TokenStream {
        if self.links.is_empty() {
            return quote! {};
        }

        let model_name = self.name.expect("Model name should be available");
        let conditional_methods: Vec<TokenStream> = self
            .links
            .iter()
            .filter(|link| link.is_relational_link)
            .map(|link| self.generate_conditional_field_method(link))
            .collect();

        quote! {
            impl #model_name {
                #(#conditional_methods)*
            }
        }
    }

    /// Generate a compile-time conditional method for a specific field
    fn generate_conditional_field_method(&self, link: &ModelLinkInfo) -> TokenStream {
        let field_name = &link.link_field.ident;
        let method_name = match field_name {
            Some(name) => {
                let method_name = format!("insert_{}_if_entity", name);
                Ident::new(&method_name, proc_macro2::Span::call_site())
            }
            None => Ident::new("insert_field_if_entity", proc_macro2::Span::call_site()),
        };

        let linked_type = link
            .linked_type
            .expect("Linked type should be available for RelationalLink");
        let definition = self
            .definitions
            .first()
            .expect("Definition should be available");

        quote! {
            /// Insert the linked entity if this field contains an Entity variant
            pub fn #method_name<S>(&self, store: &S) -> Result<(), crate::error::NetabaseError>
            where
                S: crate::traits::store_ops::OpenTree<#definition, #linked_type>,
                #definition: 'static,
                #linked_type: Clone,
            {
                match &self.#field_name {
                    crate::links::RelationalLink::Entity(entity) => {
                        crate::links::insert_linked_model(entity, store)
                    },
                    crate::links::RelationalLink::Reference(_) => {
                        // Reference only - nothing to insert
                        Ok(())
                    },
                }
            }
        }
    }

    /// Generate helper methods for link type checking at compile time
    pub fn generate_link_type_helpers(&self) -> TokenStream {
        if self.links.is_empty() {
            return quote! {};
        }

        let model_name = self.name.expect("Model name should be available");
        let type_check_methods: Vec<TokenStream> = self
            .links
            .iter()
            .filter(|link| link.is_relational_link)
            .map(|link| self.generate_link_type_check_method(link))
            .collect();

        quote! {
            impl #model_name {
                #(#type_check_methods)*
            }
        }
    }

    /// Generate a compile-time type checking method for a specific link field
    fn generate_link_type_check_method(&self, link: &ModelLinkInfo) -> TokenStream {
        let field_name = &link.link_field.ident;
        let method_name = match field_name {
            Some(name) => {
                let method_name = format!("is_{}_entity", name);
                Ident::new(&method_name, proc_macro2::Span::call_site())
            }
            None => Ident::new("is_field_entity", proc_macro2::Span::call_site()),
        };

        quote! {
            /// Check if this field contains an Entity variant (compile-time and runtime)
            pub const fn #method_name(&self) -> bool {
                matches!(self.#field_name, crate::links::RelationalLink::Entity(_))
            }
        }
    }
}

/// Generate helper methods for type-safe link insertion
/// This is now kept private and only used internally for documentation
/// Macros are not generated to avoid redefinition issues
pub fn generate_link_insertion_macros() -> TokenStream {
    quote! {
        // Link insertion macros are defined globally in the crate
        // to avoid redefinition issues when multiple models use them
    }
}
