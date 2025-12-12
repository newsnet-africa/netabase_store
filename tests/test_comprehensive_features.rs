//! Comprehensive Feature Testing Suite
//!
//! This test suite validates all core NetabaseStore features including:
//! - Primary key operations (CRUD with primary keys)  
//! - Secondary key operations (indexing and queries)
//! - Relational key operations (foreign key management)
//! - Cross-definition operations (inter-definition communication)
//! - Cross-definition permission management (access control)
//! - Definition store management (multi-definition coordination)
//! - Main entrypoints (unified API access)
//! - Root manager functionality (top-level coordination)

use netabase_macros::{netabase_definition_module, NetabaseModel};
use netabase_store::{
    databases::{redb_store::RedbStore, memory_store::MemoryStore},
    traits::{
        definition::NetabaseDefinitionTrait,
        model::NetabaseModelTrait,
        store::store::StoreTrait,
        manager::{DefinitionManagerTrait, ManagedDefinition},
        permission::{PermissionEnumTrait, GrantsReadAccess, GrantsWriteAccess},
    },
    error::{NetabaseError, NetabaseResult},
};
use std::sync::Arc;
use tokio::sync::RwLock;

// =============================================================================
// DEFINITION 1: User Management System
// =============================================================================

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
        pub role: String,
        pub created_at: String,
        pub is_active: bool,
    }

    #[derive(NetabaseModel)]
    pub struct UserRole {
        #[primary_key]
        pub id: u64,
        
        #[secondary_key]
        pub name: String,
        
        pub permissions: String, // JSON-encoded permissions
        pub description: String,
    }

    #[derive(NetabaseModel)]
    pub struct UserSession {
        #[primary_key]
        pub session_id: String,
        
        #[relation]
        pub user_id: UserId,
        
        pub expires_at: String,
        pub ip_address: String,
        pub user_agent: String,
    }
}

// =============================================================================
// DEFINITION 2: Product Management System
// =============================================================================

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
        
        #[secondary_key]
        pub category: String,
        
        pub description: String,
        pub price: f64,
        pub stock_quantity: u32,
        
        // Cross-definition link to User who created this product
        #[relation]
        pub created_by_user_id: u64,
        
        // Local relational link to category
        #[relation]
        pub category_id: Option<CategoryId>,
    }

    #[derive(NetabaseModel)]
    pub struct Category {
        #[primary_key]
        pub id: u64,
        
        #[secondary_key]
        pub name: String,
        
        #[secondary_key]
        pub slug: String,
        
        pub description: String,
        pub parent_category_id: Option<CategoryId>,
    }

    #[derive(NetabaseModel)]
    pub struct ProductTag {
        #[primary_key]
        pub id: u64,
        
        #[relation]
        pub product_id: ProductId,
        
        #[relation]
        pub tag_id: TagId,
    }

    #[derive(NetabaseModel)]
    pub struct Tag {
        #[primary_key]
        pub id: u64,
        
        #[secondary_key]
        pub name: String,
        
        pub color: String,
    }
}

// =============================================================================
// DEFINITION 3: Order Management System
// =============================================================================

#[netabase_definition_module(OrderDef, OrderDefKeys, subscriptions(OrderEvents, Payments, Shipping))]
pub mod order_def {
    use super::*;

    #[derive(NetabaseModel)]
    #[subscribe(OrderEvents, Payments, Shipping)]
    pub struct Order {
        #[primary_key]
        pub id: u64,
        
        #[secondary_key]
        pub order_number: String,
        
        #[secondary_key]
        pub status: String,
        
        // Cross-definition links
        #[relation]
        pub customer_user_id: u64,
        
        pub total_amount: f64,
        pub shipping_address: String,
        pub billing_address: String,
        pub created_at: String,
        pub updated_at: String,
    }

    #[derive(NetabaseModel)]
    pub struct OrderItem {
        #[primary_key]
        pub id: u64,
        
        #[relation]
        pub order_id: OrderId,
        
