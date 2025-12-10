//! Primary key wrapper generation
//!
//! Generates wrapper types for primary keys with all necessary trait implementations.
//!
//! # Example Output
//!
//! For a `User` model with `id: u64` as primary key, generates:
//!
//! ```rust,ignore
//! #[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Encode, Decode)]
//! pub struct UserId(pub u64);
//!
//! // redb::Value implementation (delegates to inner type)
//! impl redb::Value for UserId { ... }
//!
//! // redb::Key implementation (delegates to inner type)
//! impl redb::Key for UserId { ... }
//!
//! // Sled backend: bincode serialization
//! impl TryFrom<Vec<u8>> for UserId { ... }
//! impl TryFrom<UserId> for Vec<u8> { ... }
//! ```

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::parse::metadata::ModelMetadata;

/// Generate primary key wrapper and implementations
pub fn generate_primary_key(model: &ModelMetadata) -> syn::Result<TokenStream> {
    let model_name = &model.name;
    
    // Get primary key field
    let pk_field = model.primary_key_field()
        .ok_or_else(|| syn::Error::new_spanned(
            model_name,
            format!("Model '{}' must have a primary key field", model_name)
        ))?;
    
    let pk_type = &pk_field.ty;
    let wrapper_name = format_ident!("{}Id", model_name);
    let type_name_str = wrapper_name.to_string();
    
    // Generate the wrapper struct
    let wrapper_struct = generate_wrapper_struct(&wrapper_name, pk_type);
    
    // Generate redb::Value implementation
    let value_impl = generate_redb_value_impl(&wrapper_name, pk_type, &type_name_str);
    
    // Generate redb::Key implementation
    let key_impl = generate_redb_key_impl(&wrapper_name, pk_type);
    
    // Generate Sled backend conversions (bincode)
    let sled_conversions = generate_sled_conversions(&wrapper_name, pk_type);
    
    Ok(quote! {
        #wrapper_struct
        #value_impl
        #key_impl
        #sled_conversions
    })
}

/// Generate the wrapper struct with derives
fn generate_wrapper_struct(wrapper_name: &Ident, inner_type: &syn::Type) -> TokenStream {
    quote! {
        #[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, bincode::Encode, bincode::Decode)]
        pub struct #wrapper_name(pub #inner_type);
    }
}

/// Generate redb::Value implementation that delegates to inner type
fn generate_redb_value_impl(
    wrapper_name: &Ident,
    inner_type: &syn::Type,
    type_name_str: &str,
) -> TokenStream {
    quote! {
        impl redb::Value for #wrapper_name {
            type SelfType<'a> = #wrapper_name;
            type AsBytes<'a> = <#inner_type as redb::Value>::AsBytes<'a>;

            fn fixed_width() -> Option<usize> {
                <#inner_type as redb::Value>::fixed_width()
            }

            fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
            where
                Self: 'a,
            {
                Self(<#inner_type as redb::Value>::from_bytes(data))
            }

            fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
            where
                Self: 'a,
                Self: 'b,
            {
                <#inner_type as redb::Value>::as_bytes(&value.0)
            }

            fn type_name() -> redb::TypeName {
                redb::TypeName::new(#type_name_str)
            }
        }
    }
}

/// Generate redb::Key implementation that delegates to inner type
fn generate_redb_key_impl(wrapper_name: &Ident, inner_type: &syn::Type) -> TokenStream {
    quote! {
        impl redb::Key for #wrapper_name {
            fn compare(data1: &[u8], data2: &[u8]) -> std::cmp::Ordering {
                <#inner_type as redb::Key>::compare(data1, data2)
            }
        }
    }
}

/// Generate TryFrom conversions for Sled backend (uses bincode)
fn generate_sled_conversions(wrapper_name: &Ident, inner_type: &syn::Type) -> TokenStream {
    quote! {
        // Deserialize from Vec<u8> (Sled -> Wrapper)
        impl TryFrom<Vec<u8>> for #wrapper_name {
            type Error = bincode::error::DecodeError;
            
            fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
                let (value, _): (#inner_type, usize) =
                    bincode::decode_from_slice(&data, bincode::config::standard())?;
                Ok(#wrapper_name(value))
            }
        }

        // Serialize to Vec<u8> (Wrapper -> Sled)
        impl TryFrom<#wrapper_name> for Vec<u8> {
            type Error = bincode::error::EncodeError;
            
            fn try_from(value: #wrapper_name) -> Result<Self, Self::Error> {
                bincode::encode_to_vec(value.0, bincode::config::standard())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::metadata::{ModelMetadata, FieldMetadata};
    use syn::parse_quote;

    #[test]
    fn test_generate_primary_key_wrapper() {
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
        
        let result = generate_primary_key(&model);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let code = tokens.to_string();

        // Print for debugging
        eprintln!("Generated code:\n{}", code);

        // Verify key elements are present
        assert!(code.contains("pub struct UserId"));
        assert!(code.contains("pub u64"));
        assert!(code.contains("impl redb :: Value for UserId"));
        assert!(code.contains("impl redb :: Key for UserId"));
        assert!(code.contains("TryFrom"));
        assert!(code.contains("Vec < u8 >"));
    }

    #[test]
    fn test_error_no_primary_key() {
        let model = ModelMetadata::new(
            parse_quote!(User),
            parse_quote!(pub)
        );
        
        let result = generate_primary_key(&model);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must have a primary key"));
    }
}
