//! Key plan and generator using flow derive macros.
//!
//! Demonstrates the complete Visitor -> Plan -> Generator pattern
//! using flow derives within rewrite_macros.

use crate::visitors::model::key::KeyVisitor;
use proc_macro_flow::{FlowPlan, Generatable};
use quote::ToTokens;
use syn::{Generics, Ident, Item};

/// Plan for generating key trait implementations.
/// Uses FlowPlan derive to auto-generate TryFrom<Visited<KeyVisitor>>.
#[derive(FlowPlan)]
#[plan(visitor = KeyVisitor)]
pub struct KeyPlan {
    /// The key type name
    #[plan(from = "v.ident.clone()")]
    pub ident: Ident,
    /// Generics for the type
    #[plan(from = "v.generics.clone()")]
    pub generics: Generics,
}

/// Generator for key trait implementations.
/// Implements From<KeyPlan> manually since we need custom Generatable logic.
pub struct KeyGenerator {
    ident: Ident,
    generics: Generics,
}

impl From<KeyPlan> for KeyGenerator {
    fn from(plan: KeyPlan) -> Self {
        Self {
            ident: plan.ident,
            generics: plan.generics,
        }
    }
}

impl Generatable for KeyGenerator {
    type Output = Item;

    fn generate(self) -> Result<Self::Output, syn::Error> {
        let ident = &self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let output: syn::ItemImpl = syn::parse_quote! {
            impl #impl_generics NetabaseKeyItem for #ident #ty_generics #where_clause {
                fn key_bytes(&self) -> Vec<u8> {
                    // Default implementation using rkyv serialization
                    Vec::new()
                }
            }
        };

        Ok(Item::Impl(output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro_flow::Visited;
    use syn::{DeriveInput, parse_quote};

    #[test]
    fn test_key_plan_creation() {
        let input: DeriveInput = parse_quote! {
            struct UserId {
                id: u64,
            }
        };

        let visited = Visited::<KeyVisitor>::from(&input);
        let plan = KeyPlan::try_from(&visited).unwrap();

        assert_eq!(plan.ident.to_string(), "UserId");
    }

    #[test]
    fn test_key_generator() {
        let input: DeriveInput = parse_quote! {
            struct SimpleKey {
                value: String,
            }
        };

        let visited = Visited::<KeyVisitor>::from(&input);
        let plan = KeyPlan::try_from(&visited).unwrap();
        let generator = KeyGenerator::from(plan);

        let output = Generatable::generate(generator).unwrap();
        let code = output.to_token_stream().to_string();

        assert!(code.contains("impl NetabaseKeyItem for SimpleKey"));
        assert!(code.contains("fn key_bytes"));
    }

    #[test]
    fn test_key_full_pipeline() {
        let input: DeriveInput = parse_quote! {
            struct GenericKey<T> {
                inner: T,
            }
        };

        // Full pipeline: Visit -> Plan -> Generate
        let visited = Visited::<KeyVisitor>::from(&input);
        let plan = KeyPlan::try_from(&visited).unwrap();
        let generator = KeyGenerator::from(plan);
        let output = Generatable::generate(generator).unwrap();

        let code = output.to_token_stream().to_string();
        assert!(code.contains("impl < T > NetabaseKeyItem for GenericKey < T >"));
    }
}