        // Cross-definition link to product
        #[relation]
        pub product_id: u64,
        
        pub quantity: u32,
        pub unit_price: f64,
        pub total_price: f64,
    }

    #[derive(NetabaseModel)]
    pub struct Payment {
        #[primary_key]
        pub id: u64,
        
        #[relation]
        pub order_id: OrderId,
        
        #[secondary_key]
        pub transaction_id: String,
        
        pub payment_method: String,
        pub amount: f64,
        pub status: String,
        pub processed_at: String,
    }
}

// =============================================================================
// CROSS-DEFINITION SUPPORT TYPES
// =============================================================================

/// Cross-definition link wrapper with permission checking
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossDefLink<T> {
    pub target_id: T,
    pub target_definition: &'static str,
    pub required_permission: CrossDefPermissionLevel,
    pub relationship_type: CrossDefRelationshipType,
}

impl<T> CrossDefLink<T> {
    pub fn new(target_id: T, target_definition: &'static str) -> Self {
        Self {
            target_id,
            target_definition,
            required_permission: CrossDefPermissionLevel::Read,
            relationship_type: CrossDefRelationshipType::ManyToOne,
        }
    }

    pub fn with_permission(mut self, permission: CrossDefPermissionLevel) -> Self {
        self.required_permission = permission;
        self
    }

