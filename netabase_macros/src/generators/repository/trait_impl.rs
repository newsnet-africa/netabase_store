//! Repository trait implementation generator.
//!
//! Generates implementations of `NetabaseRepository` and `InRepository`
//! traits for the repository and its constituent definitions.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::visitors::repository::RepositoryVisitor;

/// Generator for repository trait implementations
pub struct RepositoryTraitGenerator<'a> {
    visitor: &'a RepositoryVisitor,
}

impl<'a> RepositoryTraitGenerator<'a> {
    pub fn new(visitor: &'a RepositoryVisitor) -> Self {
        Self { visitor }
    }

    /// Generate all trait implementations
    pub fn generate(&self) -> TokenStream {
        let in_repository_impls = self.generate_in_repository_impls();
        let netabase_repository_impl = self.generate_netabase_repository_impl();

        quote! {
            #in_repository_impls
            #netabase_repository_impl
        }
    }

    /// Generate `InRepository<RepoName>` implementations for each definition
    ///
    /// ```rust,ignore
    /// impl netabase_store::traits::registery::repository::InRepository<MyRepo> for Employee {
    ///     fn repository_discriminant() -> MyRepoDefinitionDiscriminant {
    ///         MyRepoDefinitionDiscriminant::Employee
    ///     }
    /// }
    /// ```
    fn generate_in_repository_impls(&self) -> TokenStream {
        let repo_name = &self.visitor.repository_name;
        let discriminant_name = format_ident!("{}DefinitionDiscriminant", repo_name);

        let impls: Vec<_> = self.visitor.definitions
            .iter()
            .map(|def| {
                let def_name = &def.name;
                quote! {
                    impl netabase_store::traits::registery::repository::InRepository<#repo_name> for #def_name {
                        type RepositoryDiscriminant = #discriminant_name;

                        #[inline]
                        fn repository_discriminant() -> Self::RepositoryDiscriminant {
                            #discriminant_name::#def_name
                        }
                    }
                }
            })
            .collect();

        quote! {
            #(#impls)*
        }
    }

    /// Generate `NetabaseRepository` implementation for the repository marker struct
    ///
    /// ```rust,ignore
    /// impl netabase_store::traits::registery::repository::NetabaseRepository for MyRepo {
    ///     type RepositoryDefinition = MyRepoDefinition;
    ///     type RepositoryDiscriminant = MyRepoDefinitionDiscriminant;
    ///     type RepositoryModelKeys = MyRepoModelDiscriminant;
    ///     // ... methods
    /// }
    /// ```
    fn generate_netabase_repository_impl(&self) -> TokenStream {
        let repo_name = &self.visitor.repository_name;
        let definition_enum = format_ident!("{}Definition", repo_name);
        let def_discriminant = format_ident!("{}DefinitionDiscriminant", repo_name);
        let model_discriminant = format_ident!("{}ModelDiscriminant", repo_name);

        let def_names: Vec<_> = self
            .visitor
            .definitions
            .iter()
            .map(|d| d.name.to_string())
            .collect();
        let def_count = def_names.len();

        let model_count: usize = self
            .visitor
            .definitions
            .iter()
            .map(|d| d.visitor.models.len())
            .sum();

        quote! {
            impl netabase_store::traits::registery::repository::NetabaseRepository for #repo_name {
                type RepositoryDefinition = #definition_enum;
                type RepositoryDiscriminant = #def_discriminant;
                type RepositoryModelKeys = #model_discriminant;

                #[inline]
                fn name() -> &'static str {
                    stringify!(#repo_name)
                }

                #[inline]
                fn definition_count() -> usize {
                    #def_count
                }

                #[inline]
                fn model_count() -> usize {
                    #model_count
                }
            }
        }
    }
}
