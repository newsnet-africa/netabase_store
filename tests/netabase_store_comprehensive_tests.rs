//! Comprehensive tests for NetabaseStore unified API
//!
//! This test suite verifies that the NetabaseStore wrapper provides
//! consistent behavior across all backends while maintaining access
//! to backend-specific features.

#![cfg(not(target_arch = "wasm32"))]
#![cfg(feature = "native")]

use netabase_store::{NetabaseStore, NetabaseModel, netabase_definition_module, netabase};
use netabase_store::traits::batch::{Batchable, BatchBuilder};

// Test schema
#[netabase_definition_module(TestDefinition, TestKeys)]
mod schema {
    use super::*;

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        Eq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(TestDefinition)]
    pub struct Product {
        #[primary_key]
        pub id: u64,
        pub name: String,
        pub price: u64,
        #[secondary_key]
        pub category: String,
        #[secondary_key]
        pub in_stock: bool,
    }

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        Eq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(TestDefinition)]
    pub struct Customer {
        #[primary_key]
        pub id: u64,
        pub name: String,
        #[secondary_key]
        pub email: String,
    }
}

use schema::*;

// Helper function to create test products
fn create_test_products(count: u64) -> Vec<Product> {
    (0..count)
        .map(|i| Product {
            id: i,
            name: format!("Product {}", i),
            price: 100 + i * 10,
            category: if i % 3 == 0 {
                "Electronics".to_string()
            } else if i % 3 == 1 {
                "Books".to_string()
            } else {
                "Clothing".to_string()
            },
            in_stock: i % 2 == 0,
        })
        .collect()
}

// ========================================
// Test Suite for Sled Backend
// ========================================

#[cfg(feature = "sled")]
mod sled_tests {
    use super::*;

    #[test]
    fn test_sled_store_creation() {
        let store = NetabaseStore::<TestDefinition, _>::temp();
        assert!(store.is_ok());
    }

    #[test]
    fn test_sled_basic_crud() {
        let store = NetabaseStore::<TestDefinition, _>::temp().unwrap();
        let tree = store.open_tree::<Product>();

        let product = Product {
            id: 1,
            name: "Test Product".to_string(),
            price: 199,
            category: "Test".to_string(),
            in_stock: true,
        };

        // Create
        tree.put(product.clone()).unwrap();

        // Read
        let retrieved = tree.get(ProductPrimaryKey(1)).unwrap();
        assert_eq!(retrieved, Some(product.clone()));

        // Update
        let mut updated = product.clone();
        updated.price = 299;
        tree.put(updated.clone()).unwrap();

        let retrieved = tree.get(ProductPrimaryKey(1)).unwrap();
        assert_eq!(retrieved.unwrap().price, 299);

        // Delete
        let removed = tree.remove(ProductPrimaryKey(1)).unwrap();
        assert_eq!(removed, Some(updated));

        let retrieved = tree.get(ProductPrimaryKey(1)).unwrap();
        assert_eq!(retrieved, None);
    }

    #[test]
    fn test_sled_secondary_keys() {
        let store = NetabaseStore::<TestDefinition, _>::temp().unwrap();
        let tree = store.open_tree::<Product>();

        let products = create_test_products(10);
        for product in &products {
            tree.put(product.clone()).unwrap();
        }

        // Query by category
        let electronics = tree
            .get_by_secondary_key(ProductSecondaryKeys::Category(
                ProductCategorySecondaryKey("Electronics".to_string()),
            ))
            .unwrap();

        assert_eq!(electronics.len(), 4); // 0, 3, 6, 9

        // Query by stock status
        let in_stock = tree
            .get_by_secondary_key(ProductSecondaryKeys::InStock(
                ProductInStockSecondaryKey(true),
            ))
            .unwrap();

        assert_eq!(in_stock.len(), 5); // 0, 2, 4, 6, 8
    }

    #[test]
    fn test_sled_batch_operations() {
        let store = NetabaseStore::<TestDefinition, _>::temp().unwrap();
        let tree = store.open_tree::<Product>();

        let products = create_test_products(100);

        // Batch insert
        let mut batch = tree.create_batch().unwrap();
        for product in &products {
            batch.put(product.clone()).unwrap();
        }
        batch.commit().unwrap();

        // Verify all inserted
        assert_eq!(tree.len(), 100);

        // Batch remove
        let mut batch = tree.create_batch().unwrap();
        for i in 0..50 {
            batch.remove(ProductPrimaryKey(i)).unwrap();
        }
        batch.commit().unwrap();

        // Verify removed
        assert_eq!(tree.len(), 50);
    }

    #[test]
    fn test_sled_iteration() {
        let store = NetabaseStore::<TestDefinition, _>::temp().unwrap();
        let tree = store.open_tree::<Product>();

        let products = create_test_products(10);
        for product in &products {
            tree.put(product.clone()).unwrap();
        }

        let mut count = 0;
        for result in tree.iter() {
            let (_key, _product) = result.unwrap();
            count += 1;
        }

        assert_eq!(count, 10);
    }

