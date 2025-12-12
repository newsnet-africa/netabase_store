//! TOML Schema System Integration Tests
//!
//! This test suite validates the TOML-based schema system including:
//! - TOML schema parsing and validation
//! - Code generation from TOML schemas
//! - Manager schema coordination
//! - Cross-definition validation
//! - Tree naming consistency from TOML
//! - Permission validation from TOML

use netabase_store::{
    codegen::{
        parse_definition_schema, parse_manager_schema,
        validate_definition_schema, validate_manager_schema,
        validate_cross_definition_references,
        generate_definition_code, generate_manager_code,
        generate_tree_names,
    },
    error::NetabaseResult,
};
use std::path::Path;

#[cfg(test)]
mod toml_schema_tests {
    use super::*;

    // =============================================================================
    // TOML PARSING TESTS
    // =============================================================================

    #[test]
    fn test_parse_user_definition_schema() {
        let schema = parse_definition_schema("schemas/User.netabase.toml").unwrap();
        
        assert_eq!(schema.definition.name, "User");
        assert_eq!(schema.definition.version, "1");
        
        // Check model fields
        assert_eq!(schema.model.fields.len(), 7);
        assert_eq!(schema.model.fields[0].name, "id");
        assert_eq!(schema.model.fields[0].r#type, "u64");
        assert_eq!(schema.model.fields[1].name, "email");
        assert_eq!(schema.model.fields[1].r#type, "String");
        
        // Check primary key
        assert_eq!(schema.keys.primary.field, "id");
        assert_eq!(schema.keys.primary.key_type, Some("UserId".to_string()));
        
        // Check secondary keys
        let secondary = schema.keys.secondary.as_ref().unwrap();
        assert_eq!(secondary.len(), 3);
        assert_eq!(secondary[0].name, "Email");
        assert_eq!(secondary[0].field, "email");
        assert!(secondary[0].unique);
        assert_eq!(secondary[1].name, "Username");
        assert!(secondary[1].unique);
        assert_eq!(secondary[2].name, "Role");
        assert!(!secondary[2].unique);
        
        // Check subscriptions
        let subscriptions = schema.subscriptions.as_ref().unwrap();
        assert_eq!(subscriptions.len(), 2);
        assert_eq!(subscriptions[0].name, "UserEvents");
        assert_eq!(subscriptions[1].name, "Authentication");
        
        // Check permissions
        let permissions = schema.permissions.as_ref().unwrap();
        let can_ref_from = permissions.can_reference_from.as_ref().unwrap();
        assert_eq!(can_ref_from.len(), 2);
        assert!(can_ref_from.contains(&"Product".to_string()));
        assert!(can_ref_from.contains(&"Order".to_string()));
    }

    #[test]
    fn test_parse_product_definition_schema() {
        let schema = parse_definition_schema("schemas/Product.netabase.toml").unwrap();
        
        assert_eq!(schema.definition.name, "Product");
        
        // Check relational keys
        let relational = schema.keys.relational.as_ref().unwrap();
        assert_eq!(relational.len(), 1);
        assert_eq!(relational[0].name, "CreatedBy");
        assert_eq!(relational[0].target_definition, "User");
        assert_eq!(relational[0].target_model, "User");
        assert_eq!(relational[0].target_key_type, "UserId");
        
        // Check permissions
        let permissions = schema.permissions.as_ref().unwrap();
        let can_ref_to = permissions.can_reference_to.as_ref().unwrap();
        assert_eq!(can_ref_to.len(), 1);
        assert!(can_ref_to.contains(&"User".to_string()));
    }

    #[test]
    fn test_parse_manager_schema() {
        let schema = parse_manager_schema("ecommerce.root.netabase.toml").unwrap();
        
        assert_eq!(schema.manager.name, "EcommerceManager");
        assert_eq!(schema.manager.version, "1");
        assert_eq!(schema.manager.root_path, "./data");
        
        // Check definitions
        assert_eq!(schema.definitions.len(), 3);
        assert_eq!(schema.definitions[0].name, "User");
        assert_eq!(schema.definitions[0].schema_file, "schemas/User.netabase.toml");
        assert_eq!(schema.definitions[1].name, "Product");
        assert_eq!(schema.definitions[2].name, "Order");
        
        // Check permission roles
        let permissions = schema.permissions.as_ref().unwrap();
        let roles = permissions.roles.as_ref().unwrap();
        assert_eq!(roles.len(), 4);
        
        // Check Admin role
        let admin_role = &roles[0];
        assert_eq!(admin_role.name, "Admin");
        assert_eq!(admin_role.level, Some("ReadWrite".to_string()));
        let admin_defs = admin_role.definitions.as_ref().unwrap();
        assert_eq!(admin_defs[0], "*");
        
        // Check Manager role
        let manager_role = &roles[1];
        assert_eq!(manager_role.name, "Manager");
        let manager_read = manager_role.read.as_ref().unwrap();
        assert_eq!(manager_read.len(), 3);
        assert!(manager_read.contains(&"User".to_string()));
        let manager_write = manager_role.write.as_ref().unwrap();
        assert_eq!(manager_write.len(), 2);
        assert!(!manager_write.contains(&"User".to_string()));
        
        // Check Customer role
        let customer_role = &roles[2];
        assert_eq!(customer_role.name, "Customer");
        let customer_read = customer_role.read.as_ref().unwrap();
        assert_eq!(customer_read.len(), 1);
        assert!(customer_read.contains(&"Product".to_string()));
        let customer_write = customer_role.write.as_ref().unwrap();
        assert_eq!(customer_write.len(), 0);
    }

    // =============================================================================
    // TOML VALIDATION TESTS
    // =============================================================================

    #[test]
    fn test_validate_user_definition_schema() {
        let schema = parse_definition_schema("schemas/User.netabase.toml").unwrap();
        let result = validate_definition_schema(&schema);
        
        assert!(result.is_valid, "Validation errors: {:?}", result.errors);
        assert!(result.errors.is_empty());
        
        // Should have some warnings about custom types maybe, but no errors
        println!("Validation warnings: {:?}", result.warnings);
    }

    #[test]
    fn test_validate_all_definition_schemas() {
        let user_schema = parse_definition_schema("schemas/User.netabase.toml").unwrap();
        let product_schema = parse_definition_schema("schemas/Product.netabase.toml").unwrap();
        let order_schema = parse_definition_schema("schemas/Order.netabase.toml").unwrap();
        
        let user_result = validate_definition_schema(&user_schema);
        let product_result = validate_definition_schema(&product_schema);
        let order_result = validate_definition_schema(&order_schema);
        
        assert!(user_result.is_valid, "User validation errors: {:?}", user_result.errors);
        assert!(product_result.is_valid, "Product validation errors: {:?}", product_result.errors);
        assert!(order_result.is_valid, "Order validation errors: {:?}", order_result.errors);
    }

    #[test]
    fn test_validate_manager_schema() {
        let schema = parse_manager_schema("ecommerce.root.netabase.toml").unwrap();
        let result = validate_manager_schema(&schema);
        
        assert!(result.is_valid, "Manager validation errors: {:?}", result.errors);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_cross_definition_references() {
        let user_schema = parse_definition_schema("schemas/User.netabase.toml").unwrap();
        let product_schema = parse_definition_schema("schemas/Product.netabase.toml").unwrap();
        let order_schema = parse_definition_schema("schemas/Order.netabase.toml").unwrap();
        
        let schemas = vec![
            ("User".to_string(), user_schema),
            ("Product".to_string(), product_schema),
            ("Order".to_string(), order_schema),
        ];
        
        let result = validate_cross_definition_references(&schemas);
        
        assert!(result.is_valid, "Cross-definition validation errors: {:?}", result.errors);
        assert!(result.errors.is_empty());
    }

    // =============================================================================
    // TREE NAMING GENERATION TESTS
    // =============================================================================

    #[test]
    fn test_generate_tree_names_from_user_schema() {
        let schema = parse_definition_schema("schemas/User.netabase.toml").unwrap();
        let tree_names = generate_tree_names(&schema).unwrap();
        
        assert_eq!(tree_names.main_tree, "User::User::Main");
        assert_eq!(tree_names.hash_tree, "User::User::Hash");
        
        // Check secondary tree names
        assert_eq!(tree_names.secondary_trees.len(), 3);
        assert!(tree_names.secondary_trees.contains(&"User::User::Secondary::Email".to_string()));
        assert!(tree_names.secondary_trees.contains(&"User::User::Secondary::Username".to_string()));
        assert!(tree_names.secondary_trees.contains(&"User::User::Secondary::Role".to_string()));
        
        // Check subscription tree names
        assert_eq!(tree_names.subscription_trees.len(), 2);
        assert!(tree_names.subscription_trees.contains(&"User::User::Subscription::UserEvents".to_string()));
        assert!(tree_names.subscription_trees.contains(&"User::User::Subscription::Authentication".to_string()));
    }

    #[test]
    fn test_generate_tree_names_from_product_schema() {
        let schema = parse_definition_schema("schemas/Product.netabase.toml").unwrap();
        let tree_names = generate_tree_names(&schema).unwrap();
        
        assert_eq!(tree_names.main_tree, "Product::Product::Main");
        assert_eq!(tree_names.hash_tree, "Product::Product::Hash");
        
        // Check secondary tree names
        assert_eq!(tree_names.secondary_trees.len(), 3);
        assert!(tree_names.secondary_trees.contains(&"Product::Product::Secondary::Sku".to_string()));
        assert!(tree_names.secondary_trees.contains(&"Product::Product::Secondary::Name".to_string()));
        assert!(tree_names.secondary_trees.contains(&"Product::Product::Secondary::Category".to_string()));
        
        // Check relational tree names
        assert_eq!(tree_names.relational_trees.len(), 1);
        assert!(tree_names.relational_trees.contains(&"Product::Product::Relational::CreatedBy".to_string()));
    }

    // =============================================================================
    // CODE GENERATION TESTS
    // =============================================================================

    #[test]
    fn test_generate_code_from_user_schema() {
        let schema = parse_definition_schema("schemas/User.netabase.toml").unwrap();
        let generated = generate_definition_code(&schema).unwrap();
        
        assert_eq!(generated.definition_name, "User");
        assert_eq!(generated.models.len(), 1);
        assert_eq!(generated.models[0].name, "User");
        assert_eq!(generated.models[0].fields.len(), 7);
        
        // Check primary key
        assert_eq!(generated.keys.primary.key_type, "UserId");
        assert_eq!(generated.keys.primary.field, "id");
        
        // Check secondary keys
        assert_eq!(generated.keys.secondary.len(), 3);
        assert_eq!(generated.keys.secondary[0].name, "Email");
        assert_eq!(generated.keys.secondary[0].field, "email");
        assert!(generated.keys.secondary[0].unique);
        
        // Check that trait implementations and tree manager code are generated
        assert!(!generated.traits.is_empty());
        assert!(!generated.tree_manager.is_empty());
    }

    #[test]
    fn test_generate_manager_code() {
        let manager_schema = parse_manager_schema("ecommerce.root.netabase.toml").unwrap();
        
        let user_schema = parse_definition_schema("schemas/User.netabase.toml").unwrap();
        let product_schema = parse_definition_schema("schemas/Product.netabase.toml").unwrap();
        let order_schema = parse_definition_schema("schemas/Order.netabase.toml").unwrap();
        
        let definition_schemas = vec![
            ("User".to_string(), user_schema),
            ("Product".to_string(), product_schema),
            ("Order".to_string(), order_schema),
        ];
        
        let generated = generate_manager_code(&manager_schema, &definition_schemas).unwrap();
        
        // Should contain all definitions
        assert!(generated.contains("User"));
        assert!(generated.contains("Product"));
        assert!(generated.contains("Order"));
        
        // Should contain manager struct
        assert!(generated.contains("EcommerceManagerDefinitions"));
        assert!(generated.contains("EcommerceManagerManager"));
        
        println!("Generated manager code length: {} bytes", generated.len());
    }

    // =============================================================================
    // INVALID SCHEMA TESTS
    // =============================================================================

    #[test]
    fn test_invalid_schema_validation() {
        // Test schema with missing primary key field
        let invalid_toml = r#"
        [definition]
        name = "Invalid"
        version = "1"

        [model]
        fields = [
            { name = "email", type = "String" },
        ]

        [keys]
        [keys.primary]
        field = "id"  # This field doesn't exist in the model
        "#;

        let schema = netabase_store::codegen::parse_definition_schema_from_str(invalid_toml).unwrap();
        let result = validate_definition_schema(&schema);
        
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
        
        // Should have an error about missing primary key field
        assert!(result.errors.iter().any(|e| e.message.contains("Primary key field") && e.message.contains("not found")));
    }

    #[test]
    fn test_invalid_cross_definition_references() {
        let invalid_toml = r#"
        [definition]
        name = "InvalidProduct"
        version = "1"

        [model]
        fields = [
            { name = "id", type = "u64" },
            { name = "name", type = "String" },
        ]

        [keys]
        [keys.primary]
        field = "id"

        [[keys.relational]]
        name = "CreatedBy"
        target_definition = "NonexistentDefinition"  # This doesn't exist
        target_model = "User"
        target_key_type = "UserId"
        "#;

        let valid_user_schema = parse_definition_schema("schemas/User.netabase.toml").unwrap();
        let invalid_schema = netabase_store::codegen::parse_definition_schema_from_str(invalid_toml).unwrap();
        
        let schemas = vec![
            ("User".to_string(), valid_user_schema),
            ("InvalidProduct".to_string(), invalid_schema),
        ];
        
        let result = validate_cross_definition_references(&schemas);
        
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
        
        // Should have an error about unknown definition reference
        assert!(result.errors.iter().any(|e| 
            e.message.contains("NonexistentDefinition") && e.message.contains("unknown definition")
        ));
    }

    // =============================================================================
    // EDGE CASE TESTS
    // =============================================================================

    #[test]
    fn test_empty_secondary_keys() {
        let toml = r#"
        [definition]
        name = "Simple"
        version = "1"

        [model]
        fields = [
            { name = "id", type = "u64" },
            { name = "name", type = "String" },
        ]

        [keys]
        [keys.primary]
        field = "id"
        "#;

        let schema = netabase_store::codegen::parse_definition_schema_from_str(toml).unwrap();
        let result = validate_definition_schema(&schema);
        
        assert!(result.is_valid);
        assert!(schema.keys.secondary.is_none());
        
        let tree_names = generate_tree_names(&schema).unwrap();
        assert!(tree_names.secondary_trees.is_empty());
    }

    #[test]
    fn test_complex_field_types() {
        let toml = r#"
        [definition]
        name = "Complex"
        version = "1"

        [model]
        fields = [
            { name = "id", type = "u64" },
            { name = "data", type = "Vec<u8>" },
            { name = "optional_name", type = "Option<String>" },
            { name = "map", type = "HashMap<String, u32>" },
            { name = "timestamp", type = "chrono::DateTime<chrono::Utc>" },
        ]

        [keys]
        [keys.primary]
        field = "id"
        "#;

        let schema = netabase_store::codegen::parse_definition_schema_from_str(toml).unwrap();
        let result = validate_definition_schema(&schema);
        
        // Should be valid but may have warnings about custom types
        assert!(result.is_valid);
        
        // Should warn about custom types that need to be in scope
        assert!(result.warnings.iter().any(|w| w.contains("chrono::")));
        assert!(result.warnings.iter().any(|w| w.contains("HashMap")));
    }

    // =============================================================================
    // INTEGRATION TESTS
    // =============================================================================

    #[test]
    fn test_full_toml_to_code_pipeline() {
        // Test the complete pipeline: TOML -> Parse -> Validate -> Generate
        let schema = parse_definition_schema("schemas/User.netabase.toml").unwrap();
        
        // Validate
        let validation = validate_definition_schema(&schema);
        assert!(validation.is_valid, "Schema validation failed: {:?}", validation.errors);
        
        // Generate tree names
        let tree_names = generate_tree_names(&schema).unwrap();
        assert!(!tree_names.main_tree.is_empty());
        assert!(!tree_names.secondary_trees.is_empty());
        
        // Generate code
        let generated = generate_definition_code(&schema).unwrap();
        assert!(!generated.traits.is_empty());
        assert!(!generated.tree_manager.is_empty());
        
        // Verify generated code contains expected elements
        let formatted_code = netabase_store::codegen::generator::format_generated_code(&generated).unwrap();
        assert!(formatted_code.contains("struct User"));
        assert!(formatted_code.contains("struct UserId"));
        assert!(formatted_code.contains("UserSecondaryKeys"));
        assert!(formatted_code.contains("NetabaseModelTrait"));
        assert!(formatted_code.contains("MAIN_TREE_NAME"));
    }

    #[test]
    fn test_manager_coordination_pipeline() {
        // Test the complete manager pipeline
        let manager_schema = parse_manager_schema("ecommerce.root.netabase.toml").unwrap();
        
        // Validate manager schema
        let manager_validation = validate_manager_schema(&manager_schema);
        assert!(manager_validation.is_valid, "Manager validation failed: {:?}", manager_validation.errors);
        
        // Load and validate all definition schemas
        let definition_schemas = netabase_store::codegen::load_all_definition_schemas(
            &manager_schema,
            Some(Path::new("."))
        ).unwrap();
        
        assert_eq!(definition_schemas.len(), 3);
        
        // Validate cross-definition references
        let cross_validation = validate_cross_definition_references(&definition_schemas);
        assert!(cross_validation.is_valid, "Cross-validation failed: {:?}", cross_validation.errors);
        
        // Generate complete manager code
        let manager_code = generate_manager_code(&manager_schema, &definition_schemas).unwrap();
        assert!(!manager_code.is_empty());
        assert!(manager_code.contains("EcommerceManagerDefinitions"));
        
        println!("Manager code generated successfully - {} bytes", manager_code.len());
    }
}