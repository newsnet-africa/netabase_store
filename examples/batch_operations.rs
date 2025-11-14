//! Batch Operations Example
//!
//! This example demonstrates the power of batch operations in netabase_store.
//! Batch operations allow you to:
//! - Perform multiple operations atomically
//! - Achieve 10-100x better performance for bulk operations
//! - Ensure consistency (all operations succeed or all fail)
//!
//! Run this example with:
//! ```bash
//! cargo run --example batch_operations --features native
//! ```

use netabase_store::NetabaseStore;
use netabase_store::netabase_definition_module;
use netabase_store::traits::batch::{BatchBuilder, Batchable};
use netabase_store::traits::store_ops::OpenTree;
use std::time::Instant;

#[netabase_definition_module(AppDefinition, AppKeys)]
pub mod models {
    use netabase_store::{NetabaseModel, netabase};

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(AppDefinition)]
    pub struct Product {
        #[primary_key]
        pub id: u64,
        pub name: String,
        pub price: f64,
        #[secondary_key]
        pub category: String,
        pub in_stock: bool,
    }

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(AppDefinition)]
    pub struct Order {
        #[primary_key]
        pub order_id: String,
        pub customer_id: u64,
        pub product_ids: Vec<u64>,
        pub total: f64,
    }
}

use models::*;

fn main() -> anyhow::Result<()> {
    println!("=== Netabase Store: Batch Operations Example ===\n");

    // Create a temporary store
    let store = NetabaseStore::<AppDefinition, _>::temp()?;
    let product_tree = store.open_tree::<Product>();

    // Example 1: Basic Batch Operations
    println!("Example 1: Basic Batch Insert");
    println!("-------------------------------");
    basic_batch_example(&product_tree)?;

    // Example 2: Performance Comparison
    println!("\nExample 2: Performance Comparison");
    println!("----------------------------------");
    performance_comparison(&store)?;

    // Example 3: Mixed Operations (Put and Remove)
    println!("\nExample 3: Mixed Batch Operations");
    println!("----------------------------------");
    mixed_batch_operations(&product_tree)?;

    // Example 4: Cross-Model Batching
    println!("\nExample 4: Cross-Model Operations");
    println!("----------------------------------");
    cross_model_example(&store)?;

    println!("\n✅ All batch operation examples completed successfully!");

    Ok(())
}

/// Example 1: Basic batch insert of multiple products
fn basic_batch_example(
    product_tree: &netabase_store::databases::sled_store::SledStoreTree<'_, AppDefinition, Product>,
) -> anyhow::Result<()> {
    // Create a batch
    let mut batch = product_tree.create_batch()?;

    // Add multiple products to the batch
    let products = vec![
        Product {
            id: 1,
            name: "Laptop".to_string(),
            price: 999.99,
            category: "Electronics".to_string(),
            in_stock: true,
        },
        Product {
            id: 2,
            name: "Mouse".to_string(),
            price: 29.99,
            category: "Electronics".to_string(),
            in_stock: true,
        },
        Product {
            id: 3,
            name: "Desk".to_string(),
            price: 299.99,
            category: "Furniture".to_string(),
            in_stock: false,
        },
    ];

    for product in products {
        batch.put(product)?;
    }

    // Commit all operations atomically
    batch.commit()?;

    // Verify the products were inserted
    let count = product_tree.len();
    println!(
        "✓ Successfully inserted {} products using batch operation",
        count
    );

    // Query by secondary key to verify
    let electronics = product_tree.get_by_secondary_key(ProductSecondaryKeys::Category(
        ProductCategorySecondaryKey("Electronics".to_string()),
    ))?;
    println!("✓ Found {} electronics products", electronics.len());

    Ok(())
}