    #[test]
    fn test_sled_backend_specific_features() {
        let store = NetabaseStore::<TestDefinition, _>::temp().unwrap();
        let tree = store.open_tree::<Product>();

        tree.put(Product {
            id: 1,
            name: "Test".to_string(),
            price: 100,
            category: "Test".to_string(),
            in_stock: true,
        })
        .unwrap();

        // Test flush (Sled-specific)
        let flushed_bytes = store.flush().unwrap();
        assert!(flushed_bytes > 0);

        // Test generate_id (Sled-specific)
        let id1 = store.generate_id().unwrap();
        let id2 = store.generate_id().unwrap();
        assert!(id2 > id1);
    }

    #[test]
    fn test_sled_multiple_models() {
        let store = NetabaseStore::<TestDefinition, _>::temp().unwrap();
        let product_tree = store.open_tree::<Product>();
        let customer_tree = store.open_tree::<Customer>();

        product_tree
            .put(Product {
                id: 1,
                name: "Product".to_string(),
                price: 100,
                category: "Test".to_string(),
                in_stock: true,
            })
            .unwrap();

        customer_tree
            .put(Customer {
                id: 1,
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
            })
            .unwrap();

        assert_eq!(product_tree.len(), 1);
        assert_eq!(customer_tree.len(), 1);
    }

    #[test]
    fn test_sled_clear() {
        let store = NetabaseStore::<TestDefinition, _>::temp().unwrap();
        let tree = store.open_tree::<Product>();

        let products = create_test_products(10);
        for product in &products {
            tree.put(product.clone()).unwrap();
        }

        assert_eq!(tree.len(), 10);

        tree.clear().unwrap();

        assert_eq!(tree.len(), 0);
        assert!(tree.is_empty());
    }
}

// ========================================
// Test Suite for Redb Backend
// ========================================

#[cfg(feature = "redb")]
mod redb_tests {
    use super::*;

