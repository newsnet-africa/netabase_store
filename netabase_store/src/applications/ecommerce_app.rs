//! E-commerce Application using Netabase - Simplified Version
//! 
//! This application demonstrates the basic functionality of the Netabase system.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Simplified User model for testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: u64,
    pub email: String,
    pub username: String,
    pub name: String,
    pub active: bool,
}

/// Simplified Product model for testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Product {
    pub id: u64,
    pub sku: String,
    pub name: String,
    pub price: f64,
    pub created_by: u64, // User ID
}

/// Simplified Order model for testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Order {
    pub id: u64,
    pub order_number: String,
    pub customer_id: u64, // User ID
    pub total: f64,
    pub status: OrderStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
}

/// Simple application demonstration
pub struct EcommerceApp {
    users: HashMap<u64, User>,
    products: HashMap<u64, Product>,
    orders: HashMap<u64, Order>,
}

impl EcommerceApp {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            products: HashMap::new(),
            orders: HashMap::new(),
        }
    }

    pub fn add_user(&mut self, user: User) {
        self.users.insert(user.id, user);
    }

    pub fn add_product(&mut self, product: Product) {
        self.products.insert(product.id, product);
    }

    pub fn add_order(&mut self, order: Order) {
        self.orders.insert(order.id, order);
    }

    pub fn get_user(&self, id: u64) -> Option<&User> {
        self.users.get(&id)
    }

    pub fn get_product(&self, id: u64) -> Option<&Product> {
        self.products.get(&id)
    }

    pub fn get_order(&self, id: u64) -> Option<&Order> {
        self.orders.get(&id)
    }

    pub fn get_all_users(&self) -> Vec<&User> {
        self.users.values().collect()
    }

    pub fn get_all_products(&self) -> Vec<&Product> {
        self.products.values().collect()
    }

    pub fn get_all_orders(&self) -> Vec<&Order> {
        self.orders.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecommerce_app_basic_functionality() {
        let mut app = EcommerceApp::new();

        // Add a user
        let user = User {
            id: 1,
            email: "test@example.com".to_string(),
            username: "testuser".to_string(),
            name: "Test User".to_string(),
            active: true,
        };
        app.add_user(user.clone());

        // Add a product
        let product = Product {
            id: 1,
            sku: "LAPTOP-001".to_string(),
            name: "Gaming Laptop".to_string(),
            price: 999.99,
            created_by: 1, // Created by user 1
        };
        app.add_product(product.clone());

        // Add an order
        let order = Order {
            id: 1,
            order_number: "ORD-001".to_string(),
            customer_id: 1, // Customer is user 1
            total: 999.99,
            status: OrderStatus::Pending,
        };
        app.add_order(order.clone());

        // Test retrieval
        assert_eq!(app.get_user(1), Some(&user));
        assert_eq!(app.get_product(1), Some(&product));
        assert_eq!(app.get_order(1), Some(&order));

        // Test collections
        assert_eq!(app.get_all_users().len(), 1);
        assert_eq!(app.get_all_products().len(), 1);
        assert_eq!(app.get_all_orders().len(), 1);
    }

    #[test]
    fn test_order_status_enum() {
        let pending_order = Order {
            id: 1,
            order_number: "ORD-001".to_string(),
            customer_id: 1,
            total: 100.0,
            status: OrderStatus::Pending,
        };

        assert_eq!(pending_order.status, OrderStatus::Pending);
        assert_ne!(pending_order.status, OrderStatus::Delivered);
    }
}