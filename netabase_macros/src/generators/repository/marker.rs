//! Marker struct generator for repository collision prevention.
//!
//! Generates zero-sized marker structs that uniquely identify repositories
//! at the type level, preventing accidental discriminant collisions.

use proc_macro2::TokenStream;
use quote::quote;

use crate::visitors::repository::RepositoryVisitor;

/// Generator for repository marker structs
pub struct MarkerGenerator<'a> {
    visitor: &'a RepositoryVisitor,
}

impl<'a> MarkerGenerator<'a> {
    pub fn new(visitor: &'a RepositoryVisitor) -> Self {
        Self { visitor }
    }

    /// Generate the marker struct for the repository
    ///
    /// This creates a zero-sized type (ZST) that uniquely identifies the repository:
    /// ```rust,ignore
    /// /// Marker type for the `MyRepo` repository.
    /// ///
    /// /// This type uniquely identifies the repository at compile time
    /// /// and prevents cross-repository access via type-level constraints.
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    /// pub struct MyRepo;
    /// ```
    pub fn generate(&self) -> TokenStream {
        let repo_name = &self.visitor.repository_name;
        let doc_comment = format!(
            "Marker type for the `{}` repository.\n\n\
             This type uniquely identifies the repository at compile time\n\
             and prevents cross-repository access via type-level constraints.",
            repo_name
        );

        quote! {
            #[doc = #doc_comment]
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
            pub struct #repo_name;
        }
    }
}
