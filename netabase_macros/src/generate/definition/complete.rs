//! Complete definition code generation orchestrator
//!
//! Combines all definition-level and model-level generators to produce
//! complete boilerplate for an entire netabase definition module.

use proc_macro2::TokenStream;
use quote::quote;

use crate::parse::metadata::ModuleMetadata;
use super::{
    generate_definition_enum,
    generate_keys_enum,
    generate_associated_types_enum,
    generate_permissions_enum,
    generate_definition_trait,
    generate_associated_types_ext,
};
use crate::generate::model::generate_complete_model;
use crate::generate::model::cross_definition_links::generate_cross_definition_support_types;
use crate::generate::tree_naming::{generate_tree_naming_impl, generate_cross_definition_helpers};

/// Generate all code for a complete definition module
///
/// This orchestrates:
/// 1. Model-level code for each model (wrappers, enums, traits)
/// 2. Definition-level enums (Definition, Keys, ModelAssociatedTypes, Permissions)
/// 3. Definition-level trait implementations
pub fn generate_complete_definition(module: &ModuleMetadata) -> syn::Result<TokenStream> {
    let definition_name = &module.definition_name;
    
    // Generate code for each model
    let model_code: Result<Vec<_>, _> = module.models.iter()
        .map(|model| generate_complete_model(model, definition_name))
        .collect();
    let model_code = model_code?;
    
    // Generate definition-level enums
    let definition_enum = generate_definition_enum(module);
    let keys_enum = generate_keys_enum(module);
    let associated_types_enum = generate_associated_types_enum(module);
    let permissions_enum = generate_permissions_enum(module);

    // Generate definition-level trait implementations
    let definition_trait = generate_definition_trait(module);
    let associated_types_ext = generate_associated_types_ext(module);

    // Generate tree naming helpers for each model
    let tree_naming: Vec<_> = module.models.iter()
        .map(|model| generate_tree_naming_impl(model, definition_name))
        .collect();

    // Generate cross-definition lookup helpers
    let cross_def_helpers = generate_cross_definition_helpers(module);

    // Generate backend-specific extension traits
    let redb_extension = super::generate_redb_extension(module);
    let sled_extension = super::generate_sled_extension(module);

    // Generate TreeManager implementation
    let tree_manager = super::generate_tree_manager(module);
    
    // Generate cross-definition support types (Phase 8)
    let cross_definition_support = generate_cross_definition_support_types();

    // Combine everything
    Ok(quote! {
        // Cross-definition support types (Phase 8) - placed first for availability
        #cross_definition_support
        
        // Model-level code (wrappers, enums, traits for each model)
        #(#model_code)*

        // Definition-level enums
        #definition_enum
        #keys_enum
        #associated_types_enum
        #permissions_enum

        // Definition-level trait implementations
        #definition_trait
        #associated_types_ext

        // Tree naming implementations (standardized format)
        #(#tree_naming)*

        // Cross-definition helpers
        #cross_def_helpers

        // Backend-specific extension traits (Phase 6)
        #redb_extension
        #sled_extension

        // TreeManager trait implementation (Phase 7)
        #tree_manager

        // Cross-definition linking complete (Phase 8)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::metadata::{ModuleMetadata, ModelMetadata, FieldMetadata};
    use syn::parse_quote;

    #[test]
    fn test_generate_complete_definition_minimal() {
        let mut module = ModuleMetadata::new(
            parse_quote!(my_mod),
            parse_quote!(MyDef),
            parse_quote!(MyDefKeys)
        );
        
        // Add one simple model
        let mut user = ModelMetadata::new(parse_quote!(User), parse_quote!(pub));
        let mut user_id = FieldMetadata::new(
            parse_quote!(id),
            parse_quote!(u64),
            parse_quote!(pub)
        );
        user_id.is_primary_key = true;
        user.add_field(user_id);
        module.add_model(user);
        
        let result = generate_complete_definition(&module);
        assert!(result.is_ok());
        
        let code = result.unwrap().to_string();
        
        // Verify model-level code
        assert!(code.contains("pub struct UserId"));
        
        // Verify definition-level code
        assert!(code.contains("pub enum MyDef"));
        assert!(code.contains("pub enum MyDefKeys"));
        assert!(code.contains("pub enum MyDefModelAssociatedTypes"));
        assert!(code.contains("pub enum MyDefPermissions"));
    }

    #[test]
    fn test_generate_complete_definition_multiple_models() {
        let mut module = ModuleMetadata::new(
            parse_quote!(app_mod),
            parse_quote!(AppDef),
            parse_quote!(AppDefKeys)
        );
        
        // Add User model
        let mut user = ModelMetadata::new(parse_quote!(User), parse_quote!(pub));
        let mut user_id = FieldMetadata::new(
            parse_quote!(id),
            parse_quote!(u64),
            parse_quote!(pub)
        );
        user_id.is_primary_key = true;
        user.add_field(user_id);
        
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
        let mut post_id = FieldMetadata::new(
            parse_quote!(id),
            parse_quote!(u64),
            parse_quote!(pub)
        );
        post_id.is_primary_key = true;
        post.add_field(post_id);
        module.add_model(post);
        
        let result = generate_complete_definition(&module);
        assert!(result.is_ok());
        
        let code = result.unwrap().to_string();
        
        // Verify both models
        assert!(code.contains("pub struct UserId"));
        assert!(code.contains("pub struct PostId"));
        assert!(code.contains("pub struct UserEmail"));
        assert!(code.contains("pub enum UserSecondaryKeys"));
        
        // Verify definition enums include both models
        assert!(code.contains("User") && code.contains("User"));
        assert!(code.contains("Post") && code.contains("Post"));
    }
}
