//! ModelAssociatedTypes enum generation
//!
//! Generates the enum that holds all types associated with models in the definition.
//! This is used by wrapping methods in model trait implementations.
//!
//! # Example Output
//!
//! ```rust,ignore
//! #[derive(Debug, Clone)]
//! pub enum MyDefinitionModelAssociatedTypes {
//!     UserPrimaryKey(UserId),
//!     UserModel(User),
//!     UserSecondaryKey(UserSecondaryKeys),
//!     UserRelationalKey(UserRelationalKeys),
//!     UserSubscriptionKey(UserSubscriptions),
//!     UserSecondaryKeyDiscriminant(UserSecondaryKeysDiscriminants),
//!     UserRelationalKeyDiscriminant(UserRelationalKeysDiscriminants),
//!     UserSubscriptionKeyDiscriminant(UserSubscriptionsDiscriminants),
//!     
//!     PostPrimaryKey(PostId),
//!     PostModel(Post),
//!     // ... etc
//! }
//! ```

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::parse::metadata::ModuleMetadata;

/// Generate the ModelAssociatedTypes enum
pub fn generate_associated_types_enum(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    let enum_name = format_ident!("{}ModelAssociatedTypes", definition_name);
    
    // Generate 8 variants for each model
    let variants: Vec<_> = module.models.iter().flat_map(|model| {
        let model_name = &model.name;
        
        let pk_variant = format_ident!("{}PrimaryKey", model_name);
        let model_variant = format_ident!("{}Model", model_name);
        let sk_variant = format_ident!("{}SecondaryKey", model_name);
        let rk_variant = format_ident!("{}RelationalKey", model_name);
        let sub_variant = format_ident!("{}SubscriptionKey", model_name);
        let sk_disc_variant = format_ident!("{}SecondaryKeyDiscriminant", model_name);
        let rk_disc_variant = format_ident!("{}RelationalKeyDiscriminant", model_name);
        let sub_disc_variant = format_ident!("{}SubscriptionKeyDiscriminant", model_name);
        
        let pk_type = format_ident!("{}Id", model_name);
        let sk_enum = format_ident!("{}SecondaryKeys", model_name);
        let rk_enum = format_ident!("{}RelationalKeys", model_name);
        let sub_enum = format_ident!("{}Subscriptions", model_name);
        let sk_disc_type = format_ident!("{}SecondaryKeysDiscriminants", model_name);
        let rk_disc_type = format_ident!("{}RelationalKeysDiscriminants", model_name);
        let sub_disc_type = format_ident!("{}SubscriptionsDiscriminants", model_name);
        
        vec![
            quote! { #pk_variant(#pk_type) },
            quote! { #model_variant(#model_name) },
            quote! { #sk_variant(#sk_enum) },
            quote! { #rk_variant(#rk_enum) },
            quote! { #sub_variant(#sub_enum) },
            quote! { #sk_disc_variant(#sk_disc_type) },
            quote! { #rk_disc_variant(#rk_disc_type) },
            quote! { #sub_disc_variant(#sub_disc_type) },
        ]
    }).collect();

    // Generate variants for nested modules
    let nested_variants: Vec<_> = module.nested_modules.iter().map(|nested| {
        let definition_name = &nested.definition_name;
        let module_name = &nested.module_name;
        // We use the nested definition's associated types enum
        let nested_assoc_type = format_ident!("{}ModelAssociatedTypes", definition_name);
        
        quote! {
            #definition_name(#module_name::#nested_assoc_type)
        }
    }).collect();
    
    quote! {
        #[derive(Debug, Clone)]
        pub enum #enum_name {
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
    fn test_generate_associated_types_enum() {
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
        
        let tokens = generate_associated_types_enum(&module);
        let code = tokens.to_string();
        
        // Verify structure
        assert!(code.contains("pub enum MyDefinitionModelAssociatedTypes"));
        assert!(code.contains("UserPrimaryKey") && code.contains("UserId"));
        assert!(code.contains("UserModel") && code.contains("User"));
        assert!(code.contains("UserSecondaryKey") && code.contains("UserSecondaryKeys"));
        assert!(code.contains("PostPrimaryKey") && code.contains("PostId"));
        assert!(code.contains("PostModel") && code.contains("Post"));
    }
}
