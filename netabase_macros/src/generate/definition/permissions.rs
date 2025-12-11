//! Permissions enum generation
//!
//! Generates the permissions enum for a definition module.
//! This enum handles access control for the definition and its nested definitions.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::parse::metadata::ModuleMetadata;

/// Generate the permissions enum and trait implementations
pub fn generate_permissions_enum(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    let permissions_name = format_ident!("{}Permissions", definition_name);
    let discriminants_name = format_ident!("{}PermissionsDiscriminants", definition_name);
    
    // Generate variants for nested definitions
    let nested_variants: Vec<_> = module.nested_modules.iter().map(|nested| {
        let variant_name = &nested.definition_name;
        let module_name = &nested.module_name;
        let nested_perm_type = format_ident!("{}Permissions", nested.definition_name);
        
        quote! {
            #variant_name(#module_name::#nested_perm_type)
        }
    }).collect();

    // Generate match arms for permission_level
    let nested_variants_level: Vec<_> = module.nested_modules.iter().map(|nested| {
        let variant_name = &nested.definition_name;
        quote! {
            Self::#variant_name(p) => netabase_store::traits::permission::PermissionEnumTrait::permission_level(p)
        }
    }).collect();
    
    quote! {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, strum::EnumDiscriminants, bincode::Encode, bincode::Decode)]
        #[strum_discriminants(derive(strum::EnumIter, Hash, strum::AsRefStr, bincode::Encode, bincode::Decode))]
        #[strum_discriminants(name(#discriminants_name))]
        pub enum #permissions_name {
            /// Full access to this definition and all children
            All,
            /// No access
            None,
            /// Read-only access to this definition
            ReadOnly,
            /// Access delegated to a nested definition
            #(#nested_variants,)*
        }

        // Implement DiscriminantName for the generated discriminant enum
        impl netabase_store::traits::definition::DiscriminantName for #discriminants_name {}

        impl netabase_store::traits::permission::PermissionEnumTrait for #permissions_name {
            fn permission_level(&self) -> netabase_store::traits::permission::PermissionLevel {
                use netabase_store::traits::permission::PermissionLevel;
                match self {
                    Self::All => PermissionLevel::ReadWrite,
                    Self::None => PermissionLevel::None,
                    Self::ReadOnly => PermissionLevel::Read,
                    #(#nested_variants_level,)*
                }
            }

            fn grants_access_to<R>(&self, _definition: &R::Discriminant) -> bool
            where
                R: strum::IntoDiscriminant,
                R::Discriminant: strum::IntoEnumIterator + std::hash::Hash + Eq + std::fmt::Debug + Clone,
            {
                // TODO: Implement precise discriminant checking
                true
            }
        }
    }
}