//! Standardized tree naming convention
//!
//! Implements the naming format: {Definition}::{Model}::{TreeType}::{TreeName}
//!
//! # Format
//! - Main tree: `{Def}::{Model}::Main`
//! - Secondary: `{Def}::{Model}::Secondary::{KeyName}`
//! - Relational: `{Def}::{Model}::Relational::{LinkName}`
//! - Subscription: `{Def}::{Model}::Subscription::{SubName}`
//! - Hash tree: `{Def}::{Model}::Hash`
//!
//! # Benefits
//! 1. Namespace isolation prevents collisions
//! 2. Predictable - tree name is deterministic
//! 3. Cross-definition lookup works seamlessly
//! 4. Easy parsing by splitting on `::`

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::parse::metadata::{ModelMetadata, ModuleMetadata};

/// Generate tree name constants and helper functions for a model
pub fn generate_tree_naming_impl(
    model: &ModelMetadata,
    definition_name: &Ident,
) -> TokenStream {
    let model_name = &model.name;
    let def_str = definition_name.to_string();
    let model_str = model_name.to_string();
    
    // Main tree name
    let main_tree = format!("{}::{}::Main", def_str, model_str);
    
    // Hash tree name
    let hash_tree = format!("{}::{}::Hash", def_str, model_str);
    
    // Secondary tree names
    let secondary_trees: Vec<_> = model.secondary_key_fields()
        .iter()
        .map(|field| {
            let field_name = pascal_case(&field.name);
            format!("{}::{}::Secondary::{}", def_str, model_str, field_name)
        })
        .collect();
    
    // Relational tree names
    let relational_trees: Vec<_> = model.relational_fields()
        .iter()
        .map(|field| {
            let field_name = pascal_case(&field.name);
            format!("{}::{}::Relational::{}", def_str, model_str, field_name)
        })
        .collect();
    
    // Subscription tree names
    let subscription_trees: Vec<_> = model.subscriptions
        .iter()
        .map(|sub| {
            let sub_str = sub.to_string();
            format!("{}::{}::Subscription::{}", def_str, model_str, sub_str)
        })
        .collect();
    
    // Generate constants
    let secondary_count = secondary_trees.len();
    let relational_count = relational_trees.len();
    let subscription_count = subscription_trees.len();
    
    quote! {
        // Tree name constants for efficient access
        impl #model_name {
            /// Main tree name (primary key storage)
            pub const MAIN_TREE_NAME: &'static str = #main_tree;
            
            /// Hash tree name (content-addressed storage)
            pub const HASH_TREE_NAME: &'static str = #hash_tree;
            
            /// All secondary index tree names
            pub const SECONDARY_TREE_NAMES: [&'static str; #secondary_count] = [
                #(#secondary_trees,)*
            ];
            
            /// All relational link tree names
            pub const RELATIONAL_TREE_NAMES: [&'static str; #relational_count] = [
                #(#relational_trees,)*
            ];
            
            /// All subscription tree names
            pub const SUBSCRIPTION_TREE_NAMES: [&'static str; #subscription_count] = [
                #(#subscription_trees,)*
            ];
            
            /// Get the main tree name for this model
            pub fn main_tree_name() -> &'static str {
                Self::MAIN_TREE_NAME
            }
            
            /// Get all tree names for this model
            pub fn all_tree_names() -> Vec<&'static str> {
                let mut names = vec![Self::MAIN_TREE_NAME, Self::HASH_TREE_NAME];
                names.extend_from_slice(&Self::SECONDARY_TREE_NAMES);
                names.extend_from_slice(&Self::RELATIONAL_TREE_NAMES);
                names.extend_from_slice(&Self::SUBSCRIPTION_TREE_NAMES);
                names
            }
        }
    }
}

/// Generate cross-definition tree lookup helpers
pub fn generate_cross_definition_helpers(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    let def_str = definition_name.to_string();
    
    // Generate lookup functions for each model
    let model_lookups: Vec<_> = module.models.iter().map(|model| {
        let model_name = &model.name;
        let model_str = model_name.to_string();
        let main_tree = format!("{}::{}::Main", def_str, model_str);
        
        quote! {
            #model_name => Some(#main_tree),
        }
    }).collect();

    // Nested definitions don't have a main tree in this store
    let nested_lookups: Vec<_> = module.nested_modules.iter().map(|nested| {
        let nested_name = &nested.definition_name;
        quote! {
            #nested_name => None,
        }
    }).collect();
    
    let discriminants = quote::format_ident!("{}Discriminants", definition_name);
    
    quote! {
        impl #definition_name {
            /// Look up the main tree name for any model in this definition
            pub fn lookup_main_tree(model: &#discriminants) -> Option<&'static str> {
                match model {
                    #(#discriminants::#model_lookups)*
                    #(#discriminants::#nested_lookups)*
                }
            }
            
            /// Get the definition name prefix for tree naming
            pub fn definition_prefix() -> &'static str {
                #def_str
            }
        }
    }
}

/// Convert snake_case to PascalCase
fn pascal_case(ident: &Ident) -> String {
    ident.to_string()
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::metadata::{ModelMetadata, FieldMetadata, ModuleMetadata};
    use syn::parse_quote;

    #[test]
    fn test_tree_naming_format() {
        let mut model = ModelMetadata::new(parse_quote!(User), parse_quote!(pub));
        
        let mut pk = FieldMetadata::new(
            parse_quote!(id),
            parse_quote!(u64),
            parse_quote!(pub)
        );
        pk.is_primary_key = true;
        model.add_field(pk);
        
        let mut email = FieldMetadata::new(
            parse_quote!(email),
            parse_quote!(String),
            parse_quote!(pub)
        );
        email.is_secondary_key = true;
        model.add_field(email);
        
        model.add_subscription(parse_quote!(Updates));
        
        let def_name = parse_quote!(UserDef);
        let tokens = generate_tree_naming_impl(&model, &def_name);
        let code = tokens.to_string();
        
        // Verify standardized format
        assert!(code.contains("UserDef") && code.contains("User") && code.contains("Main"));
        assert!(code.contains("Hash"));
        assert!(code.contains("Secondary") && code.contains("Email"));
        assert!(code.contains("Subscription") && code.contains("Updates"));
    }

    #[test]
    fn test_generate_cross_definition_helpers() {
        let mut module = ModuleMetadata::new(
            parse_quote!(app_mod),
            parse_quote!(AppDef),
            parse_quote!(AppDefKeys)
        );
        
        let mut user = ModelMetadata::new(parse_quote!(User), parse_quote!(pub));
        let mut user_pk = FieldMetadata::new(
            parse_quote!(id),
            parse_quote!(u64),
            parse_quote!(pub)
        );
        user_pk.is_primary_key = true;
        user.add_field(user_pk);
        module.add_model(user);
        
        let tokens = generate_cross_definition_helpers(&module);
        let code = tokens.to_string();
        
        assert!(code.contains("lookup_main_tree"));
        assert!(code.contains("AppDef") && code.contains("User") && code.contains("Main"));
    }
}
