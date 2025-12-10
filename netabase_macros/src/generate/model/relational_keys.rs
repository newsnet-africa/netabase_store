//! Relational keys generation
//!
//! Generates enums and metadata for relational fields that link to other models.
//!
//! # Example Output
//!
//! For a `User` model with `posts: Vec<PostId>` relation:
//!
//! ```rust,ignore
//! // Enum for all relational keys (uses field types directly, no wrappers)
//! #[derive(Debug, Clone, strum::EnumDiscriminants, bincode::Encode, bincode::Decode)]
//! #[strum_discriminants(derive(Hash, strum::AsRefStr, bincode::Encode, bincode::Decode, strum::EnumIter))]
//! #[strum_discriminants(name(UserRelationalKeysDiscriminants))]
//! pub enum UserRelationalKeys {
//!     Posts(Vec<PostId>),
//! }
//!
//! // Tree names enum
//! #[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, strum::EnumIter, strum::AsRefStr)]
//! pub enum UserRelationalTreeNames {
//!     Posts,
//! }
//!
//! // Iterator wrapper
//! pub struct UserRelationalKeysIter;
//! ```

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::parse::metadata::ModelMetadata;

/// Generate all relational key structures
pub fn generate_relational_keys(model: &ModelMetadata) -> TokenStream {
    let relational_fields = model.relational_fields();
    
    // If no relational keys, return empty
    if relational_fields.is_empty() {
        return quote! {};
    }
    
    let model_name = &model.name;
    
    // Generate the combined enum
    let enum_def = generate_relational_keys_enum(model_name, &relational_fields);
    
    // Generate tree names enum
    let tree_names_enum = generate_tree_names_enum(model_name, &relational_fields);
    
    // Generate iterator wrapper
    let iterator_wrapper = generate_iterator_wrapper(model_name);
    
    quote! {
        #enum_def
        #tree_names_enum
        #iterator_wrapper
    }
}

/// Generate the enum for relational keys (uses field types directly)
fn generate_relational_keys_enum(
    model_name: &Ident,
    fields: &[&crate::parse::metadata::FieldMetadata],
) -> TokenStream {
    let enum_name = format_ident!("{}RelationalKeys", model_name);
    let discriminants_name = format_ident!("{}RelationalKeysDiscriminants", model_name);
    
    // Generate enum variants using the field types directly
    let variants: Vec<_> = fields.iter().map(|field| {
        let field_name = &field.name;
        let field_type = &field.ty;
        let field_name_pascal = pascal_case(field_name);
        let variant_name = format_ident!("{}", field_name_pascal);
        
        quote! {
            #variant_name(#field_type)
        }
    }).collect();
    
    quote! {
        #[derive(Debug, Clone, strum::EnumDiscriminants, bincode::Encode, bincode::Decode)]
        #[strum_discriminants(derive(Hash, strum::AsRefStr, bincode::Encode, bincode::Decode, strum::EnumIter))]
        #[strum_discriminants(name(#discriminants_name))]
        pub enum #enum_name {
            #(#variants,)*
        }
        
        // Implement DiscriminantName trait for discriminants
        impl crate::traits::DiscriminantName for #discriminants_name {
            fn discriminant_name(&self) -> &str {
                self.as_ref()
            }
        }
    }
}

/// Generate tree names enum for database tree identification
fn generate_tree_names_enum(
    model_name: &Ident,
    fields: &[&crate::parse::metadata::FieldMetadata],
) -> TokenStream {
    let enum_name = format_ident!("{}RelationalTreeNames", model_name);
    
    let variants: Vec<_> = fields.iter().map(|field| {
        let field_name = &field.name;
        let variant_name = pascal_case(field_name);
        format_ident!("{}", variant_name)
    }).collect();
    
    quote! {
        #[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, strum::EnumIter, strum::AsRefStr)]
        pub enum #enum_name {
            #(#variants,)*
        }
        
        impl crate::traits::DiscriminantName for #enum_name {
            fn discriminant_name(&self) -> &str {
                self.as_ref()
            }
        }
    }
}

/// Generate iterator wrapper struct
fn generate_iterator_wrapper(model_name: &Ident) -> TokenStream {
    let iter_name = format_ident!("{}RelationalKeysIter", model_name);
    
    quote! {
        pub struct #iter_name;
    }
}

/// Convert snake_case identifier to PascalCase string
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
    use crate::parse::metadata::{ModelMetadata, FieldMetadata};
    use syn::parse_quote;

    #[test]
    fn test_generate_relational_keys_with_fields() {
        let mut model = ModelMetadata::new(
            parse_quote!(User),
            parse_quote!(pub)
        );
        
        // Add primary key
        let mut pk_field = FieldMetadata::new(
            parse_quote!(id),
            parse_quote!(u64),
            parse_quote!(pub)
        );
        pk_field.is_primary_key = true;
        model.add_field(pk_field);
        
        // Add relational keys
        let mut posts_field = FieldMetadata::new(
            parse_quote!(posts),
            parse_quote!(Vec<PostId>),
            parse_quote!(pub)
        );
        posts_field.is_relation = true;
        model.add_field(posts_field);
        
        let mut comments_field = FieldMetadata::new(
            parse_quote!(user_comments),
            parse_quote!(Vec<CommentId>),
            parse_quote!(pub)
        );
        comments_field.is_relation = true;
        model.add_field(comments_field);
        
        let tokens = generate_relational_keys(&model);
        let code = tokens.to_string();
        
        // Verify key elements
        assert!(code.contains("pub enum UserRelationalKeys"));
        assert!(code.contains("Posts") && code.contains("Vec < PostId >"));
        assert!(code.contains("UserComments") && code.contains("Vec < CommentId >"));
        assert!(code.contains("UserRelationalKeysDiscriminants"));
        assert!(code.contains("UserRelationalTreeNames"));
        assert!(code.contains("UserRelationalKeysIter"));
    }

    #[test]
    fn test_generate_relational_keys_no_fields() {
        let model = ModelMetadata::new(
            parse_quote!(User),
            parse_quote!(pub)
        );
        
        let tokens = generate_relational_keys(&model);
        let code = tokens.to_string();
        
        // Should be empty when no relational keys
        assert!(code.is_empty());
    }
}
