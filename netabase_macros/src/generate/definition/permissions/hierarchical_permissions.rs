//! Hierarchical permissions system for nested definitions
//!
//! Generates permission enums that form a tree-like hierarchy where:
//! - Parent definitions can manage permissions for children
//! - Sibling definitions can access each other based on parent grants
//! - Cross-definition relationships are enforced through enum-based type safety
//! - Permissions propagate up the tree for management decisions

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::parse::metadata::{ModuleMetadata, PermissionLevel, ChildPermissionGrant};

/// Generate hierarchical permission enum for a definition with children
pub fn generate_hierarchical_permissions(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    let permissions_name = format_ident!("{}Permissions", definition_name);
    let discriminants_name = format_ident!("{}PermissionsDiscriminants", definition_name);
    
    // Generate permission manager enum that can delegate to children
    let manager_enum = generate_permission_manager_enum(module);
    
    // Generate cross-definition link types for type safety
    let cross_def_types = generate_cross_definition_link_types(module);
    
    // Generate permission checker trait implementations
    let permission_checkers = generate_permission_checkers(module);
    
    quote! {
        #manager_enum
        #cross_def_types
        #permission_checkers
    }
}

/// Generate the main permission manager enum
fn generate_permission_manager_enum(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    let permissions_name = format_ident!("{}PermissionManager", definition_name);
    let discriminants_name = format_ident!("{}PermissionManagerDiscriminants", definition_name);
    
    // Root level permissions (self-management)
    let self_variants = quote! {
        /// Full administrative access to this definition
        Admin,
        /// Read-write access to all models in this definition
        ReadWrite,
        /// Read-only access to all models in this definition
        ReadOnly,
        /// No access to this definition
        None,
    };

    // Child delegation variants
    let child_variants: Vec<_> = module.nested_modules.iter().map(|child| {
        let child_def = &child.definition_name;
        let child_perm_manager = format_ident!("{}PermissionManager", child_def);
        let variant_name = format_ident!("Delegate{}", child_def);
        
        quote! {
            /// Delegate permission decision to child definition
            #variant_name(#child_perm_manager)
        }
    }).collect();

    // Sibling access variants (controlled by parent)
    let sibling_variants: Vec<_> = module.nested_modules.iter().enumerate().map(|(i, child)| {
        let child_def = &child.definition_name;
        let variant_name = format_ident!("CrossAccess{}", child_def);
        
        // Generate cross-access permissions for this sibling
        let other_siblings: Vec<_> = module.nested_modules.iter().enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, sibling)| {
                let sibling_def = &sibling.definition_name;
                let sibling_perm = format_ident!("{}PermissionLevel", sibling_def);
                
                quote! {
                    pub #sibling_def: #sibling_perm
                }
            })
            .collect();
        
        quote! {
            /// Cross-sibling access permissions for #child_def
            #variant_name {
                #(#other_siblings,)*
            }
        }
    }).collect();

    // Permission level enum for granular control
    let permission_level_enum = quote! {
        /// Granular permission levels for cross-definition access
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[derive(strum::EnumDiscriminants, bincode::Encode, bincode::Decode)]
        #[strum_discriminants(derive(strum::EnumIter, strum::AsRefStr))]
        pub enum PermissionLevel {
            None,
            Read,
            Write,
            ReadWrite,
            Admin,
        }

        impl PermissionLevel {
            pub fn can_read(&self) -> bool {
                matches!(self, PermissionLevel::Read | PermissionLevel::ReadWrite | PermissionLevel::Admin)
            }

            pub fn can_write(&self) -> bool {
                matches!(self, PermissionLevel::Write | PermissionLevel::ReadWrite | PermissionLevel::Admin)
            }

            pub fn can_manage(&self) -> bool {
                matches!(self, PermissionLevel::Admin)
            }
        }
    };

    quote! {
        #permission_level_enum

        /// Hierarchical permission manager for #definition_name
        /// 
        /// This enum handles:
        /// - Self-permissions (Admin, ReadWrite, ReadOnly, None)
        /// - Child delegation (DelegateChildName)
        /// - Cross-sibling access (CrossAccessChildName)
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        #[derive(strum::EnumDiscriminants, bincode::Encode, bincode::Decode)]
        #[strum_discriminants(derive(strum::EnumIter, strum::AsRefStr, bincode::Encode, bincode::Decode))]
        #[strum_discriminants(name(#discriminants_name))]
        pub enum #permissions_name {
            #self_variants
            #(#child_variants,)*
            #(#sibling_variants,)*
        }

        impl netabase_store::traits::definition::DiscriminantName for #discriminants_name {}
    }
}

