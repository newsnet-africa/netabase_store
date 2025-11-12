use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, ItemStruct, parse_quote};

use crate::item_info::netabase_definitions::ModuleInfo;
use crate::util::append_ident;

/// Generate a struct that holds redb TableDefinitions for all models in the schema
///
/// This generates:
/// ```ignore
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
pub fn generate_tables_struct(
    modules: &Vec<ModuleInfo<'_>>,
    definition: &Ident,
) -> ItemStruct {
    let tables_name = Ident::new(&format!("{}Tables", definition), definition.span());

    // Generate fields for each model
    let fields: Vec<TokenStream> = modules
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

    parse_quote! {
        #[cfg(feature = "redb")]
        #[derive(Debug, Clone, Copy)]
        pub struct #tables_name {
            #(#fields),*
        }
    }
}

/// Generate the implementation for the tables struct with a const constructor
pub fn generate_tables_impl(
    modules: &Vec<ModuleInfo<'_>>,
    definition: &Ident,
) -> TokenStream {
    let tables_name = Ident::new(&format!("{}Tables", definition), definition.span());

    // Generate initializers for each field
    let field_inits: Vec<TokenStream> = modules
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
        #[cfg(feature = "redb")]
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
}

/// Convert PascalCase to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_is_uppercase = false;

    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            // Don't add underscore at the start
            if i > 0 && !prev_is_uppercase {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
            prev_is_uppercase = true;
        } else {
            result.push(ch);
            prev_is_uppercase = false;
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
        assert_eq!(to_snake_case("HTTPResponse"), "h_t_t_p_response");
        assert_eq!(to_snake_case("simpleModel"), "simple_model");
    }
}