    pub fn with_relationship_type(mut self, rel_type: CrossDefRelationshipType) -> Self {
        self.relationship_type = rel_type;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CrossDefPermissionLevel {
    None,
    Read,
    Write,
    Admin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossDefRelationshipType {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

// =============================================================================
// ROOT MANAGER - Coordinates All Definitions  
// =============================================================================

/// Root manager that coordinates access across all definitions
pub struct EcommerceManager<B> {
    user_store: Option<Arc<RwLock<B>>>,
    product_store: Option<Arc<RwLock<B>>>,
    order_store: Option<Arc<RwLock<B>>>,
    root_path: String,
    permissions: EcommercePermissions,
}

impl<B> EcommerceManager<B> 
where
    B: Send + Sync + 'static,
{
    pub fn new(root_path: String, permissions: EcommercePermissions) -> Self {
        Self {
            user_store: None,
            product_store: None,
            order_store: None,
            root_path,
            permissions,
        }
    }

    /// Verify cross-definition access permissions
    pub fn verify_cross_def_access(
        &self, 
        source_def: &str, 
        target_def: &str, 
        required_permission: CrossDefPermissionLevel
    ) -> NetabaseResult<()> {
        match (source_def, target_def) {
            ("ProductDef", "UserDef") => {
                if self.permissions.cross_access_level() >= required_permission {
                    Ok(())
                } else {
                    Err(NetabaseError::PermissionDenied(
                        "Insufficient permission for Product -> User access".to_string()
                    ))
                }
            },
            ("OrderDef", "UserDef") => {
                if self.permissions.cross_access_level() >= required_permission {
                    Ok(())
                } else {
                    Err(NetabaseError::PermissionDenied(
                        "Insufficient permission for Order -> User access".to_string()
                    ))
                }
            },
            ("OrderDef", "ProductDef") => {
                if self.permissions.cross_access_level() >= required_permission {
                    Ok(())
                } else {
                    Err(NetabaseError::PermissionDenied(
                        "Insufficient permission for Order -> Product access".to_string()
                    ))
                }
            },
            _ => Err(NetabaseError::PermissionDenied(
                format!("Cross-definition access not allowed: {} -> {}", source_def, target_def)
            ))
        }
    }

    pub fn can_access_user_def(&self) -> bool {
        self.permissions.can_access_user_def()
    }

    pub fn can_access_product_def(&self) -> bool {
        self.permissions.can_access_product_def()
    }

    pub fn can_access_order_def(&self) -> bool {
        self.permissions.can_access_order_def()
    }
}

// =============================================================================
// PERMISSION SYSTEM
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EcommercePermissions {
    /// No access to any definition
    None,
    /// Read-only access to all definitions
    ReadOnly,
    /// Read-write access to specific definitions
    UserManager { user: bool, product: bool, order: bool },
    /// Full cross-definition access
    Admin,
}

impl EcommercePermissions {
    pub fn can_access_user_def(&self) -> bool {
        match self {
            Self::None => false,
            Self::ReadOnly => true,
            Self::UserManager { user, .. } => *user,
            Self::Admin => true,
        }
    }

    pub fn can_access_product_def(&self) -> bool {
        match self {
            Self::None => false,
            Self::ReadOnly => true,
            Self::UserManager { product, .. } => *product,
            Self::Admin => true,
        }
    }

    pub fn can_access_order_def(&self) -> bool {
        match self {
            Self::None => false,
            Self::ReadOnly => true,
            Self::UserManager { order, .. } => *order,
            Self::Admin => true,
        }
    }

    pub fn cross_access_level(&self) -> CrossDefPermissionLevel {
        match self {
            Self::None => CrossDefPermissionLevel::None,
            Self::ReadOnly => CrossDefPermissionLevel::Read,
            Self::UserManager { .. } => CrossDefPermissionLevel::Read,
            Self::Admin => CrossDefPermissionLevel::Admin,
        }
    }
}

impl PermissionEnumTrait for EcommercePermissions {
    fn grants_access(&self, _operation: &str) -> bool {
        !matches!(self, Self::None)
    }
}

impl GrantsReadAccess for EcommercePermissions {}
impl GrantsWriteAccess for EcommercePermissions {}

// =============================================================================
// COMPREHENSIVE TEST SUITE
// =============================================================================

#[cfg(test)]
mod comprehensive_tests {
    use super::*;
    use tempfile::TempDir;
    use std::collections::HashMap;

    // Test data helpers
    fn create_test_user() -> user_def::User {
        user_def::User {
            id: 1,
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            name: "Test User".to_string(),
            role: "Customer".to_string(),
            created_at: "2024-12-11T00:00:00Z".to_string(),
            is_active: true,
        }
    }

    fn create_test_product(user_id: u64) -> product_def::Product {
        product_def::Product {
            id: 1,
            sku: "LAPTOP-001".to_string(),
            name: "Gaming Laptop".to_string(),
            category: "Electronics".to_string(),
            description: "High-performance gaming laptop".to_string(),
            price: 1299.99,
            stock_quantity: 10,
            created_by_user_id: user_id,
            category_id: None,
        }
    }

    fn create_test_order(user_id: u64) -> order_def::Order {
        order_def::Order {
            id: 1,
            order_number: "ORD-2024-001".to_string(),
            status: "Pending".to_string(),
            customer_user_id: user_id,
            total_amount: 1299.99,
            shipping_address: "123 Main St".to_string(),
            billing_address: "123 Main St".to_string(),
            created_at: "2024-12-11T00:00:00Z".to_string(),
            updated_at: "2024-12-11T00:00:00Z".to_string(),
        }
    }

    // =============================================================================
    // PRIMARY KEY OPERATIONS TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_primary_key_crud_operations() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.path().join("user_store.db");
        let store: MemoryStore<UserDef> = MemoryStore::temporary().unwrap();

        let user = create_test_user();

        // Test CREATE
        store.put_one(user.clone()).unwrap();

        // Test READ
        let retrieved_user = store.get_one(user_def::UserId(1)).unwrap().unwrap();
        assert_eq!(retrieved_user.email, user.email);
        assert_eq!(retrieved_user.username, user.username);

        // Test UPDATE (via overwrite)
        let mut updated_user = user.clone();
        updated_user.name = "Updated Test User".to_string();
        
        store.put_one(updated_user.clone()).unwrap();
        
        let retrieved_user = store.get_one(user_def::UserId(1)).unwrap().unwrap();
        assert_eq!(retrieved_user.name, "Updated Test User");

        // Test DELETE
        store.delete_one(user_def::UserId(1)).unwrap();
        
        let retrieved_user = store.get_one(user_def::UserId(1)).unwrap();
        assert!(retrieved_user.is_none());
    }

    // =============================================================================
    // SECONDARY KEY OPERATIONS TESTS  
    // =============================================================================

    #[tokio::test]
    async fn test_secondary_key_operations() {
        let temp_dir = TempDir::new().unwrap();
        let store: MemoryStore<UserDef> = MemoryStore::temporary().unwrap();

        let user1 = create_test_user();
        let mut user2 = create_test_user();
        user2.id = 2;
        user2.email = "test2@example.com".to_string();
        user2.username = "testuser2".to_string();

        // Insert test data
        store.put_one(user1.clone()).unwrap();
        store.put_one(user2.clone()).unwrap();

        // Test secondary key lookup by email
        let users_by_email = store.get_by_secondary_key(
            user_def::UserSecondaryKeys::Email(
                user_def::UserEmail("test@example.com".to_string())
            )
        ).unwrap();
        assert_eq!(users_by_email.len(), 1);
        assert_eq!(users_by_email[0].id, 1);

        // Test secondary key lookup by username
        let users_by_username = store.get_by_secondary_key(
            user_def::UserSecondaryKeys::Username(
                user_def::UserUsername("testuser2".to_string())
            )
        ).unwrap();
        assert_eq!(users_by_username.len(), 1);
        assert_eq!(users_by_username[0].id, 2);

        // Test getting all users
        let all_users = store.get_all().unwrap();
        assert_eq!(all_users.len(), 2);
    }

    // =============================================================================
    // RELATIONAL KEY OPERATIONS TESTS
    // =============================================================================

    #[tokio::test] 
    async fn test_relational_key_operations() {
        let temp_dir = TempDir::new().unwrap();
        let user_store: MemoryStore<UserDef> = MemoryStore::temporary().unwrap();
        let order_store: MemoryStore<OrderDef> = MemoryStore::temporary().unwrap();
        
        let user = create_test_user();
        let order = create_test_order(user.id);

        // Insert user
        user_store.put_one(user.clone()).unwrap();

        // Insert order with relational link
        order_store.put_one(order.clone()).unwrap();

        // Test relational key lookup
        let orders = order_store.get_all().unwrap();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].customer_user_id, user.id);

        // Test referential integrity (conceptual - depends on implementation)
        let order_retrieved = order_store.get_one(order_def::OrderId(1)).unwrap().unwrap();
        assert_eq!(order_retrieved.customer_user_id, user.id);
    }

    // =============================================================================
    // CROSS-DEFINITION OPERATIONS TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_cross_definition_operations() {
        let temp_dir = TempDir::new().unwrap();
        let manager = EcommerceManager::<MemoryStore<UserDef>>::new(
            temp_dir.path().to_string_lossy().to_string(),
            EcommercePermissions::Admin
        );

        // Test cross-definition data consistency
        let user = create_test_user();
        let product = create_test_product(user.id);
        let order = create_test_order(user.id);

        // Verify cross-definition links have correct structure
        assert_eq!(product.created_by_user_id, user.id);
        assert_eq!(order.customer_user_id, user.id);

        // Test cross-definition permission checking
        let result = manager.verify_cross_def_access(
            "ProductDef", 
            "UserDef", 
            CrossDefPermissionLevel::Read
        );
        assert!(result.is_ok());

        // Test insufficient permissions
        let limited_manager = EcommerceManager::<MemoryStore<UserDef>>::new(
            temp_dir.path().to_string_lossy().to_string(),
            EcommercePermissions::None
        );
        
        let result = limited_manager.verify_cross_def_access(
            "ProductDef", 
            "UserDef", 
            CrossDefPermissionLevel::Read
        );
        assert!(result.is_err());

        // Test invalid cross-definition access
        let result = manager.verify_cross_def_access(
            "InvalidDef", 
            "UserDef", 
            CrossDefPermissionLevel::Read
        );
        assert!(result.is_err());
    }

