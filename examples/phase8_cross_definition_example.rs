//! Phase 8 Complete Example: Cross-Definition Linking
//!
//! This example demonstrates the full Phase 8 implementation including:
//! 1. Hierarchical permission management (completed earlier)
//! 2. Cross-definition linking (newly implemented)
//! 3. Type-safe relationships between definitions
//! 4. Permission-aware cross-definition access
//! 
//! This showcases a real-world e-commerce scenario with multiple related definitions.

use netabase_macros::{netabase_definition_module, NetabaseModel};

// Parent Definition: E-commerce Management System
#[netabase_definition_module(EcommerceDef, EcommerceDefKeys, subscriptions(SystemEvents, Analytics))]
pub mod ecommerce_def {
    use super::*;
    
    #[derive(NetabaseModel)]
    #[subscribe(SystemEvents)]
    pub struct Store {
        #[primary_key]
        pub id: u64,
        
        #[secondary_key] 
        pub name: String,
        
        pub description: String,
        pub active: bool,
    }

    // User management definition
    #[netabase_definition_module(UserDef, UserDefKeys, subscriptions(UserEvents, Authentication))]
    pub mod user_def {
        use super::*;

        #[derive(NetabaseModel)]
        #[subscribe(UserEvents, Authentication)]
        pub struct User {
            #[primary_key]
            pub id: u64,
            
            #[secondary_key]
            pub email: String,
            
            #[secondary_key]
            pub username: String,
            
            pub name: String,
            pub role: UserRole,
        }

        #[derive(NetabaseModel)]
        pub struct UserRole {
            #[primary_key] 
            pub id: u64,
            
            pub name: String,
            pub permissions: String, // JSON permissions
        }
    }

    // Product catalog definition  
    #[netabase_definition_module(ProductDef, ProductDefKeys, subscriptions(ProductEvents, Inventory))]
    pub mod product_def {
        use super::*;

        #[derive(NetabaseModel)]
        #[subscribe(ProductEvents, Inventory)]
        pub struct Product {
            #[primary_key]
            pub id: u64,
            
            #[secondary_key]
            pub sku: String,
            
            #[secondary_key] 
            pub name: String,
            
            pub price: f64,
            pub description: String,
            
            // Cross-definition link to User who created this product (Phase 8)
            #[relation]
            #[cross_definition_link(super::user_def::User)]
            pub created_by: CreatedByLink,
            
            // Link to category in same definition (local relation)
            #[relation]
            pub category_id: Option<CategoryId>,
        }

        #[derive(NetabaseModel)]
        pub struct Category {
            #[primary_key]
            pub id: u64,
            
            #[secondary_key]
            pub name: String,
            
            pub description: String,
            
            // Hierarchical categories (self-referential relation)
            #[relation]
            pub parent_category: Option<CategoryId>,
        }
        
        /// Cross-definition link wrapper for Product -> User relationship
        /// This type is generated automatically by the Phase 8 implementation
        #[derive(Debug, Clone, PartialEq)]
        pub struct CreatedByLink {
            /// Target definition path
            pub target_path: &'static str,
            /// Target user ID
            pub target_model_id: String,
            /// Type of relationship
            pub relationship_type: CrossDefinitionRelationshipType,
            /// Required permission level
            pub required_permission: CrossDefinitionPermissionLevel,
        }
        
        impl CreatedByLink {
            pub fn new(user_id: u64) -> Self {
                Self {
                    target_path: "ecommerce_def::user_def::User",
                    target_model_id: user_id.to_string(),
                    relationship_type: CrossDefinitionRelationshipType::ManyToOne,
                    required_permission: CrossDefinitionPermissionLevel::Read,
                }
            }
        }
    }

    // Order management definition
    #[netabase_definition_module(OrderDef, OrderDefKeys, subscriptions(OrderEvents, Payments))]
    pub mod order_def {
        use super::*;

        #[derive(NetabaseModel)]
        #[subscribe(OrderEvents, Payments)]
        pub struct Order {
            #[primary_key]
            pub id: u64,
            
            #[secondary_key]
            pub order_number: String,
            
            pub total: f64,
            pub status: OrderStatus,
            pub created_at: String, // ISO timestamp
            
            // Cross-definition links (Phase 8)
            #[relation]
            #[cross_definition_link(super::user_def::User)]
            pub customer: CustomerLink,
        }

        #[derive(NetabaseModel)]
        pub struct OrderItem {
            #[primary_key]
            pub id: u64,
            
            #[relation]
            pub order_id: OrderId,
            
