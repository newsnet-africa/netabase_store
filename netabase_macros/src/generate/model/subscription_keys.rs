//! Subscription keys generation
//!
//! Generates enums for subscription topics that models subscribe to.
//!
//! # Example Output
//!
//! For a `User` model with `#[subscribe(Updates, Premium)]`:
//!
//! ```rust,ignore
//! #[derive(Debug, Clone, strum::EnumDiscriminants, bincode::Encode, bincode::Decode)]
//! #[strum_discriminants(derive(Hash, strum::AsRefStr, bincode::Encode, bincode::Decode, strum::EnumIter))]
//! #[strum_discriminants(name(UserSubscriptionsDiscriminants))]
//! pub enum UserSubscriptions {
//!     Updates,
//!     Premium,
//! }
//!
//! #[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, strum::EnumIter, strum::AsRefStr)]
//! pub enum UserSubscriptionTreeNames {
//!     Updates,
//!     Premium,
//! }
//! ```

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::parse::metadata::ModelMetadata;

/// Generate subscription enums for models that subscribe to topics
pub fn generate_subscription_keys(model: &ModelMetadata) -> TokenStream {
    // If no subscriptions, return empty
    if model.subscriptions.is_empty() {
        return quote! {};
    }
    
    let model_name = &model.name;
    
    // Generate the subscriptions enum
    let enum_def = generate_subscriptions_enum(model_name, &model.subscriptions);
    
    // Generate tree names enum
    let tree_names_enum = generate_tree_names_enum(model_name, &model.subscriptions);
    
    quote! {
        #enum_def
        #tree_names_enum
    }
}

/// Generate the subscriptions enum (simple variants, no data)
fn generate_subscriptions_enum(
    model_name: &Ident,
    subscriptions: &[Ident],
) -> TokenStream {
    let enum_name = format_ident!("{}Subscriptions", model_name);
    let discriminants_name = format_ident!("{}SubscriptionsDiscriminants", model_name);
    
    // Generate simple enum variants (no data)
    let variants: Vec<_> = subscriptions.iter().collect();
    
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
    subscriptions: &[Ident],
) -> TokenStream {
    let enum_name = format_ident!("{}SubscriptionTreeNames", model_name);
    
    let variants: Vec<_> = subscriptions.iter().collect();
    
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::metadata::ModelMetadata;
    use syn::parse_quote;

    #[test]
    fn test_generate_subscription_keys_with_subscriptions() {
        let mut model = ModelMetadata::new(
            parse_quote!(User),
            parse_quote!(pub)
        );
        
        // Add subscriptions
        model.add_subscription(parse_quote!(Updates));
        model.add_subscription(parse_quote!(Premium));
        model.add_subscription(parse_quote!(Newsletter));
        
        let tokens = generate_subscription_keys(&model);
        let code = tokens.to_string();
        
        // Verify key elements
        assert!(code.contains("pub enum UserSubscriptions"));
        assert!(code.contains("Updates"));
        assert!(code.contains("Premium"));
        assert!(code.contains("Newsletter"));
        assert!(code.contains("UserSubscriptionsDiscriminants"));
        assert!(code.contains("UserSubscriptionTreeNames"));
    }

    #[test]
    fn test_generate_subscription_keys_no_subscriptions() {
        let model = ModelMetadata::new(
            parse_quote!(User),
            parse_quote!(pub)
        );
        
        let tokens = generate_subscription_keys(&model);
        let code = tokens.to_string();
        
        // Should be empty when no subscriptions
        assert!(code.is_empty());
    }
}
