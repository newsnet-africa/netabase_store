use crate::visitors::model::field::ModelFieldVisitor;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// Generator for immutable model constructors and accessors
pub struct ConstructorGenerator<'a> {
    visitor: &'a ModelFieldVisitor,
}

impl<'a> ConstructorGenerator<'a> {
    pub fn new(visitor: &'a ModelFieldVisitor) -> Self {
        Self { visitor }
    }

    /// Generate the submodule containing constructor and accessors
    pub fn generate(&self) -> TokenStream {
        // Only generate for immutable models
        let Some(subs) = &self.visitor.subscriptions else {
            return TokenStream::new();
        };
        if !subs.immutable {
            return TokenStream::new();
        }

        let model_name = &self.visitor.model_name;
        // Generate submodule name: snake_case of ModelName + _ctor
        let model_name_str = model_name.to_string();
        // Simple snake case conversion for module name
        let module_name_str = format!("{}_ctor", to_snake_case(&model_name_str));
        let module_name = format_ident!("{}", module_name_str);

        // Collect all fields for the constructor args and struct initialization
        let fields = self.visitor.all_fields();

        let ctor_args: Vec<_> = fields
            .iter()
            .map(|f| {
                let name = &f.name;
                let ty = &f.ty;
                quote! { #name: #ty }
            })
            .collect();

        let field_inits: Vec<_> = fields
            .iter()
            .map(|f| {
                let name = &f.name;
                quote! { #name }
            })
            .collect();

        // Generate getters for all fields
        let getters: Vec<_> = fields
            .iter()
            .map(|f| {
                let name = &f.name;
                let ty = &f.ty;
                let getter_name = format_ident!("get_{}", name);

                quote! {
                    #[inline]
                    pub fn #getter_name(&self) -> &#ty {
                        &self.#name
                    }
                }
            })
            .collect();

        quote! {
            pub mod #module_name {
                use super::*;

                /// Immutable constructor. Returns the model hash and the model.
                pub fn new(
                    #(#ctor_args),*
                ) -> Result<(netabase_store::subscription_hash::ModelHash, #model_name), Box<dyn std::error::Error>> {
                    let model = #model_name {
                        #(#field_inits),*
                    };

                    let hash = netabase_store::subscription_hash::ModelHash::from_data(&model)?;

                    Ok((hash, model))
                }
            }

            // Implement getters on the model itself
            impl #model_name {
                #(#getters)*
            }
        }
    }
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}
