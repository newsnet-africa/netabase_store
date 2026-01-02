/// Comprehensive test that validates all README examples work correctly.
///
/// This test file verifies that every code example in README.md compiles and runs.
mod common;

use bincode::{Decode, Encode};
use netabase_macros::{NetabaseModel, netabase_definition};
use netabase_store::databases::redb::RedbStore;
use netabase_store::errors::NetabaseResult;
use netabase_store::query::{QueryConfig, QueryResult};
use netabase_store::traits::database::transaction::{NetabaseRoTransaction, NetabaseRwTransaction};

// ============================================================================
// Quick Start Example
// ============================================================================

#[netabase_definition]
mod blog {
    use super::*;

    #[derive(Debug, Clone, PartialEq, NetabaseModel, Encode, Decode)]
    pub struct User {
        #[primary]
        pub id: u64,
        pub username: String,
        #[secondary]
        pub email: String,
    }

    #[derive(Debug, Clone, PartialEq, NetabaseModel, Encode, Decode)]
    pub struct Post {
        #[primary]
        pub id: u64,
        pub title: String,
        pub content: String,
        #[secondary]
        pub author_id: u64,
    }
}

#[test]
fn test_readme_quick_start() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<blog::Blog>("readme_quick_start")?;

    // Create records in a write transaction
    {
        let txn = store.begin_write()?;

        let user = blog::User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        };

        txn.create(&user)?;

        let post = blog::Post {
            id: 1,
            title: "Hello World".to_string(),
            content: "My first post!".to_string(),
            author_id: 1,
        };

        txn.create(&post)?;
        txn.commit()?;
    }

    // Query records in a read transaction
    {
        let txn = store.begin_read()?;

        // Read by primary key
        let user: Option<blog::User> = txn.read(&1u64)?;
        assert!(user.is_some());
        assert_eq!(user.as_ref().unwrap().username, "alice");

        // Read by secondary key (email)
        let users_by_email = txn.read_by_secondary::<blog::User, _>(&"alice@example.com")?;
        assert_eq!(users_by_email.len(), 1);

        // Query posts by author
        let posts = txn.read_by_secondary::<blog::Post, _>(&1u64)?;
        assert_eq!(posts.len(), 1);
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

// ============================================================================
// Model Definition Examples
// ============================================================================

#[netabase_definition]
mod models_basic {
    use super::*;

    #[derive(NetabaseModel, Clone, Encode, Decode)]
    pub struct User {
        #[primary]
        pub id: u64,
        pub username: String,
        pub email: String,
        pub created_at: u64,
    }

    #[derive(NetabaseModel, Clone, Encode, Decode)]
    pub struct Product {
        #[primary]
        pub sku: String,
        pub name: String,
        pub price: u64,
        #[secondary]
        pub category: String,
        #[secondary]
        pub manufacturer: String,
    }
}

#[test]
fn test_readme_basic_models() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) =
        common::create_test_db::<models_basic::ModelsBasic>("readme_basic_models")?;

    {
        let txn = store.begin_write()?;

        let user = models_basic::User {
            id: 1,
            username: "test".to_string(),
            email: "test@example.com".to_string(),
            created_at: 123456,
        };
        txn.create(&user)?;

        let product = models_basic::Product {
            sku: "ABC123".to_string(),
            name: "Widget".to_string(),
            price: 1999,
            category: "Electronics".to_string(),
            manufacturer: "ACME".to_string(),
        };
        txn.create(&product)?;

        txn.commit()?;
    }

    {
        let txn = store.begin_read()?;

        let user = txn.read::<models_basic::User>(&1u64)?;
        assert!(user.is_some());

        let product = txn.read::<models_basic::Product>(&"ABC123".to_string())?;
        assert!(product.is_some());
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

// ============================================================================
// Secondary Index Examples
// ============================================================================

#[netabase_definition]
mod library {
    use super::*;

    #[derive(NetabaseModel, Clone, Encode, Decode)]
    pub struct Book {
        #[primary]
        pub isbn: String,
        pub title: String,
        #[secondary]
        pub author: String,
        #[secondary]
        pub genre: String,
        #[secondary]
        pub year: u32,
        pub pages: u32,
        pub available: bool,
    }
}

#[test]
fn test_readme_secondary_indexes() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<library::Library>("readme_secondary_idx")?;

    {
        let txn = store.begin_write()?;

        txn.create(&library::Book {
            isbn: "978-0-441-17271-9".to_string(),
            title: "Foundation".to_string(),
            author: "Isaac Asimov".to_string(),
            genre: "Science Fiction".to_string(),
            year: 1951,
            pages: 255,
            available: true,
        })?;

        txn.create(&library::Book {
            isbn: "978-0-553-38768-1".to_string(),
            title: "I, Robot".to_string(),
            author: "Isaac Asimov".to_string(),
            genre: "Science Fiction".to_string(),
            year: 1950,
            pages: 224,
            available: true,
        })?;

        txn.create(&library::Book {
            isbn: "978-0-345-33968-3".to_string(),
            title: "Dune".to_string(),
            author: "Frank Herbert".to_string(),
            genre: "Science Fiction".to_string(),
            year: 1965,
            pages: 688,
            available: false,
        })?;

        txn.commit()?;
    }

    {
        let txn = store.begin_read()?;

        // Query by genre
        let scifi_books = txn.read_by_secondary::<library::Book, _>(&"Science Fiction")?;
        assert_eq!(scifi_books.len(), 3);

        // Query by year
        let books_1951 = txn.read_by_secondary::<library::Book, _>(&1951u32)?;
        assert_eq!(books_1951.len(), 1);

        // Query by author
        let asimov_books = txn.read_by_secondary::<library::Book, _>(&"Isaac Asimov")?;
        assert_eq!(asimov_books.len(), 2);
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

// ============================================================================
// CRUD Operations
// ============================================================================

#[netabase_definition]
mod crud_test {
    use super::*;

    #[derive(Clone, NetabaseModel, Encode, Decode)]
    pub struct Item {
        #[primary]
        pub id: u64,
        pub name: String,
        pub value: u32,
    }
}

#[test]
fn test_readme_crud_operations() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<crud_test::CrudTest>("readme_crud")?;

    // Create
    {
        let txn = store.begin_write()?;

        let item = crud_test::Item {
            id: 1,
            name: "Test Item".to_string(),
            value: 100,
        };

        txn.create(&item)?;
        txn.commit()?;
    }

    // Read
    {
        let txn = store.begin_read()?;

        let item: Option<crud_test::Item> = txn.read(&1u64)?;
        assert!(item.is_some());
        assert_eq!(item.unwrap().name, "Test Item");
    }

    // Update
    {
        let txn = store.begin_write()?;

        let mut item: crud_test::Item = txn.read(&1u64)?.expect("Item not found");
        item.name = "Updated Item".to_string();
        item.value = 200;

        txn.update(&item)?;
        txn.commit()?;
    }

    // Verify update
    {
        let txn = store.begin_read()?;

        let item: crud_test::Item = txn.read(&1u64)?.unwrap();
        assert_eq!(item.name, "Updated Item");
        assert_eq!(item.value, 200);
    }

    // Delete
    {
        let txn = store.begin_write()?;
        txn.delete::<crud_test::Item>(&1u64)?;
        txn.commit()?;
    }

    // Verify deletion
    {
        let txn = store.begin_read()?;
        let item: Option<crud_test::Item> = txn.read(&1u64)?;
        assert!(item.is_none());
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

// ============================================================================
// Batch Operations
// ============================================================================

#[test]
fn test_readme_batch_operations() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<crud_test::CrudTest>("readme_batch")?;

    {
        let txn = store.begin_write()?;

        // Batch create
        for i in 1..=100 {
            let item = crud_test::Item {
                id: i,
                name: format!("Item {}", i),
                value: i as u32 * 10,
            };
            txn.create(&item)?;
        }

        txn.commit()?;
    }

    // Verify all created
    {
        let txn = store.begin_read()?;

        for i in 1..=100 {
            let item: Option<crud_test::Item> = txn.read(&i)?;
            assert!(item.is_some());
        }
    }

    // Batch update
    {
        let txn = store.begin_write()?;

        for i in 1..=50 {
            let mut item: crud_test::Item = txn.read(&i)?.unwrap();
            item.value += 1000;
            txn.update(&item)?;
        }

        txn.commit()?;
    }

    // Batch delete
    {
        let txn = store.begin_write()?;

        for i in 51..=100 {
            txn.delete::<crud_test::Item>(&i)?;
        }

        txn.commit()?;
    }

    // Verify final state
    {
        let txn = store.begin_read()?;

        // First 50 should exist with updated values
        for i in 1..=50 {
            let item: crud_test::Item = txn.read(&i)?.unwrap();
            assert!(item.value >= 1000);
        }

        // Last 50 should be deleted
        for i in 51..=100 {
            let item: Option<crud_test::Item> = txn.read(&i)?;
            assert!(item.is_none());
        }
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

// ============================================================================
// Transaction Isolation
// ============================================================================

#[test]
fn test_readme_transaction_isolation() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<crud_test::CrudTest>("readme_isolation")?;

    // Create initial data
    {
        let txn = store.begin_write()?;
        txn.create(&crud_test::Item {
            id: 1,
            name: "Original".to_string(),
            value: 100,
        })?;
        txn.commit()?;
    }

    // Start a write transaction but don't commit yet
    let write_txn = store.begin_write()?;
    let mut item = write_txn.read::<crud_test::Item>(&1)?.unwrap();
    item.name = "Modified".to_string();
    item.value = 200;
    write_txn.update(&item)?;
    // Not committed yet

    // Read transactions see the old committed state
    {
        let read_txn = store.begin_read()?;
        let item = read_txn.read::<crud_test::Item>(&1)?.unwrap();
        assert_eq!(item.name, "Original");
        assert_eq!(item.value, 100);
    }

    // Commit the write
    write_txn.commit()?;

    // Now reads see the new state
    {
        let read_txn = store.begin_read()?;
        let item = read_txn.read::<crud_test::Item>(&1)?.unwrap();
        assert_eq!(item.name, "Modified");
        assert_eq!(item.value, 200);
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

// ============================================================================
// Query Configuration
// ============================================================================

#[test]
fn test_readme_query_config() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<crud_test::CrudTest>("readme_query")?;

    // Create test data
    {
        let txn = store.begin_write()?;
        for i in 1..=100 {
            txn.create(&crud_test::Item {
                id: i,
                name: format!("Item {}", i),
                value: i as u32,
            })?;
        }
        txn.commit()?;
    }

    {
        let txn = store.begin_read()?;

        // Basic query - fetch all
        let config = QueryConfig::all();
        let result = txn.query::<crud_test::Item>(config)?;
        assert_eq!(result.len(), 100);

        // With limit
        let config = QueryConfig::default().with_limit(10);
        let result = txn.query::<crud_test::Item>(config)?;
        assert_eq!(result.len(), 10);

        // With pagination
        let config = QueryConfig::default().with_limit(20).with_offset(40);
        let result = txn.query::<crud_test::Item>(config)?;
        assert_eq!(result.len(), 20);

        // Count only
        let config = QueryConfig::default().count_only();
        let result = txn.query::<crud_test::Item>(config)?;
        assert_eq!(result.count().unwrap(), 100);

        // Range query
        let config = QueryConfig::new(20u64..30u64);
        let result = txn.query::<crud_test::Item>(config)?;
        assert_eq!(result.len(), 10);

        // First record
        let config = QueryConfig::first();
        let result = txn.query::<crud_test::Item>(config)?;
        assert_eq!(result.len(), 1);
    }

    common::cleanup_test_db(db_path);
    Ok(())
}

// ============================================================================
// Query Results
// ============================================================================

#[test]
fn test_readme_query_results() {
    // Single variant
    let single = QueryResult::Single(Some(42));
    assert_eq!(single.len(), 1);
    assert!(!single.is_empty());
    assert_eq!(single.as_single(), Some(&42));
    assert_eq!(single.clone().unwrap_single(), 42);

    // Multiple variant
    let multiple = QueryResult::Multiple(vec![1, 2, 3, 4, 5]);
    assert_eq!(multiple.len(), 5);
    assert!(!multiple.is_empty());
    assert_eq!(multiple.as_multiple(), Some(&vec![1, 2, 3, 4, 5]));

    // Count variant
    let count: QueryResult<i32> = QueryResult::Count(100);
    assert_eq!(count.len(), 100);
    assert!(!count.is_empty());
    assert_eq!(count.count(), Some(100));

    // Conversion to vec
    let vec = QueryResult::Multiple(vec![10, 20, 30]);
    assert_eq!(vec.into_vec(), vec![10, 20, 30]);
}

// ============================================================================
// E-Commerce Complete Example
// ============================================================================

#[netabase_definition]
mod shop {
    use super::*;

    #[derive(Debug, Clone, NetabaseModel, Encode, Decode)]
    pub struct Customer {
        #[primary]
        pub id: u64,
        pub name: String,
        #[secondary]
        pub email: String,
        pub created_at: u64,
    }

    #[derive(Debug, Clone, NetabaseModel, Encode, Decode)]
    pub struct Product {
        #[primary]
        pub sku: String,
        pub name: String,
        pub price: u64,
        #[secondary]
        pub category: String,
        pub stock: u32,
    }

    #[derive(Debug, Clone, NetabaseModel, Encode, Decode)]
    pub struct Order {
        #[primary]
        pub id: u64,
        #[secondary]
        pub customer_id: u64,
        pub product_sku: String,
        pub quantity: u32,
        pub total_price: u64,
        pub timestamp: u64,
    }
}

#[test]
fn test_readme_ecommerce_example() -> Result<(), Box<dyn std::error::Error>> {
    let (store, db_path) = common::create_test_db::<shop::Shop>("readme_ecommerce")?;

    // Add customers
    {
        let txn = store.begin_write()?;

        txn.create(&shop::Customer {
            id: 1,
            name: "Alice Johnson".into(),
            email: "alice@example.com".into(),
            created_at: 1704153600,
        })?;

        txn.commit()?;
    }

    // Add products
    {
        let txn = store.begin_write()?;

        txn.create(&shop::Product {
            sku: "LAPTOP-001".into(),
            name: "Professional Laptop".into(),
            price: 129999,
            category: "Electronics".into(),
            stock: 50,
        })?;

        txn.create(&shop::Product {
            sku: "MOUSE-001".into(),
            name: "Wireless Mouse".into(),
            price: 2999,
            category: "Accessories".into(),
            stock: 200,
        })?;

        txn.commit()?;
    }

    // Place an order
    {
        let txn = store.begin_write()?;

        // Check stock
        let mut product: shop::Product = txn
            .read(&"LAPTOP-001".to_string())?
            .expect("Product not found");

        assert!(product.stock >= 1, "Out of stock");

        // Create order
        txn.create(&shop::Order {
            id: 1,
            customer_id: 1,
            product_sku: "LAPTOP-001".into(),
            quantity: 1,
            total_price: product.price,
            timestamp: 1704240000,
        })?;

        // Update stock
        product.stock -= 1;
        txn.update(&product)?;

        txn.commit()?;
    }

    // Query customer orders
    {
        let txn = store.begin_read()?;

        let customer_orders = txn.read_by_secondary::<shop::Order, _>(&1u64)?;
        assert_eq!(customer_orders.len(), 1);

        for order in customer_orders {
            let product: shop::Product = txn.read(&order.product_sku)?.expect("Product not found");
            assert_eq!(product.name, "Professional Laptop");
            assert_eq!(product.stock, 49); // Reduced by 1
        }
    }

    // Find all electronics
    {
        let txn = store.begin_read()?;

        let electronics = txn.read_by_secondary::<shop::Product, _>(&"Electronics")?;
        assert_eq!(electronics.len(), 1);
        assert_eq!(electronics[0].name, "Professional Laptop");
    }

    common::cleanup_test_db(db_path);
    Ok(())
}
