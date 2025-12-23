use proc_macro2::TokenStream;
use quote::quote;
use crate::visitors::global::GlobalVisitor;

/// Generator for global enum that wraps all definitions
pub struct GlobalEnumGenerator<'a> {
    visitor: &'a GlobalVisitor,
}

impl<'a> GlobalEnumGenerator<'a> {
    pub fn new(visitor: &'a GlobalVisitor) -> Self {
        Self { visitor }
    }

    /// Generate the global enum wrapping all definitions
    pub fn generate_global_enum(&self) -> TokenStream {
        let global_name = &self.visitor.global_name;

        let variants: Vec<_> = self.visitor.definitions
            .iter()
            .map(|def| {
                let def_name = &def.definition_name;
                quote! { #def_name(#def_name) }
            })
            .collect();

        quote! {
            #[derive(
                Clone, Debug,
                strum::EnumDiscriminants,
                bincode::Encode, bincode::Decode,
                serde::Serialize, serde::Deserialize,
                PartialEq, Eq, PartialOrd, Ord, Hash
            )]
            #[strum_discriminants(name(GlobalTreeName))]
            #[strum_discriminants(derive(strum::AsRefStr))]
            pub enum #global_name {
                #(#variants),*
            }
        }
    }
}
