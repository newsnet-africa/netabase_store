//! Test cross-definition linking functionality

use crate::generate::model::cross_definition_links::{
    generate_model_cross_definition_links,
    generate_cross_definition_support_types,
};
use crate::parse::metadata::{
    ModelMetadata, FieldMetadata, CrossDefinitionLink, RelationshipType, PermissionLevel
};
use syn::{parse_quote, Ident};

#[test]
fn test_generate_cross_definition_support_types() {
    let tokens = generate_cross_definition_support_types();
    let code = tokens.to_string();
    
    // Verify all necessary types are generated
    assert!(code.contains("CrossDefinitionRelationshipType"));
    assert!(code.contains("CrossDefinitionPermissionLevel"));
    assert!(code.contains("CrossDefinitionLinked"));
    assert!(code.contains("CrossDefinitionResolver"));
    
    // Verify enum variants
    assert!(code.contains("OneToOne"));
    assert!(code.contains("OneToMany")); 
    assert!(code.contains("ManyToOne"));
    assert!(code.contains("ManyToMany"));
    
    // Verify permission levels
    assert!(code.contains("None"));
    assert!(code.contains("Read"));
    assert!(code.contains("Write"));
    assert!(code.contains("ReadWrite"));
    assert!(code.contains("Admin"));
}

#[test]
fn test_generate_model_cross_definition_links_no_links() {
    // Model with no cross-definition links should generate empty code
    let model = create_simple_model();
    let definition_name: Ident = parse_quote!(TestDef);
    
    let result = generate_model_cross_definition_links(&model, &definition_name).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_generate_model_cross_definition_links_with_link() {
    // Model with cross-definition link
    let mut model = create_simple_model();
    
    // Add a field with cross-definition link
    let mut cross_link_field = FieldMetadata::new(
        parse_quote!(related_item),
        parse_quote!(String),
        parse_quote!(pub)
    );
    
    cross_link_field.cross_definition_link = Some(CrossDefinitionLink {
        target_path: parse_quote!(other_def::OtherModel),
        target_model: Some(parse_quote!(OtherModel)),
        required_permission: PermissionLevel::Read,
        relationship_type: RelationshipType::ManyToOne,
    });
    
    model.fields.push(cross_link_field);
    
    let definition_name: Ident = parse_quote!(TestDef);
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

#[test]
fn test_cross_definition_wrapper_generation() {
    let mut model = create_simple_model();
    
    // Add multiple cross-definition links
    let mut link1 = FieldMetadata::new(
        parse_quote!(user_link),
        parse_quote!(String),
        parse_quote!(pub)
    );
    link1.cross_definition_link = Some(CrossDefinitionLink {
        target_path: parse_quote!(user_def::User),
        target_model: Some(parse_quote!(User)),
        required_permission: PermissionLevel::Read,
        relationship_type: RelationshipType::ManyToOne,
    });
    
    let mut link2 = FieldMetadata::new(
        parse_quote!(category_link),
        parse_quote!(String),
        parse_quote!(pub)
    );
    link2.cross_definition_link = Some(CrossDefinitionLink {
        target_path: parse_quote!(category_def::Category),
        target_model: Some(parse_quote!(Category)),
        required_permission: PermissionLevel::ReadWrite,
        relationship_type: RelationshipType::OneToMany,
    });
    
    model.fields.push(link1);
    model.fields.push(link2);
    
    let definition_name: Ident = parse_quote!(TestDef);
    let result = generate_model_cross_definition_links(&model, &definition_name).unwrap();
    let code = result.to_string();
    
    // Should generate wrapper types for both links
    assert!(code.contains("TestModeluser_linkLink"));
    assert!(code.contains("TestModelcategory_linkLink"));
    
    // Should generate enum with both variants
    assert!(code.contains("TestModelCrossDefinitionLinks"));
    assert!(code.contains("user_linkLink"));
    assert!(code.contains("category_linkLink"));
    
    // Should include proper methods
    assert!(code.contains("target_id"));
    assert!(code.contains("target_path"));
    assert!(code.contains("can_access_with_permission"));
}

#[test]
fn test_cross_definition_path_parsing() {
    use crate::generate::model::cross_definition_links::parse_cross_definition_path;
    
    // Test complex path
    let path = parse_quote!(inner::InnerDefinition::InnerModel);
    let (def_path, model_name) = parse_cross_definition_path(&path).unwrap();
    
    assert_eq!(model_name.to_string(), "InnerModel");
    let def_str = def_path.to_string();
    assert!(def_str.contains("inner") && def_str.contains("InnerDefinition"));
    
    // Test simple path  
    let simple_path = parse_quote!(SimpleModel);
    let (simple_def_path, simple_model_name) = parse_cross_definition_path(&simple_path).unwrap();
    
    assert_eq!(simple_model_name.to_string(), "SimpleModel");
    assert!(simple_def_path.is_empty());
}

#[test] 
fn test_cross_definition_enum_methods() {
    let mut model = create_simple_model();
    
    let mut link_field = FieldMetadata::new(
        parse_quote!(target),
        parse_quote!(String),
        parse_quote!(pub)
    );
    
    link_field.cross_definition_link = Some(CrossDefinitionLink {
        target_path: parse_quote!(external::ExternalModel),
        target_model: Some(parse_quote!(ExternalModel)),
        required_permission: PermissionLevel::Admin,
        relationship_type: RelationshipType::OneToOne,
    });
    
    model.fields.push(link_field);
    
    let definition_name: Ident = parse_quote!(TestDef);
    let result = generate_model_cross_definition_links(&model, &definition_name).unwrap();
    let code = result.to_string();
    
    // Should have enum implementation methods
    assert!(code.contains("relationship_type(&self)"));
    assert!(code.contains("required_permission(&self)"));
    assert!(code.contains("target_path(&self)"));
    
    // Should have match statements for the methods
    assert!(code.contains("match self"));
}

/// Helper function to create a simple model for testing
fn create_simple_model() -> ModelMetadata {
    let mut model = ModelMetadata::new(
        parse_quote!(TestModel),
        parse_quote!(pub)
    );
    
    // Add a primary key field
    let mut pk_field = FieldMetadata::new(
        parse_quote!(id),
        parse_quote!(u64),
        parse_quote!(pub)
    );
    pk_field.is_primary_key = true;
    model.fields.push(pk_field);
    
    // Add a regular field
    let data_field = FieldMetadata::new(
        parse_quote!(data),
        parse_quote!(String),
        parse_quote!(pub)
    );
    model.fields.push(data_field);
    
    model
}