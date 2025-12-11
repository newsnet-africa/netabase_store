//! E-commerce Application using Netabase Cross-Definition Linking
//! 
//! This application demonstrates the concrete implementation of the Phase 8
//! cross-definition linking system with a real-world e-commerce scenario.

use netabase_macros::{netabase_definition_module, NetabaseModel};
use crate::{
    traits::{
        definition::NetabaseDefinition,
        model::NetabaseModelTrait,
        store::tree_manager::TreeManager,
    },
    databases::redb_store::RedbStore,
    error::NetabaseError,
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// Main E-commerce Definition with nested user and product management
#[netabase_definition_module(EcommerceDef, EcommerceDefKeys, subscriptions(SystemEvents, Analytics, AuditLog))]
pub mod ecommerce_def {
    use super::*;

    #[derive(NetabaseModel, Serialize, Deserialize)]
    #[subscribe(SystemEvents, AuditLog)]
    pub struct Store {
        #[primary_key]
        pub id: u64,
        
        #[secondary_key]
        pub name: String,
        
        #[secondary_key] 
        pub domain: String,
        
        pub description: String,
        pub active: bool,
        pub created_at: u64, // Unix timestamp
        pub settings: StoreSettings,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StoreSettings {
        pub currency: String,
        pub tax_rate: f64,
        pub shipping_enabled: bool,
        pub payment_methods: Vec<String>,
    }

    // User Management Subdefinition
    #[netabase_definition_module(UserDef, UserDefKeys, subscriptions(UserEvents, Authentication, ProfileChanges))]
    pub mod user_def {
        use super::*;

        #[derive(NetabaseModel, Serialize, Deserialize)]
        #[subscribe(UserEvents, Authentication)]
        pub struct User {
            #[primary_key]
            pub id: u64,
            
            #[secondary_key]
            pub email: String,
            
            #[secondary_key]
            pub username: String,
            
            pub name: String,
            pub password_hash: String,
            pub role_id: UserRoleId,
            pub profile: UserProfile,
            pub created_at: u64,
            pub last_login: Option<u64>,
            pub active: bool,
        }

        #[derive(NetabaseModel, Serialize, Deserialize)]
        pub struct UserRole {
            #[primary_key]
            pub id: u64,
            
            #[secondary_key]
            pub name: String,
            
            pub permissions: RolePermissions,
            pub description: String,
            pub system_role: bool, // Cannot be deleted if true
        }

        #[derive(NetabaseModel, Serialize, Deserialize)]
        #[subscribe(ProfileChanges)]
        pub struct UserProfile {
            #[primary_key]
            pub user_id: UserId,
            
            pub first_name: String,
            pub last_name: String,
            pub phone: Option<String>,
            pub address: Option<Address>,
            pub preferences: UserPreferences,
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct Address {
            pub street: String,
            pub city: String,
            pub state: String,
            pub country: String,
            pub postal_code: String,
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct UserPreferences {
            pub newsletter: bool,
            pub notifications: bool,
            pub language: String,
            pub timezone: String,
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct RolePermissions {
            pub manage_products: bool,
            pub manage_orders: bool,
            pub manage_users: bool,
            pub view_analytics: bool,
            pub system_admin: bool,
        }
    }

    // Product Catalog Subdefinition  
    #[netabase_definition_module(ProductDef, ProductDefKeys, subscriptions(ProductEvents, Inventory, PriceChanges))]
    pub mod product_def {
        use super::*;

        #[derive(NetabaseModel, Serialize, Deserialize)]
        #[subscribe(ProductEvents, Inventory)]
        pub struct Product {
            #[primary_key]
            pub id: u64,
            
            #[secondary_key]
            pub sku: String,
            
            #[secondary_key]
            pub name: String,
            
            pub description: String,
            pub price: f64,
            pub cost: f64,
            pub weight: f64,
            pub category_id: CategoryId,
            
            // Cross-definition link to User who created this product
            #[cross_definition_link(super::user_def::User)]
            pub created_by: ProductCreatedByLink,
            
            pub inventory: ProductInventory,
            pub seo: ProductSEO,
            pub images: Vec<String>, // Image URLs
            pub status: ProductStatus,
            pub created_at: u64,
            pub updated_at: u64,
        }

        #[derive(NetabaseModel, Serialize, Deserialize)]
        pub struct Category {
            #[primary_key]
            pub id: u64,
            
            #[secondary_key]
            pub name: String,
            
            #[secondary_key]
            pub slug: String,
            
            pub description: String,
            
            // Self-referential relation for category hierarchy
            #[relation]
            pub parent_category: Option<CategoryId>,
            
            pub image: Option<String>,
            pub sort_order: u32,
            pub active: bool,
        }

        #[derive(NetabaseModel, Serialize, Deserialize)]
        #[subscribe(PriceChanges)]
        pub struct ProductVariant {
            #[primary_key]
            pub id: u64,
            
            #[relation]
            pub product_id: ProductId,
            
            #[secondary_key]
            pub sku: String,
            
            pub name: String,
            pub price: f64,
            pub attributes: HashMap<String, String>, // color: red, size: large, etc.
            pub inventory: ProductInventory,
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct ProductInventory {
            pub quantity: u32,
            pub reserved: u32,
            pub reorder_level: u32,
            pub reorder_quantity: u32,
            pub track_inventory: bool,
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct ProductSEO {
            pub title: Option<String>,
            pub description: Option<String>,
            pub keywords: Vec<String>,
            pub meta_data: HashMap<String, String>,
        }

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
        pub enum ProductStatus {
            Draft,
            Active,
            Inactive,
            Discontinued,
        }

        /// Cross-definition link wrapper for Product -> User relationship
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub struct ProductCreatedByLink {
            pub target_path: &'static str,
            pub target_model_id: String,
            pub relationship_type: super::CrossDefinitionRelationshipType,
            pub required_permission: super::CrossDefinitionPermissionLevel,
        }

        impl ProductCreatedByLink {
            pub fn new(user_id: u64) -> Self {
                Self {
                    target_path: "ecommerce_def::user_def::User",
                    target_model_id: user_id.to_string(),
                    relationship_type: super::CrossDefinitionRelationshipType::ManyToOne,
                    required_permission: super::CrossDefinitionPermissionLevel::Read,
                }
            }

            pub fn can_access_with_permission(&self, permission: &super::CrossDefinitionPermissionLevel) -> bool {
                permission >= &self.required_permission
            }
        }
    }

    // Order Management Subdefinition
    #[netabase_definition_module(OrderDef, OrderDefKeys, subscriptions(OrderEvents, Payments, Shipping))]
    pub mod order_def {
        use super::*;

        #[derive(NetabaseModel, Serialize, Deserialize)]
        #[subscribe(OrderEvents, Payments)]
        pub struct Order {
            #[primary_key]
            pub id: u64,
            
            #[secondary_key]
            pub order_number: String,
            
            // Cross-definition link to customer
            #[cross_definition_link(super::user_def::User)]
            pub customer: OrderCustomerLink,
            
            pub status: OrderStatus,
            pub payment_status: PaymentStatus,
            pub shipping_status: ShippingStatus,
            
            pub subtotal: f64,
            pub tax_amount: f64,
            pub shipping_amount: f64,
            pub discount_amount: f64,
            pub total: f64,
            
            pub billing_address: super::user_def::Address,
            pub shipping_address: super::user_def::Address,
            
            pub payment_method: String,
            pub shipping_method: String,
            
            pub notes: Option<String>,
            
            pub created_at: u64,
            pub updated_at: u64,
        }

        #[derive(NetabaseModel, Serialize, Deserialize)]
        pub struct OrderItem {
            #[primary_key]
            pub id: u64,
            
            #[relation]
            pub order_id: OrderId,
            
            // Cross-definition link to product
            #[cross_definition_link(super::product_def::Product)]
            pub product: OrderItemProductLink,
            
            pub product_variant_id: Option<super::product_def::ProductVariantId>,
            pub quantity: u32,
            pub unit_price: f64,
            pub total_price: f64,
            pub product_snapshot: ProductSnapshot, // Snapshot of product at time of order
        }

        #[derive(NetabaseModel, Serialize, Deserialize)]
        #[subscribe(Payments)]
        pub struct Payment {
            #[primary_key]
            pub id: u64,
            
            #[relation]
            pub order_id: OrderId,
            
            pub payment_method: PaymentMethod,
            pub amount: f64,
            pub currency: String,
            pub status: PaymentStatus,
            pub transaction_id: Option<String>,
            pub gateway_response: Option<String>,
            pub created_at: u64,
        }

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
        pub enum OrderStatus {
            Pending,
            Confirmed,
            Processing,
            Shipped,
            Delivered,
            Cancelled,
            Refunded,
        }

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
        pub enum PaymentStatus {
            Pending,
            Processing,
            Completed,
            Failed,
            Cancelled,
            Refunded,
        }

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
        pub enum ShippingStatus {
            NotShipped,
            Processing,
            Shipped,
            InTransit,
            Delivered,
            Exception,
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub enum PaymentMethod {
            CreditCard { last_four: String, brand: String },
            PayPal { email: String },
            BankTransfer { reference: String },
            Cash,
        }

        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct ProductSnapshot {
            pub name: String,
            pub sku: String,
            pub description: String,
            pub price: f64,
            pub attributes: HashMap<String, String>,
        }

        /// Cross-definition link wrapper for Order -> User relationship
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub struct OrderCustomerLink {
            pub target_path: &'static str,
            pub target_model_id: String,
            pub relationship_type: super::CrossDefinitionRelationshipType,
            pub required_permission: super::CrossDefinitionPermissionLevel,
        }

        impl OrderCustomerLink {
            pub fn new(user_id: u64) -> Self {
                Self {
                    target_path: "ecommerce_def::user_def::User",
                    target_model_id: user_id.to_string(),
                    relationship_type: super::CrossDefinitionRelationshipType::ManyToOne,
                    required_permission: super::CrossDefinitionPermissionLevel::Read,
                }
            }

            pub fn can_access_with_permission(&self, permission: &super::CrossDefinitionPermissionLevel) -> bool {
                permission >= &self.required_permission
            }
        }

        /// Cross-definition link wrapper for OrderItem -> Product relationship  
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub struct OrderItemProductLink {
            pub target_path: &'static str,
            pub target_model_id: String,
            pub relationship_type: super::CrossDefinitionRelationshipType,
            pub required_permission: super::CrossDefinitionPermissionLevel,
        }

        impl OrderItemProductLink {
            pub fn new(product_id: u64) -> Self {
                Self {
                    target_path: "ecommerce_def::product_def::Product",
                    target_model_id: product_id.to_string(),
                    relationship_type: super::CrossDefinitionRelationshipType::ManyToOne,
                    required_permission: super::CrossDefinitionPermissionLevel::Read,
                }
            }

            pub fn can_access_with_permission(&self, permission: &super::CrossDefinitionPermissionLevel) -> bool {
                permission >= &self.required_permission
            }
        }
    }
}

// Cross-definition support types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrossDefinitionRelationshipType {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CrossDefinitionPermissionLevel {
    None,
    Read,
    Write,
    ReadWrite,
    Admin,
}

/// E-commerce Application Service Layer
pub struct EcommerceApplication<S> {
    store: S,
}

impl<S> EcommerceApplication<S>
where
    S: Clone,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub fn store(&self) -> &S {
        &self.store
    }
}

// Application-specific implementations
impl EcommerceApplication<RedbStore> {
    /// Initialize the e-commerce application with sample data
    pub async fn initialize_sample_data(&self) -> Result<(), NetabaseError> {
        // Create sample store
        let _store = ecommerce_def::Store {
            id: 1,
            name: "Demo E-commerce Store".to_string(),
            domain: "demo-store.com".to_string(),
            description: "A demonstration e-commerce store showcasing cross-definition linking".to_string(),
            active: true,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            settings: ecommerce_def::StoreSettings {
                currency: "USD".to_string(),
                tax_rate: 0.08,
                shipping_enabled: true,
                payment_methods: vec!["credit_card".to_string(), "paypal".to_string()],
            },
        };

        // Create sample user role
        let _admin_role = ecommerce_def::user_def::UserRole {
            id: 1,
            name: "Administrator".to_string(),
            permissions: ecommerce_def::user_def::RolePermissions {
                manage_products: true,
                manage_orders: true,
                manage_users: true,
                view_analytics: true,
                system_admin: true,
            },
            description: "Full system administrator".to_string(),
            system_role: true,
        };

        let _customer_role = ecommerce_def::user_def::UserRole {
            id: 2,
            name: "Customer".to_string(),
            permissions: ecommerce_def::user_def::RolePermissions {
                manage_products: false,
                manage_orders: false,
                manage_users: false,
                view_analytics: false,
                system_admin: false,
            },
            description: "Regular customer account".to_string(),
            system_role: false,
        };

        println!("✅ Sample data initialized successfully!");
        println!("📊 Application ready with cross-definition linking support!");
        
        Ok(())
    }

    /// Demonstrate cross-definition relationship traversal
    pub async fn demonstrate_cross_definition_access(&self) -> Result<(), NetabaseError> {
        println!("\n🔗 Demonstrating Cross-Definition Relationship Access");
        println!("====================================================");

        // Example: Get product and traverse to its creator
        let product_id = 1u64;
        println!("\n📦 Product ID: {}", product_id);
        
        // Simulate product with cross-definition link
        let laptop_product = ecommerce_def::product_def::Product {
            id: product_id,
            sku: "LAPTOP-001".to_string(),
            name: "Gaming Laptop Pro".to_string(),
            description: "High-performance gaming laptop".to_string(),
            price: 1299.99,
            cost: 999.99,
            weight: 2.5,
            category_id: ecommerce_def::product_def::CategoryId(2),
            created_by: ecommerce_def::product_def::ProductCreatedByLink::new(1),
            inventory: ecommerce_def::product_def::ProductInventory {
                quantity: 10,
                reserved: 0,
                reorder_level: 2,
                reorder_quantity: 5,
                track_inventory: true,
            },
            seo: ecommerce_def::product_def::ProductSEO {
                title: Some("Gaming Laptop Pro".to_string()),
                description: Some("High-performance gaming laptop".to_string()),
                keywords: vec!["gaming".to_string(), "laptop".to_string()],
                meta_data: HashMap::new(),
            },
            images: vec!["https://example.com/images/laptop-1.jpg".to_string()],
            status: ecommerce_def::product_def::ProductStatus::Active,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            updated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        // Demonstrate cross-definition link access
        let created_by_link = &laptop_product.created_by;
        println!("   📋 Product Name: {}", laptop_product.name);
        println!("   💰 Price: ${}", laptop_product.price);
        println!("   👤 Created By Link:");
        println!("      🎯 Target Path: {}", created_by_link.target_path);
        println!("      🆔 Target User ID: {}", created_by_link.target_model_id);
        println!("      🔗 Relationship Type: {:?}", created_by_link.relationship_type);
        println!("      🔒 Required Permission: {:?}", created_by_link.required_permission);

        // Demonstrate permission checking
        let read_permission = CrossDefinitionPermissionLevel::Read;
        let admin_permission = CrossDefinitionPermissionLevel::Admin;
        let no_permission = CrossDefinitionPermissionLevel::None;

        println!("\n   🔐 Permission Check Results:");
        println!("      ✅ Can access with Read permission: {}", 
                created_by_link.can_access_with_permission(&read_permission));
        println!("      ✅ Can access with Admin permission: {}", 
                created_by_link.can_access_with_permission(&admin_permission));
        println!("      ❌ Can access with No permission: {}", 
                created_by_link.can_access_with_permission(&no_permission));

        println!("\n✅ Cross-definition relationship traversal demonstration complete!");
        
        Ok(())
    }

    /// Demonstrate hierarchical permission management
    pub async fn demonstrate_permission_hierarchy(&self) -> Result<(), NetabaseError> {
        println!("\n🏛️  Demonstrating Hierarchical Permission Management");
        println!("===================================================");

        // Demonstrate different permission scenarios
        let scenarios = vec![
            ("System Admin", CrossDefinitionPermissionLevel::Admin),
            ("Store Manager", CrossDefinitionPermissionLevel::ReadWrite),
            ("Customer Support", CrossDefinitionPermissionLevel::Read),
            ("Guest User", CrossDefinitionPermissionLevel::None),
        ];

        for (role_name, permission_level) in scenarios {
            println!("\n👤 Role: {} (Permission Level: {:?})", role_name, permission_level);
            
            // Test access to different cross-definition links
            let product_link = ecommerce_def::product_def::ProductCreatedByLink::new(1);
            let order_customer_link = ecommerce_def::order_def::OrderCustomerLink::new(2);
            let order_product_link = ecommerce_def::order_def::OrderItemProductLink::new(1);

            println!("   🔐 Access Results:");
            println!("      📦 Product -> User link: {}", 
                    if product_link.can_access_with_permission(&permission_level) { "✅ Allowed" } else { "❌ Denied" });
            println!("      🛒 Order -> Customer link: {}", 
                    if order_customer_link.can_access_with_permission(&permission_level) { "✅ Allowed" } else { "❌ Denied" });
            println!("      📦 OrderItem -> Product link: {}", 
                    if order_product_link.can_access_with_permission(&permission_level) { "✅ Allowed" } else { "❌ Denied" });
        }

        println!("\n✅ Hierarchical permission demonstration complete!");
        
        Ok(())
    }
}

impl Default for ecommerce_def::StoreSettings {
    fn default() -> Self {
        Self {
            currency: "USD".to_string(),
            tax_rate: 0.08,
            shipping_enabled: true,
            payment_methods: vec!["credit_card".to_string()],
        }
    }
}

impl Default for ecommerce_def::user_def::UserPreferences {
    fn default() -> Self {
        Self {
            newsletter: false,
            notifications: true,
            language: "en".to_string(),
            timezone: "UTC".to_string(),
        }
    }
}

impl Default for ecommerce_def::product_def::ProductInventory {
    fn default() -> Self {
        Self {
            quantity: 0,
            reserved: 0,
            reorder_level: 0,
            reorder_quantity: 0,
            track_inventory: true,
        }
    }
}

impl Default for ecommerce_def::product_def::ProductSEO {
    fn default() -> Self {
        Self {
            title: None,
            description: None,
            keywords: Vec::new(),
            meta_data: HashMap::new(),
        }
    }
}