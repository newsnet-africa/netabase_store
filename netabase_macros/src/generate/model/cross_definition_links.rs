//! Cross-definition linking code generation
//!
//! This module generates wrapper types and related code for cross-definition relationships.
//! Implements the approach specified in Phase 8 of the implementation plan.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Path};

use crate::parse::metadata::{ModuleMetadata, ModelMetadata, FieldMetadata, CrossDefinitionLink, RelationshipType};

/// Generate cross-definition link wrapper types and implementations for a module
pub fn generate_cross_definition_links(module: &ModuleMetadata) -> syn::Result<TokenStream> {
    let mut tokens = TokenStream::new();
    
    // Generate wrappers for each model that has cross-definition links
    for model in &module.models {
        let model_links = generate_model_cross_definition_links(model, &module.definition_name)?;
        tokens.extend(model_links);
    }
    
    Ok(tokens)
}

/// Generate cross-definition link wrappers for a specific model
pub fn generate_model_cross_definition_links(model: &ModelMetadata, definition_name: &Ident) -> syn::Result<TokenStream> {
    let mut tokens = TokenStream::new();
    
    // Find all cross-definition link fields
    let cross_definition_fields: Vec<_> = model.fields.iter()
        .filter(|field| field.cross_definition_link.is_some())
        .collect();
    
    if cross_definition_fields.is_empty() {
        return Ok(tokens);
    }
    
    // Generate wrapper types for each cross-definition link
    for field in cross_definition_fields.iter() {
        let link = field.cross_definition_link.as_ref().unwrap();
        let wrapper_type = generate_cross_definition_wrapper_type(
            &model.name, 
            &field.name, 
            link,
            definition_name
        )?;
        tokens.extend(wrapper_type);
    }
    
    // Generate enum for all cross-definition links in this model
    let links_enum = generate_cross_definition_links_enum(model, &cross_definition_fields)?;
    tokens.extend(links_enum);
    
    Ok(tokens)
}

/// Generate wrapper type for a cross-definition link
fn generate_cross_definition_wrapper_type(
    source_model: &Ident,
    field_name: &Ident,
    link: &CrossDefinitionLink,
    _definition_name: &Ident,
) -> syn::Result<TokenStream> {
    // Generate wrapper type name: {SourceModel}{FieldName}Link
    let wrapper_name = format_ident!("{}{}Link", source_model, field_name);
    
    // Extract target model name and definition from path
    let (target_definition, target_model_name) = parse_cross_definition_path(&link.target_path)?;
    
    let relationship_type = match link.relationship_type {
        RelationshipType::OneToOne => quote! { CrossDefinitionRelationshipType::OneToOne },
        RelationshipType::OneToMany => quote! { CrossDefinitionRelationshipType::OneToMany },
        RelationshipType::ManyToOne => quote! { CrossDefinitionRelationshipType::ManyToOne },
        RelationshipType::ManyToMany => quote! { CrossDefinitionRelationshipType::ManyToMany },
    };
    
    let permission_level = match link.required_permission {
        crate::parse::metadata::PermissionLevel::None => quote! { CrossDefinitionPermissionLevel::None },
        crate::parse::metadata::PermissionLevel::Read => quote! { CrossDefinitionPermissionLevel::Read },
        crate::parse::metadata::PermissionLevel::Write => quote! { CrossDefinitionPermissionLevel::Write },
        crate::parse::metadata::PermissionLevel::ReadWrite => quote! { CrossDefinitionPermissionLevel::ReadWrite },
        crate::parse::metadata::PermissionLevel::Admin => quote! { CrossDefinitionPermissionLevel::Admin },
    };
    
    Ok(quote! {
        /// Cross-definition link wrapper for #source_model -> #target_model_name
        #[derive(Debug, Clone, PartialEq)]
        pub struct #wrapper_name {
            /// Target definition reference path
            pub target_path: &'static str,
            /// Target model identifier
            pub target_model_id: String,
            /// Type of relationship
            pub relationship_type: CrossDefinitionRelationshipType,
            /// Required permission level for this link
            pub required_permission: CrossDefinitionPermissionLevel,
        }
        
        impl #wrapper_name {
            /// Create a new cross-definition link
            pub fn new(target_model_id: String) -> Self {
                Self {
                    target_path: stringify!(#target_definition),
                    target_model_id,
                    relationship_type: #relationship_type,
                    required_permission: #permission_level,
                }
            }
            
            /// Get the target model ID
            pub fn target_id(&self) -> &str {
                &self.target_model_id
            }
            
            /// Get the target definition path
            pub fn target_path(&self) -> &'static str {
                self.target_path
            }
            
            /// Check if access is allowed with given permission
            pub fn can_access_with_permission(&self, permission: &CrossDefinitionPermissionLevel) -> bool {
                permission >= &self.required_permission
            }
        }
        
        impl std::fmt::Display for #wrapper_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}::{}", self.target_path, self.target_model_id)
            }
        }
    })
}

