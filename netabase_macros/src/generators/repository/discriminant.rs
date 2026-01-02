//! Discriminant enum generator for repositories.
//!
//! Generates strongly-typed discriminant enums for definitions and models
//! within a repository, using strum for enum utilities.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::visitors::repository::RepositoryVisitor;

/// Generator for repository discriminant enums
pub struct DiscriminantGenerator<'a> {
    visitor: &'a RepositoryVisitor,
}

impl<'a> DiscriminantGenerator<'a> {
    pub fn new(visitor: &'a RepositoryVisitor) -> Self {
        Self { visitor }
    }

    /// Generate all discriminant enums for the repository
    pub fn generate(&self) -> TokenStream {
        let definition_discriminant = self.generate_definition_discriminant();
        let model_discriminant = self.generate_model_discriminant();
        let definition_enum = self.generate_definition_enum();

        quote! {
            #definition_discriminant
            #model_discriminant
            #definition_enum
        }
    }

    /// Generate the definition discriminant enum
    ///
    /// ```rust,ignore
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::AsRefStr, strum::EnumIter)]
    /// pub enum MyRepoDefinitionDiscriminant {
    ///     Employee,
    ///     Inventory,
    /// }
    /// ```
    fn generate_definition_discriminant(&self) -> TokenStream {
        let repo_name = &self.visitor.repository_name;
        let discriminant_name = format_ident!("{}DefinitionDiscriminant", repo_name);

        let variants: Vec<_> = self
            .visitor
            .definitions
            .iter()
            .map(|def| &def.name)
            .collect();

        if variants.is_empty() {
            return quote! {
                #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
                pub enum #discriminant_name {}
            };
        }

        let doc_comment = format!(
            "Discriminant enum for definitions in the `{}` repository.\n\n\
             Each variant represents a definition that belongs to this repository.",
            repo_name
        );

        quote! {
            #[doc = #doc_comment]
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::AsRefStr, strum::EnumIter)]
            pub enum #discriminant_name {
                #(#variants),*
            }
        }
    }

    /// Generate the model discriminant enum (flattened across all definitions)
    ///
    /// ```rust,ignore
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::AsRefStr, strum::EnumIter)]
    /// pub enum MyRepoModelDiscriminant {
    ///     EmployeeUser,
    ///     EmployeeShift,
    ///     InventoryProduct,
    /// }
    /// ```
    fn generate_model_discriminant(&self) -> TokenStream {
        let repo_name = &self.visitor.repository_name;
        let discriminant_name = format_ident!("{}ModelDiscriminant", repo_name);

        let variants: Vec<_> = self
            .visitor
            .definitions
            .iter()
            .flat_map(|def| {
                def.visitor
                    .models
                    .iter()
                    .map(move |model| format_ident!("{}{}", def.name, model.name))
            })
            .collect();

        if variants.is_empty() {
            return quote! {
                #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
                pub enum #discriminant_name {}
            };
        }

        let doc_comment = format!(
            "Discriminant enum for all models in the `{}` repository.\n\n\
             Each variant represents a model across all definitions, named as `{{Definition}}{{Model}}`.",
            repo_name
        );

        quote! {
            #[doc = #doc_comment]
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::AsRefStr, strum::EnumIter)]
            pub enum #discriminant_name {
                #(#variants),*
            }
        }
    }

    /// Generate the definition wrapper enum with strum discriminants
    ///
    /// ```rust,ignore
    /// #[derive(Debug, Clone, strum::EnumDiscriminants)]
    /// #[strum_discriminants(name(MyRepoDefinitionDiscriminant))]
    /// #[strum_discriminants(derive(Hash, strum::AsRefStr, strum::EnumIter))]
    /// pub enum MyRepoDefinition {
    ///     Employee(Employee),
    ///     Inventory(Inventory),
    /// }
    /// ```
    fn generate_definition_enum(&self) -> TokenStream {
        let repo_name = &self.visitor.repository_name;
        let enum_name = format_ident!("{}Definition", repo_name);

        let variants: Vec<_> = self
            .visitor
            .definitions
            .iter()
            .map(|def| {
                let name = &def.name;
                quote! { #name(#name) }
            })
            .collect();

        if variants.is_empty() {
            return quote! {
                #[derive(Debug, Clone)]
                pub enum #enum_name {}
            };
        }

        let doc_comment = format!(
            "Enum wrapping all definitions in the `{}` repository.\n\n\
             Use this for type-safe cross-definition operations within the repository.",
            repo_name
        );

        // Generate From impls for each definition
        let from_impls: Vec<_> = self
            .visitor
            .definitions
            .iter()
            .map(|def| {
                let def_name = &def.name;
                quote! {
                    impl From<#def_name> for #enum_name {
                        #[inline]
                        fn from(value: #def_name) -> Self {
                            #enum_name::#def_name(value)
                        }
                    }
                }
            })
            .collect();

        quote! {
            #[doc = #doc_comment]
            #[derive(Debug, Clone)]
            pub enum #enum_name {
                #(#variants),*
            }

            #(#from_impls)*
        }
    }
}
