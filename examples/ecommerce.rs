use netabase_store::{
    databases::redb_store::RedbStore,
    traits::store::store::StoreTrait,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete E-commerce System Example
/// 
/// This example demonstrates:
/// - Cross-definition linking
/// - Hierarchical permissions
/// - Complex data relationships
/// - Real-world business logic

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProductCategory {
    Electronics,
    Clothing,
    Books,
    Home,
    Sports,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderStatus {
    Pending,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    Admin,
    Customer,
    Seller,
    Support,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Address {
    pub street: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaymentInfo {
    pub method: String,
    pub last_four: String,
    pub expiry: String,
}

/// User entity with hierarchical permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub addresses: Vec<Address>,
    pub payment_info: Option<PaymentInfo>,
    pub created_at: u64,
    pub last_login: Option<u64>,
    pub is_active: bool,
}

/// Product entity with cross-definition linking to orders and reviews
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Product {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub price: u64, // in cents
    pub category: ProductCategory,
    pub stock_quantity: u32,
    pub seller_id: u64,
    pub images: Vec<String>,
    pub tags: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub is_active: bool,
}

/// Order entity linking users and products
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Order {
    pub id: u64,
    pub user_id: u64,
    pub items: Vec<OrderItem>,
    pub total_amount: u64, // in cents
    pub status: OrderStatus,
    pub shipping_address: Address,
    pub payment_info: PaymentInfo,
    pub created_at: u64,
    pub updated_at: u64,
    pub tracking_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderItem {
    pub product_id: u64,
    pub quantity: u32,
    pub price_at_time: u64, // in cents
}

/// Review entity with multi-definition relationships
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Review {
    pub id: u64,
    pub user_id: u64,
    pub product_id: u64,
    pub rating: u8, // 1-5 stars
    pub title: String,
    pub comment: String,
    pub created_at: u64,
    pub helpful_votes: u32,
    pub verified_purchase: bool,
}

/// Shopping cart with session management
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Cart {
    pub id: u64,
    pub user_id: Option<u64>, // None for guest carts
    pub session_id: String,
    pub items: Vec<CartItem>,
    pub created_at: u64,
    pub updated_at: u64,
    pub expires_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CartItem {
    pub product_id: u64,
    pub quantity: u32,
    pub added_at: u64,
}

/// E-commerce application demonstrating all features
pub struct EcommerceApp {
    pub user_store: UserStore<InMemoryBackend>,
    pub product_store: ProductStore<InMemoryBackend>,
    pub order_store: OrderStore<InMemoryBackend>,
    pub review_store: ReviewStore<InMemoryBackend>,
    pub cart_store: CartStore<InMemoryBackend>,
}

impl EcommerceApp {
    /// Initialize the e-commerce application
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(EcommerceApp {
            user_store: UserStore::new_in_memory()?,
            product_store: ProductStore::new_in_memory()?,
            order_store: OrderStore::new_in_memory()?,
            review_store: ReviewStore::new_in_memory()?,
            cart_store: CartStore::new_in_memory()?,
        })
    }
    
    /// Create a new user account
    pub fn create_user(&mut self, user: User) -> Result<(), Box<dyn std::error::Error>> {
        // Validate user data
        if user.username.len() < 3 {
            return Err("Username must be at least 3 characters".into());
        }
        
        if !user.email.contains('@') {
            return Err("Invalid email format".into());
        }
        
        // Check for duplicate username
        for existing_user in self.user_store.scan()? {
            if existing_user.username == user.username {
                return Err("Username already exists".into());
            }
        }
        
        // Check for duplicate email
        for existing_user in self.user_store.scan()? {
            if existing_user.email == user.email {
                return Err("Email already registered".into());
            }
        }
        
        self.user_store.insert(user.id, user)?;
        Ok(())
    }
    
    /// Add a new product (seller permission required)
    pub fn add_product(&mut self, product: Product) -> Result<(), Box<dyn std::error::Error>> {
        // Validate seller exists and has permission
        if let Some(seller) = self.user_store.get(&product.seller_id)? {
            if !matches!(seller.role, UserRole::Seller | UserRole::Admin) {
                return Err("User does not have seller permissions".into());
            }
        } else {
            return Err("Seller not found".into());
        }
        
        // Validate product data
        if product.name.is_empty() {
            return Err("Product name cannot be empty".into());
        }
        
        if product.price == 0 {
            return Err("Product price must be greater than 0".into());
        }
        
        self.product_store.insert(product.id, product)?;
        Ok(())
    }
    
    /// Create an order with cross-definition validation
    pub fn create_order(&mut self, order: Order) -> Result<(), Box<dyn std::error::Error>> {
        // Validate user exists
        if self.user_store.get(&order.user_id)?.is_none() {
            return Err("User not found".into());
        }
        
        // Validate all products exist and are available
        let mut total_calculated = 0;
        for item in &order.items {
            if let Some(product) = self.product_store.get(&item.product_id)? {
                if !product.is_active {
                    return Err(format!("Product {} is not available", product.name).into());
                }
                
                if product.stock_quantity < item.quantity {
                    return Err(format!("Insufficient stock for product {}", product.name).into());
                }
                
                total_calculated += item.price_at_time * item.quantity as u64;
            } else {
                return Err(format!("Product {} not found", item.product_id).into());
            }
        }
        
        // Validate calculated total matches order total
        if total_calculated != order.total_amount {
            return Err("Order total does not match calculated total".into());
        }
        
        // Update product stock quantities
        for item in &order.items {
            if let Some(mut product) = self.product_store.get(&item.product_id)? {
                product.stock_quantity -= item.quantity;
                product.updated_at = order.created_at;
                self.product_store.insert(product.id, product)?;
            }
        }
        
        // Insert order with cross-definition validation
        self.order_store.insert_with_cross_validation(&self.user_store, order.id, order)?;
        Ok(())
    }
    
    /// Add a product review with validation
    pub fn add_review(&mut self, review: Review) -> Result<(), Box<dyn std::error::Error>> {
        // Validate user exists
        if self.user_store.get(&review.user_id)?.is_none() {
            return Err("User not found".into());
        }
        
        // Validate product exists
        if self.product_store.get(&review.product_id)?.is_none() {
            return Err("Product not found".into());
        }
        
        // Check if user has purchased this product (for verified reviews)
        let mut has_purchased = false;
        for order in self.order_store.scan()? {
            if order.user_id == review.user_id {
                for item in &order.items {
                    if item.product_id == review.product_id {
                        has_purchased = true;
                        break;
                    }
                }
            }
            if has_purchased {
                break;
            }
        }
        
        let mut review = review;
        review.verified_purchase = has_purchased;
        
        // Validate rating
        if !(1..=5).contains(&review.rating) {
            return Err("Rating must be between 1 and 5".into());
        }
        
        self.review_store.insert(review.id, review)?;
        Ok(())
    }
    
    /// Manage shopping cart
    pub fn add_to_cart(&mut self, cart_id: u64, product_id: u64, quantity: u32) -> Result<(), Box<dyn std::error::Error>> {
        // Validate product exists and is available
        if let Some(product) = self.product_store.get(&product_id)? {
            if !product.is_active {
                return Err("Product is not available".into());
            }
            
            if product.stock_quantity < quantity {
                return Err("Insufficient stock".into());
            }
        } else {
            return Err("Product not found".into());
        }
        
        // Get or create cart
        let mut cart = if let Some(existing_cart) = self.cart_store.get(&cart_id)? {
            existing_cart
        } else {
            return Err("Cart not found".into());
        };
        
        // Check if item already in cart
        let mut found = false;
        for item in &mut cart.items {
            if item.product_id == product_id {
                item.quantity += quantity;
                found = true;
                break;
            }
        }
        
        if !found {
            cart.items.push(CartItem {
                product_id,
                quantity,
                added_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            });
        }
        
        cart.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        self.cart_store.insert(cart_id, cart)?;
        Ok(())
    }
    
    /// Get user order history
    pub fn get_user_orders(&self, user_id: u64) -> Result<Vec<Order>, Box<dyn std::error::Error>> {
        let orders: Vec<Order> = self.order_store
            .scan()?
            .filter(|order| order.user_id == user_id)
            .collect();
        Ok(orders)
    }
    
    /// Get product reviews
    pub fn get_product_reviews(&self, product_id: u64) -> Result<Vec<Review>, Box<dyn std::error::Error>> {
        let reviews: Vec<Review> = self.review_store
            .scan()?
            .filter(|review| review.product_id == product_id)
            .collect();
        Ok(reviews)
    }
    
    /// Calculate product average rating
    pub fn get_product_rating(&self, product_id: u64) -> Result<Option<f64>, Box<dyn std::error::Error>> {
        let reviews: Vec<Review> = self.review_store
            .scan()?
            .filter(|review| review.product_id == product_id)
            .collect();
        
        if reviews.is_empty() {
            return Ok(None);
        }
        
        let total: u32 = reviews.iter().map(|r| r.rating as u32).sum();
        let average = total as f64 / reviews.len() as f64;
        Ok(Some(average))
    }
    
    /// Search products by category and filters
    pub fn search_products(
        &self,
        category: Option<ProductCategory>,
        min_price: Option<u64>,
        max_price: Option<u64>,
        search_term: Option<&str>,
    ) -> Result<Vec<Product>, Box<dyn std::error::Error>> {
        let mut products: Vec<Product> = self.product_store
            .scan()?
            .filter(|product| product.is_active)
            .collect();
        
        if let Some(cat) = category {
            products.retain(|p| p.category == cat);
        }
        
        if let Some(min) = min_price {
            products.retain(|p| p.price >= min);
        }
        
        if let Some(max) = max_price {
            products.retain(|p| p.price <= max);
        }
        
        if let Some(term) = search_term {
            let term = term.to_lowercase();
            products.retain(|p| {
                p.name.to_lowercase().contains(&term) ||
                p.description.to_lowercase().contains(&term) ||
                p.tags.iter().any(|tag| tag.to_lowercase().contains(&term))
            });
        }
        
        // Sort by relevance (simple sorting by name for this example)
        products.sort_by(|a, b| a.name.cmp(&b.name));
        
        Ok(products)
    }
    
    /// Admin function: Get sales analytics
    pub fn get_sales_analytics(&self) -> Result<SalesAnalytics, Box<dyn std::error::Error>> {
        let orders: Vec<Order> = self.order_store.scan()?.collect();
        
        let total_revenue: u64 = orders.iter()
            .filter(|o| matches!(o.status, OrderStatus::Delivered))
            .map(|o| o.total_amount)
            .sum();
        
        let total_orders = orders.len();
        let delivered_orders = orders.iter()
            .filter(|o| matches!(o.status, OrderStatus::Delivered))
            .count();
        
        let mut product_sales: HashMap<u64, u32> = HashMap::new();
        for order in &orders {
            for item in &order.items {
                *product_sales.entry(item.product_id).or_insert(0) += item.quantity;
            }
        }
        
        let top_products: Vec<(u64, u32)> = {
            let mut sales_vec: Vec<(u64, u32)> = product_sales.into_iter().collect();
            sales_vec.sort_by(|a, b| b.1.cmp(&a.1));
            sales_vec.into_iter().take(10).collect()
        };
        
        Ok(SalesAnalytics {
            total_revenue,
            total_orders,
            delivered_orders,
            top_products,
        })
    }
}

/// Sales analytics structure
#[derive(Debug)]
pub struct SalesAnalytics {
    pub total_revenue: u64,
    pub total_orders: usize,
    pub delivered_orders: usize,
    pub top_products: Vec<(u64, u32)>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 E-commerce System Demo");
    
    let mut app = EcommerceApp::new()?;
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    
    // Create users
    println!("\n📝 Creating users...");
    let admin = User {
        id: 1,
        username: "admin".to_string(),
        email: "admin@ecommerce.com".to_string(),
        role: UserRole::Admin,
        addresses: vec![],
        payment_info: None,
        created_at: current_time,
        last_login: Some(current_time),
        is_active: true,
    };
    app.create_user(admin)?;
    
    let seller = User {
        id: 2,
        username: "seller1".to_string(),
        email: "seller@example.com".to_string(),
        role: UserRole::Seller,
        addresses: vec![],
        payment_info: None,
        created_at: current_time,
        last_login: Some(current_time),
        is_active: true,
    };
    app.create_user(seller)?;
    
    let customer = User {
        id: 3,
        username: "customer1".to_string(),
        email: "customer@example.com".to_string(),
        role: UserRole::Customer,
        addresses: vec![Address {
            street: "123 Main St".to_string(),
            city: "Anytown".to_string(),
            state: "CA".to_string(),
            zip: "90210".to_string(),
            country: "USA".to_string(),
        }],
        payment_info: Some(PaymentInfo {
            method: "Credit Card".to_string(),
            last_four: "1234".to_string(),
            expiry: "12/25".to_string(),
        }),
        created_at: current_time,
        last_login: Some(current_time),
        is_active: true,
    };
    app.create_user(customer)?;
    
    // Add products
    println!("📦 Adding products...");
    let laptop = Product {
        id: 1,
        name: "Gaming Laptop".to_string(),
        description: "High-performance gaming laptop with RTX graphics".to_string(),
        price: 149999, // $1499.99
        category: ProductCategory::Electronics,
        stock_quantity: 10,
        seller_id: 2,
        images: vec!["laptop1.jpg".to_string(), "laptop2.jpg".to_string()],
        tags: vec!["gaming".to_string(), "laptop".to_string(), "RTX".to_string()],
        created_at: current_time,
        updated_at: current_time,
        is_active: true,
    };
    app.add_product(laptop)?;
    
    let book = Product {
        id: 2,
        name: "Rust Programming Book".to_string(),
        description: "Learn Rust programming language".to_string(),
        price: 3999, // $39.99
        category: ProductCategory::Books,
        stock_quantity: 50,
        seller_id: 2,
        images: vec!["book1.jpg".to_string()],
        tags: vec!["programming".to_string(), "rust".to_string(), "book".to_string()],
        created_at: current_time,
        updated_at: current_time,
        is_active: true,
    };
    app.add_product(book)?;
    
    // Create shopping cart
    println!("🛒 Creating shopping cart...");
    let cart = Cart {
        id: 1,
        user_id: Some(3),
        session_id: "session123".to_string(),
        items: vec![],
        created_at: current_time,
        updated_at: current_time,
        expires_at: current_time + 3600, // 1 hour
    };
    app.cart_store.insert(1, cart)?;
    
    // Add items to cart
    app.add_to_cart(1, 1, 1)?; // Add laptop
    app.add_to_cart(1, 2, 2)?; // Add 2 books
    
    // Create order
    println!("📋 Creating order...");
    let order = Order {
        id: 1,
        user_id: 3,
        items: vec![
            OrderItem {
                product_id: 1,
                quantity: 1,
                price_at_time: 149999,
            },
            OrderItem {
                product_id: 2,
                quantity: 2,
                price_at_time: 3999,
            },
        ],
        total_amount: 157997, // $1499.99 + 2 * $39.99
        status: OrderStatus::Pending,
        shipping_address: Address {
            street: "123 Main St".to_string(),
            city: "Anytown".to_string(),
            state: "CA".to_string(),
            zip: "90210".to_string(),
            country: "USA".to_string(),
        },
        payment_info: PaymentInfo {
            method: "Credit Card".to_string(),
            last_four: "1234".to_string(),
            expiry: "12/25".to_string(),
        },
        created_at: current_time,
        updated_at: current_time,
        tracking_number: None,
    };
    app.create_order(order)?;
    
    // Add product review
    println!("⭐ Adding product review...");
    let review = Review {
        id: 1,
        user_id: 3,
        product_id: 2, // Review the book
        rating: 5,
        title: "Excellent Book!".to_string(),
        comment: "Great resource for learning Rust programming.".to_string(),
        created_at: current_time,
        helpful_votes: 0,
        verified_purchase: false, // Will be set by the system
    };
    app.add_review(review)?;
    
    // Demonstrate cross-definition queries
    println!("\n🔍 Running cross-definition queries...");
    
    // Get user order history
    let user_orders = app.get_user_orders(3)?;
    println!("Customer orders: {}", user_orders.len());
    
    // Get product reviews
    let product_reviews = app.get_product_reviews(2)?;
    println!("Book reviews: {}", product_reviews.len());
    
    // Get product rating
    if let Some(rating) = app.get_product_rating(2)? {
        println!("Book average rating: {:.1}/5.0", rating);
    }
    
    // Search products
    let electronics = app.search_products(
        Some(ProductCategory::Electronics),
        Some(100000),
        Some(200000),
        None,
    )?;
    println!("Electronics in price range: {}", electronics.len());
    
    let search_results = app.search_products(
        None,
        None,
        None,
        Some("rust"),
    )?;
    println!("Products matching 'rust': {}", search_results.len());
    
    // Admin analytics
    println!("\n📊 Sales Analytics:");
    let analytics = app.get_sales_analytics()?;
    println!("Total Revenue: ${:.2}", analytics.total_revenue as f64 / 100.0);
    println!("Total Orders: {}", analytics.total_orders);
    println!("Delivered Orders: {}", analytics.delivered_orders);
    println!("Top Products: {:?}", analytics.top_products);
    
    println!("\n✅ E-commerce demo completed successfully!");
    println!("🎉 All cross-definition linking and permissions working perfectly!");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_user_creation() {
        let mut app = EcommerceApp::new().unwrap();
        let user = User {
            id: 1,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            role: UserRole::Customer,
            addresses: vec![],
            payment_info: None,
            created_at: 1640000000,
            last_login: None,
            is_active: true,
        };
        
        assert!(app.create_user(user).is_ok());
    }
    
    #[test]
    fn test_duplicate_username() {
        let mut app = EcommerceApp::new().unwrap();
        let user1 = User {
            id: 1,
            username: "duplicate".to_string(),
            email: "test1@example.com".to_string(),
            role: UserRole::Customer,
            addresses: vec![],
            payment_info: None,
            created_at: 1640000000,
            last_login: None,
            is_active: true,
        };
        
        let user2 = User {
            id: 2,
            username: "duplicate".to_string(),
            email: "test2@example.com".to_string(),
            role: UserRole::Customer,
            addresses: vec![],
            payment_info: None,
            created_at: 1640000000,
            last_login: None,
            is_active: true,
        };
        
        assert!(app.create_user(user1).is_ok());
        assert!(app.create_user(user2).is_err());
    }
    
    #[test]
    fn test_product_seller_validation() {
        let mut app = EcommerceApp::new().unwrap();
        
        // Create a customer (not a seller)
        let customer = User {
            id: 1,
            username: "customer".to_string(),
            email: "customer@example.com".to_string(),
            role: UserRole::Customer,
            addresses: vec![],
            payment_info: None,
            created_at: 1640000000,
            last_login: None,
            is_active: true,
        };
        app.create_user(customer).unwrap();
        
        // Try to add product as customer (should fail)
        let product = Product {
            id: 1,
            name: "Test Product".to_string(),
            description: "Test".to_string(),
            price: 1000,
            category: ProductCategory::Electronics,
            stock_quantity: 1,
            seller_id: 1, // Customer ID
            images: vec![],
            tags: vec![],
            created_at: 1640000000,
            updated_at: 1640000000,
            is_active: true,
        };
        
        assert!(app.add_product(product).is_err());
    }
    
    #[test]
    fn test_order_validation() {
        let mut app = EcommerceApp::new().unwrap();
        
        // Setup user and product
        let user = User {
            id: 1,
            username: "customer".to_string(),
            email: "customer@example.com".to_string(),
            role: UserRole::Customer,
            addresses: vec![],
            payment_info: None,
            created_at: 1640000000,
            last_login: None,
            is_active: true,
        };
        app.create_user(user).unwrap();
        
        let seller = User {
            id: 2,
            username: "seller".to_string(),
            email: "seller@example.com".to_string(),
            role: UserRole::Seller,
            addresses: vec![],
            payment_info: None,
            created_at: 1640000000,
            last_login: None,
            is_active: true,
        };
        app.create_user(seller).unwrap();
        
        let product = Product {
            id: 1,
            name: "Test Product".to_string(),
            description: "Test".to_string(),
            price: 1000,
            category: ProductCategory::Electronics,
            stock_quantity: 5,
            seller_id: 2,
            images: vec![],
            tags: vec![],
            created_at: 1640000000,
            updated_at: 1640000000,
            is_active: true,
        };
        app.add_product(product).unwrap();
        
        // Create valid order
        let order = Order {
            id: 1,
            user_id: 1,
            items: vec![OrderItem {
                product_id: 1,
                quantity: 2,
                price_at_time: 1000,
            }],
            total_amount: 2000,
            status: OrderStatus::Pending,
            shipping_address: Address {
                street: "123 Test St".to_string(),
                city: "Test City".to_string(),
                state: "TS".to_string(),
                zip: "12345".to_string(),
                country: "Test Country".to_string(),
            },
            payment_info: PaymentInfo {
                method: "Test".to_string(),
                last_four: "1234".to_string(),
                expiry: "12/25".to_string(),
            },
            created_at: 1640000000,
            updated_at: 1640000000,
            tracking_number: None,
        };
        
        assert!(app.create_order(order).is_ok());
        
        // Check stock was updated
        let updated_product = app.product_store.get(&1).unwrap().unwrap();
        assert_eq!(updated_product.stock_quantity, 3); // 5 - 2 = 3
    }
}