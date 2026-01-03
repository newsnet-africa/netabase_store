//! Repository code generators.
//!
//! This module contains generators for repository-level code including:
//! - Marker structs for collision prevention
//! - Discriminant enums for type-safe repository/definition identification
//! - Repository trait implementations
//! - Schema TOML and hash generation
//! - Store management for multi-definition databases

mod discriminant;
mod marker;
mod schema;
mod store;
mod trait_impl;

pub use discriminant::DiscriminantGenerator;
pub use marker::MarkerGenerator;
pub use schema::SchemaGenerator;
pub use store::StoreGenerator;
pub use trait_impl::RepositoryTraitGenerator;

use proc_macro2::TokenStream;
use quote::quote;

use crate::visitors::repository::RepositoryVisitor;

/// Main repository generator that coordinates all sub-generators
pub struct RepositoryGenerator<'a> {
    visitor: &'a RepositoryVisitor,
}

impl<'a> RepositoryGenerator<'a> {
    pub fn new(visitor: &'a RepositoryVisitor) -> Self {
        Self { visitor }
    }

    /// Generate all repository code
    pub fn generate(&self) -> TokenStream {
        let marker = MarkerGenerator::new(self.visitor).generate();
        let discriminants = DiscriminantGenerator::new(self.visitor).generate();
        let trait_impls = RepositoryTraitGenerator::new(self.visitor).generate();
        let schema = SchemaGenerator::new(self.visitor).generate();
        let store = StoreGenerator::new(self.visitor).generate();

        quote! {
            #marker
            #discriminants
            #trait_impls
            #schema
            #store
        }
    }
}
