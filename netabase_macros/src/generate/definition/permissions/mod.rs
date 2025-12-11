//! Permissions enum generation
//!
//! Generates the permissions enum for a definition module.
//! This enum handles access control for the definition and its nested definitions.
//! 
//! Phase 8 Enhancement: Now supports hierarchical permissions where parent
//! definitions manage child permissions and enforce cross-definition access
//! through enum-based type safety.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::parse::metadata::ModuleMetadata;

pub mod hierarchical_permissions;

#[cfg(test)]
mod hierarchical_permissions_test;

pub use hierarchical_permissions::generate_hierarchical_permissions;

/// Generate the permissions enum and trait implementations
/// 
/// This is the main entry point that determines whether to generate
/// a simple permission enum (for leaf definitions) or a hierarchical
/// permission manager (for definitions with children).
pub fn generate_permissions_enum(module: &ModuleMetadata) -> TokenStream {
    if module.nested_modules.is_empty() {
        // Leaf definition - generate simple permissions
        generate_simple_permissions_enum(module)
    } else {
        // Parent definition - generate hierarchical permission manager
        generate_hierarchical_permissions(module)
    }
}

/// Generate simple permissions enum for leaf definitions (no children)
fn generate_simple_permissions_enum(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    let permissions_name = format_ident!("{}Permissions", definition_name);
    let discriminants_name = format_ident!("{}PermissionsDiscriminants", definition_name);
    
    quote! {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, strum::EnumDiscriminants, bincode::Encode, bincode::Decode)]
        #[strum_discriminants(derive(strum::EnumIter, Hash, strum::AsRefStr, bincode::Encode, bincode::Decode))]
        #[strum_discriminants(name(#discriminants_name))]
        pub enum #permissions_name {
            /// Full access to this definition
            All,
            /// No access
            None,
            /// Read-only access to this definition
            ReadOnly,
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
                }
            }

            fn grants_access_to<R>(&self, _definition: &R::Discriminant) -> bool
            where
                R: strum::IntoDiscriminant,
                R::Discriminant: strum::IntoEnumIterator + std::hash::Hash + Eq + std::fmt::Debug + Clone,
            {
                !matches!(self, Self::None)
            }
        }
    }
}