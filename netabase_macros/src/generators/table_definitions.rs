use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, ItemStruct, parse_quote};

use crate::item_info::netabase_definitions::ModuleInfo;
use crate::util::append_ident;

/// Generate a struct that holds redb TableDefinitions for all models in the schema
///
/// This generates:
/// ```no_run
/// pub struct {Definition}Tables {
///     pub users: redb::TableDefinition<'static, UserPrimaryKey, User>,
///     pub posts: redb::TableDefinition<'static, PostPrimaryKey, Post>,
/// }
///
/// impl {Definition}Tables {
///     pub const fn new() -> Self {
///         Self {
///             users: redb::TableDefinition::new("users"),
///             posts: redb::TableDefinition::new("posts"),
///         }
///     }
/// }
/// ```
pub fn generate_tables_struct(modules: &Vec<ModuleInfo<'_>>, definition: &Ident) -> ItemStruct {
    let tables_name = Ident::new(&format!("{}Tables", definition), definition.span());

    // Generate fields for each model
    let _fields: Vec<TokenStream> = modules
        .iter()
        .flat_map(|module| {
            module.models.iter().map(|model_struct| {
                let model_name = &model_struct.ident;
                let field_name = to_snake_case(&model_name.to_string());
                let field_ident = Ident::new(&field_name, model_name.span());

                // Build full paths to model and key types
                let mut model_path = module.path.clone();
                model_path.push(model_name.clone().into());

                let key_name = append_ident(model_name, "PrimaryKey");
                let mut key_path = module.path.clone();
                key_path.push(key_name.into());

                // Wrap in BincodeWrapper for compatibility with current implementation
                // This allows models without native redb::Value impls to work
                quote! {
                    pub #field_ident: ::netabase_store::redb::TableDefinition<
                        'static,
                        ::netabase_store::databases::redb_store::BincodeWrapper<#key_path>,
                        ::netabase_store::databases::redb_store::BincodeWrapper<#model_path>
                    >
                }
            })
        })
        .collect();

    #[cfg(feature = "redb")]
    let out = parse_quote! {
        #[derive(Clone, Copy)]
        pub struct #tables_name {
            #(#_fields),*
        }
    };
    #[cfg(not(feature = "redb"))]
    let out = parse_quote! {
        #[derive(Clone, Copy)]
        pub struct #tables_name;
    };
    out
}

/// Generate the implementation for the tables struct with a const constructor
pub fn generate_tables_impl(_modules: &Vec<ModuleInfo<'_>>, definition: &Ident) -> TokenStream {
    let tables_name = Ident::new(&format!("{}Tables", definition), definition.span());

    // Conditionally generate based on macro's compile-time features
    #[cfg(feature = "redb")]
    let impl_block = {
        // Generate initializers for each field
        let field_inits: Vec<TokenStream> = _modules
            .iter()
            .flat_map(|module| {
                module.models.iter().map(|model_struct| {
                    let model_name = &model_struct.ident;
                    let field_name = to_snake_case(&model_name.to_string());
                    let field_ident = Ident::new(&field_name, model_name.span());

                    // Table name is the snake_case version of the model name
                    let table_name_str = field_name.clone();

                    quote! {
                        #field_ident: ::netabase_store::redb::TableDefinition::new(#table_name_str)
                    }
                })
            })
            .collect();

        quote! {
            impl #tables_name {
                /// Create a new tables definition with all table names initialized
                ///
                /// This is a const function so it can be used in static contexts.
                pub const fn new() -> Self {
                    Self {
                        #(#field_inits),*
                    }
                }
            }
        }
    };

    #[cfg(not(feature = "redb"))]
    let impl_block = {
        quote! {
            impl #tables_name {
                /// Create a new empty tables definition (redb feature not enabled)
                ///
                /// This is a placeholder when redb is not enabled.
                pub const fn new() -> Self {
                    Self
                }
            }
        }
    };

    impl_block
}

/// Convert PascalCase to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, ch) in chars.iter().enumerate() {
        if ch.is_uppercase() {
            // Add underscore before uppercase letter if:
            // 1. Not at the start
            // 2. Previous char is lowercase OR next char is lowercase (handles sequences like "HTTPResponse")
            if i > 0
                && (chars[i - 1].is_lowercase()
                    || (i + 1 < chars.len() && chars[i + 1].is_lowercase()))
            {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(*ch);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("User"), "user");
        assert_eq!(to_snake_case("UserPost"), "user_post");
        assert_eq!(to_snake_case("HTTPResponse"), "http_response");
        assert_eq!(to_snake_case("simpleModel"), "simple_model");
    }
}
