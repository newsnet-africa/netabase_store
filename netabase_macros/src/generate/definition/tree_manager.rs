//! TreeManager trait implementation generator
//!
//! Generates a TreeManager implementation for each definition that provides
//! methods to access tree names by model discriminant.
//!
//! # Purpose
//!
//! The TreeManager trait provides a unified interface for:
//! - Getting main tree names by model discriminant
//! - Getting secondary tree names for a specific model
//! - Getting relational tree names for a specific model
//! - Getting subscription tree names for a specific model
//! - Getting all tree names for the entire definition
//! 
//! Phase 8 Enhancement: Added permission checking methods for hierarchical
//! access control and cross-definition relationship management.
//!
//! This enables generic code that works across any definition.

use proc_macro2::TokenStream;
use quote::quote;

use crate::parse::metadata::{ModuleMetadata, PermissionLevel};

/// Generate TreeManager trait implementation for a definition
///
/// The TreeManager trait provides discriminant-based tree name lookup,
/// which is essential for generic store operations.
///
/// # Generated Methods
///
/// - `get_main_tree_name(discriminant)` -> Option<&'static str>
/// - `get_hash_tree_name(discriminant)` -> Option<&'static str>
/// - `get_secondary_tree_names(discriminant)` -> Vec<&'static str>
/// - `get_relational_tree_names(discriminant)` -> Vec<&'static str>
/// - `get_subscription_tree_names(discriminant)` -> Vec<&'static str>
/// - `get_all_tree_names(discriminant)` -> Vec<&'static str>
/// - `get_all_definition_tree_names()` -> Vec<&'static str>
pub fn generate_tree_manager(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    let discriminants = quote::format_ident!("{}Discriminants", definition_name);

    let main_tree_method = generate_get_main_tree_name(module);
    let hash_tree_method = generate_get_hash_tree_name(module);
    let secondary_trees_method = generate_get_secondary_tree_names(module);
    let relational_trees_method = generate_get_relational_tree_names(module);
    let subscription_trees_method = generate_get_subscription_tree_names(module);
    let all_model_trees_method = generate_get_all_tree_names(module);
    let all_definition_trees_method = generate_get_all_definition_tree_names(module);

    quote! {
        /// TreeManager trait for accessing tree names by model discriminant
        /// 
        /// Phase 8 Enhancement: Now includes permission checking methods for
        /// hierarchical access control and cross-definition relationship management.
        pub trait TreeManager {
            /// Get the main tree name for a model
            fn get_main_tree_name(discriminant: &#discriminants) -> Option<&'static str>;

            /// Get the hash tree name for a model
            fn get_hash_tree_name(discriminant: &#discriminants) -> Option<&'static str>;

            /// Get all secondary tree names for a model
            fn get_secondary_tree_names(discriminant: &#discriminants) -> Vec<&'static str>;

            /// Get all relational tree names for a model
            fn get_relational_tree_names(discriminant: &#discriminants) -> Vec<&'static str>;

            /// Get all subscription tree names for a model
            fn get_subscription_tree_names(discriminant: &#discriminants) -> Vec<&'static str>;

            /// Get all tree names (main + hash + secondary + relational + subscription) for a model
            fn get_all_tree_names(discriminant: &#discriminants) -> Vec<&'static str>;

            /// Get all tree names for the entire definition (all models)
            fn get_all_definition_tree_names() -> Vec<&'static str>;
            
            /// Get permission level required to access a specific model (Phase 8)
            fn get_access_permission_required(discriminant: &#discriminants) -> PermissionLevel;
            
            /// Check if cross-definition access is allowed for a model (Phase 8)
            fn allows_cross_definition_access(discriminant: &#discriminants) -> bool;
        }

        impl TreeManager for #definition_name {
            #main_tree_method
            #hash_tree_method
            #secondary_trees_method
            #relational_trees_method
            #subscription_trees_method
            #all_model_trees_method
            #all_definition_trees_method
            
            /// Get permission level required to access a specific model
            fn get_access_permission_required(discriminant: &#discriminants) -> PermissionLevel {
                match discriminant {
                    // All models in this definition require at least Read permission
                    _ => PermissionLevel::Read,
                }
            }
            
            /// Check if cross-definition access is allowed for a model
            fn allows_cross_definition_access(discriminant: &#discriminants) -> bool {
                // TODO: Implement based on model's cross-definition fields
                false
            }
        }
    }
}

/// Generate get_main_tree_name method
fn generate_get_main_tree_name(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    let discriminants = quote::format_ident!("{}Discriminants", definition_name);

    let match_arms: Vec<_> = module.models.iter().map(|model| {
        let model_name = &model.name;
        quote! {
            #discriminants::#model_name => Some(#model_name::MAIN_TREE_NAME)
        }
    }).collect();

    // Nested modules don't have a single main tree name relative to this definition
    let nested_arms: Vec<_> = module.nested_modules.iter().map(|nested| {
        let definition_name = &nested.definition_name;
        quote! {
            #discriminants::#definition_name => None
        }
    }).collect();

    quote! {
        fn get_main_tree_name(discriminant: &#discriminants) -> Option<&'static str> {
            match discriminant {
                #(#match_arms,)*
                #(#nested_arms,)*
            }
        }
    }
}

/// Generate get_hash_tree_name method
fn generate_get_hash_tree_name(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    let discriminants = quote::format_ident!("{}Discriminants", definition_name);

    let match_arms: Vec<_> = module.models.iter().map(|model| {
        let model_name = &model.name;
        quote! {
            #discriminants::#model_name => Some(#model_name::HASH_TREE_NAME)
        }
    }).collect();

    // Nested modules don't have a single hash tree name relative to this definition
    let nested_arms: Vec<_> = module.nested_modules.iter().map(|nested| {
        let definition_name = &nested.definition_name;
        quote! {
            #discriminants::#definition_name => None
        }
    }).collect();

    quote! {
        fn get_hash_tree_name(discriminant: &#discriminants) -> Option<&'static str> {
            match discriminant {
                #(#match_arms,)*
                #(#nested_arms,)*
            }
        }
    }
}

/// Generate get_secondary_tree_names method
fn generate_get_secondary_tree_names(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    let discriminants = quote::format_ident!("{}Discriminants", definition_name);

    let match_arms: Vec<_> = module.models.iter().map(|model| {
        let model_name = &model.name;
        quote! {
            #discriminants::#model_name => #model_name::SECONDARY_TREE_NAMES.to_vec()
        }
    }).collect();

    let nested_arms: Vec<_> = module.nested_modules.iter().map(|nested| {
        let definition_name = &nested.definition_name;
        quote! {
            #discriminants::#definition_name => Vec::new()
        }
    }).collect();

    quote! {
        fn get_secondary_tree_names(discriminant: &#discriminants) -> Vec<&'static str> {
            match discriminant {
                #(#match_arms,)*
                #(#nested_arms,)*
            }
        }
    }
}

/// Generate get_relational_tree_names method
fn generate_get_relational_tree_names(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    let discriminants = quote::format_ident!("{}Discriminants", definition_name);

    let match_arms: Vec<_> = module.models.iter().map(|model| {
        let model_name = &model.name;
        quote! {
            #discriminants::#model_name => #model_name::RELATIONAL_TREE_NAMES.to_vec()
        }
    }).collect();

    let nested_arms: Vec<_> = module.nested_modules.iter().map(|nested| {
        let definition_name = &nested.definition_name;
        quote! {
            #discriminants::#definition_name => Vec::new()
        }
    }).collect();

    quote! {
        fn get_relational_tree_names(discriminant: &#discriminants) -> Vec<&'static str> {
            match discriminant {
                #(#match_arms,)*
                #(#nested_arms,)*
            }
        }
    }
}

/// Generate get_subscription_tree_names method
fn generate_get_subscription_tree_names(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    let discriminants = quote::format_ident!("{}Discriminants", definition_name);

    let match_arms: Vec<_> = module.models.iter().map(|model| {
        let model_name = &model.name;
        quote! {
            #discriminants::#model_name => #model_name::SUBSCRIPTION_TREE_NAMES.to_vec()
        }
    }).collect();

    let nested_arms: Vec<_> = module.nested_modules.iter().map(|nested| {
        let definition_name = &nested.definition_name;
        quote! {
            #discriminants::#definition_name => Vec::new()
        }
    }).collect();

    quote! {
        fn get_subscription_tree_names(discriminant: &#discriminants) -> Vec<&'static str> {
            match discriminant {
                #(#match_arms,)*
                #(#nested_arms,)*
            }
        }
    }
}

/// Generate get_all_tree_names method (for a specific model)
fn generate_get_all_tree_names(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    let discriminants = quote::format_ident!("{}Discriminants", definition_name);

    let match_arms: Vec<_> = module.models.iter().map(|model| {
        let model_name = &model.name;
        quote! {
            #discriminants::#model_name => #model_name::all_tree_names()
        }
    }).collect();

    let nested_arms: Vec<_> = module.nested_modules.iter().map(|nested| {
        let definition_name = &nested.definition_name;
        quote! {
            #discriminants::#definition_name => Vec::new()
        }
    }).collect();

    quote! {
        fn get_all_tree_names(discriminant: &#discriminants) -> Vec<&'static str> {
            match discriminant {
                #(#match_arms,)*
                #(#nested_arms,)*
            }
        }
    }
}

/// Generate get_all_definition_tree_names method (for entire definition)
fn generate_get_all_definition_tree_names(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    let discriminants = quote::format_ident!("{}Discriminants", definition_name);

    // Use strum to iterate over all discriminants
    quote! {
        fn get_all_definition_tree_names() -> Vec<&'static str> {
            use strum::IntoEnumIterator;
            let mut all_trees = Vec::new();

            for discriminant in #discriminants::iter() {
                all_trees.extend(Self::get_all_tree_names(&discriminant));
            }

            all_trees
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::metadata::{ModuleMetadata, ModelMetadata, FieldMetadata};
    use syn::parse_quote;

    #[test]
    fn test_generate_tree_manager_minimal() {
        let mut module = ModuleMetadata::new(
            parse_quote!(test_mod),
            parse_quote!(TestDef),
            parse_quote!(TestDefKeys)
        );

        let mut user = ModelMetadata::new(parse_quote!(User), parse_quote!(pub));
        let mut pk = FieldMetadata::new(
            parse_quote!(id),
            parse_quote!(u64),
            parse_quote!(pub)
        );
        pk.is_primary_key = true;
        user.add_field(pk);
        module.add_model(user);

        let result = generate_tree_manager(&module);
        let code = result.to_string();

        assert!(code.contains("trait TreeManager"));
        assert!(code.contains("fn get_main_tree_name"));
        assert!(code.contains("fn get_hash_tree_name"));
        assert!(code.contains("fn get_secondary_tree_names"));
        assert!(code.contains("fn get_relational_tree_names"));
        assert!(code.contains("fn get_subscription_tree_names"));
        assert!(code.contains("fn get_all_tree_names"));
        assert!(code.contains("fn get_all_definition_tree_names"));
        assert!(code.contains("impl TreeManager for TestDef"));
    }

    #[test]
    fn test_generate_tree_manager_multiple_models() {
        let mut module = ModuleMetadata::new(
            parse_quote!(app_mod),
            parse_quote!(AppDef),
            parse_quote!(AppDefKeys)
        );

        // Add User model
        let mut user = ModelMetadata::new(parse_quote!(User), parse_quote!(pub));
        let mut user_pk = FieldMetadata::new(
            parse_quote!(id),
            parse_quote!(u64),
            parse_quote!(pub)
        );
        user_pk.is_primary_key = true;
        user.add_field(user_pk);

        let mut email = FieldMetadata::new(
            parse_quote!(email),
            parse_quote!(String),
            parse_quote!(pub)
        );
        email.is_secondary_key = true;
        user.add_field(email);

        module.add_model(user);

        // Add Post model
        let mut post = ModelMetadata::new(parse_quote!(Post), parse_quote!(pub));
        let mut post_pk = FieldMetadata::new(
            parse_quote!(id),
            parse_quote!(u64),
            parse_quote!(pub)
        );
        post_pk.is_primary_key = true;
        post.add_field(post_pk);
        module.add_model(post);

        let result = generate_tree_manager(&module);
        let code = result.to_string();

        // Check that both models are included in match arms
        assert!(code.contains("AppDefDiscriminants :: User"));
        assert!(code.contains("AppDefDiscriminants :: Post"));
        assert!(code.contains("User :: MAIN_TREE_NAME"));
        assert!(code.contains("Post :: MAIN_TREE_NAME"));
        assert!(code.contains("User :: SECONDARY_TREE_NAMES"));
    }

    #[test]
    fn test_tree_manager_all_definition_trees() {
        let mut module = ModuleMetadata::new(
            parse_quote!(my_mod),
            parse_quote!(MyDef),
            parse_quote!(MyDefKeys)
        );

        let mut user = ModelMetadata::new(parse_quote!(User), parse_quote!(pub));
        let mut pk = FieldMetadata::new(
            parse_quote!(id),
            parse_quote!(u64),
            parse_quote!(pub)
        );
        pk.is_primary_key = true;
        user.add_field(pk);
        module.add_model(user);

        let result = generate_tree_manager(&module);
        let code = result.to_string();

        // Check that the all_definition_trees method uses strum iteration
        assert!(code.contains("get_all_definition_tree_names"));
        assert!(code.contains("use strum :: IntoEnumIterator"));
        assert!(code.contains("MyDefDiscriminants :: iter"));
    }
}
