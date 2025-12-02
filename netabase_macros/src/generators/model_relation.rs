//! Generator for model relation enums and trait implementations
//!
//! This module generates relation-related types and traits for models that contain
//! RelationalLink fields, following the same patterns as the key generation.

use heck::ToPascalCase;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{ItemEnum, parse_quote};

use crate::{
    item_info::netabase_model::ModelLinkInfo, util::append_ident,
    visitors::model_visitor::ModelVisitor,
};
use syn::Type;

/// Type of wrapper around a RelationalLink field
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WrapperType {
    /// Direct RelationalLink<D, M>
    None,
    /// Option<RelationalLink<D, M>>
    Option,
    /// Vec<RelationalLink<D, M>>
    Vec,
    /// Box<RelationalLink<D, M>>
    Box,
}

impl<'a> ModelVisitor<'a> {
    /// Generate the relations enum for models with RelationalLink fields
    /// This creates a layered enum structure with user-defined relation names
    ///
    /// Returns None if the model has no relational links
    pub fn generate_relations_enum(&self) -> Option<ItemEnum> {
        if self.links.is_empty() {
            return None;
        }

        let model_name = self.name.expect("Model name should be available");
        let relations_name = append_ident(model_name, "Relations");

        // Generate relation type variants with metadata
        let variants: Vec<TokenStream> = self
            .links
            .iter()
            .filter(|link| link.is_relational_link)
            .enumerate()
            .map(|(index, link)| self.generate_enhanced_relation_variant(link, index))
            .collect();

        if variants.is_empty() {
            return None;
        }

        let relations_enum: ItemEnum = parse_quote!(
            #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
                ::netabase_store::netabase_deps::strum::EnumDiscriminants,
                ::netabase_store::netabase_deps::strum::EnumIter,
                ::netabase_store::netabase_deps::bincode::Encode,
                ::netabase_store::netabase_deps::bincode::Decode,
                ::netabase_store::netabase_deps::serde::Serialize,
                ::netabase_store::netabase_deps::serde::Deserialize
            )]
            #[strum_discriminants(derive(Hash,
                ::netabase_store::netabase_deps::strum::EnumIter,
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
    #[allow(dead_code)] // Reserved for future use
    fn generate_relation_variant(&self, link: &ModelLinkInfo, _index: usize) -> TokenStream {
        let field_name = link
            .link_field
            .ident
            .as_ref()
            .expect("Field should have a name");
        let variant_name = self.get_variant_name_for_link(link);
        let linked_type = link
            .linked_type
            .expect("RelationalLink should have a linked type");

        quote! {
            #[doc = concat!("Relation to ", stringify!(#linked_type), " via field ", stringify!(#field_name))]
            #variant_name
        }
    }

    /// Convert a field name to a variant name using PascalCase (e.g., "author_posts" -> "AuthorPosts")
    fn field_name_to_variant_name(&self, field_name: &Ident) -> Ident {
        let field_str = field_name.to_string();
        let variant_str = field_str.to_pascal_case();
        Ident::new(&variant_str, field_name.span())
    }

    /// Get the correct variant name for a relation link, using the user-defined relation name if provided
    fn get_variant_name_for_link(&self, link: &ModelLinkInfo) -> Ident {
        let field_name = link
            .link_field
            .ident
            .as_ref()
            .expect("Field should have a name");

        // Use user-defined name if provided, otherwise use field name
        if let Some(ref user_name) = link.relation_name {
            // Convert user-defined name to proper PascalCase variant format
            let variant_str = user_name.to_pascal_case();
            Ident::new(&variant_str, field_name.span())
        } else {
            self.field_name_to_variant_name(field_name)
        }
    }