/// Generate type-safe cross-definition link types using enums
fn generate_cross_definition_link_types(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    
    // Generate link enums for each model that has cross-definition relationships
    let link_enums: Vec<_> = module.models.iter().filter_map(|model| {
        let cross_def_fields = model.cross_definition_fields();
        if cross_def_fields.is_empty() {
            return None;
        }

        let model_name = &model.name;
        let link_enum_name = format_ident!("{}CrossDefinitionLinks", model_name);
        let discriminants_name = format_ident!("{}CrossDefinitionLinksDiscriminants", model_name);
        
        let variants: Vec<_> = cross_def_fields.iter().map(|field| {
            if let Some(ref link) = field.cross_definition_link {
                let field_name = &field.name;
                let variant_name = format_ident!("{}", pascal_case(&field_name));
                let target_path = &link.target_path;
                
                // Use the target model type if specified, otherwise use the path
                if let Some(ref target_model) = link.target_model {
                    quote! {
                        #variant_name(#target_path::#target_model)
                    }
                } else {
                    quote! {
                        #variant_name(#target_path)
                    }
                }
            } else {
                // This shouldn't happen since we filtered, but provide a fallback
                let field_name = &field.name;
                let variant_name = format_ident!("{}", pascal_case(&field_name));
                quote! {
                    #variant_name
                }
            }
        }).collect();

        Some(quote! {
            /// Cross-definition links for #model_name model
            /// 
            /// This enum provides type-safe access to related models in other definitions.
            /// Each variant represents a field that links to a model in a different definition.
            #[derive(Debug, Clone, PartialEq, Eq, Hash)]
            #[derive(strum::EnumDiscriminants, bincode::Encode, bincode::Decode)]
            #[strum_discriminants(derive(strum::EnumIter, strum::AsRefStr, bincode::Encode, bincode::Decode))]
            #[strum_discriminants(name(#discriminants_name))]
            pub enum #link_enum_name {
                #(#variants,)*
            }

            impl netabase_store::traits::definition::DiscriminantName for #discriminants_name {}
        })
    }).collect();

    quote! {
        #(#link_enums)*
    }
}

/// Generate permission checker trait implementations
fn generate_permission_checkers(module: &ModuleMetadata) -> TokenStream {
    let definition_name = &module.definition_name;
    let permissions_name = format_ident!("{}PermissionManager", definition_name);
    
    // Generate permission checking logic
    let can_access_impl = generate_can_access_implementation(module);
    let can_cross_access_impl = generate_cross_access_implementation(module);
    let permission_propagation = generate_permission_propagation(module);

    quote! {
        impl #permissions_name {
            #can_access_impl
            #can_cross_access_impl
            #permission_propagation
        }

        // Implement core permission traits
        impl netabase_store::traits::permission::PermissionEnumTrait for #permissions_name {
            fn permission_level(&self) -> netabase_store::traits::permission::PermissionLevel {
                use netabase_store::traits::permission::PermissionLevel as CoreLevel;
                match self {
                    Self::Admin => CoreLevel::ReadWrite, // Map to highest available
                    Self::ReadWrite => CoreLevel::ReadWrite,
                    Self::ReadOnly => CoreLevel::Read,
                    Self::None => CoreLevel::None,
                    // For delegated permissions, recursively check
                    _ => {
                        // TODO: Implement recursive permission level checking
                        CoreLevel::Read
                    }
                }
            }

            fn grants_access_to<R>(&self, definition: &R::Discriminant) -> bool
            where
                R: strum::IntoDiscriminant,
                R::Discriminant: strum::IntoEnumIterator + std::hash::Hash + Eq + std::fmt::Debug + Clone,
            {
                // TODO: Implement precise discriminant checking for hierarchical access
                !matches!(self, Self::None)
            }
        }
    }
}

