//! Secondary keys generation
//!
//! Generates wrapper types, enums, and iterators for secondary key fields.
//!
//! # Example Output
//!
//! For a `User` model with `email: String` and `username: String` as secondary keys:
//!
//! ```rust,ignore
//! // Individual wrappers
//! #[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Encode, Decode)]
//! pub struct UserEmail(pub String);
//!
//! #[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Encode, Decode)]
//! pub struct UserName(pub String);
//!
//! // Enum combining all secondary keys
//! #[derive(Debug, Clone, strum::EnumDiscriminants, Encode, Decode)]
//! #[strum_discriminants(derive(Hash, strum::AsRefStr, Encode, Decode, strum::EnumIter))]
//! #[strum_discriminants(name(UserSecondaryKeysDiscriminants))]
//! pub enum UserSecondaryKeys {
//!     Email(UserEmail),
//!     Name(UserName),
//! }
//!
//! // Tree names enum (for database tree identification)
//! #[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, strum::EnumIter, strum::AsRefStr)]
//! pub enum UserSecondaryTreeNames {
//!     Email,
//!     Name,
//! }
//!
//! // Iterator wrapper
//! pub struct UserSecondaryKeysIter;
//! ```
//!
//! Each wrapper and enum gets full redb and Sled backend implementations.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::parse::metadata::ModelMetadata;

/// Generate all secondary key structures and implementations
pub fn generate_secondary_keys(model: &ModelMetadata) -> TokenStream {
    let secondary_fields = model.secondary_key_fields();
    
    // If no secondary keys, return empty
    if secondary_fields.is_empty() {
        return quote! {};
    }
    
    let model_name = &model.name;
    
    // Generate individual wrappers for each secondary key field
    let wrappers: Vec<_> = secondary_fields.iter()
        .map(|field| generate_secondary_key_wrapper(model_name, field))
        .collect();
    
    // Generate the combined enum
    let enum_def = generate_secondary_keys_enum(model_name, &secondary_fields);
    
    // Generate tree names enum
    let tree_names_enum = generate_tree_names_enum(model_name, &secondary_fields);
    
    // Generate iterator wrapper
    let iterator_wrapper = generate_iterator_wrapper(model_name);
    
    quote! {
        #(#wrappers)*
        #enum_def
        #tree_names_enum
        #iterator_wrapper
    }
}

/// Generate a wrapper for a single secondary key field
fn generate_secondary_key_wrapper(
    model_name: &Ident,
    field: &crate::parse::metadata::FieldMetadata,
) -> TokenStream {
    let field_name = &field.name;
    let field_type = &field.ty;
    
    // Convert field name to PascalCase for wrapper name
    let field_name_pascal = pascal_case(field_name);
    let wrapper_name = format_ident!("{}{}", model_name, field_name_pascal);
    let type_name_str = wrapper_name.to_string();
    
    quote! {
        #[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, bincode::Encode, bincode::Decode)]
        pub struct #wrapper_name(pub #field_type);
        
        impl redb::Value for #wrapper_name {
            type SelfType<'a> = #wrapper_name where Self: 'a;
            type AsBytes<'a> = <#field_type as redb::Value>::AsBytes<'a> where Self: 'a;

            fn fixed_width() -> Option<usize> {
                <#field_type as redb::Value>::fixed_width()
            }

            fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
            where
                Self: 'a,
            {
                Self(<#field_type as redb::Value>::from_bytes(data))
            }

            fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
            where
                Self: 'a,
                Self: 'b,
            {
                <#field_type as redb::Value>::as_bytes(&value.0)
            }

            fn type_name() -> redb::TypeName {
                redb::TypeName::new(#type_name_str)
            }
        }
        
        impl redb::Key for #wrapper_name {
            fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
                <#field_type as redb::Key>::compare(data1, data2)
            }
        }
        
        impl TryFrom<Vec<u8>> for #wrapper_name {
            type Error = bincode::error::DecodeError;
            
            fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
                let (value, _): (#field_type, usize) =
                    bincode::decode_from_slice(&data, bincode::config::standard())?;
                Ok(#wrapper_name(value))
            }
        }

        impl TryFrom<#wrapper_name> for Vec<u8> {
            type Error = bincode::error::EncodeError;
            
            fn try_from(value: #wrapper_name) -> Result<Self, Self::Error> {
                bincode::encode_to_vec(value.0, bincode::config::standard())
            }
        }
    }
}

/// Generate the enum that combines all secondary keys
fn generate_secondary_keys_enum(
    model_name: &Ident,
    fields: &[&crate::parse::metadata::FieldMetadata],
) -> TokenStream {
    let enum_name = format_ident!("{}SecondaryKeys", model_name);
    let discriminants_name = format_ident!("{}SecondaryKeysDiscriminants", model_name);
    
    // Generate enum variants
    let variants: Vec<_> = fields.iter().map(|field| {
        let field_name = &field.name;
        let field_name_pascal = pascal_case(field_name);
        let variant_name = format_ident!("{}", field_name_pascal);
        let wrapper_name = format_ident!("{}{}", model_name, field_name_pascal);
        
        quote! {
            #variant_name(#wrapper_name)
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
    let enum_name = format_ident!("{}SecondaryTreeNames", model_name);
    
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
    let iter_name = format_ident!("{}SecondaryKeysIter", model_name);
    
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
    fn test_pascal_case() {
        let ident: Ident = parse_quote!(email_address);
        assert_eq!(pascal_case(&ident), "EmailAddress");
        
        let ident: Ident = parse_quote!(user_name);
        assert_eq!(pascal_case(&ident), "UserName");
        
        let ident: Ident = parse_quote!(id);
        assert_eq!(pascal_case(&ident), "Id");
    }

    #[test]
    fn test_generate_secondary_keys_with_fields() {
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
        
        // Add secondary keys
        let mut email_field = FieldMetadata::new(
            parse_quote!(email),
            parse_quote!(String),
            parse_quote!(pub)
        );
        email_field.is_secondary_key = true;
        model.add_field(email_field);
        
        let mut username_field = FieldMetadata::new(
            parse_quote!(username),
            parse_quote!(String),
            parse_quote!(pub)
        );
        username_field.is_secondary_key = true;
        model.add_field(username_field);
        
        let tokens = generate_secondary_keys(&model);
        let code = tokens.to_string();

        eprintln!("Generated secondary keys code:\n{}", code);

        // Verify key elements
        assert!(code.contains("pub struct UserEmail"));
        assert!(code.contains("pub struct UserUsername")); // username -> Username
        assert!(code.contains("pub enum UserSecondaryKeys"));
        assert!(code.contains("Email") && code.contains("UserEmail"));
        assert!(code.contains("Username") && code.contains("UserUsername"));
        assert!(code.contains("UserSecondaryKeysDiscriminants"));
        assert!(code.contains("UserSecondaryTreeNames"));
        assert!(code.contains("UserSecondaryKeysIter"));
    }

    #[test]
    fn test_generate_secondary_keys_no_fields() {
        let model = ModelMetadata::new(
            parse_quote!(User),
            parse_quote!(pub)
        );
        
        let tokens = generate_secondary_keys(&model);
        let code = tokens.to_string();
        
        // Should be empty when no secondary keys
        assert!(code.is_empty());
    }
}