/// Generate enum for all cross-definition links in a model
fn generate_cross_definition_links_enum(
    model: &ModelMetadata, 
    cross_definition_fields: &[&FieldMetadata]
) -> syn::Result<TokenStream> {
    if cross_definition_fields.is_empty() {
        return Ok(TokenStream::new());
    }
    
    let model_name = &model.name;
    let enum_name = format_ident!("{}CrossDefinitionLinks", model_name);
    
    // Generate enum variants for each cross-definition link
    let variants: Vec<_> = cross_definition_fields.iter().map(|field| {
        let field_name = &field.name;
        let wrapper_name = format_ident!("{}{}Link", model_name, field_name);
        let variant_name = format_ident!("{}Link", field_name);
        
        quote! {
            /// Link via field #field_name
            #variant_name(#wrapper_name)
        }
    }).collect();
    
    // Generate match arms for the implementations
    let relationship_type_arms: Vec<_> = cross_definition_fields.iter().map(|field| {
        let field_name = &field.name;
        let variant_name = format_ident!("{}Link", field_name);
        quote! { Self::#variant_name(link) => link.relationship_type }
    }).collect();
    
    let required_permission_arms: Vec<_> = cross_definition_fields.iter().map(|field| {
        let field_name = &field.name;
        let variant_name = format_ident!("{}Link", field_name);
        quote! { Self::#variant_name(link) => link.required_permission }
    }).collect();

    let target_path_arms: Vec<_> = cross_definition_fields.iter().map(|field| {
        let field_name = &field.name;
        let variant_name = format_ident!("{}Link", field_name);
        quote! { Self::#variant_name(link) => link.target_path() }
    }).collect();
    
    Ok(quote! {
        /// Enum representing all cross-definition links for #model_name
        #[derive(Debug, Clone)]
        pub enum #enum_name {
            #(#variants,)*
        }
        
        impl #enum_name {
            /// Get the relationship type of this link
            pub fn relationship_type(&self) -> CrossDefinitionRelationshipType {
                match self {
                    #(#relationship_type_arms,)*
                }
            }
            
            /// Get the required permission level for this link
            pub fn required_permission(&self) -> CrossDefinitionPermissionLevel {
                match self {
                    #(#required_permission_arms,)*
                }
            }
            
            /// Get the target path for this link
            pub fn target_path(&self) -> &'static str {
                match self {
                    #(#target_path_arms,)*
                }
            }
        }
    })
}

/// Generate supporting types for cross-definition relationships
pub fn generate_cross_definition_support_types() -> TokenStream {
    quote! {
        /// Types of cross-definition relationships
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum CrossDefinitionRelationshipType {
            /// One-to-one relationship
            OneToOne,
            /// One-to-many relationship 
            OneToMany,
            /// Many-to-one relationship
            ManyToOne,
            /// Many-to-many relationship
            ManyToMany,
        }
        
        /// Permission levels for cross-definition access
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub enum CrossDefinitionPermissionLevel {
            /// No access allowed
            None,
            /// Read-only access
            Read,
            /// Write-only access (rarely used)
            Write,
            /// Full read-write access
            ReadWrite,
            /// Administrative access
            Admin,
        }
        
        /// Trait for models that support cross-definition linking
        pub trait CrossDefinitionLinked {
            /// Type representing all cross-definition links for this model
            type CrossDefinitionLinks;
            
            /// Get all cross-definition links for this model instance
            fn get_cross_definition_links(&self) -> Vec<Self::CrossDefinitionLinks>;
            
            /// Check if a specific cross-definition link is available
            fn has_cross_definition_link(&self, link_type: &str) -> bool;
        }
        
        /// Helper trait for cross-definition path resolution
        pub trait CrossDefinitionResolver {
            /// Resolve a cross-definition path to a concrete type
            fn resolve_cross_definition_path(path: &str) -> Option<String>;
            
            /// Validate that a cross-definition link is valid
            fn validate_cross_definition_link(
                source: &str, 
                target: &str, 
                permission: CrossDefinitionPermissionLevel
            ) -> bool;
        }
    }
}