/// Example 2: Performance comparison between individual puts and batch operations
fn performance_comparison(
    store: &NetabaseStore<
        AppDefinition,
        netabase_store::databases::sled_store::SledStore<AppDefinition>,
    >,
) -> anyhow::Result<()> {
    const NUM_ITEMS: u64 = 1000;

    // Test individual puts
    let product_tree_individual = store.open_tree::<Product>();
    let start = Instant::now();

    for i in 0..NUM_ITEMS {
        let product = Product {
            id: i + 1000, // Offset to avoid conflicts
            name: format!("Product {}", i),
            price: (i as f64) * 1.5,
            category: "Test".to_string(),
            in_stock: i % 2 == 0,
        };
        product_tree_individual.put(product)?;
    }

    let individual_duration = start.elapsed();
    println!(
        "Individual puts: {} items in {:?}",
        NUM_ITEMS, individual_duration
    );

    // Test batch operations
    let product_tree_batch = store.open_tree::<Product>();
    let start = Instant::now();

    let mut batch = product_tree_batch.create_batch()?;
    for i in 0..NUM_ITEMS {
        let product = Product {
            id: i + 2000, // Different offset
            name: format!("Product {}", i),
            price: (i as f64) * 1.5,
            category: "Test".to_string(),
            in_stock: i % 2 == 0,
        };
        batch.put(product)?;
    }
    batch.commit()?;

    let batch_duration = start.elapsed();
    println!(
        "Batch operation: {} items in {:?}",
        NUM_ITEMS, batch_duration
    );

    // Calculate speedup
    let speedup = individual_duration.as_micros() as f64 / batch_duration.as_micros() as f64;
    println!("✓ Batch operations were {:.2}x faster!", speedup);

    Ok(())
}

/// Example 3: Mixed batch operations (both puts and removes)
fn mixed_batch_operations(
    product_tree: &netabase_store::databases::sled_store::SledStoreTree<'_, AppDefinition, Product>,
) -> anyhow::Result<()> {
    // First, add some products
    let mut setup_batch = product_tree.create_batch()?;
    for i in 10..20 {
        let product = Product {
            id: i,
            name: format!("Temp Product {}", i),
            price: (i as f64) * 10.0,
            category: "Temporary".to_string(),
            in_stock: true,
        };
        setup_batch.put(product)?;
    }
    setup_batch.commit()?;

    println!("✓ Added 10 temporary products");

    // Now perform mixed operations
    let mut mixed_batch = product_tree.create_batch()?;

    // Update some products
    for i in 10..15 {
        let product = Product {
            id: i,
            name: format!("Updated Product {}", i),
            price: (i as f64) * 15.0, // New price
            category: "Updated".to_string(),
            in_stock: false,
        };
        mixed_batch.put(product)?;
    }

    // Remove some products
    for i in 15..20 {
        mixed_batch.remove(ProductPrimaryKey(i))?;
    }

    // Commit all changes atomically
    mixed_batch.commit()?;

    println!("✓ Updated 5 products and removed 5 products in one atomic operation");

    // Verify the changes
    let updated = product_tree.get_by_secondary_key(ProductSecondaryKeys::Category(
        ProductCategorySecondaryKey("Updated".to_string()),
    ))?;
    println!("✓ Found {} updated products", updated.len());

    Ok(())
}

/// Example 4: Using batches with multiple model types
fn cross_model_example(
    store: &NetabaseStore<
        AppDefinition,
        netabase_store::databases::sled_store::SledStore<AppDefinition>,
    >,
) -> anyhow::Result<()> {
    let product_tree = store.open_tree::<Product>();
    let order_tree = store.open_tree::<Order>();

    // Batch insert products
    let mut product_batch = product_tree.create_batch()?;
    let product_ids = vec![100, 101, 102];

    for &id in &product_ids {
        let product = Product {
            id,
            name: format!("Product {}", id),
            price: (id as f64) * 2.5,
            category: "Sale".to_string(),
            in_stock: true,
        };
        product_batch.put(product)?;
    }
    product_batch.commit()?;

    // Batch insert orders referencing those products
    let mut order_batch = order_tree.create_batch()?;

    for i in 0..5 {
        let order = Order {
            order_id: format!("ORD-{:04}", i),
            customer_id: i + 1000,
            product_ids: product_ids.clone(),
            total: 300.0 + (i as f64 * 10.0),
        };
        order_batch.put(order)?;
    }
    order_batch.commit()?;

    println!("✓ Inserted {} products", product_ids.len());
    println!("✓ Inserted 5 orders referencing those products");
    println!("✓ All operations completed atomically within their model type");

    Ok(())
}
