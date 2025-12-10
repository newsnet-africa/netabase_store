//! NetabaseModelTrait implementation generation
//!
//! Generates the core trait implementation for each model, including:
//! - Associated types (Keys, SecondaryKeys, etc.)
//! - primary_key() method
//! - compute_hash() method using blake3
//! - Wrapping methods for converting to definition types
//!
//! # Example Output
//!
//! ```rust,ignore
//! impl NetabaseModelTrait<UserDefinition> for User {
//!     type Keys = UserKeys;
//!     const MODEL_TREE_NAME: UserDefinitionDiscriminants = UserDefinitionDiscriminants::User;
//!     type SecondaryKeys = UserSecondaryKeysIter;
//!     type RelationalKeys = UserRelationalKeysIter;
//!     type SubscriptionEnum = UserSubscriptions;
//!     type Hash = [u8; 32];
//!
//!     fn primary_key(&self) -> UserId {
//!         UserId(self.id)
//!     }
//!
//!     fn compute_hash(&self) -> Self::Hash {
//!         let mut hasher = blake3::Hasher::new();
//!         // Hash each field in order
//!         hasher.update(&self.id.to_le_bytes());
//!         hasher.update(self.email.as_bytes());
//!         // ... etc
//!         let hash = hasher.finalize();
//!         *hash.as_bytes()
//!     }
//!
//!     // Wrapping methods...
//! }
//! ```

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Type};

use crate::parse::metadata::ModelMetadata;

/// Generate NetabaseModelTrait implementation for a model
pub fn generate_model_trait(
    model: &ModelMetadata,
    definition_name: &Ident,
) -> syn::Result<TokenStream> {
    let model_name = &model.name;
    
    // Get primary key field
    let pk_field = model.primary_key_field()
        .ok_or_else(|| syn::Error::new_spanned(
            model_name,
            format!("Model '{}' must have a primary key field", model_name)
        ))?;
    
    let pk_field_name = &pk_field.name;
    let pk_wrapper = format_ident!("{}Id", model_name);
    
    // Generate associated types
    let associated_types = generate_associated_types(model, definition_name);
    
    // Generate primary_key method
    let primary_key_method = quote! {
        fn primary_key(&self) -> #pk_wrapper {
            #pk_wrapper(self.#pk_field_name)
        }
    };
    
    // Generate compute_hash method
    let compute_hash_method = generate_compute_hash_method(model)?;
    
    // Generate wrapping methods
    let wrapping_methods = generate_wrapping_methods(model, definition_name);
    
    let definition_discriminants = format_ident!("{}Discriminants", definition_name);
    
    Ok(quote! {
        impl netabase_store::NetabaseModelTrait<#definition_name> for #model_name {
            #associated_types
            
            #primary_key_method
            
            #compute_hash_method
            
            #wrapping_methods
        }
    })
}

/// Generate associated type definitions
fn generate_associated_types(
    model: &ModelMetadata,
    definition_name: &Ident,
) -> TokenStream {
    let model_name = &model.name;
    let keys_type = format_ident!("{}Keys", model_name);
    let secondary_iter = format_ident!("{}SecondaryKeysIter", model_name);
    let relational_iter = format_ident!("{}RelationalKeysIter", model_name);
    let subscriptions_enum = format_ident!("{}Subscriptions", model_name);
    let definition_discriminants = format_ident!("{}Discriminants", definition_name);
    
    quote! {
        type Keys = #keys_type;
        const MODEL_TREE_NAME: #definition_discriminants = #definition_discriminants::#model_name;
        type SecondaryKeys = #secondary_iter;
        type RelationalKeys = #relational_iter;
        type SubscriptionEnum = #subscriptions_enum;
        type Hash = [u8; 32];
    }
}

/// Generate the compute_hash method that hashes all fields
fn generate_compute_hash_method(model: &ModelMetadata) -> syn::Result<TokenStream> {
    let hash_statements = model.fields.iter().map(|field| {
        let field_name = &field.name;
        let field_type = &field.ty;
        
        generate_hash_statement(field_name, field_type)
    });
    
    Ok(quote! {
        fn compute_hash(&self) -> Self::Hash {
            let mut hasher = blake3::Hasher::new();
            #(#hash_statements)*
            let hash = hasher.finalize();
            *hash.as_bytes()
        }
    })
}

