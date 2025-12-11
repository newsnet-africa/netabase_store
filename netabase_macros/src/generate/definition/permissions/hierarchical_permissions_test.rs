//! Example test for Phase 8: Nested Definitions with Hierarchical Permissions
//!
//! This test demonstrates the enhanced functionality where parent definitions
//! manage child definitions through a tree-like permission system with
//! enum-based type safety for cross-definition relationships.

use quote::quote;
use syn::parse_quote;

use crate::parse::metadata::{
    ModuleMetadata, ModelMetadata, FieldMetadata, PermissionLevel, 
    ChildPermissionGrant, CrossDefinitionLink, RelationshipType
};
use super::hierarchical_permissions::generate_hierarchical_permissions;

#[test]
fn test_hierarchical_permission_system() {
    // Create a parent definition (Restaurant) with child definitions
    let mut restaurant_module = ModuleMetadata::new(
        parse_quote!(restaurant_def),
        parse_quote!(RestaurantDef),
        parse_quote!(RestaurantDefKeys)
    );

    // Add a model to the parent definition
    let mut restaurant_model = ModelMetadata::new(parse_quote!(Restaurant), parse_quote!(pub));
    let mut pk = FieldMetadata::new(
        parse_quote!(id),
        parse_quote!(u64),
        parse_quote!(pub)
    );
    pk.is_primary_key = true;
    restaurant_model.add_field(pk);
    restaurant_module.add_model(restaurant_model);

    // Create child definitions
    let mut user_module = create_user_definition();
    let mut product_module = create_product_definition_with_cross_link();

    // Set up permission hierarchy - Restaurant can manage both User and Product
    restaurant_module.add_child_permission(ChildPermissionGrant {
        child_name: parse_quote!(UserDef),
        permission_level: PermissionLevel::Admin,
        cross_sibling_access: true, // Users can access Products
    });

    restaurant_module.add_child_permission(ChildPermissionGrant {
        child_name: parse_quote!(ProductDef),
        permission_level: PermissionLevel::ReadWrite,
        cross_sibling_access: false, // Products cannot access Users directly
    });

    // Add children to parent
    restaurant_module.add_nested_module(user_module);
    restaurant_module.add_nested_module(product_module);

    // Generate hierarchical permission system
    let tokens = generate_hierarchical_permissions(&restaurant_module);
    let code = tokens.to_string();

    // Verify key components are generated
    assert!(code.contains("RestaurantDefPermissionManager"));
    assert!(code.contains("DelegateUserDef"));
    assert!(code.contains("DelegateProductDef"));
    assert!(code.contains("CrossAccessUserDef"));
    assert!(code.contains("CrossAccessProductDef"));
    assert!(code.contains("PermissionLevel"));
    
    // Verify permission checking methods
    assert!(code.contains("can_access_"));
    assert!(code.contains("can_cross_access_"));
    assert!(code.contains("can_manage_child_permissions"));
    assert!(code.contains("propagate_permission_check"));

    println!("Generated hierarchical permission code:\n{}", code);
}

#[test]
fn test_cross_definition_link_types() {
    let product_module = create_product_definition_with_cross_link();
    let tokens = generate_hierarchical_permissions(&product_module);
    let code = tokens.to_string();

    // Verify cross-definition link types are generated
    assert!(code.contains("CrossDefinitionLinks"));
    assert!(code.contains("CreatedBy")); // The field name in PascalCase

    println!("Generated cross-definition link types:\n{}", code);
}

