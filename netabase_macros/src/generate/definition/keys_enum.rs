//! Keys enum generation
//!
//! Generates the keys enum that holds all primary key types from all models.
//!
//! # Example Output
//!
//! ```rust,ignore
//! #[derive(Debug, Clone, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
//! pub enum MyDefinitionKeys {
//!     UserId(UserId),
//!     PostId(PostId),
//! }
//! ```

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::parse::metadata::ModuleMetadata;

/// Generate the keys enum that holds all primary key types
pub fn generate_keys_enum(module: &ModuleMetadata) -> TokenStream {
    let keys_name = &module.keys_name;
    
    // Generate variants for each model's primary key
    let variants: Vec<_> = module.models.iter().map(|model| {
        let model_name = &model.name;
        let key_type = format_ident!("{}Id", model_name);
        let variant_name = format_ident!("{}Id", model_name);
        
        quote! {
            #variant_name(#key_type)
        }
    }).collect();

    // Generate variants for each nested module's keys
    let nested_variants: Vec<_> = module.nested_modules.iter().map(|nested| {
        let keys_name = &nested.keys_name;
        let module_name = &nested.module_name;
        quote! {
            #keys_name(#module_name::#keys_name)
        }
    }).collect();
    
    quote! {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
        pub enum #keys_name {
            #(#variants,)*
            #(#nested_variants,)*
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::metadata::{ModuleMetadata, ModelMetadata, FieldMetadata};
    use syn::parse_quote;

    #[test]
    fn test_generate_keys_enum() {
        let mut module = ModuleMetadata::new(
            parse_quote!(test_mod),
            parse_quote!(MyDefinition),
            parse_quote!(MyDefinitionKeys)
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
        
        let tokens = generate_keys_enum(&module);
        let code = tokens.to_string();
        
        // Verify structure
        assert!(code.contains("pub enum MyDefinitionKeys"));
        assert!(code.contains("UserId") && code.contains("UserId"));
        assert!(code.contains("PostId") && code.contains("PostId"));
    }
}