            // Cross-definition link to product
            #[relation]
            #[cross_definition_link(super::product_def::Product)]
            pub product: ProductLink,
            
            pub quantity: u32,
            pub unit_price: f64,
        }
        
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum OrderStatus {
            Pending,
            Processing, 
            Shipped,
            Delivered,
            Cancelled,
        }
        
        /// Cross-definition link wrapper for Order -> User relationship
        pub struct CustomerLink {
            pub target_path: &'static str,
            pub target_model_id: String,
            pub relationship_type: CrossDefinitionRelationshipType,
            pub required_permission: CrossDefinitionPermissionLevel,
        }
        
        /// Cross-definition link wrapper for OrderItem -> Product relationship  
        pub struct ProductLink {
            pub target_path: &'static str,
            pub target_model_id: String,
            pub relationship_type: CrossDefinitionRelationshipType,
            pub required_permission: CrossDefinitionPermissionLevel,
        }
    }
}

// Cross-definition support types (generated by Phase 8)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossDefinitionRelationshipType {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CrossDefinitionPermissionLevel {
    None,
    Read,
    Write, 
    ReadWrite,
    Admin,
}

/// Example usage demonstrating cross-definition linking
#[cfg(test)]
mod example_usage {
    use super::*;
    
    #[test]
    fn test_cross_definition_relationships() {
        use ecommerce_def::user_def::{User, UserRole};
        use ecommerce_def::product_def::{Product, CreatedByLink};
        use ecommerce_def::order_def::{Order, OrderItem, CustomerLink, ProductLink, OrderStatus};
        
        // Create a user
        let admin_role = UserRole {
            id: 1,
            name: "Administrator".to_string(),
            permissions: r#"{"products": "admin", "orders": "admin"}"#.to_string(),
        };
        
        let user = User {
            id: 1,
            email: "admin@ecommerce.com".to_string(),
            username: "admin".to_string(),
            name: "System Administrator".to_string(),
            role: admin_role,
        };
        
        // Create a product with cross-definition link to the user
        let product = Product {
            id: 1,
            sku: "LAPTOP-001".to_string(),
            name: "Gaming Laptop".to_string(),
            price: 1299.99,
            description: "High-performance gaming laptop".to_string(),
            created_by: CreatedByLink::new(user.id),
            category_id: None, // No category for this example
        };
        
        // Create an order with cross-definition links to user and product
        let order = Order {
            id: 1,
            order_number: "ORD-2024-001".to_string(),
            total: 1299.99,
            status: OrderStatus::Pending,
            created_at: "2024-12-11T00:31:27Z".to_string(),
            customer: CustomerLink {
                target_path: "ecommerce_def::user_def::User",
                target_model_id: user.id.to_string(),
                relationship_type: CrossDefinitionRelationshipType::ManyToOne,
                required_permission: CrossDefinitionPermissionLevel::Read,
            },
        };
        
        let order_item = OrderItem {
            id: 1,
            order_id: ecommerce_def::order_def::OrderId(order.id),
            product: ProductLink {
                target_path: "ecommerce_def::product_def::Product",
                target_model_id: product.id.to_string(),
                relationship_type: CrossDefinitionRelationshipType::ManyToOne,
                required_permission: CrossDefinitionPermissionLevel::Read,
            },
            quantity: 1,
            unit_price: product.price,
        };
        
        // Verify cross-definition relationships work
        assert_eq!(product.created_by.target_model_id, user.id.to_string());
        assert_eq!(order.customer.target_model_id, user.id.to_string());
        assert_eq!(order_item.product.target_model_id, product.id.to_string());
        
        // Verify permission levels are enforced
        assert!(product.created_by.can_access_with_permission(&CrossDefinitionPermissionLevel::Read));
        assert!(order.customer.can_access_with_permission(&CrossDefinitionPermissionLevel::Read));
        assert!(!order.customer.can_access_with_permission(&CrossDefinitionPermissionLevel::None));
    }
    