#[test]
fn test_tree_permission_propagation() {
    // Test that permission checks can propagate up the hierarchy
    let mut parent = ModuleMetadata::new(
        parse_quote!(parent_def),
        parse_quote!(ParentDef),
        parse_quote!(ParentDefKeys)
    );

    // Create nested hierarchy: Parent -> Child -> GrandChild
    let mut child = ModuleMetadata::new(
        parse_quote!(child_def),
        parse_quote!(ChildDef),
        parse_quote!(ChildDefKeys)
    );

    let grandchild = ModuleMetadata::new(
        parse_quote!(grandchild_def),
        parse_quote!(GrandChildDef),
        parse_quote!(GrandChildDefKeys)
    );

    // Set up hierarchical permissions
    child.add_child_permission(ChildPermissionGrant {
        child_name: parse_quote!(GrandChildDef),
        permission_level: PermissionLevel::Read,
        cross_sibling_access: false,
    });

    parent.add_child_permission(ChildPermissionGrant {
        child_name: parse_quote!(ChildDef),
        permission_level: PermissionLevel::Admin,
        cross_sibling_access: false,
    });

    child.add_nested_module(grandchild);
    parent.add_nested_module(child);

    let tokens = generate_hierarchical_permissions(&parent);
    let code = tokens.to_string();

    // Should generate delegation patterns for multi-level hierarchy
    assert!(code.contains("DelegateChildDef"));
    assert!(code.contains("propagate_permission_check"));

    println!("Generated hierarchical delegation code:\n{}", code);
}

/// Helper function to create a User definition
fn create_user_definition() -> ModuleMetadata {
    let mut user_module = ModuleMetadata::new(
        parse_quote!(user_def),
        parse_quote!(UserDef),
        parse_quote!(UserDefKeys)
    );

    let mut user_model = ModelMetadata::new(parse_quote!(User), parse_quote!(pub));
    
    // Primary key
    let mut pk = FieldMetadata::new(
        parse_quote!(id),
        parse_quote!(u64),
        parse_quote!(pub)
    );
    pk.is_primary_key = true;
    user_model.add_field(pk);

    // Secondary key
    let mut email = FieldMetadata::new(
        parse_quote!(email),
        parse_quote!(String),
        parse_quote!(pub)
    );
    email.is_secondary_key = true;
    user_model.add_field(email);

    user_module.add_model(user_model);
    user_module
}

/// Helper function to create a Product definition with cross-definition links
fn create_product_definition_with_cross_link() -> ModuleMetadata {
    let mut product_module = ModuleMetadata::new(
        parse_quote!(product_def),
        parse_quote!(ProductDef),
        parse_quote!(ProductDefKeys)
    );

    let mut product_model = ModelMetadata::new(parse_quote!(Product), parse_quote!(pub));
    
    // Primary key
    let mut pk = FieldMetadata::new(
        parse_quote!(id),
        parse_quote!(u64),
        parse_quote!(pub)
    );
    pk.is_primary_key = true;
    product_model.add_field(pk);

    // Cross-definition relationship to User
    let mut created_by = FieldMetadata::new(
        parse_quote!(created_by),
        parse_quote!(UserId),
        parse_quote!(pub)
    );
    created_by.is_relation = true;
    created_by.cross_definition_link = Some(CrossDefinitionLink {
        target_path: parse_quote!(user_def::UserDef),
        target_model: Some(parse_quote!(User)),
        required_permission: PermissionLevel::Read,
        relationship_type: RelationshipType::ManyToOne,
    });
    product_model.add_field(created_by);

    product_module.add_model(product_model);
    product_module
}

#[test]
fn test_enum_based_type_safety() {
    // Test that enums are generated for type-safe cross-definition relationships
    let mut parent = ModuleMetadata::new(
        parse_quote!(app_def),
        parse_quote!(AppDef),
        parse_quote!(AppDefKeys)
    );

    // Add multiple child definitions to test enum variants
    let user_child = create_user_definition();
    let product_child = create_product_definition_with_cross_link();
    
    parent.add_nested_module(user_child);
    parent.add_nested_module(product_child);

    let tokens = generate_hierarchical_permissions(&parent);
    let code = tokens.to_string();

    // Should generate enum discriminants for type safety
    assert!(code.contains("EnumDiscriminants"));
    
    // Should generate proper enum variants for each child
    assert!(code.contains("DelegateUserDef"));
    assert!(code.contains("DelegateProductDef"));

    println!("Generated enum-based type safety code:\n{}", code);
}