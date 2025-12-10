//! Definition enum generation
//!
//! Generates the main definition enum that wraps all models.
//!
//! # Example Output
//!
//! For a module with User and Post models:
//!
//! ```rust,ignore
//! #[derive(Debug, Clone, strum::EnumDiscriminants, bincode::Encode, bincode::Decode)]
//! #[strum_discriminants(derive(Hash, strum::AsRefStr, bincode::Encode, bincode::Decode, strum::EnumIter))]
//! #[strum_discriminants(name(MyDefinitionDiscriminants))]
//! pub enum MyDefinition {
//!     User(User),
//!     Post(Post),
//! }
//! ```

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::parse::metadata::{ModuleMetadata, ModelMetadata};

/// Generate the definition enum that wraps all models
pub fn generate_definition_enum(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    let discriminants_name = format_ident!("{}Discriminants", definition_name);
    
    // Generate variants for each model
    let variants: Vec<_> = module.models.iter().map(|model| {
        let model_name = &model.name;
        quote! {
            #model_name(#model_name)
        }
    }).collect();

    // Generate variants for each nested module
    let nested_variants: Vec<_> = module.nested_modules.iter().map(|nested| {
        let definition_name = &nested.definition_name;
        let module_name = &nested.module_name;
        quote! {
            #definition_name(#module_name::#definition_name)
        }
    }).collect();
    
    quote! {
        #[derive(Debug, Clone, strum::EnumDiscriminants, bincode::Encode, bincode::Decode)]
        #[strum_discriminants(derive(Hash, strum::AsRefStr, bincode::Encode, bincode::Decode, strum::EnumIter))]
        #[strum_discriminants(name(#discriminants_name))]
        pub enum #definition_name {
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
    fn test_generate_definition_enum() {
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
        
        let tokens = generate_definition_enum(&module);
        let code = tokens.to_string();
        
        // Verify structure
        assert!(code.contains("pub enum MyDefinition"));
        assert!(code.contains("User") && code.contains("User"));
        assert!(code.contains("Post") && code.contains("Post"));
        assert!(code.contains("MyDefinitionDiscriminants"));
        assert!(code.contains("strum :: EnumDiscriminants"));
    }
}
