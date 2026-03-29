//! Key visitor using FlowVisitor derive macro.
//!
//! This module demonstrates the use of flow derive macros for simpler
//! visitor patterns within rewrite_macros.

use proc_macro_flow::{FlowVisitor, FlowAttribute};
use syn::{Generics, Ident};

/// A simple visitor for key types that extracts basic struct metadata.
/// Uses FlowVisitor derive to auto-generate the From<&DeriveInput> impl.
#[derive(FlowVisitor, FlowAttribute)]
#[visit(no_traversal)]
pub struct KeyVisitor {
    /// The identifier of the type
    #[visit(ident)]
    pub ident: Ident,
    /// The generics of the type
    #[visit(generics)]
    pub generics: Generics,
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{DeriveInput, parse_quote};

    #[test]
    fn test_key_visitor_extraction() {
        let input: DeriveInput = parse_quote! {
            struct MyKey {
                id: u64,
            }
        };

        let visitor = KeyVisitor::from(&input);
        assert_eq!(visitor.ident.to_string(), "MyKey");
        assert!(visitor.generics.params.is_empty());
    }

    #[test]
    fn test_key_visitor_with_generics() {
        let input: DeriveInput = parse_quote! {
            struct GenericKey<T, U> {
                key: T,
                value: U,
            }
        };

        let visitor = KeyVisitor::from(&input);
        assert_eq!(visitor.ident.to_string(), "GenericKey");
        assert_eq!(visitor.generics.params.len(), 2);
    }
}
