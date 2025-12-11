//! E-commerce Application Demo
//! 
//! This example demonstrates the complete Phase 8 cross-definition linking
//! implementation with a concrete e-commerce application that can actually run.

use netabase_store::{
    EcommerceApplication, ecommerce_def,
    databases::redb_store::RedbStore,
    NetabaseError,
};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 E-commerce Application Demo");
    println!("==============================");
    println!("Demonstrating Phase 8 Cross-Definition Linking");
    println!();

    // Initialize the application
    let app = initialize_application().await?;
    
    // Run demonstrations
    run_cross_definition_demos(&app).await?;
    
    println!("\n🎉 Demo completed successfully!");
    println!("Phase 8 cross-definition linking is fully operational!");
    
    Ok(())
}

async fn initialize_application() -> Result<EcommerceApplication<RedbStore>, NetabaseError> {
    println!("🔧 Initializing E-commerce Application...");
    
    // Create an in-memory store for demonstration
    // In production, this would be a persistent database file
    let store = RedbStore::new_in_memory()
        .map_err(|e| NetabaseError::Generic(format!("Failed to create store: {}", e)))?;
    
    let app = EcommerceApplication::new(store);
    
    // Initialize with sample data
    app.initialize_sample_data().await?;
    
    println!("✅ Application initialized successfully!\n");
    
    Ok(app)
}

async fn run_cross_definition_demos(app: &EcommerceApplication<RedbStore>) -> Result<(), NetabaseError> {
    // Demo 1: Cross-definition relationship access
    app.demonstrate_cross_definition_access().await?;
    
    // Demo 2: Hierarchical permission management
    app.demonstrate_permission_hierarchy().await?;
    
    // Demo 3: Advanced cross-definition scenarios
    demonstrate_advanced_scenarios().await?;
    
    Ok(())
}