/// Generate implementation for checking access to local models
fn generate_can_access_implementation(module: &ModuleMetadata) -> TokenStream {
    let model_checks: Vec<_> = module.models.iter().map(|model| {
        let model_name = &model.name;
        let function_name = format_ident!("can_access_{}", model_name.to_string().to_lowercase());
        quote! {
            /// Check if this permission allows access to #model_name model
            pub fn #function_name(&self) -> bool {
                match self {
                    Self::Admin | Self::ReadWrite | Self::ReadOnly => true,
                    Self::None => false,
                    // For delegated permissions, check child permission
                    _ => false, // TODO: Implement child delegation logic
                }
            }
        }
    }).collect();

    quote! {
        #(#model_checks)*
    }
}

/// Generate implementation for cross-definition access checking
fn generate_cross_access_implementation(module: &ModuleMetadata) -> TokenStream {
    let cross_checks: Vec<_> = module.nested_modules.iter().map(|child| {
        let child_def = &child.definition_name;
        let check_fn = format_ident!("can_cross_access_{}", child_def.to_string().to_lowercase());
        
        quote! {
            /// Check if this permission allows cross-access to #child_def
            pub fn #check_fn(&self, target_permission: &PermissionLevel) -> bool {
                match self {
                    Self::Admin => true, // Admin can access everything
                    // Check specific cross-access variants
                    _ => {
                        // TODO: Implement specific cross-access logic based on variant
                        false
                    }
                }
            }
        }
    }).collect();

    quote! {
        #(#cross_checks)*
    }
}

/// Generate permission propagation logic for hierarchical management
fn generate_permission_propagation(module: &ModuleMetadata) -> TokenStream {
    quote! {
        /// Propagate permission check up the hierarchy
        /// 
        /// This allows parent definitions to make permission decisions
        /// for their children and manage cross-sibling access.
        pub fn propagate_permission_check<F>(&self, check: F) -> bool
        where
            F: Fn() -> bool,
        {
            match self {
                Self::Admin => true,
                Self::None => false,
                _ => check(), // Delegate to the specific permission logic
            }
        }

        /// Check if this permission can manage child permissions
        pub fn can_manage_child_permissions(&self) -> bool {
            matches!(self, Self::Admin)
        }

        /// Get the effective permission level for a specific operation
        pub fn effective_permission_for_operation(&self, operation: &str) -> PermissionLevel {
            match (self, operation) {
                (Self::Admin, _) => PermissionLevel::Admin,
                (Self::ReadWrite, "read" | "write") => PermissionLevel::ReadWrite,
                (Self::ReadOnly, "read") => PermissionLevel::Read,
                (Self::None, _) => PermissionLevel::None,
                _ => PermissionLevel::None,
            }
        }
    }
}

/// Convert snake_case to PascalCase
fn pascal_case(ident: &syn::Ident) -> String {
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
    use crate::parse::metadata::{ModuleMetadata, ModelMetadata, FieldMetadata};
    use syn::parse_quote;

    #[test]
    fn test_generate_hierarchical_permissions() {
        let mut module = ModuleMetadata::new(
            parse_quote!(app_def),
            parse_quote!(AppDef),
            parse_quote!(AppDefKeys)
        );

        // Add a nested module
        let nested = ModuleMetadata::new(
            parse_quote!(user_def),
            parse_quote!(UserDef),
            parse_quote!(UserDefKeys)
        );
        module.add_nested_module(nested);

        let tokens = generate_hierarchical_permissions(&module);
        let code = tokens.to_string();
        
        // Should contain permission manager enum
        assert!(code.contains("AppDefPermissionManager"));
        assert!(code.contains("DelegateUserDef"));
        assert!(code.contains("CrossAccessUserDef"));
    }
}