    /// Generate an enhanced variant for a relation field with user-defined names and metadata
    fn generate_enhanced_relation_variant(
        &self,
        link: &ModelLinkInfo,
        _index: usize,
    ) -> TokenStream {
        let field_name = link
            .link_field
            .ident
            .as_ref()
            .expect("Field should have a name");

        let variant_name = self.get_variant_name_for_link(link);

        let linked_type = link
            .linked_type
            .expect("RelationalLink should have a linked type");

        // Include relation name if provided
        let name_doc = if let Some(ref rel_name) = link.relation_name {
            format!(" as '{}'", rel_name)
        } else {
            String::new()
        };

        quote! {
            #[doc = concat!("Relation to ", stringify!(#linked_type), " via field ", stringify!(#field_name), #name_doc)]
            #variant_name
        }
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
                let variant_name = self.get_variant_name_for_link(link);
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
                let variant_name = self.get_variant_name_for_link(link);
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
                let variant_name = self.get_variant_name_for_link(link);
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
    #[allow(dead_code)]
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
                let variant_name = self.get_variant_name_for_link(link);
                quote! {
                    map.insert(
                        #relations_name::#variant_name.into(),
                        #relations_name::#variant_name
                    );
                }
            })
            .collect();

        // Collect all linked model types to add as OpenTree bounds
        let linked_types: Vec<_> = self
            .links
            .iter()
            .filter(|link| link.is_relational_link)
            .filter_map(|link| link.linked_type)
            .collect();

        // Generate OpenTree bounds for each linked type
        let _open_tree_bounds: Vec<TokenStream> = linked_types
            .iter()
            .map(|linked_type| {
                quote! {
                    S: ::netabase_store::traits::store_ops::OpenTree<#definition, #linked_type>
                }
            })
            .collect();

        let insert_relations_statements = self.generate_relation_insertion_statements();
        let insert_relations_only_statements = self.generate_relation_only_insertion_statements();
        let recursive_statements = self.generate_recursive_relation_insertion_statements();

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
                    S: ::netabase_store::traits::relation::MultiModelStore<#definition>
                        + ::netabase_store::traits::store_ops::OpenTree<#definition, Self>,
                    Self: Clone + ::netabase_store::traits::model::NetabaseModelTrait<#definition>,
                {
                    // Insert all linked entities first using helper methods
                    #insert_relations_statements

                    // Then insert this model using OpenTree
                    let tree: <S as ::netabase_store::traits::store_ops::OpenTree<#definition, Self>>::Tree<'_> = <S as ::netabase_store::traits::store_ops::OpenTree<#definition, Self>>::open_tree(store);
                    ::netabase_store::traits::store_ops::StoreOps::put_raw(&tree, self.clone())
                }

                fn insert_relations_only<S>(&self, store: &S) -> Result<(), ::netabase_store::error::NetabaseError>
                where
                    S: ::netabase_store::traits::relation::MultiModelStore<#definition>,
                    Self: Clone,
                {
                    #insert_relations_only_statements
                    Ok(())
                }

                fn insert_relations_recursive<S>(&self, store: &S, level: ::netabase_store::links::RecursionLevel) -> Result<(), ::netabase_store::error::NetabaseError>
                where
                    S: ::netabase_store::traits::relation::MultiModelStore<#definition>,
                    Self: Clone + ::netabase_store::traits::model::NetabaseModelTrait<#definition>
                        + ::netabase_store::links::HasCustomRelationInsertion<#definition>,
                {
                    self.insert_relations_recursive_with_depth(store, level, 0)
                }

                fn insert_relations_recursive_with_depth<S>(&self, store: &S, level: ::netabase_store::links::RecursionLevel, current_depth: u8) -> Result<(), ::netabase_store::error::NetabaseError>
                where
                    S: ::netabase_store::traits::relation::MultiModelStore<#definition>,
                    Self: Clone + ::netabase_store::traits::model::NetabaseModelTrait<#definition>
                        + ::netabase_store::links::HasCustomRelationInsertion<#definition>,
                {
                    // Insert relations recursively if we should recurse
                    if level.should_recurse(current_depth) {
                        #recursive_statements
                    }
                    Ok(())
                }
            }
        })
    }

    /// Generate code to insert all related entities
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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

    fn generate_recursive_relation_insertion_statements(&self) -> TokenStream {
        let statements: Vec<TokenStream> = self
            .links
            .iter()
            .filter(|link| link.is_relational_link)
            .map(|link| self.generate_single_recursive_relation_insertion(link))
            .collect();

        quote! {
            #(#statements)*
        }
    }

    fn generate_single_recursive_relation_insertion(&self, link: &ModelLinkInfo) -> TokenStream {
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

        // Detect wrapper type
        let field_type = &link.link_field.ty;
        let wrapper_type = Self::detect_wrapper_type(field_type);

        match wrapper_type {
            WrapperType::None => {
                quote! {
                    match &self.#field_name {
                        ::netabase_store::links::RelationalLink::Entity(entity) => {
                            // Insert the entity using the store's MultiModelStore capability
                            store.insert_model_erased(entity)?;

                            // If the entity has relations and we have recursion depth left, recurse
                            if <#linked_type as ::netabase_store::links::HasCustomRelationInsertion<#definition>>::HAS_RELATIONS {
                                let next_level = level.next_level();
                                if next_level.should_recurse(current_depth + 1) {
                                    entity.insert_relations_recursive_with_depth(store, next_level, current_depth + 1)?;
                                }
                            }
                        },
                        ::netabase_store::links::RelationalLink::Reference(_) => {
                            // References don't need insertion
                        },
                    }
                }
            }
            WrapperType::Option => {
                quote! {
                    if let Some(link) = &self.#field_name {
                        match link {
                            ::netabase_store::links::RelationalLink::Entity(entity) => {
                                // Insert the entity using the store's MultiModelStore capability
                                store.insert_model_erased(entity)?;

                                // If the entity has relations and we have recursion depth left, recurse
                                if <#linked_type as ::netabase_store::links::HasCustomRelationInsertion<#definition>>::HAS_RELATIONS {
                                    let next_level = level.next_level();
                                    if next_level.should_recurse(current_depth + 1) {
                                        entity.insert_relations_recursive_with_depth(store, next_level, current_depth + 1)?;
                                    }
                                }
                            },
                            ::netabase_store::links::RelationalLink::Reference(_) => {
                                // References don't need insertion
                            },
                        }
                    }
                }
            }
            WrapperType::Vec => {
                quote! {
                    for link in &self.#field_name {
                        if let ::netabase_store::links::RelationalLink::Entity(entity) = link {
                            // Insert the entity using the store's MultiModelStore capability
                            store.insert_model_erased(entity)?;

                            // If the entity has relations and we have recursion depth left, recurse
                            if <#linked_type as ::netabase_store::links::HasCustomRelationInsertion<#definition>>::HAS_RELATIONS {
                                let next_level = level.next_level();
                                if next_level.should_recurse(current_depth + 1) {
                                    entity.insert_relations_recursive_with_depth(store, next_level, current_depth + 1)?;
                                }
                            }
                        }
                    }
                }
            }
            WrapperType::Box => {
                quote! {
                    match &*self.#field_name {
                        ::netabase_store::links::RelationalLink::Entity(entity) => {
                            // Insert the entity using the store's MultiModelStore capability
                            store.insert_model_erased(entity)?;

                            // If the entity has relations and we have recursion depth left, recurse
                            if <#linked_type as ::netabase_store::links::HasCustomRelationInsertion<#definition>>::HAS_RELATIONS {
                                let next_level = level.next_level();
                                if next_level.should_recurse(current_depth + 1) {
                                    entity.insert_relations_recursive_with_depth(store, next_level, current_depth + 1)?;
                                }
                            }
                        },
                        ::netabase_store::links::RelationalLink::Reference(_) => {
                            // References don't need insertion
                        },
                    }
                }
            }
        }
    }

    /// Generate simple insertion statements for all relation fields
    #[allow(dead_code)] // Reserved for future use
    fn generate_simple_insertion_statements(&self) -> TokenStream {
        let insertion_statements: Vec<TokenStream> = self
            .links
            .iter()
            .filter(|link| link.is_relational_link)
            .map(|link| {
                let field_name = link
                    .link_field
                    .ident
                    .as_ref()
                    .expect("Field should have a name");

                let linked_type = link
                    .linked_type
                    .expect("RelationalLink should have a linked type");

                quote! {
                    match &self.#field_name {
                        ::netabase_store::links::RelationalLink::Entity(entity) => {
                            let entity_tree = store.open_tree::<#linked_type>();
                            ::netabase_store::traits::store_ops::StoreOps::put_raw(&entity_tree, entity.clone())?;
                        },
                        ::netabase_store::links::RelationalLink::Reference(_) => {
                            // Reference only - nothing to insert
                        },
                    }
                }
            })
            .collect();

        quote! {
            #(#insertion_statements)*
        }
    }

    /// Generate a simple insertion method for relation fields
    pub fn generate_relation_insertion_impl(&self) -> Option<TokenStream> {
        if self.links.is_empty() {
            return None;
        }

        let model_name = self.name.expect("Model name should be available");
        let definition = self
            .definitions
            .first()
            .expect("Definition should be available");

        // Collect all linked model types to add as OpenTree bounds
        let linked_types: Vec<_> = self
            .links
            .iter()
            .filter(|link| link.is_relational_link)
            .filter_map(|link| link.linked_type)
            .collect();

        // Generate OpenTree bounds for each linked type
        let _open_tree_bounds: Vec<TokenStream> = linked_types
            .iter()
            .map(|linked_type| {
                quote! {
                    S: ::netabase_store::traits::store_ops::OpenTree<#definition, #linked_type>
                }
            })
            .collect();

        // Generate simple insertion statements for each relation field
        let relation_insertions: Vec<TokenStream> = self
            .links
            .iter()
            .filter(|link| link.is_relational_link)
            .map(|link| {
                let field_name = link
                    .link_field
                    .ident
                    .as_ref()
                    .expect("Field should have a name");

                let linked_type = link
                    .linked_type
                    .expect("RelationalLink should have a linked type");

                // Detect wrapper type
                let field_type = &link.link_field.ty;
                let wrapper_type = Self::detect_wrapper_type(field_type);

                match wrapper_type {
                    WrapperType::None => {
                        quote! {
                            // Insert entity if it's an Entity variant
                            match &self.#field_name {
                                ::netabase_store::links::RelationalLink::Entity(entity) => {
                                    let entity_tree: <S as ::netabase_store::traits::store_ops::OpenTree<#definition, #linked_type>>::Tree<'_> = <S as ::netabase_store::traits::store_ops::OpenTree<#definition, #linked_type>>::open_tree(store);
                                    ::netabase_store::traits::store_ops::StoreOps::put_raw(&entity_tree, entity.clone())?;
                                },
                                ::netabase_store::links::RelationalLink::Reference(_) => {
                                    // Reference only - nothing to insert
                                },
                            }
                        }
                    },
                    WrapperType::Option => {
                        quote! {
                            // Insert entity if Some and it's an Entity variant
                            if let Some(link) = &self.#field_name {
                                match link {
                                    ::netabase_store::links::RelationalLink::Entity(entity) => {
                                        let entity_tree: <S as ::netabase_store::traits::store_ops::OpenTree<#definition, #linked_type>>::Tree<'_> = <S as ::netabase_store::traits::store_ops::OpenTree<#definition, #linked_type>>::open_tree(store);
                                        ::netabase_store::traits::store_ops::StoreOps::put_raw(&entity_tree, entity.clone())?;
                                    },
                                    ::netabase_store::links::RelationalLink::Reference(_) => {
                                        // Reference only - nothing to insert
                                    },
                                }
                            }
                        }
                    },
                    WrapperType::Vec => {
                        quote! {
                            // Insert all entities in the vector
                            let entity_tree: <S as ::netabase_store::traits::store_ops::OpenTree<#definition, #linked_type>>::Tree<'_> = <S as ::netabase_store::traits::store_ops::OpenTree<#definition, #linked_type>>::open_tree(store);
                            for link in &self.#field_name {
                                if let ::netabase_store::links::RelationalLink::Entity(entity) = link {
                                    ::netabase_store::traits::store_ops::StoreOps::put_raw(&entity_tree, entity.clone())?;
                                }
                            }
                        }
                    },
                    WrapperType::Box => {
                        quote! {
                            // Insert entity if it's an Entity variant (dereferencing the Box)
                            match &*self.#field_name {
                                ::netabase_store::links::RelationalLink::Entity(entity) => {
                                    let entity_tree: <S as ::netabase_store::traits::store_ops::OpenTree<#definition, #linked_type>>::Tree<'_> = <S as ::netabase_store::traits::store_ops::OpenTree<#definition, #linked_type>>::open_tree(store);
                                    ::netabase_store::traits::store_ops::StoreOps::put_raw(&entity_tree, entity.clone())?;
                                },
                                ::netabase_store::links::RelationalLink::Reference(_) => {
                                    // Reference only - nothing to insert
                                },
                            }
                        }
                    },
                }
            })
            .collect();

        Some(quote! {
            impl #model_name {
                /// Insert this model with all its related entities (non-recursive)
                pub fn insert_with_relations<S>(&self, store: &S) -> Result<(), ::netabase_store::error::NetabaseError>
                where
                    S: ::netabase_store::traits::store_ops::OpenTree<#definition, Self>,
                    #(#_open_tree_bounds,)*
                    Self: Clone + ::netabase_store::traits::model::NetabaseModelTrait<#definition>,
                {
                    // First insert all related entities
                    #(#relation_insertions)*

                    // Then insert the main model
                    let main_tree: <S as ::netabase_store::traits::store_ops::OpenTree<#definition, Self>>::Tree<'_> = <S as ::netabase_store::traits::store_ops::OpenTree<#definition, Self>>::open_tree(store);
                    ::netabase_store::traits::store_ops::StoreOps::put_raw(&main_tree, self.clone())?;

                    Ok(())
                }
            }
        })
    }

    #[allow(dead_code)]
    fn generate_single_relation_insertion(&self, link: &ModelLinkInfo) -> TokenStream {
        let field_name = link
            .link_field
            .ident
            .as_ref()
            .expect("Field should have a name");

        // Call the helper method defined in the separate impl block
        // This works because the helper method has its own OpenTree bounds
        let insert_method = syn::Ident::new(
            &format!("insert_{}_if_entity", field_name),
            field_name.span(),
        );

        quote! {
            // Helper method has its own bounds, so this will work
            Self::#insert_method(self, store)?;
        }
    }

    /// Generate insertion code for a single RelationalLink field (relations only)
    #[allow(dead_code)]
    fn generate_single_relation_only_insertion(&self, link: &ModelLinkInfo) -> TokenStream {
        let field_name = link
            .link_field
            .ident
            .as_ref()
            .expect("Field should have a name");

        // Call the helper method defined in the separate impl block
        let insert_method = syn::Ident::new(
            &format!("insert_{}_if_entity", field_name),
            field_name.span(),
        );

        quote! {
            Self::#insert_method(self, store)?;
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

        // Detect if field is wrapped in Vec/Option/Box
        let field_type = &link.link_field.ty;
        let wrapper_type = Self::detect_wrapper_type(field_type);

        match wrapper_type {
            WrapperType::None => {
                // Direct RelationalLink - original implementation
                quote! {
                    /// Get the relational link for this field
                    #[doc = concat!("Get the ", stringify!(#field_name), " relation link")]
                    pub fn #method_name(&self) -> &::netabase_store::links::RelationalLink<#definition, #linked_type> {
                        &self.#field_name
                    }

                    /// Hydrate the linked entity if it's a reference
                    #[doc = concat!("Hydrate the ", stringify!(#field_name), " entity from the store")]
                    pub fn #hydrate_method_name<T>(&self, store: T) -> Result<Option<#linked_type>, ::netabase_store::error::NetabaseError>
                    where
                        T: ::netabase_store::traits::store_ops::StoreOps<#definition, #linked_type>,
                    {
                        self.#field_name.clone().hydrate(&store)
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
                        S: ::netabase_store::traits::store_ops::OpenTree<#definition, #linked_type>,
                    {
                        match &self.#field_name {
                            ::netabase_store::links::RelationalLink::Entity(entity) => {
                                let tree: <S as ::netabase_store::traits::store_ops::OpenTree<#definition, #linked_type>>::Tree<'_> = <S as ::netabase_store::traits::store_ops::OpenTree<#definition, #linked_type>>::open_tree(store);
                                ::netabase_store::traits::store_ops::StoreOps::put_raw(&tree, entity.clone())
                            },
                            ::netabase_store::links::RelationalLink::Reference(_) => {
                                // Reference only - nothing to insert
                                Ok(())
                            },
                        }
                    }
                }
            }
            WrapperType::Option => {
                // Option<RelationalLink> - methods work with Option
                quote! {
                    /// Get the optional relational link for this field
                    #[doc = concat!("Get the ", stringify!(#field_name), " relation link (optional)")]
                    pub fn #method_name(&self) -> &Option<::netabase_store::links::RelationalLink<#definition, #linked_type>> {
                        &self.#field_name
                    }

                    /// Hydrate the linked entity if it exists and is a reference
                    #[doc = concat!("Hydrate the ", stringify!(#field_name), " entity from the store if present")]
                    pub fn #hydrate_method_name<T>(&self, store: T) -> Result<Option<#linked_type>, ::netabase_store::error::NetabaseError>
                    where
                        T: ::netabase_store::traits::store_ops::StoreOps<#definition, #linked_type>,
                    {
                        match &self.#field_name {
                            Some(link) => link.clone().hydrate(&store),
                            None => Ok(None),
                        }
                    }

                    /// Check if this optional field contains an entity (vs reference or None)
                    #[doc = concat!("Check if ", stringify!(#field_name), " contains a full entity")]
                    pub fn #is_entity_method_name(&self) -> bool {
                        self.#field_name.as_ref().map_or(false, |link| link.is_entity())
                    }

                    /// Check if this optional field contains a reference (vs entity or None)
                    #[doc = concat!("Check if ", stringify!(#field_name), " contains a reference")]
                    pub fn #is_reference_method_name(&self) -> bool {
                        self.#field_name.as_ref().map_or(false, |link| link.is_reference())
                    }

                    /// Insert the linked entity if this optional field contains an Entity variant
                    #[doc = concat!("Insert the ", stringify!(#field_name), " entity if it's an Entity variant")]
                    pub fn #insert_if_entity_method_name<S>(&self, store: &S) -> Result<(), ::netabase_store::error::NetabaseError>
                    where
                        S: ::netabase_store::traits::store_ops::OpenTree<#definition, #linked_type>,
                    {
                        if let Some(link) = &self.#field_name {
                            match link {
                                ::netabase_store::links::RelationalLink::Entity(entity) => {
                                    let tree: <S as ::netabase_store::traits::store_ops::OpenTree<#definition, #linked_type>>::Tree<'_> = <S as ::netabase_store::traits::store_ops::OpenTree<#definition, #linked_type>>::open_tree(store);
                                    ::netabase_store::traits::store_ops::StoreOps::put_raw(&tree, entity.clone())
                                },
                                ::netabase_store::links::RelationalLink::Reference(_) => Ok(()),
                            }
                        } else {
                            Ok(())
                        }
                    }
                }
            }
            WrapperType::Vec => {
                // Vec<RelationalLink> - methods work with collections
                quote! {
                    /// Get the vector of relational links for this field
                    #[doc = concat!("Get the ", stringify!(#field_name), " relation links (vector)")]
                    pub fn #method_name(&self) -> &Vec<::netabase_store::links::RelationalLink<#definition, #linked_type>> {
                        &self.#field_name
                    }

                    /// Hydrate all linked entities that are references
                    #[doc = concat!("Hydrate all ", stringify!(#field_name), " entities from the store")]
                    pub fn #hydrate_method_name<T>(&self, store: T) -> Result<Vec<#linked_type>, ::netabase_store::error::NetabaseError>
                    where
                        T: ::netabase_store::traits::store_ops::StoreOps<#definition, #linked_type>,
                    {
                        let mut results = Vec::new();
                        for link in &self.#field_name {
                            if let Some(entity) = link.clone().hydrate(&store)? {
                                results.push(entity);
                            }
                        }
                        Ok(results)
                    }

                    /// Check if all items in this vector contain entities (vs references)
                    #[doc = concat!("Check if all ", stringify!(#field_name), " contain full entities")]
                    pub fn #is_entity_method_name(&self) -> bool {
                        !self.#field_name.is_empty() && self.#field_name.iter().all(|link| link.is_entity())
                    }

                    /// Check if all items in this vector contain references (vs entities)
                    #[doc = concat!("Check if all ", stringify!(#field_name), " contain references")]
                    pub fn #is_reference_method_name(&self) -> bool {
                        !self.#field_name.is_empty() && self.#field_name.iter().all(|link| link.is_reference())
                    }

                    /// Insert all linked entities that are Entity variants
                    #[doc = concat!("Insert all ", stringify!(#field_name), " entities that are Entity variants")]
                    pub fn #insert_if_entity_method_name<S>(&self, store: &S) -> Result<(), ::netabase_store::error::NetabaseError>
                    where
                        S: ::netabase_store::traits::store_ops::OpenTree<#definition, #linked_type>,
                    {
                        let tree: <S as ::netabase_store::traits::store_ops::OpenTree<#definition, #linked_type>>::Tree<'_> = <S as ::netabase_store::traits::store_ops::OpenTree<#definition, #linked_type>>::open_tree(store);
                        for link in &self.#field_name {
                            if let ::netabase_store::links::RelationalLink::Entity(entity) = link {
                                ::netabase_store::traits::store_ops::StoreOps::put_raw(&tree, entity.clone())?;
                            }
                        }
                        Ok(())
                    }
                }
            }
            WrapperType::Box => {
                // Box<RelationalLink> - methods dereference the Box
                quote! {
                    /// Get the boxed relational link for this field
                    #[doc = concat!("Get the ", stringify!(#field_name), " relation link (boxed)")]
                    pub fn #method_name(&self) -> &::netabase_store::links::RelationalLink<#definition, #linked_type> {
                        &*self.#field_name
                    }

                    /// Hydrate the linked entity if it's a reference
                    #[doc = concat!("Hydrate the ", stringify!(#field_name), " entity from the store")]
                    pub fn #hydrate_method_name<T>(&self, store: T) -> Result<Option<#linked_type>, ::netabase_store::error::NetabaseError>
                    where
                        T: ::netabase_store::traits::store_ops::StoreOps<#definition, #linked_type>,
                    {
                        (*self.#field_name).clone().hydrate(&store)
                    }

                    /// Check if this boxed field contains an entity (vs reference)
                    #[doc = concat!("Check if ", stringify!(#field_name), " contains a full entity")]
                    pub fn #is_entity_method_name(&self) -> bool {
                        (*self.#field_name).is_entity()
                    }

                    /// Check if this boxed field contains a reference (vs entity)
                    #[doc = concat!("Check if ", stringify!(#field_name), " contains a reference")]
                    pub fn #is_reference_method_name(&self) -> bool {
                        (*self.#field_name).is_reference()
                    }

                    /// Insert the linked entity if this boxed field contains an Entity variant
                    #[doc = concat!("Insert the ", stringify!(#field_name), " entity if it's an Entity variant")]
                    pub fn #insert_if_entity_method_name<S>(&self, store: &S) -> Result<(), ::netabase_store::error::NetabaseError>
                    where
                        S: ::netabase_store::traits::store_ops::OpenTree<#definition, #linked_type>,
                    {
                        match &*self.#field_name {
                            ::netabase_store::links::RelationalLink::Entity(entity) => {
                                let tree: <S as ::netabase_store::traits::store_ops::OpenTree<#definition, #linked_type>>::Tree<'_> = <S as ::netabase_store::traits::store_ops::OpenTree<#definition, #linked_type>>::open_tree(store);
                                ::netabase_store::traits::store_ops::StoreOps::put_raw(&tree, entity.clone())
                            },
                            ::netabase_store::links::RelationalLink::Reference(_) => {
                                // Reference only - nothing to insert
                                Ok(())
                            },
                        }
                    }
                }
            }
        }
    }

    /// Detect if a type is wrapped in Vec/Option/Box
    fn detect_wrapper_type(field_type: &Type) -> WrapperType {
        if let Type::Path(type_path) = field_type
            && let Some(segment) = type_path.path.segments.last()
        {
            let ident = segment.ident.to_string();
            match ident.as_str() {
                "Vec" => return WrapperType::Vec,
                "Option" => return WrapperType::Option,
                "Box" => return WrapperType::Box,
                _ => {}
            }
        }
        WrapperType::None
    }

    /// Generate marker trait implementations for relation detection
    pub fn generate_relation_markers(&self) -> Option<TokenStream> {
        let model_name = self.name.expect("Model name should be available");
        let _relations_name = append_ident(model_name, "Relations");
        let definition = self
            .definitions
            .first()
            .expect("Definition should be available");

        let has_relations = !self.links.is_empty();

        Some(quote! {
            impl ::netabase_store::links::HasCustomRelationInsertion<#definition> for #model_name {
                const HAS_RELATIONS: bool = #has_relations;
            }
        })
    }

    /// Generate a type alias for backward-compatible RelationalLink usage
    /// Generate type alias for backward compatibility (per-model scope)
    ///
    /// Note: Global type aliases are disabled to prevent conflicts when multiple
    /// models with relations exist in the same scope. Users should migrate to
    /// the explicit 3-parameter RelationalLink<D, M, R> form.
    #[allow(dead_code)]
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
