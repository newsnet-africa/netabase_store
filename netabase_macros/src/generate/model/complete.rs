//! Complete model code generation orchestrator
//!
//! This module combines all individual generators to produce the complete
//! boilerplate code for a single model.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::parse::metadata::ModelMetadata;
use super::{
    generate_primary_key,
    generate_secondary_keys,
    generate_relational_keys,
    generate_subscription_keys,
    generate_model_trait,
};

/// Generate all boilerplate code for a single model
///
/// This is the main entry point that orchestrates all generators.
/// It produces:
/// - Primary key wrapper
/// - Secondary key wrappers and enums
/// - Relational key enums
/// - Subscription enums
/// - NetabaseModelTrait implementation
///
/// # Arguments
/// - `model`: Parsed metadata for the model
/// - `definition_name`: Name of the definition enum this model belongs to
pub fn generate_complete_model(
    model: &ModelMetadata,
    definition_name: &Ident,
) -> syn::Result<TokenStream> {
    // Generate primary key wrapper
    let primary_key = generate_primary_key(model)?;
    
    // Generate secondary keys (returns empty if none)
    let secondary_keys = generate_secondary_keys(model);
    
    // Generate relational keys (returns empty if none)
    let relational_keys = generate_relational_keys(model);
    
    // Generate subscription keys (returns empty if none)
    let subscription_keys = generate_subscription_keys(model);
    
    // Generate cross-definition link wrappers (Phase 8)
    let cross_definition_links = generate_cross_definition_links_for_model(model, definition_name)?;

    // Generate NetabaseModelTrait implementation
    let model_trait = generate_model_trait(model, definition_name)?;

    // Combine all generated code
    Ok(quote! {
        // Key wrappers and enums
        #primary_key
        #secondary_keys
        #relational_keys
        #subscription_keys
        
        // Cross-definition links (Phase 8)
        #cross_definition_links

        // Trait implementation
        #model_trait
    })
}

/// Generate cross-definition links specifically for one model
fn generate_cross_definition_links_for_model(
    model: &ModelMetadata, 
    definition_name: &Ident
) -> syn::Result<TokenStream> {
    // Check if this model has any cross-definition links
    let has_cross_links = model.fields.iter()
        .any(|field| field.cross_definition_link.is_some());
    
    if !has_cross_links {
        return Ok(TokenStream::new());
    }
    
    // Import the function from the cross_definition_links module
    use crate::generate::model::cross_definition_links::generate_model_cross_definition_links;
    generate_model_cross_definition_links(model, definition_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::metadata::{ModelMetadata, FieldMetadata};
    use syn::parse_quote;

    #[test]
    fn test_generate_complete_model_minimal() {
        // Minimal model with just primary key
        let mut model = ModelMetadata::new(
            parse_quote!(User),
            parse_quote!(pub)
        );
        
        let mut pk_field = FieldMetadata::new(
            parse_quote!(id),
            parse_quote!(u64),
            parse_quote!(pub)
        );
        pk_field.is_primary_key = true;
        model.add_field(pk_field);
        
        let definition_name = parse_quote!(UserDefinition);
        let result = generate_complete_model(&model, &definition_name);
        assert!(result.is_ok());
        
        let code = result.unwrap().to_string();
        
        // Should have primary key
        assert!(code.contains("pub struct UserId"));
    }

    #[test]
    fn test_generate_complete_model_full() {
        // Full model with all features
        let mut model = ModelMetadata::new(
            parse_quote!(User),
            parse_quote!(pub)
        );
        
        // Primary key
        let mut pk_field = FieldMetadata::new(
            parse_quote!(id),
            parse_quote!(u64),
            parse_quote!(pub)
        );
        pk_field.is_primary_key = true;
        model.add_field(pk_field);
        
        // Secondary key
        let mut email_field = FieldMetadata::new(
            parse_quote!(email),
            parse_quote!(String),
            parse_quote!(pub)
        );
        email_field.is_secondary_key = true;
        model.add_field(email_field);
        
        // Relational key
        let mut posts_field = FieldMetadata::new(
            parse_quote!(posts),
            parse_quote!(Vec<PostId>),
            parse_quote!(pub)
        );
        posts_field.is_relation = true;
        model.add_field(posts_field);
        
        // Subscription
        model.add_subscription(parse_quote!(Updates));
        
        let definition_name = parse_quote!(UserDefinition);
        let result = generate_complete_model(&model, &definition_name);
        assert!(result.is_ok());
        
        let code = result.unwrap().to_string();
        
        // Verify all components
        assert!(code.contains("pub struct UserId"));
        assert!(code.contains("pub struct UserEmail"));
        assert!(code.contains("pub enum UserSecondaryKeys"));
        assert!(code.contains("pub enum UserRelationalKeys"));
        assert!(code.contains("pub enum UserSubscriptions"));
    }
}