/// Generate a hash statement for a single field
fn generate_hash_statement(field_name: &Ident, field_type: &Type) -> TokenStream {
    // Detect the field type and generate appropriate serialization
    if is_primitive_type(field_type) {
        // Primitives: use to_le_bytes()
        quote! {
            hasher.update(&self.#field_name.to_le_bytes());
        }
    } else if is_string_type(field_type) {
        // String: use as_bytes()
        quote! {
            hasher.update(self.#field_name.as_bytes());
        }
    } else {
        // Complex types: use bincode serialization
        quote! {
            let bytes = bincode::encode_to_vec(&self.#field_name, bincode::config::standard())
                .expect("Failed to serialize field for hashing");
            hasher.update(&bytes);
        }
    }
}

/// Check if a type is a primitive (u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, bool)
fn is_primitive_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(ident) = type_path.path.get_ident() {
            let name = ident.to_string();
            matches!(
                name.as_str(),
                "u8" | "u16" | "u32" | "u64" | "u128" |
                "i8" | "i16" | "i32" | "i64" | "i128" |
                "f32" | "f64" | "bool" | "usize" | "isize"
            )
        } else {
            false
        }
    } else {
        false
    }
}

/// Check if a type is String
fn is_string_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(ident) = type_path.path.get_ident() {
            ident == "String"
        } else {
            false
        }
    } else {
        false
    }
}

/// Generate wrapping methods that convert model data to definition types
fn generate_wrapping_methods(
    model: &ModelMetadata,
    definition_name: &Ident,
) -> TokenStream {
    let model_name = &model.name;
    let definition_types = format_ident!("{}ModelAssociatedTypes", definition_name);

    // Create variant names by combining model name with suffix
    let pk_variant = format_ident!("{}PrimaryKey", model_name);
    let model_variant = format_ident!("{}Model", model_name);
    let sk_variant = format_ident!("{}SecondaryKey", model_name);
    let rk_variant = format_ident!("{}RelationalKey", model_name);
    let sub_variant = format_ident!("{}SubscriptionKey", model_name);
    let sk_disc_variant = format_ident!("{}SecondaryKeyDiscriminant", model_name);
    let rk_disc_variant = format_ident!("{}RelationalKeyDiscriminant", model_name);
    let sub_disc_variant = format_ident!("{}SubscriptionKeyDiscriminant", model_name);

    quote! {
        fn wrap_primary_key(pk: Self::PrimaryKey) -> #definition_types {
            #definition_types::#pk_variant(pk)
        }

        fn wrap_model(model: Self) -> #definition_types {
            #definition_types::#model_variant(model)
        }

        fn wrap_secondary_key(key: Self::SecondaryKeysEnum) -> #definition_types {
            #definition_types::#sk_variant(key)
        }

        fn wrap_relational_key(key: Self::RelationalKeysEnum) -> #definition_types {
            #definition_types::#rk_variant(key)
        }

        fn wrap_subscription_key(key: Self::SubscriptionEnum) -> #definition_types {
            #definition_types::#sub_variant(key)
        }

        fn wrap_secondary_discriminant(disc: Self::SecondaryKeysDiscriminant) -> #definition_types {
            #definition_types::#sk_disc_variant(disc)
        }

        fn wrap_relational_discriminant(disc: Self::RelationalKeysDiscriminant) -> #definition_types {
            #definition_types::#rk_disc_variant(disc)
        }

        fn wrap_subscription_discriminant(disc: Self::SubscriptionDiscriminant) -> #definition_types {
            #definition_types::#sub_disc_variant(disc)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::metadata::{ModelMetadata, FieldMetadata};
    use syn::parse_quote;

    #[test]
    fn test_is_primitive_type() {
        assert!(is_primitive_type(&parse_quote!(u64)));
        assert!(is_primitive_type(&parse_quote!(i32)));
        assert!(is_primitive_type(&parse_quote!(bool)));
        assert!(!is_primitive_type(&parse_quote!(String)));
        assert!(!is_primitive_type(&parse_quote!(Vec<u8>)));
    }

    #[test]
    fn test_is_string_type() {
        assert!(is_string_type(&parse_quote!(String)));
        assert!(!is_string_type(&parse_quote!(u64)));
        assert!(!is_string_type(&parse_quote!(&str)));
    }

    #[test]
    fn test_generate_model_trait() {
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
        
        // Add regular field
        let email_field = FieldMetadata::new(
            parse_quote!(email),
            parse_quote!(String),
            parse_quote!(pub)
        );
        model.add_field(email_field);
        
        let definition_name = parse_quote!(UserDefinition);
        let result = generate_model_trait(&model, &definition_name);
        assert!(result.is_ok());
        
        let code = result.unwrap().to_string();
        
        // Verify key elements
        assert!(code.contains("impl netabase_store :: NetabaseModelTrait"));
        assert!(code.contains("type Keys = UserKeys"));
        assert!(code.contains("fn primary_key"));
        assert!(code.contains("UserId (self . id)"));
        assert!(code.contains("fn compute_hash"));
        assert!(code.contains("blake3 :: Hasher :: new"));
        assert!(code.contains("wrap_primary_key"));
        assert!(code.contains("wrap_model"));
    }
}