    #[test]
    fn test_redb_store_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.redb");
        let store = NetabaseStore::<TestDefinition, _>::redb(&path);
        assert!(store.is_ok());
    }

    #[test]
    fn test_redb_basic_crud() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.redb");
        let store = NetabaseStore::<TestDefinition, _>::redb(&path).unwrap();
        let tree = store.open_tree::<Product>();

        let product = Product {
            id: 1,
            name: "Test Product".to_string(),
            price: 199,
            category: "Test".to_string(),
            in_stock: true,
        };

        // Create
        tree.put(product.clone()).unwrap();

        // Read
        let retrieved = tree.get(ProductKey::Primary(ProductPrimaryKey(1))).unwrap();
        assert_eq!(retrieved, Some(product.clone()));

        // Update
        let mut updated = product.clone();
        updated.price = 299;
        tree.put(updated.clone()).unwrap();

        let retrieved = tree.get(ProductKey::Primary(ProductPrimaryKey(1))).unwrap();
        assert_eq!(retrieved.unwrap().price, 299);

        // Delete
        let removed = tree.remove(ProductKey::Primary(ProductPrimaryKey(1))).unwrap();
        assert_eq!(removed, Some(updated));

        let retrieved = tree.get(ProductKey::Primary(ProductPrimaryKey(1))).unwrap();
        assert_eq!(retrieved, None);
    }

    #[test]
    fn test_redb_secondary_keys() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.redb");
        let store = NetabaseStore::<TestDefinition, _>::redb(&path).unwrap();
        let tree = store.open_tree::<Product>();

        let products = create_test_products(10);
        for product in &products {
            tree.put(product.clone()).unwrap();
        }

        // Query by category
        let electronics = tree
            .get_by_secondary_key(ProductSecondaryKeys::Category(
                ProductCategorySecondaryKey("Electronics".to_string()),
            ))
            .unwrap();

        assert_eq!(electronics.len(), 4);

        // Query by stock status
        let in_stock = tree
            .get_by_secondary_key(ProductSecondaryKeys::InStock(
                ProductInStockSecondaryKey(true),
            ))
            .unwrap();

        assert_eq!(in_stock.len(), 5);
    }

    #[test]
    fn test_redb_batch_operations() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.redb");
        let store = NetabaseStore::<TestDefinition, _>::redb(&path).unwrap();
        let tree = store.open_tree::<Product>();

        let products = create_test_products(100);

        // Batch insert
        let mut batch = tree.create_batch().unwrap();
        for product in &products {
            batch.put(product.clone()).unwrap();
        }
        batch.commit().unwrap();

        // Verify all inserted
        assert_eq!(tree.len().unwrap(), 100);

        // Batch remove
        let mut batch = tree.create_batch().unwrap();
        for i in 0..50 {
            batch.remove(ProductPrimaryKey(i)).unwrap();
        }
        batch.commit().unwrap();

        // Verify removed
        assert_eq!(tree.len().unwrap(), 50);
    }

    #[test]
    fn test_redb_iteration() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.redb");
        let store = NetabaseStore::<TestDefinition, _>::redb(&path).unwrap();
        let tree = store.open_tree::<Product>();

        let products = create_test_products(10);
        for product in &products {
            tree.put(product.clone()).unwrap();
        }

        // Redb's iter() returns Result<Vec<...>>
        let results = tree.iter().unwrap();
        assert_eq!(results.len(), 10);

        for (_key, _product) in results {
            // All items iterated
        }
    }

    #[test]
    fn test_redb_backend_specific_features() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.redb");
        {
            let mut store = NetabaseStore::<TestDefinition, _>::redb(&path).unwrap();
            let tree = store.open_tree::<Product>();

            tree.put(Product {
                id: 1,
                name: "Test".to_string(),
                price: 100,
                category: "Test".to_string(),
                in_stock: true,
            })
            .unwrap();

            // Test tree_names (Redb-specific)
            let names = store.tree_names();
            assert!(!names.is_empty());
        } // Drop store to release all references before integrity check

        // Test check_integrity (Redb-specific) - requires exclusive access
        {
            let mut store = NetabaseStore::<TestDefinition, _>::redb(&path).unwrap();
            let is_valid = store.check_integrity().unwrap();
            assert!(is_valid);
        }

        // Test compact (Redb-specific) - requires exclusive access
        {
            let mut store = NetabaseStore::<TestDefinition, _>::redb(&path).unwrap();
            let compacted = store.compact().unwrap();
            assert!(compacted);
        }
    }

    #[test]
    fn test_redb_multiple_models() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.redb");
        let store = NetabaseStore::<TestDefinition, _>::redb(&path).unwrap();
        let product_tree = store.open_tree::<Product>();
        let customer_tree = store.open_tree::<Customer>();

        product_tree
            .put(Product {
                id: 1,
                name: "Product".to_string(),
                price: 100,
                category: "Test".to_string(),
                in_stock: true,
            })
            .unwrap();

        customer_tree
            .put(Customer {
                id: 1,
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
            })
            .unwrap();

        assert_eq!(product_tree.len().unwrap(), 1);
        assert_eq!(customer_tree.len().unwrap(), 1);
    }

    #[test]
    fn test_redb_clear() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.redb");
        let store = NetabaseStore::<TestDefinition, _>::redb(&path).unwrap();
        let tree = store.open_tree::<Product>();

        let products = create_test_products(10);
        for product in &products {
            tree.put(product.clone()).unwrap();
        }

        assert_eq!(tree.len().unwrap(), 10);

        tree.clear().unwrap();

        assert_eq!(tree.len().unwrap(), 0);
        assert!(tree.is_empty().unwrap());
    }

    #[test]
    fn test_redb_persistence() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.redb");

        // Write data
        {
            let store = NetabaseStore::<TestDefinition, _>::redb(&path).unwrap();
            let tree = store.open_tree::<Product>();

            tree.put(Product {
                id: 1,
                name: "Persistent Product".to_string(),
                price: 100,
                category: "Test".to_string(),
                in_stock: true,
            })
            .unwrap();
        }

        // Read data
        {
            let store = NetabaseStore::<TestDefinition, _>::open_redb(&path).unwrap();
            let tree = store.open_tree::<Product>();

            let product = tree.get(ProductKey::Primary(ProductPrimaryKey(1))).unwrap();
            assert!(product.is_some());
            assert_eq!(product.unwrap().name, "Persistent Product");
        }
    }
}

// ========================================
// Cross-Backend Compatibility Tests
// ========================================

#[cfg(all(feature = "sled", feature = "redb"))]
mod cross_backend_tests {
    use super::*;

    #[test]
    fn test_same_api_across_backends() {
        // This test verifies that the same code works with both backends

        let test_product = Product {
            id: 42,
            name: "Universal Product".to_string(),
            price: 999,
            category: "Test".to_string(),
            in_stock: true,
        };

        // Test with Sled
        {
            let store = NetabaseStore::<TestDefinition, _>::temp().unwrap();
            let tree = store.open_tree::<Product>();

            tree.put(test_product.clone()).unwrap();
            let retrieved = tree.get(ProductPrimaryKey(42)).unwrap();
            assert_eq!(retrieved, Some(test_product.clone()));
        }

        // Test with Redb
        {
            let temp_dir = tempfile::tempdir().unwrap();
            let path = temp_dir.path().join("test.redb");
            let store = NetabaseStore::<TestDefinition, _>::redb(&path).unwrap();
            let tree = store.open_tree::<Product>();

            tree.put(test_product.clone()).unwrap();
            let retrieved = tree.get(ProductKey::Primary(ProductPrimaryKey(42))).unwrap();
            assert_eq!(retrieved, Some(test_product));
        }
    }
}