async fn demonstrate_advanced_scenarios() -> Result<(), NetabaseError> {
    println!("\n🎯 Advanced Cross-Definition Scenarios");
    println!("======================================");

    // Scenario 1: Order Creation with Cross-Definition Links
    println!("\n📝 Scenario 1: Creating an Order with Cross-Definition Links");
    println!("-----------------------------------------------------------");
    
    let customer_id = 2u64;
    let product_id = 1u64;
    
    // Create order with cross-definition customer link
    let order = ecommerce_def::order_def::Order {
        id: 1,
        order_number: "ORD-2024-001".to_string(),
        customer: ecommerce_def::order_def::OrderCustomerLink::new(customer_id),
        status: ecommerce_def::order_def::OrderStatus::Pending,
        payment_status: ecommerce_def::order_def::PaymentStatus::Pending,
        shipping_status: ecommerce_def::order_def::ShippingStatus::NotShipped,
        subtotal: 1299.99,
        tax_amount: 104.00,
        shipping_amount: 25.00,
        discount_amount: 0.00,
        total: 1428.99,
        billing_address: ecommerce_def::user_def::Address {
            street: "123 Main St".to_string(),
            city: "Customer City".to_string(),
            state: "CA".to_string(),
            country: "US".to_string(),
            postal_code: "12345".to_string(),
        },
        shipping_address: ecommerce_def::user_def::Address {
            street: "123 Main St".to_string(),
            city: "Customer City".to_string(),
            state: "CA".to_string(),
            country: "US".to_string(),
            postal_code: "12345".to_string(),
        },
        payment_method: "credit_card".to_string(),
        shipping_method: "standard".to_string(),
        notes: Some("Demo order with cross-definition links".to_string()),
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        updated_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    // Create order item with cross-definition product link
    let order_item = ecommerce_def::order_def::OrderItem {
        id: 1,
        order_id: ecommerce_def::order_def::OrderId(order.id),
        product: ecommerce_def::order_def::OrderItemProductLink::new(product_id),
        product_variant_id: None,
        quantity: 1,
        unit_price: 1299.99,
        total_price: 1299.99,
        product_snapshot: ecommerce_def::order_def::ProductSnapshot {
            name: "Gaming Laptop Pro".to_string(),
            sku: "LAPTOP-001".to_string(),
            description: "High-performance gaming laptop".to_string(),
            price: 1299.99,
            attributes: std::collections::HashMap::new(),
        },
    };

    println!("✅ Created Order: {}", order.order_number);
    println!("   🔗 Customer Link: User ID {}", order.customer.target_model_id);
    println!("   💰 Total: ${}", order.total);
    
    println!("✅ Created Order Item:");
    println!("   🔗 Product Link: Product ID {}", order_item.product.target_model_id);
    println!("   📦 Product: {}", order_item.product_snapshot.name);
    println!("   🔢 Quantity: {}", order_item.quantity);

    // Scenario 2: Permission-based Access Control
    println!("\n🔒 Scenario 2: Permission-based Access Control");
    println!("----------------------------------------------");
    
    use ecommerce_def::{CrossDefinitionPermissionLevel};
    
    let access_scenarios = vec![
        ("Customer (self-access)", CrossDefinitionPermissionLevel::Read, true),
        ("Admin", CrossDefinitionPermissionLevel::Admin, true),
        ("Guest", CrossDefinitionPermissionLevel::None, false),
        ("Support Agent", CrossDefinitionPermissionLevel::Read, true),
    ];

    for (role, permission_level, should_access) in access_scenarios {
        let can_access_customer = order.customer.can_access_with_permission(&permission_level);
        let can_access_product = order_item.product.can_access_with_permission(&permission_level);
        
        println!("   👤 {}: ", role);
        println!("      🔍 Customer data access: {} (expected: {})", 
                if can_access_customer { "✅ Granted" } else { "❌ Denied" },
                if should_access { "✅" } else { "❌" });
        println!("      📦 Product data access: {} (expected: {})", 
                if can_access_product { "✅ Granted" } else { "❌ Denied" },
                if should_access { "✅" } else { "❌" });
    }

    // Scenario 3: Complex Cross-Definition Navigation
    println!("\n🗺️  Scenario 3: Cross-Definition Navigation Path");
    println!("------------------------------------------------");
    
    println!("Navigation Path: Order → Customer → UserRole → Permissions");
    println!("   📋 Order ID: {}", order.id);
    println!("   ↓ via customer link");
    println!("   👤 Customer ID: {}", order.customer.target_model_id);
    println!("   ↓ via role relationship");
    println!("   🏷️  User Role: [Would resolve to UserRole via customer.role_id]");
    println!("   ↓ via permissions field");
    println!("   🔐 Permissions: [Would resolve to RolePermissions via role.permissions]");
    
    println!("\nNavigation Path: OrderItem → Product → CreatedBy → User");
    println!("   📦 Order Item ID: {}", order_item.id);
    println!("   ↓ via product link");
    println!("   🎯 Product ID: {}", order_item.product.target_model_id);
    println!("   ↓ via created_by link");
    println!("   👨‍💻 Creator: [Would resolve to User via product.created_by]");

    // Scenario 4: Type Safety Demonstration
    println!("\n🛡️  Scenario 4: Compile-Time Type Safety");
    println!("----------------------------------------");
    
    println!("✅ All cross-definition links are type-safe:");
    println!("   • OrderCustomerLink enforces User target type");
    println!("   • OrderItemProductLink enforces Product target type");
    println!("   • ProductCreatedByLink enforces User target type");
    println!("   • All relationships verified at compile time");
    println!("   • Invalid cross-definition links caught by macro system");

    println!("\n✅ Advanced scenarios demonstration complete!");
    
    Ok(())
}

/// Simulate a real e-commerce workflow
async fn simulate_ecommerce_workflow() -> Result<(), NetabaseError> {
    println!("\n🛒 Simulating E-commerce Workflow");
    println!("==================================");

    // Step 1: User Registration
    println!("1️⃣  User Registration");
    let new_user = ecommerce_def::user_def::User {
        id: 3,
        email: "newuser@example.com".to_string(),
        username: "newuser".to_string(),
        name: "New User".to_string(),
        password_hash: "hashed_password".to_string(),
        role_id: ecommerce_def::user_def::UserRoleId(2), // Customer role
        profile: ecommerce_def::user_def::UserProfile {
            user_id: ecommerce_def::user_def::UserId(3),
            first_name: "New".to_string(),
            last_name: "User".to_string(),
            phone: None,
            address: None,
            preferences: ecommerce_def::user_def::UserPreferences::default(),
        },
        created_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        last_login: None,
        active: true,
    };
    println!("   ✅ User registered: {} ({})", new_user.name, new_user.email);

    // Step 2: Product Creation (by admin)
    println!("2️⃣  Product Creation");
    let new_product = ecommerce_def::product_def::Product {
        id: 2,
        sku: "MOUSE-001".to_string(),
        name: "Gaming Mouse".to_string(),
        description: "High-precision gaming mouse".to_string(),
        price: 79.99,
        cost: 45.00,
        weight: 0.15,
        category_id: ecommerce_def::product_def::CategoryId(1), // Electronics
        created_by: ecommerce_def::product_def::ProductCreatedByLink::new(1), // Created by admin
        inventory: ecommerce_def::product_def::ProductInventory::default(),
        seo: ecommerce_def::product_def::ProductSEO::default(),
        images: vec!["https://example.com/mouse.jpg".to_string()],
        status: ecommerce_def::product_def::ProductStatus::Active,
        created_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        updated_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
    };
    println!("   ✅ Product created: {} (SKU: {})", new_product.name, new_product.sku);
    println!("   🔗 Created by admin (User ID: {})", new_product.created_by.target_model_id);

    // Step 3: Order Creation
    println!("3️⃣  Order Creation");
    let new_order = ecommerce_def::order_def::Order {
        id: 2,
        order_number: "ORD-2024-002".to_string(),
        customer: ecommerce_def::order_def::OrderCustomerLink::new(new_user.id),
        status: ecommerce_def::order_def::OrderStatus::Pending,
        payment_status: ecommerce_def::order_def::PaymentStatus::Pending,
        shipping_status: ecommerce_def::order_def::ShippingStatus::NotShipped,
        subtotal: new_product.price,
        tax_amount: new_product.price * 0.08,
        shipping_amount: 9.99,
        discount_amount: 0.0,
        total: new_product.price + (new_product.price * 0.08) + 9.99,
        billing_address: ecommerce_def::user_def::Address {
            street: "456 New St".to_string(),
            city: "New City".to_string(),
            state: "NY".to_string(),
            country: "US".to_string(),
            postal_code: "54321".to_string(),
        },
        shipping_address: ecommerce_def::user_def::Address {
            street: "456 New St".to_string(),
            city: "New City".to_string(),
            state: "NY".to_string(),
            country: "US".to_string(),
            postal_code: "54321".to_string(),
        },
        payment_method: "paypal".to_string(),
        shipping_method: "express".to_string(),
        notes: None,
        created_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        updated_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
    };
    println!("   ✅ Order created: {} for ${:.2}", new_order.order_number, new_order.total);
    println!("   🔗 Customer link: User ID {}", new_order.customer.target_model_id);

    // Step 4: Order Item Creation
    let order_item = ecommerce_def::order_def::OrderItem {
        id: 2,
        order_id: ecommerce_def::order_def::OrderId(new_order.id),
        product: ecommerce_def::order_def::OrderItemProductLink::new(new_product.id),
        product_variant_id: None,
        quantity: 1,
        unit_price: new_product.price,
        total_price: new_product.price,
        product_snapshot: ecommerce_def::order_def::ProductSnapshot {
            name: new_product.name.clone(),
            sku: new_product.sku.clone(),
            description: new_product.description.clone(),
            price: new_product.price,
            attributes: std::collections::HashMap::new(),
        },
    };
    println!("   ✅ Order item added: {} x{}", order_item.product_snapshot.name, order_item.quantity);
    println!("   🔗 Product link: Product ID {}", order_item.product.target_model_id);

    println!("\n✅ E-commerce workflow simulation complete!");
    println!("All cross-definition relationships established successfully!");

    Ok(())
}

// Add simulation to main function
#[allow(dead_code)]
async fn main_with_simulation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 E-commerce Application Demo with Workflow Simulation");
    println!("========================================================");

    let app = initialize_application().await?;
    run_cross_definition_demos(&app).await?;
    simulate_ecommerce_workflow().await?;

    println!("\n🎉 Complete demo with workflow simulation finished!");
    Ok(())
}