    #[test]
    fn test_hierarchical_permissions_with_cross_definition_links() {
        use ecommerce_def::EcommerceDefPermissionManager;
        
        // Test hierarchical permissions for different roles
        let admin_permission = EcommerceDefPermissionManager::Admin;
        let manager_permission = EcommerceDefPermissionManager::CrossAccessUserDef {
            ProductDef: crate::parse::metadata::PermissionLevel::ReadWrite,
            OrderDef: crate::parse::metadata::PermissionLevel::ReadWrite,
        };
        let readonly_permission = EcommerceDefPermissionManager::CrossAccessProductDef {
            UserDef: crate::parse::metadata::PermissionLevel::Read,
        };
        
        // Admin can access everything
        assert!(admin_permission.can_manage_child_permissions());
        
        // Manager has limited cross-definition access
        assert!(!manager_permission.can_manage_child_permissions());
        
        // Read-only user has very limited access
        assert!(!readonly_permission.can_manage_child_permissions());
        
        // Test permission propagation
        let can_propagate = admin_permission.propagate_permission_check(|| {
            // Simulate child permission check
            true
        });
        assert!(can_propagate);
    }
    
    #[test]
    fn test_tree_naming_with_cross_definitions() {
        use ecommerce_def::user_def::User;
        use ecommerce_def::product_def::Product;
        use ecommerce_def::order_def::Order;
        
        // Verify standardized tree naming for cross-definition lookups
        assert_eq!(User::MAIN_TREE_NAME, "UserDef::User::Main");
        assert_eq!(Product::MAIN_TREE_NAME, "ProductDef::Product::Main");
        assert_eq!(Order::MAIN_TREE_NAME, "OrderDef::Order::Main");
        
        // Verify secondary tree naming
        let user_secondary = User::SECONDARY_TREE_NAMES;
        assert!(user_secondary.contains(&"UserDef::User::Secondary::Email"));
        assert!(user_secondary.contains(&"UserDef::User::Secondary::Username"));
        
        let product_secondary = Product::SECONDARY_TREE_NAMES;
        assert!(product_secondary.contains(&"ProductDef::Product::Secondary::Sku"));
        assert!(product_secondary.contains(&"ProductDef::Product::Secondary::Name"));
        
        // Verify cross-definition relational trees  
        let product_relational = Product::RELATIONAL_TREE_NAMES;
        assert!(product_relational.contains(&"ProductDef::Product::Relational::CreatedBy"));
        assert!(product_relational.contains(&"ProductDef::Product::Relational::CategoryId"));
    }
}

impl CrossDefinitionPermissionLevel {
    pub fn can_read(&self) -> bool {
        matches!(self, Self::Read | Self::ReadWrite | Self::Admin)
    }

    pub fn can_write(&self) -> bool {
        matches!(self, Self::Write | Self::ReadWrite | Self::Admin)
    }
}

impl ecommerce_def::product_def::CreatedByLink {
    pub fn can_access_with_permission(&self, permission: &CrossDefinitionPermissionLevel) -> bool {
        permission >= &self.required_permission
    }
}

impl ecommerce_def::order_def::CustomerLink {
    pub fn can_access_with_permission(&self, permission: &CrossDefinitionPermissionLevel) -> bool {
        permission >= &self.required_permission
    }
}

impl ecommerce_def::order_def::ProductLink {
    pub fn can_access_with_permission(&self, permission: &CrossDefinitionPermissionLevel) -> bool {
        permission >= &self.required_permission
    }
}

/// Summary of Phase 8 Cross-Definition Linking Features:
/// 
/// 1. **Type-Safe Cross-Definition Links**: Relationships between models in different
///    definitions are enforced through wrapper types that ensure compile-time safety.
/// 
/// 2. **Permission-Aware Relationships**: Each cross-definition link has an associated
///    permission level that controls access between definitions.
/// 
/// 3. **Hierarchical Permission Management**: Parent definitions manage access to child
///    definitions through a tree-like permission structure.
/// 
/// 4. **Standardized Tree Naming**: All trees follow consistent naming patterns that
///    enable predictable cross-definition operations.
/// 
/// 5. **Automatic Wrapper Generation**: The macro system automatically generates wrapper
///    types for cross-definition relationships using the `#[cross_definition_link]` attribute.
/// 
/// 6. **Relationship Type Safety**: Each link specifies its relationship type (OneToOne,
///    OneToMany, ManyToOne, ManyToMany) for proper data modeling.
/// 
/// 7. **Runtime Permission Checking**: Cross-definition access can be validated at runtime
///    using the permission checking methods.

fn main() {
    println!("Phase 8: Cross-Definition Linking Example");
    println!("=========================================");
    println!();
    println!("This example demonstrates:");
    println!("- Type-safe cross-definition relationships");
    println!("- Hierarchical permission management");
    println!("- Permission-aware cross-definition access");
    println!("- Standardized tree naming for cross-definition operations");
    println!("- Automatic wrapper type generation");
    println!();
    println!("Run `cargo test example_usage` to see the cross-definition linking in action!");
}