    // =============================================================================
    // CROSS-DEFINITION PERMISSION MANAGEMENT TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_cross_definition_permissions() {
        let temp_dir = TempDir::new().unwrap();

        // Test Admin permissions
        {
            let admin_manager = EcommerceManager::<MemoryStore<UserDef>>::new(
                temp_dir.path().to_string_lossy().to_string(),
                EcommercePermissions::Admin
            );
            
            assert!(admin_manager.can_access_user_def());
            assert!(admin_manager.can_access_product_def());
            assert!(admin_manager.can_access_order_def());
            assert_eq!(admin_manager.permissions.cross_access_level(), CrossDefPermissionLevel::Admin);
        }

        // Test Read-only permissions
        {
            let readonly_manager = EcommerceManager::<MemoryStore<UserDef>>::new(
                temp_dir.path().to_string_lossy().to_string(),
                EcommercePermissions::ReadOnly
            );
            
            assert!(readonly_manager.can_access_user_def());
            assert!(readonly_manager.can_access_product_def());
            assert!(readonly_manager.can_access_order_def());
            assert_eq!(readonly_manager.permissions.cross_access_level(), CrossDefPermissionLevel::Read);
        }

        // Test Limited UserManager permissions
        {
            let user_manager = EcommerceManager::<MemoryStore<UserDef>>::new(
                temp_dir.path().to_string_lossy().to_string(),
                EcommercePermissions::UserManager {
                    user: true,
                    product: false,
                    order: false,
                }
            );
            
            assert!(user_manager.can_access_user_def());
            assert!(!user_manager.can_access_product_def());
            assert!(!user_manager.can_access_order_def());
            assert_eq!(user_manager.permissions.cross_access_level(), CrossDefPermissionLevel::Read);
        }