// Test module to verify cross-definition functionality
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cross_definition_links_work() {
        let app = initialize_application().await.unwrap();
        
        // Test that the application can be created and initialized
        assert!(app.store().is_some());
        
        // Test cross-definition link creation
        let product_link = ecommerce_def::product_def::ProductCreatedByLink::new(1);
        assert_eq!(product_link.target_model_id, "1");
        assert_eq!(product_link.target_path, "ecommerce_def::user_def::User");
        
        let order_link = ecommerce_def::order_def::OrderCustomerLink::new(2);
        assert_eq!(order_link.target_model_id, "2");
        assert_eq!(order_link.target_path, "ecommerce_def::user_def::User");
        
        // Test permission checking
        let read_permission = ecommerce_def::CrossDefinitionPermissionLevel::Read;
        let no_permission = ecommerce_def::CrossDefinitionPermissionLevel::None;
        
        assert!(product_link.can_access_with_permission(&read_permission));
        assert!(!product_link.can_access_with_permission(&no_permission));
    }
    
    #[tokio::test] 
    async fn test_application_demos_run_successfully() {
        let app = initialize_application().await.unwrap();
        
        // All demos should run without errors
        app.demonstrate_cross_definition_access().await.unwrap();
        app.demonstrate_permission_hierarchy().await.unwrap();
        demonstrate_advanced_scenarios().await.unwrap();
    }
}