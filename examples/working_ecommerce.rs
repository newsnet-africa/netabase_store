use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete E-commerce System Example
/// 
/// This example demonstrates a simplified e-commerce system with 
/// basic models and in-memory storage for demonstration purposes.

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
    pub card_type: String,
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

/// In-memory e-commerce application for demonstration
/// 
/// This is a simplified version using HashMaps instead of the full netabase store
/// to demonstrate the concepts without requiring the full macro-generated infrastructure
pub struct EcommerceApp {
    pub users: HashMap<u64, User>,
    pub products: HashMap<u64, Product>,
    pub orders: HashMap<u64, Order>,
    pub reviews: HashMap<u64, Review>,
    pub carts: HashMap<u64, Cart>,
    next_id: u64,
}

impl EcommerceApp {
    /// Initialize the e-commerce application
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(EcommerceApp {
            users: HashMap::new(),
            products: HashMap::new(),
            orders: HashMap::new(),
            reviews: HashMap::new(),
            carts: HashMap::new(),
            next_id: 1,
        })
    }
    
    fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    
    /// Create a new user account
    pub fn create_user(&mut self, mut user: User) -> Result<u64, Box<dyn std::error::Error>> {
        // Validate user data
        if user.username.len() < 3 {
            return Err("Username must be at least 3 characters".into());
        }

        // Check if username or email already exists
        for existing_user in self.users.values() {
            if existing_user.username == user.username {
                return Err("Username already exists".into());
            }
            if existing_user.email == user.email {
                return Err("Email already exists".into());
            }
        }

        // Generate ID and timestamps
        user.id = self.next_id();
        user.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let user_id = user.id;
        self.users.insert(user_id, user);
        Ok(user_id)
    }
    
    /// Create a new product
    pub fn create_product(&mut self, mut product: Product) -> Result<u64, Box<dyn std::error::Error>> {
        // Validate product data
        if product.name.is_empty() {
            return Err("Product name cannot be empty".into());
        }
        if product.price == 0 {
            return Err("Product price must be greater than 0".into());
        }

        // Check if seller exists
        if !self.users.contains_key(&product.seller_id) {
            return Err("Seller does not exist".into());
        }

        // Generate ID and timestamps
        product.id = self.next_id();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        product.created_at = now;
        product.updated_at = now;

        let product_id = product.id;
        self.products.insert(product_id, product);
        Ok(product_id)
    }
    
    /// Create a new order
    pub fn create_order(&mut self, mut order: Order) -> Result<u64, Box<dyn std::error::Error>> {
        // Validate user exists
        if !self.users.contains_key(&order.user_id) {
            return Err("User does not exist".into());
        }

        // Validate all products exist and calculate total
        let mut total = 0;
        for item in &order.items {
            if let Some(product) = self.products.get(&item.product_id) {
                if !product.is_active {
                    return Err(format!("Product {} is not active", product.name).into());
                }
                if product.stock_quantity < item.quantity {
                    return Err(format!("Insufficient stock for product {}", product.name).into());
                }
                total += item.price_at_time * item.quantity as u64;
            } else {
                return Err("Product does not exist".into());
            }
        }

        // Verify calculated total matches order total
        if total != order.total_amount {
            return Err("Order total does not match calculated total".into());
        }

        // Update product stock
        for item in &order.items {
            if let Some(product) = self.products.get_mut(&item.product_id) {
                product.stock_quantity -= item.quantity;
            }
        }

        // Generate ID and timestamps
        order.id = self.next_id();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        order.created_at = now;
        order.updated_at = now;

        let order_id = order.id;
        self.orders.insert(order_id, order);
        Ok(order_id)
    }
    
    /// Add a product review
    pub fn add_review(&mut self, mut review: Review) -> Result<u64, Box<dyn std::error::Error>> {
        // Validate user and product exist
        if !self.users.contains_key(&review.user_id) {
            return Err("User does not exist".into());
        }
        if !self.products.contains_key(&review.product_id) {
            return Err("Product does not exist".into());
        }

        // Validate rating
        if review.rating < 1 || review.rating > 5 {
            return Err("Rating must be between 1 and 5".into());
        }

        // Check if user has purchased this product (for verified purchase)
        let has_purchased = self.orders.values().any(|order| {
            order.user_id == review.user_id 
                && order.items.iter().any(|item| item.product_id == review.product_id)
                && matches!(order.status, OrderStatus::Delivered)
        });

        review.verified_purchase = has_purchased;

        // Generate ID and timestamp
        review.id = self.next_id();
        review.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let review_id = review.id;
        self.reviews.insert(review_id, review);
        Ok(review_id)
    }
    
    /// Search for products
    pub fn search_products(&self, query: &str) -> Vec<&Product> {
        let term = query.to_lowercase();
        self.products
            .values()
            .filter(|p| {
                p.is_active && (
                    p.name.to_lowercase().contains(&term) ||
                    p.description.to_lowercase().contains(&term) ||
                    p.tags.iter().any(|tag| tag.to_lowercase().contains(&term))
                )
            })
            .collect()
    }
    
    /// Get user's order history
    pub fn get_user_orders(&self, user_id: u64) -> Vec<&Order> {
        self.orders
            .values()
            .filter(|order| order.user_id == user_id)
            .collect()
    }
    
    /// Get product reviews
    pub fn get_product_reviews(&self, product_id: u64) -> Vec<&Review> {
        self.reviews
            .values()
            .filter(|review| review.product_id == product_id)
            .collect()
    }
    
    /// Update order status
    pub fn update_order_status(&mut self, order_id: u64, status: OrderStatus) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(order) = self.orders.get_mut(&order_id) {
            order.status = status;
            order.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            Ok(())
        } else {
            Err("Order not found".into())
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🛒 Netabase E-commerce Demo");
    println!("==========================");

    let mut app = EcommerceApp::new()?;

    // Create users
    let user1 = User {
        id: 0, // Will be assigned by create_user
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        role: UserRole::Customer,
        addresses: vec![Address {
            street: "123 Main St".to_string(),
            city: "Springfield".to_string(),
            state: "IL".to_string(),
            zip: "62701".to_string(),
            country: "USA".to_string(),
        }],
        payment_info: Some(PaymentInfo {
            card_type: "Visa".to_string(),
            last_four: "1234".to_string(),
            expiry: "12/25".to_string(),
        }),
        created_at: 0,
        last_login: None,
        is_active: true,
    };

    let seller = User {
        id: 0,
        username: "techstore".to_string(),
        email: "admin@techstore.com".to_string(),
        role: UserRole::Seller,
        addresses: vec![Address {
            street: "456 Commerce Ave".to_string(),
            city: "Tech City".to_string(),
            state: "CA".to_string(),
            zip: "90210".to_string(),
            country: "USA".to_string(),
        }],
        payment_info: None,
        created_at: 0,
        last_login: None,
        is_active: true,
    };

    let alice_id = app.create_user(user1)?;
    let seller_id = app.create_user(seller)?;

    println!("✅ Created users: Alice (ID: {}) and TechStore (ID: {})", alice_id, seller_id);

    // Create products
    let laptop = Product {
        id: 0,
        name: "Gaming Laptop".to_string(),
        description: "High-performance gaming laptop with RTX graphics".to_string(),
        price: 149999, // $1,499.99
        category: ProductCategory::Electronics,
        stock_quantity: 10,
        seller_id,
        images: vec!["laptop1.jpg".to_string(), "laptop2.jpg".to_string()],
        tags: vec!["gaming".to_string(), "laptop".to_string(), "rtx".to_string()],
        created_at: 0,
        updated_at: 0,
        is_active: true,
    };

    let headphones = Product {
        id: 0,
        name: "Wireless Headphones".to_string(),
        description: "Premium noise-cancelling wireless headphones".to_string(),
        price: 29999, // $299.99
        category: ProductCategory::Electronics,
        stock_quantity: 25,
        seller_id,
        images: vec!["headphones1.jpg".to_string()],
        tags: vec!["audio".to_string(), "wireless".to_string(), "noise-cancelling".to_string()],
        created_at: 0,
        updated_at: 0,
        is_active: true,
    };

    let laptop_id = app.create_product(laptop)?;
    let headphones_id = app.create_product(headphones)?;

    println!("✅ Created products: Laptop (ID: {}) and Headphones (ID: {})", laptop_id, headphones_id);

    // Create an order
    let order = Order {
        id: 0,
        user_id: alice_id,
        items: vec![
            OrderItem {
                product_id: laptop_id,
                quantity: 1,
                price_at_time: 149999,
            },
            OrderItem {
                product_id: headphones_id,
                quantity: 1,
                price_at_time: 29999,
            },
        ],
        total_amount: 179998, // $1,799.98
        status: OrderStatus::Pending,
        shipping_address: Address {
            street: "123 Main St".to_string(),
            city: "Springfield".to_string(),
            state: "IL".to_string(),
            zip: "62701".to_string(),
            country: "USA".to_string(),
        },
        payment_info: PaymentInfo {
            card_type: "Visa".to_string(),
            last_four: "1234".to_string(),
            expiry: "12/25".to_string(),
        },
        created_at: 0,
        updated_at: 0,
        tracking_number: None,
    };

    let order_id = app.create_order(order)?;
    println!("✅ Created order (ID: {}) for $1,799.98", order_id);

    // Update order status
    app.update_order_status(order_id, OrderStatus::Delivered)?;
    println!("✅ Updated order status to Delivered");

    // Add a review
    let review = Review {
        id: 0,
        user_id: alice_id,
        product_id: laptop_id,
        rating: 5,
        title: "Amazing gaming performance!".to_string(),
        comment: "This laptop handles all my games at max settings. Great purchase!".to_string(),
        created_at: 0,
        helpful_votes: 0,
        verified_purchase: false, // Will be set by add_review
    };

    let review_id = app.add_review(review)?;
    println!("✅ Added verified review (ID: {})", review_id);

    // Demonstrate search functionality
    let search_results = app.search_products("gaming");
    println!("🔍 Search results for 'gaming': {} products found", search_results.len());
    for product in search_results {
        println!("   - {} (${:.2})", product.name, product.price as f64 / 100.0);
    }

    // Show user's orders
    let user_orders = app.get_user_orders(alice_id);
    println!("📦 Alice's orders: {} orders found", user_orders.len());
    for order in user_orders {
        println!("   - Order {} ({:?}) - ${:.2}", 
                 order.id, order.status, order.total_amount as f64 / 100.0);
    }

    // Show product reviews
    let laptop_reviews = app.get_product_reviews(laptop_id);
    println!("⭐ Laptop reviews: {} reviews found", laptop_reviews.len());
    for review in laptop_reviews {
        println!("   - {} stars: {} (Verified: {})", 
                 review.rating, review.title, review.verified_purchase);
    }

    println!("\n🎉 E-commerce demo completed successfully!");
    
    Ok(())
}