/// Parse a cross-definition path into definition and model parts
/// 
/// Examples:
/// - "inner::InnerDefinition::InnerModel" -> ("inner::InnerDefinition", "InnerModel")
/// - "InnerModel" -> ("", "InnerModel") 
fn parse_cross_definition_path(path: &Path) -> syn::Result<(TokenStream, Ident)> {
    let segments: Vec<_> = path.segments.iter().collect();
    
    if segments.is_empty() {
        return Err(syn::Error::new_spanned(path, "Empty cross-definition path"));
    }
    
    // Last segment is always the model name
    let model_name = segments.last().unwrap().ident.clone();
    
    // Everything before the last segment is the definition path
    if segments.len() == 1 {
        // Just the model name, no path prefix
        Ok((TokenStream::new(), model_name))
    } else {
        // Definition path + model name
        let definition_segments = &segments[..segments.len()-1];
        let definition_path = quote! { #(#definition_segments)::* };
        Ok((definition_path, model_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;
    
    #[test]
    fn test_parse_cross_definition_path() {
        let path: Path = parse_quote! { inner::InnerDefinition::InnerModel };
        let (def_path, model_name) = parse_cross_definition_path(&path).unwrap();
        
        assert_eq!(model_name, "InnerModel");
        // Definition path should contain the path parts
        let def_str = def_path.to_string();
        assert!(def_str.contains("inner") && def_str.contains("InnerDefinition"));
    }
    
    #[test]
    fn test_parse_simple_cross_definition_path() {
        let path: Path = parse_quote! { InnerModel };
        let (def_path, model_name) = parse_cross_definition_path(&path).unwrap();
        
        assert_eq!(model_name, "InnerModel");
        assert!(def_path.is_empty());
    }
    
    #[test]
    fn test_generate_support_types() {
        let tokens = generate_cross_definition_support_types();
        let code = tokens.to_string();
        
        // Verify the basic structure is generated
        assert!(code.contains("CrossDefinitionRelationshipType"));
        assert!(code.contains("CrossDefinitionPermissionLevel"));
        assert!(code.contains("CrossDefinitionLinked"));
        assert!(code.contains("CrossDefinitionResolver"));
    }
    
    #[test]
    fn test_generate_model_cross_definition_links_no_links() {
        // Model with no cross-definition links should generate empty code
        let model = create_simple_model();
        let definition_name: syn::Ident = parse_quote!(TestDef);
        
        let result = generate_model_cross_definition_links(&model, &definition_name).unwrap();
        assert!(result.is_empty());
    }
    
    #[test]
    fn test_generate_model_cross_definition_links_with_link() {
        // Model with cross-definition link
        let mut model = create_simple_model();
        
        // Add a field with cross-definition link
        let mut cross_link_field = crate::parse::metadata::FieldMetadata::new(
            parse_quote!(related_item),
            parse_quote!(String),
            parse_quote!(pub)
        );
        
        cross_link_field.cross_definition_link = Some(CrossDefinitionLink {
            target_path: parse_quote!(other_def::OtherModel),
            target_model: Some(parse_quote!(OtherModel)),
            required_permission: crate::parse::metadata::PermissionLevel::Read,
            relationship_type: crate::parse::metadata::RelationshipType::ManyToOne,
        });
        
        model.fields.push(cross_link_field);
        
        let definition_name: syn::Ident = parse_quote!(TestDef);
        let result = generate_model_cross_definition_links(&model, &definition_name).unwrap();
        let code = result.to_string();
        
        // Should generate wrapper type
        assert!(code.contains("TestModelrelated_itemLink"));
        
        // Should generate enum for cross-definition links
        assert!(code.contains("TestModelCrossDefinitionLinks"));
        
        // Should include relationship type and permission handling
        assert!(code.contains("CrossDefinitionRelationshipType"));
        assert!(code.contains("CrossDefinitionPermissionLevel"));
    }
    
    /// Helper function to create a simple model for testing
    fn create_simple_model() -> crate::parse::metadata::ModelMetadata {
        let mut model = crate::parse::metadata::ModelMetadata::new(
            parse_quote!(TestModel),
            parse_quote!(pub)
        );
        
        // Add a primary key field
        let mut pk_field = crate::parse::metadata::FieldMetadata::new(
            parse_quote!(id),
            parse_quote!(u64),
            parse_quote!(pub)
        );
        pk_field.is_primary_key = true;
        model.fields.push(pk_field);
        
        // Add a regular field
        let data_field = crate::parse::metadata::FieldMetadata::new(
            parse_quote!(data),
            parse_quote!(String),
            parse_quote!(pub)
        );
        model.fields.push(data_field);
        
        model
    }
}