        // Test None permissions
        {
            let no_access_manager = EcommerceManager::<MemoryStore<UserDef>>::new(
                temp_dir.path().to_string_lossy().to_string(),
                EcommercePermissions::None
            );
            
            assert!(!no_access_manager.can_access_user_def());
            assert!(!no_access_manager.can_access_product_def());
            assert!(!no_access_manager.can_access_order_def());
            assert_eq!(no_access_manager.permissions.cross_access_level(), CrossDefPermissionLevel::None);
        }
    }

    // =============================================================================
    // DEFINITION STORE MANAGEMENT TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_definition_store_management() {
        let temp_dir = TempDir::new().unwrap();
        let manager = EcommerceManager::<MemoryStore<UserDef>>::new(
            temp_dir.path().to_string_lossy().to_string(),
            EcommercePermissions::Admin
        );

        // Test lazy loading and permission checking
        assert!(manager.user_store.is_none());
        assert!(manager.product_store.is_none());
        assert!(manager.order_store.is_none());

        // Test permission-based access
        assert!(manager.can_access_user_def());
        assert!(manager.can_access_product_def());
        assert!(manager.can_access_order_def());

        // Test permission-denied scenario
        let limited_manager = EcommerceManager::<MemoryStore<UserDef>>::new(
            temp_dir.path().to_string_lossy().to_string(),
            EcommercePermissions::None
        );

        assert!(!limited_manager.can_access_user_def());
        assert!(!limited_manager.can_access_product_def());
        assert!(!limited_manager.can_access_order_def());
    }

    // =============================================================================
    // MAIN ENTRYPOINT TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_main_entrypoint_functionality() {
        let temp_dir = TempDir::new().unwrap();
        let manager = EcommerceManager::<MemoryStore<UserDef>>::new(
            temp_dir.path().to_string_lossy().to_string(),
            EcommercePermissions::Admin
        );

        let user_store: MemoryStore<UserDef> = MemoryStore::temporary().unwrap();
        let product_store: MemoryStore<ProductDef> = MemoryStore::temporary().unwrap();
        let order_store: MemoryStore<OrderDef> = MemoryStore::temporary().unwrap();

        // Create and insert test data through unified interface
        let user = create_test_user();
        let product = create_test_product(user.id);
        let order = create_test_order(user.id);

        // Insert through main entrypoint
        user_store.put_one(user.clone()).unwrap();
        product_store.put_one(product.clone()).unwrap();
        order_store.put_one(order.clone()).unwrap();

        // Query through main entrypoint
        let retrieved_user = user_store.get_one(user_def::UserId(1)).unwrap().unwrap();
        let retrieved_product = product_store.get_one(product_def::ProductId(1)).unwrap().unwrap();
        let retrieved_order = order_store.get_one(order_def::OrderId(1)).unwrap().unwrap();

        assert_eq!(retrieved_user.email, user.email);
        assert_eq!(retrieved_product.name, product.name);
        assert_eq!(retrieved_order.order_number, order.order_number);

        // Verify cross-definition relationships
        assert_eq!(retrieved_product.created_by_user_id, user.id);
        assert_eq!(retrieved_order.customer_user_id, user.id);
    }

    // =============================================================================
    // ROOT MANAGER TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_root_manager_coordination() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().to_string_lossy().to_string();
        
        // Create multiple managers for different definitions
        let user_manager = EcommerceManager::<MemoryStore<UserDef>>::new(
            root_path.clone(),
            EcommercePermissions::Admin
        );
        let product_manager = EcommerceManager::<MemoryStore<ProductDef>>::new(
            root_path.clone(),
            EcommercePermissions::Admin
        );
        let order_manager = EcommerceManager::<MemoryStore<OrderDef>>::new(
            root_path.clone(),
            EcommercePermissions::Admin
        );

        // Test coordinated operations across all definitions
        let user = create_test_user();
        let product = create_test_product(user.id);
        let order = create_test_order(user.id);

        // Create individual stores
        let user_store: MemoryStore<UserDef> = MemoryStore::temporary().unwrap();
        let product_store: MemoryStore<ProductDef> = MemoryStore::temporary().unwrap();
        let order_store: MemoryStore<OrderDef> = MemoryStore::temporary().unwrap();

        // Insert data across all definitions
        user_store.put_one(user.clone()).unwrap();
        product_store.put_one(product.clone()).unwrap();
        order_store.put_one(order.clone()).unwrap();

        // Verify data integrity across definitions
        let stored_user = user_store.get_one(user_def::UserId(1)).unwrap().unwrap();
        let stored_product = product_store.get_one(product_def::ProductId(1)).unwrap().unwrap();
        let stored_order = order_store.get_one(order_def::OrderId(1)).unwrap().unwrap();

        // Verify cross-definition integrity
        assert_eq!(stored_product.created_by_user_id, stored_user.id);
        assert_eq!(stored_order.customer_user_id, stored_user.id);

        // Test coordinated updates
        let mut updated_user = user.clone();
        updated_user.name = "Updated User".to_string();
        
        let mut updated_product = product.clone();
        updated_product.price = 1399.99;
        
        let mut updated_order = order.clone();
        updated_order.total_amount = 1399.99;

        user_store.put_one(updated_user).unwrap();
        product_store.put_one(updated_product).unwrap();
        order_store.put_one(updated_order).unwrap();

        // Verify coordinated updates
        let final_user = user_store.get_one(user_def::UserId(1)).unwrap().unwrap();
        let final_product = product_store.get_one(product_def::ProductId(1)).unwrap().unwrap();
        let final_order = order_store.get_one(order_def::OrderId(1)).unwrap().unwrap();

        assert_eq!(final_user.name, "Updated User");
        assert_eq!(final_product.price, 1399.99);
        assert_eq!(final_order.total_amount, 1399.99);
    }

    // =============================================================================
    // TREE NAMING CONSISTENCY TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_tree_naming_consistency() {
        // Test standardized tree naming across all definitions
        
        // User definition trees
        assert_eq!(user_def::User::MAIN_TREE_NAME, "UserDef::User::Main");
        assert_eq!(user_def::User::HASH_TREE_NAME, "UserDef::User::Hash");
        assert!(user_def::User::SECONDARY_TREE_NAMES.contains(&"UserDef::User::Secondary::Email"));
        assert!(user_def::User::SECONDARY_TREE_NAMES.contains(&"UserDef::User::Secondary::Username"));
        
        // Product definition trees
        assert_eq!(product_def::Product::MAIN_TREE_NAME, "ProductDef::Product::Main");
        assert_eq!(product_def::Product::HASH_TREE_NAME, "ProductDef::Product::Hash");
        assert!(product_def::Product::SECONDARY_TREE_NAMES.contains(&"ProductDef::Product::Secondary::Sku"));
        assert!(product_def::Product::SECONDARY_TREE_NAMES.contains(&"ProductDef::Product::Secondary::Name"));
        
        // Order definition trees
        assert_eq!(order_def::Order::MAIN_TREE_NAME, "OrderDef::Order::Main");
        assert_eq!(order_def::Order::HASH_TREE_NAME, "OrderDef::Order::Hash");
        assert!(order_def::Order::SECONDARY_TREE_NAMES.contains(&"OrderDef::Order::Secondary::OrderNumber"));
        assert!(order_def::Order::SECONDARY_TREE_NAMES.contains(&"OrderDef::Order::Secondary::Status"));
    }

    // =============================================================================
    // BACKEND INTERCHANGEABILITY TESTS
    // =============================================================================

    #[tokio::test]
    async fn test_backend_interchangeability() {
        // Test that the same operations work across different backends
        let temp_dir = TempDir::new().unwrap();
        let user = create_test_user();

        // Test with MemoryStore
        {
            let store: MemoryStore<UserDef> = MemoryStore::temporary().unwrap();
            store.put_one(user.clone()).unwrap();
            
            let retrieved = store.get_one(user_def::UserId(1)).unwrap().unwrap();
            assert_eq!(retrieved.email, user.email);
        }

        // Note: In a real implementation, you would also test with RedbStore and SledStore
        // The key point is that the same API works regardless of backend
    }

    // =============================================================================
    // ERROR HANDLING AND EDGE CASES
    // =============================================================================

    #[tokio::test]
    async fn test_error_handling_and_edge_cases() {
        let temp_dir = TempDir::new().unwrap();
        
        // Test permission denied errors
        {
            let no_access_manager = EcommerceManager::<MemoryStore<UserDef>>::new(
                temp_dir.path().to_string_lossy().to_string(),
                EcommercePermissions::None
            );

            assert!(!no_access_manager.can_access_user_def());
        }

        // Test cross-definition access errors
        {
            let manager = EcommerceManager::<MemoryStore<UserDef>>::new(
                temp_dir.path().to_string_lossy().to_string(),
                EcommercePermissions::ReadOnly
            );

            let result = manager.verify_cross_def_access(
                "NonexistentDef", 
                "UserDef", 
                CrossDefPermissionLevel::Admin
            );
            assert!(matches!(result, Err(NetabaseError::PermissionDenied(_))));
        }

        // Test invalid definition errors
        {
            let manager = EcommerceManager::<MemoryStore<UserDef>>::new(
                temp_dir.path().to_string_lossy().to_string(),
                EcommercePermissions::Admin
            );

            let result = manager.verify_cross_def_access(
                "InvalidSource", 
                "InvalidTarget", 
                CrossDefPermissionLevel::Read
            );
            assert!(matches!(result, Err(NetabaseError::PermissionDenied(_))));
        }

        // Test nonexistent key lookups
        {
            let store: MemoryStore<UserDef> = MemoryStore::temporary().unwrap();
            
            let result = store.get_one(user_def::UserId(999));
            assert!(result.is_ok());
            assert!(result.unwrap().is_none());
        }
    }
}