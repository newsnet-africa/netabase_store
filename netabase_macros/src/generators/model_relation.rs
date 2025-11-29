//! Generator for model relation enums and trait implementations
//!
//! This module generates relation-related types and traits for models that contain
//! RelationalLink fields, following the same patterns as the key generation.

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{ItemEnum, parse_quote};

use crate::{
    item_info::netabase_model::ModelLinkInfo, util::append_ident,
    visitors::model_visitor::ModelVisitor,
};

impl<'a> ModelVisitor<'a> {
    /// Generate the relations enum for models with RelationalLink fields
    ///
    /// Returns None if the model has no relational links
    pub fn generate_relations_enum(&self) -> Option<ItemEnum> {
        if self.links.is_empty() {
            return None;
        }

        let model_name = self.name.expect("Model name should be available");
        let relations_name = append_ident(model_name, "Relations");

        let variants: Vec<TokenStream> = self
            .links
            .iter()
            .filter(|link| link.is_relational_link)
            .enumerate()
            .map(|(index, link)| self.generate_relation_variant(link, index))
            .collect();

        if variants.is_empty() {
            return None;
        }

        let relations_enum: ItemEnum = parse_quote!(
            #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
                ::netabase_store::netabase_deps::strum::EnumDiscriminants,
                ::netabase_store::netabase_deps::bincode::Encode,
                ::netabase_store::netabase_deps::bincode::Decode,
                ::netabase_store::netabase_deps::serde::Serialize,
                ::netabase_store::netabase_deps::serde::Deserialize
            )]
            #[strum_discriminants(derive(Hash,
                ::netabase_store::netabase_deps::bincode::Encode,
                ::netabase_store::netabase_deps::bincode::Decode,
                ::netabase_store::netabase_deps::serde::Serialize,
                ::netabase_store::netabase_deps::serde::Deserialize))]
            #[repr(u8)]
            pub enum #relations_name {
                #(#variants),*
            }
        );

        Some(relations_enum)
    }

    /// Generate a single variant for a relation field
    fn generate_relation_variant(&self, link: &ModelLinkInfo, _index: usize) -> TokenStream {
        let field_name = link
            .link_field
            .ident
            .as_ref()
            .expect("Field should have a name");
        let variant_name = self.field_name_to_variant_name(field_name);
        let linked_type = link
            .linked_type
            .expect("RelationalLink should have a linked type");

        quote! {
            #[doc = concat!("Relation to ", stringify!(#linked_type), " via field ", stringify!(#field_name))]
            #variant_name
        }
    }

    /// Convert a field name to a variant name (e.g., "author" -> "Author")
    fn field_name_to_variant_name(&self, field_name: &Ident) -> Ident {
        let field_str = field_name.to_string();
        let variant_str = format!(
            "{}{}",
            field_str.chars().next().unwrap().to_uppercase(),
            field_str.chars().skip(1).collect::<String>()
        );
        Ident::new(&variant_str, field_name.span())
    }

    /// Generate the NetabaseRelationDiscriminant trait implementation
    pub fn generate_relation_discriminant_impl(&self) -> Option<TokenStream> {
        if self.links.is_empty() {
            return None;
        }

        let model_name = self.name.expect("Model name should be available");
        let relations_name = append_ident(model_name, "Relations");

        let field_name_arms: Vec<TokenStream> = self
            .links
            .iter()
            .filter(|link| link.is_relational_link)
            .map(|link| {
                let field_name = link
                    .link_field
                    .ident
                    .as_ref()
                    .expect("Field should have a name");
                let variant_name = self.field_name_to_variant_name(field_name);
                let field_name_str = field_name.to_string();
                quote! {
                    #relations_name::#variant_name => #field_name_str,
                }
            })
            .collect();

        let target_model_arms: Vec<TokenStream> = self
            .links
            .iter()
            .filter(|link| link.is_relational_link)
            .map(|link| {
                let field_name = link
                    .link_field
                    .ident
                    .as_ref()
                    .expect("Field should have a name");
                let variant_name = self.field_name_to_variant_name(field_name);
                let linked_type = link
                    .linked_type
                    .expect("RelationalLink should have a linked type");
                let type_name = quote!(#linked_type).to_string();
                quote! {
                    #relations_name::#variant_name => #type_name,
                }
            })
            .collect();

        let all_variants: Vec<TokenStream> = self
            .links
            .iter()
            .filter(|link| link.is_relational_link)
            .map(|link| {
                let field_name = link
                    .link_field
                    .ident
                    .as_ref()
                    .expect("Field should have a name");
                let variant_name = self.field_name_to_variant_name(field_name);
                quote! {
                    #relations_name::#variant_name,
                }
            })
            .collect();

        Some(quote! {
            impl ::netabase_store::traits::relation::NetabaseRelationDiscriminant for #relations_name {
                fn field_name(&self) -> &'static str {
                    match self {
                        #(#field_name_arms)*
                    }
                }

                fn target_model_name(&self) -> &'static str {
                    match self {
                        #(#target_model_arms)*
                    }
                }

                fn all_variants() -> Vec<Self> {
                    vec![
                        #(#all_variants)*
                    ]
                }
            }
        })
    }

    /// Generate the NetabaseRelationTrait implementation for the model
    pub fn generate_relation_trait_impl(&self) -> Option<TokenStream> {
        if self.links.is_empty() {
            return None;
        }

        let model_name = self.name.expect("Model name should be available");
        let relations_name = append_ident(model_name, "Relations");
        let definition = self
            .definitions
            .first()
            .expect("Definition should be available");

        let relation_map_entries: Vec<TokenStream> = self
            .links
            .iter()
            .filter(|link| link.is_relational_link)
            .map(|link| {
                let field_name = link
                    .link_field
                    .ident
                    .as_ref()
                    .expect("Field should have a name");
                let variant_name = self.field_name_to_variant_name(field_name);
                quote! {
                    map.insert(
                        #relations_name::#variant_name.into(),
                        #relations_name::#variant_name
                    );
                }
            })
            .collect();

        let insert_relations_statements = self.generate_relation_insertion_statements();
        let insert_relations_only_statements = self.generate_relation_only_insertion_statements();

        Some(quote! {
            impl ::netabase_store::traits::relation::NetabaseRelationTrait<#definition> for #model_name {
                type Relations = #relations_name;

                fn relations(&self) -> std::collections::HashMap<
                    <Self::Relations as ::netabase_store::netabase_deps::strum::IntoDiscriminant>::Discriminant,
                    Self::Relations
                > {
                    let mut map = std::collections::HashMap::new();
                    #(#relation_map_entries)*
                    map
                }

                fn insert_with_relations<S>(&self, store: &S) -> Result<(), ::netabase_store::error::NetabaseError>
                where
                    S: ::netabase_store::traits::relation::MultiModelStore<#definition>,
                    Self: Clone,
                {
                    // Insert all linked entities first
                    #insert_relations_statements

                    // Then insert this model
                    store.insert_model_erased(self)
                }

                fn insert_relations_only<S>(&self, store: &S) -> Result<(), ::netabase_store::error::NetabaseError>
                where
                    S: ::netabase_store::traits::relation::MultiModelStore<#definition>,
                    Self: Clone,
                {
                    #insert_relations_only_statements
                    Ok(())
                }
            }
        })
    }

    /// Generate code to insert all related entities
    fn generate_relation_insertion_statements(&self) -> TokenStream {
        let statements: Vec<TokenStream> = self
            .links
            .iter()
            .filter(|link| link.is_relational_link)
            .map(|link| self.generate_single_relation_insertion(link))
            .collect();

        quote! {
            #(#statements)*
        }
    }

    /// Generate code to insert only related entities (not the main model)
    fn generate_relation_only_insertion_statements(&self) -> TokenStream {
        let statements: Vec<TokenStream> = self
            .links
            .iter()
            .filter(|link| link.is_relational_link)
            .map(|link| self.generate_single_relation_only_insertion(link))
            .collect();

        quote! {
            #(#statements)*
        }
    }

    /// Generate insertion code for a single RelationalLink field
    fn generate_single_relation_insertion(&self, link: &ModelLinkInfo) -> TokenStream {
        let field_name = link
            .link_field
            .ident
            .as_ref()
            .expect("Field should have a name");

        quote! {
            match &self.#field_name {
                ::netabase_store::links::RelationalLink::Entity(entity) => {
                    store.insert_model_erased(entity)?;
                },
                ::netabase_store::links::RelationalLink::Reference(_) => {
                    // Reference only - nothing to insert
                },
                ::netabase_store::links::RelationalLink::_RelationMarker(_) => {
                    unreachable!("RelationMarker should never be instantiated");
                },
            }
        }
    }

    /// Generate insertion code for a single RelationalLink field (relations only)
    fn generate_single_relation_only_insertion(&self, link: &ModelLinkInfo) -> TokenStream {
        let field_name = link
            .link_field
            .ident
            .as_ref()
            .expect("Field should have a name");

        quote! {
            match &self.#field_name {
                ::netabase_store::links::RelationalLink::Entity(entity) => {
                    store.insert_model_erased(entity)?;
                },
                ::netabase_store::links::RelationalLink::Reference(_) => {
                    // Reference only - nothing to insert
                },
                ::netabase_store::links::RelationalLink::_RelationMarker(_) => {
                    unreachable!("RelationMarker should never be instantiated");
                },
            }
        }
    }

    /// Generate helper methods for individual relation field access and type aliases
    pub fn generate_relation_helpers(&self) -> Option<TokenStream> {
        if self.links.is_empty() {
            return None;
        }

        let model_name = self.name.expect("Model name should be available");
        let helper_methods: Vec<TokenStream> = self
            .links
            .iter()
            .filter(|link| link.is_relational_link)
            .map(|link| self.generate_relation_field_helper(link))
            .collect();

        Some(quote! {
            impl #model_name {
                #(#helper_methods)*
            }
        })
    }

    /// Generate a helper method for accessing a specific relation field
    fn generate_relation_field_helper(&self, link: &ModelLinkInfo) -> TokenStream {
        let field_name = link
            .link_field
            .ident
            .as_ref()
            .expect("Field should have a name");
        let linked_type = link
            .linked_type
            .expect("RelationalLink should have a linked type");
        let definition = self
            .definitions
            .first()
            .expect("Definition should be available");

        let method_name = Ident::new(&format!("get_{}", field_name), field_name.span());
        let hydrate_method_name = Ident::new(&format!("hydrate_{}", field_name), field_name.span());
        let is_entity_method_name =
            Ident::new(&format!("is_{}_entity", field_name), field_name.span());
        let is_reference_method_name =
            Ident::new(&format!("is_{}_reference", field_name), field_name.span());
        let insert_if_entity_method_name = Ident::new(
            &format!("insert_{}_if_entity", field_name),
            field_name.span(),
        );

        quote! {
            /// Get the relational link for this field
            #[doc = concat!("Get the ", stringify!(#field_name), " relation link")]
            pub fn #method_name(&self) -> &::netabase_store::links::RelationalLink<#definition, #linked_type, Self::Relations> {
                &self.#field_name
            }

            /// Hydrate the linked entity if it's a reference
            #[doc = concat!("Hydrate the ", stringify!(#field_name), " entity from the store")]
            pub fn #hydrate_method_name<T>(&self, store: T) -> Result<Option<#linked_type>, ::netabase_store::error::NetabaseError>
            where
                T: ::netabase_store::store_ops::StoreOps<#definition, #linked_type>,
            {
                self.#field_name.clone().hydrate(store)
            }

            /// Check if this field contains an entity (vs reference)
            #[doc = concat!("Check if ", stringify!(#field_name), " contains a full entity")]
            pub fn #is_entity_method_name(&self) -> bool {
                self.#field_name.is_entity()
            }

            /// Check if this field contains a reference (vs entity)
            #[doc = concat!("Check if ", stringify!(#field_name), " contains a reference")]
            pub fn #is_reference_method_name(&self) -> bool {
                self.#field_name.is_reference()
            }

            /// Insert the linked entity if this field contains an Entity variant
            #[doc = concat!("Insert the ", stringify!(#field_name), " entity if it's an Entity variant")]
            pub fn #insert_if_entity_method_name<S>(&self, store: &S) -> Result<(), ::netabase_store::error::NetabaseError>
            where
                S: ::netabase_store::traits::relation::MultiModelStore<#definition>,
            {
                match &self.#field_name {
                    ::netabase_store::links::RelationalLink::Entity(entity) => {
                        store.insert_model_erased(entity)
                    },
                    ::netabase_store::links::RelationalLink::Reference(_) => {
                        // Reference only - nothing to insert
                        Ok(())
                    },
                    ::netabase_store::links::RelationalLink::_RelationMarker(_) => {
                        unreachable!("RelationMarker should never be instantiated");
                    },
                }
            }
        }
    }

    /// Generate marker trait implementations for relation detection
    pub fn generate_relation_markers(&self) -> Option<TokenStream> {
        if self.links.is_empty() {
            return None;
        }

        let model_name = self.name.expect("Model name should be available");
        let relations_name = append_ident(model_name, "Relations");
        let definition = self
            .definitions
            .first()
            .expect("Definition should be available");

        Some(quote! {
            impl ::netabase_store::links::HasCustomRelationInsertion<#definition> for #model_name {
                const HAS_RELATIONS: bool = true;
            }
        })
    }

    /// Generate a type alias for backward-compatible RelationalLink usage
    /// Generate type alias for backward compatibility (per-model scope)
    ///
    /// Note: Global type aliases are disabled to prevent conflicts when multiple
    /// models with relations exist in the same scope. Users should migrate to
    /// the explicit 3-parameter RelationalLink<D, M, R> form.
    pub fn generate_relation_type_alias(&self) -> Option<TokenStream> {
        if self.links.is_empty() {
            return None;
        }

        let model_name = self.name.expect("Model name should be available");
        let relations_name = append_ident(model_name, "Relations");
        let alias_name = append_ident(model_name, "RelationalLink");

        // Generate a model-specific type alias to avoid conflicts
        // This allows backward compatibility while preventing name collisions
        Some(quote! {
            // Model-specific RelationalLink alias for backward compatibility
            #[allow(non_camel_case_types)]
            pub type #alias_name<D, M> = ::netabase_store::links::RelationalLink<D, M, #relations_name>;

            // Re-export the generic RelationalLink for this model's scope
            #[allow(non_camel_case_types)]
            pub use #alias_name as RelationalLink;
        })
    }
}
