#![cfg(not(target_arch = "wasm32"))]
#![cfg(not(feature = "paxos"))]

/// Comprehensive CRUD and secondary key tests for all three backends:
/// - SledStore (native)
/// - RedbStore (native)
/// - IndexedDBStore (WASM)
///
/// This test suite verifies that all backends can:
/// 1. Create/initialize stores
/// 2. Perform CRUD operations (Create, Read, Update, Delete)
/// 3. Query by secondary keys
/// 4. Handle multiple models in the same definition
/// 5. Iterate over records
/// 6. Clear and flush data

use netabase_store::{netabase, netabase_definition_module, NetabaseModel};

// Test schema shared across all backends
#[netabase_definition_module(TestDefinition, TestKeys)]
mod test_schema {
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
    pub struct User {
        #[primary_key]
        pub id: u64,
        pub username: String,
        #[secondary_key]
        pub email: String,
        #[secondary_key]
        pub age: u32,
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
    pub struct Product {
        #[primary_key]
        pub id: String,
        pub name: String,
        pub price: u64,
        #[secondary_key]
        pub category: String,
        #[secondary_key]
        pub in_stock: bool,
    }
}

use test_schema::*;

// ============================================================================
// SLED STORE TESTS (Native only)
// ============================================================================

#[cfg(feature = "native")]
mod sled_tests {
    use super::*;
    use netabase_store::databases::sled_store::SledStore;

    #[test]
    fn test_sled_create_store() {
        let store = SledStore::<TestDefinition>::temp();
        assert!(store.is_ok(), "Failed to create SledStore");
    }

    #[test]
    fn test_sled_crud_operations() {
        let store = SledStore::<TestDefinition>::temp().unwrap();
        let user_tree = store.open_tree::<User>();

        // CREATE
        let alice = User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 30,
        };
        assert!(user_tree.put(alice.clone()).is_ok(), "Failed to insert user");

        // READ
        let retrieved = user_tree.get(UserPrimaryKey(1)).unwrap();
        assert_eq!(Some(alice.clone()), retrieved, "Failed to retrieve user");

        // UPDATE
        let updated_alice = User {
            id: 1,
            username: "alice_updated".to_string(),
            email: "alice_new@example.com".to_string(),
            age: 31,
        };
        assert!(user_tree.put(updated_alice.clone()).is_ok(), "Failed to update user");

        let retrieved = user_tree.get(UserPrimaryKey(1)).unwrap();
        assert_eq!(Some(updated_alice), retrieved, "Updated user doesn't match");

        // DELETE
        let removed = user_tree.remove(UserPrimaryKey(1)).unwrap();
        assert!(removed.is_some(), "Failed to remove user");

        let retrieved = user_tree.get(UserPrimaryKey(1)).unwrap();
        assert_eq!(None, retrieved, "User should be deleted");
    }

    #[test]
    fn test_sled_secondary_key_single_result() {
        let store = SledStore::<TestDefinition>::temp().unwrap();
        let user_tree = store.open_tree::<User>();

        let alice = User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 30,
        };
        let bob = User {
            id: 2,
            username: "bob".to_string(),
            email: "bob@example.com".to_string(),
            age: 25,
        };

        user_tree.put(alice.clone()).unwrap();
        user_tree.put(bob.clone()).unwrap();

        // Query by email secondary key
        let results = user_tree
            .get_by_secondary_key(UserSecondaryKeys::Email(UserEmailSecondaryKey(
                "alice@example.com".to_string(),
            )))
            .unwrap();

        assert_eq!(1, results.len(), "Should find exactly one user");
        assert_eq!(alice, results[0], "Should find Alice");
    }

    #[test]
    fn test_sled_secondary_key_multiple_results() {
        let store = SledStore::<TestDefinition>::temp().unwrap();
        let user_tree = store.open_tree::<User>();

        let users = vec![
            User {
                id: 1,
                username: "alice".to_string(),
                email: "alice@example.com".to_string(),
                age: 30,
            },
            User {
                id: 2,
                username: "bob".to_string(),
                email: "bob@example.com".to_string(),
                age: 30,
            },
            User {
                id: 3,
                username: "carol".to_string(),
                email: "carol@example.com".to_string(),
                age: 25,
            },
        ];

        for user in &users {
            user_tree.put(user.clone()).unwrap();
        }

        // Query by age secondary key (should find 2 users with age 30)
        let results = user_tree
            .get_by_secondary_key(UserSecondaryKeys::Age(UserAgeSecondaryKey(30)))
            .unwrap();

        assert_eq!(2, results.len(), "Should find 2 users with age 30");
        assert!(results.contains(&users[0]), "Should include Alice");
        assert!(results.contains(&users[1]), "Should include Bob");
    }

    #[test]
    fn test_sled_multiple_models() {
        let store = SledStore::<TestDefinition>::temp().unwrap();

        // Test User model
        let user_tree = store.open_tree::<User>();
        let alice = User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 30,
        };
        user_tree.put(alice.clone()).unwrap();

        // Test Product model
        let product_tree = store.open_tree::<Product>();
        let laptop = Product {
            id: "LAPTOP-001".to_string(),
            name: "ThinkPad X1".to_string(),
            price: 1299,
            category: "Electronics".to_string(),
            in_stock: true,
        };
        product_tree.put(laptop.clone()).unwrap();

        // Verify both models are stored correctly
        assert_eq!(Some(alice), user_tree.get(UserPrimaryKey(1)).unwrap());
        assert_eq!(
            Some(laptop),
            product_tree.get(ProductPrimaryKey("LAPTOP-001".to_string())).unwrap()
        );
    }

    #[test]
    fn test_sled_iteration() {
        let store = SledStore::<TestDefinition>::temp().unwrap();
        let user_tree = store.open_tree::<User>();

        let users = vec![
            User {
                id: 1,
                username: "alice".to_string(),
                email: "alice@example.com".to_string(),
                age: 30,
            },
            User {
                id: 2,
                username: "bob".to_string(),
                email: "bob@example.com".to_string(),
                age: 25,
            },
            User {
                id: 3,
                username: "carol".to_string(),
                email: "carol@example.com".to_string(),
                age: 35,
            },
        ];

        for user in &users {
            user_tree.put(user.clone()).unwrap();
        }

        let mut retrieved = Vec::new();
        for result in user_tree.iter() {
            let (_, user) = result.unwrap();
            retrieved.push(user);
        }

        assert_eq!(3, retrieved.len(), "Should retrieve all 3 users");
        for user in &users {
            assert!(retrieved.contains(user), "Should contain user {:?}", user);
        }
    }

    #[test]
    fn test_sled_clear_and_len() {
        let store = SledStore::<TestDefinition>::temp().unwrap();
        let user_tree = store.open_tree::<User>();

        assert!(user_tree.is_empty(), "Tree should be empty initially");
        assert_eq!(0, user_tree.len(), "Length should be 0");

        let alice = User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 30,
        };
        user_tree.put(alice).unwrap();

        assert!(!user_tree.is_empty(), "Tree should not be empty");
        assert_eq!(1, user_tree.len(), "Length should be 1");

        user_tree.clear().unwrap();
        assert!(user_tree.is_empty(), "Tree should be empty after clear");
    }

    #[test]
    fn test_sled_string_primary_key() {
        let store = SledStore::<TestDefinition>::temp().unwrap();
        let product_tree = store.open_tree::<Product>();

        let laptop = Product {
            id: "LAPTOP-001".to_string(),
            name: "ThinkPad X1".to_string(),
            price: 1299,
            category: "Electronics".to_string(),
            in_stock: true,
        };

        product_tree.put(laptop.clone()).unwrap();

        let retrieved = product_tree
            .get(ProductPrimaryKey("LAPTOP-001".to_string()))
            .unwrap();
        assert_eq!(Some(laptop), retrieved);
    }

    #[test]
    fn test_sled_secondary_key_with_bool() {
        let store = SledStore::<TestDefinition>::temp().unwrap();
        let product_tree = store.open_tree::<Product>();

        let products = vec![
            Product {
                id: "PROD-1".to_string(),
                name: "Product 1".to_string(),
                price: 100,
                category: "Electronics".to_string(),
                in_stock: true,
            },
            Product {
                id: "PROD-2".to_string(),
                name: "Product 2".to_string(),
                price: 200,
                category: "Electronics".to_string(),
                in_stock: true,
            },
            Product {
                id: "PROD-3".to_string(),
                name: "Product 3".to_string(),
                price: 300,
                category: "Books".to_string(),
                in_stock: false,
            },
        ];

        for product in &products {
            product_tree.put(product.clone()).unwrap();
        }

        // Query by in_stock = true
        let results = product_tree
            .get_by_secondary_key(ProductSecondaryKeys::InStock(ProductInStockSecondaryKey(true)))
            .unwrap();

        assert_eq!(2, results.len(), "Should find 2 in-stock products");
        assert!(results.contains(&products[0]));
        assert!(results.contains(&products[1]));
    }
}

// ============================================================================
// MEMORY STORE TESTS (Always available)
// ============================================================================

mod memory_tests {
    use super::*;
    use netabase_store::databases::memory_store::MemoryStore;

    #[test]
    fn test_memory_create_store() {
        let store = MemoryStore::<TestDefinition>::new();
        assert!(store.tree_names().len() > 0, "Store should have trees");
    }

    #[test]
    fn test_memory_crud_operations() {
        let store = MemoryStore::<TestDefinition>::new();
        let user_tree = store.open_tree::<User>();

        // CREATE
        let alice = User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 30,
        };
        assert!(user_tree.put(alice.clone()).is_ok(), "Failed to insert user");

        // READ
        let retrieved = user_tree.get(UserPrimaryKey(1)).unwrap();
        assert_eq!(Some(alice.clone()), retrieved, "Failed to retrieve user");

        // UPDATE
        let updated_alice = User {
            id: 1,
            username: "alice_updated".to_string(),
            email: "alice_new@example.com".to_string(),
            age: 31,
        };
        assert!(user_tree.put(updated_alice.clone()).is_ok(), "Failed to update user");

        let retrieved = user_tree.get(UserPrimaryKey(1)).unwrap();
        assert_eq!(Some(updated_alice), retrieved, "Updated user doesn't match");

        // DELETE
        let removed = user_tree.remove(UserPrimaryKey(1)).unwrap();
        assert!(removed.is_some(), "Failed to remove user");

        let retrieved = user_tree.get(UserPrimaryKey(1)).unwrap();
        assert_eq!(None, retrieved, "User should be deleted");
    }

    #[test]
    fn test_memory_secondary_key_single_result() {
        let store = MemoryStore::<TestDefinition>::new();
        let user_tree = store.open_tree::<User>();

        let alice = User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 30,
        };
        let bob = User {
            id: 2,
            username: "bob".to_string(),
            email: "bob@example.com".to_string(),
            age: 25,
        };

        user_tree.put(alice.clone()).unwrap();
        user_tree.put(bob.clone()).unwrap();

        // Query by email secondary key
        let results = user_tree
            .get_by_secondary_key(UserSecondaryKeys::Email(UserEmailSecondaryKey(
                "alice@example.com".to_string(),
            )))
            .unwrap();

        assert_eq!(1, results.len(), "Should find exactly one user");
        assert_eq!(alice, results[0], "Should find Alice");
    }

    #[test]
    fn test_memory_secondary_key_multiple_results() {
        let store = MemoryStore::<TestDefinition>::new();
        let user_tree = store.open_tree::<User>();

        let users = vec![
            User {
                id: 1,
                username: "alice".to_string(),
                email: "alice@example.com".to_string(),
                age: 30,
            },
            User {
                id: 2,
                username: "bob".to_string(),
                email: "bob@example.com".to_string(),
                age: 30,
            },
            User {
                id: 3,
                username: "carol".to_string(),
                email: "carol@example.com".to_string(),
                age: 25,
            },
        ];

        for user in &users {
            user_tree.put(user.clone()).unwrap();
        }

        // Query by age secondary key (should find 2 users with age 30)
        let results = user_tree
            .get_by_secondary_key(UserSecondaryKeys::Age(UserAgeSecondaryKey(30)))
            .unwrap();

        assert_eq!(2, results.len(), "Should find 2 users with age 30");
        assert!(results.contains(&users[0]), "Should include Alice");
        assert!(results.contains(&users[1]), "Should include Bob");
    }

    #[test]
    fn test_memory_multiple_models() {
        let store = MemoryStore::<TestDefinition>::new();

        // Test User model
        let user_tree = store.open_tree::<User>();
        let alice = User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 30,
        };
        user_tree.put(alice.clone()).unwrap();

        // Test Product model
        let product_tree = store.open_tree::<Product>();
        let laptop = Product {
            id: "LAPTOP-001".to_string(),
            name: "ThinkPad X1".to_string(),
            price: 1299,
            category: "Electronics".to_string(),
            in_stock: true,
        };
        product_tree.put(laptop.clone()).unwrap();

        // Verify both models are stored correctly
        assert_eq!(Some(alice), user_tree.get(UserPrimaryKey(1)).unwrap());
        assert_eq!(
            Some(laptop),
            product_tree.get(ProductPrimaryKey("LAPTOP-001".to_string())).unwrap()
        );
    }

    #[test]
    fn test_memory_iteration() {
        let store = MemoryStore::<TestDefinition>::new();
        let user_tree = store.open_tree::<User>();

        let users = vec![
            User {
                id: 1,
                username: "alice".to_string(),
                email: "alice@example.com".to_string(),
                age: 30,
            },
            User {
                id: 2,
                username: "bob".to_string(),
                email: "bob@example.com".to_string(),
                age: 25,
            },
            User {
                id: 3,
                username: "carol".to_string(),
                email: "carol@example.com".to_string(),
                age: 35,
            },
        ];

        for user in &users {
            user_tree.put(user.clone()).unwrap();
        }

        let mut retrieved = Vec::new();
        for result in user_tree.iter() {
            let (_, user) = result.unwrap();
            retrieved.push(user);
        }

        assert_eq!(3, retrieved.len(), "Should retrieve all 3 users");
        for user in &users {
            assert!(retrieved.contains(user), "Should contain user {:?}", user);
        }
    }

    #[test]
    fn test_memory_clear_and_len() {
        let store = MemoryStore::<TestDefinition>::new();
        let user_tree = store.open_tree::<User>();

        assert!(user_tree.is_empty(), "Tree should be empty initially");
        assert_eq!(0, user_tree.len(), "Length should be 0");

        let alice = User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 30,
        };
        user_tree.put(alice).unwrap();

        assert!(!user_tree.is_empty(), "Tree should not be empty");
        assert_eq!(1, user_tree.len(), "Length should be 1");

        user_tree.clear().unwrap();
        assert!(user_tree.is_empty(), "Tree should be empty after clear");
    }

    #[test]
    fn test_memory_string_primary_key() {
        let store = MemoryStore::<TestDefinition>::new();
        let product_tree = store.open_tree::<Product>();

        let laptop = Product {
            id: "LAPTOP-001".to_string(),
            name: "ThinkPad X1".to_string(),
            price: 1299,
            category: "Electronics".to_string(),
            in_stock: true,
        };

        product_tree.put(laptop.clone()).unwrap();

        let retrieved = product_tree
            .get(ProductPrimaryKey("LAPTOP-001".to_string()))
            .unwrap();
        assert_eq!(Some(laptop), retrieved);
    }

    #[test]
    fn test_memory_secondary_key_with_bool() {
        let store = MemoryStore::<TestDefinition>::new();
        let product_tree = store.open_tree::<Product>();

        let products = vec![
            Product {
                id: "PROD-1".to_string(),
                name: "Product 1".to_string(),
                price: 100,
                category: "Electronics".to_string(),
                in_stock: true,
            },
            Product {
                id: "PROD-2".to_string(),
                name: "Product 2".to_string(),
                price: 200,
                category: "Electronics".to_string(),
                in_stock: true,
            },
            Product {
                id: "PROD-3".to_string(),
                name: "Product 3".to_string(),
                price: 300,
                category: "Books".to_string(),
                in_stock: false,
            },
        ];

        for product in &products {
            product_tree.put(product.clone()).unwrap();
        }

        // Query by in_stock = true
        let results = product_tree
            .get_by_secondary_key(ProductSecondaryKeys::InStock(ProductInStockSecondaryKey(true)))
            .unwrap();

        assert_eq!(2, results.len(), "Should find 2 in-stock products");
        assert!(results.contains(&products[0]));
        assert!(results.contains(&products[1]));
    }
}

// ============================================================================
// REDB STORE TESTS (Native only)
// ============================================================================

#[cfg(feature = "native")]
mod redb_tests {
    use super::*;
    use netabase_store::databases::redb_store::RedbStore;

    #[test]
    fn test_redb_create_store() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.redb");
        let store = RedbStore::<TestDefinition>::new(db_path.to_str().unwrap());
        assert!(store.is_ok(), "Failed to create RedbStore");
    }

    #[test]
    fn test_redb_crud_operations() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_crud.redb");
        let store = RedbStore::<TestDefinition>::new(db_path.to_str().unwrap()).unwrap();
        let user_tree = store.open_tree::<User>();

        // CREATE
        let alice = User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 30,
        };
        assert!(user_tree.insert(alice.clone()).is_ok(), "Failed to insert user");

        // READ
        let retrieved = user_tree.get(UserKey::Primary(UserPrimaryKey(1))).unwrap();
        assert_eq!(Some(alice.clone()), retrieved, "Failed to retrieve user");

        // UPDATE
        let updated_alice = User {
            id: 1,
            username: "alice_updated".to_string(),
            email: "alice_new@example.com".to_string(),
            age: 31,
        };
        assert!(user_tree.insert(updated_alice.clone()).is_ok(), "Failed to update user");

        let retrieved = user_tree.get(UserKey::Primary(UserPrimaryKey(1))).unwrap();
        assert_eq!(Some(updated_alice), retrieved, "Updated user doesn't match");

        // DELETE
        let removed = user_tree.remove(UserKey::Primary(UserPrimaryKey(1))).unwrap();
        assert!(removed.is_some(), "Failed to remove user");

        let retrieved = user_tree.get(UserKey::Primary(UserPrimaryKey(1))).unwrap();
        assert_eq!(None, retrieved, "User should be deleted");
    }

    #[test]
    fn test_redb_secondary_key_single_result() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_sec_single.redb");
        let store = RedbStore::<TestDefinition>::new(db_path.to_str().unwrap()).unwrap();
        let user_tree = store.open_tree::<User>();

        let alice = User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 30,
        };
        let bob = User {
            id: 2,
            username: "bob".to_string(),
            email: "bob@example.com".to_string(),
            age: 25,
        };

        user_tree.insert(alice.clone()).unwrap();
        user_tree.insert(bob.clone()).unwrap();

        // Query by email secondary key
        let results = user_tree
            .get_by_secondary_key(UserSecondaryKeys::Email(UserEmailSecondaryKey(
                "alice@example.com".to_string(),
            )))
            .unwrap();

        assert_eq!(1, results.len(), "Should find exactly one user");
        assert_eq!(alice, results[0], "Should find Alice");
    }

    #[test]
    fn test_redb_secondary_key_multiple_results() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_sec_multi.redb");
        let store = RedbStore::<TestDefinition>::new(db_path.to_str().unwrap()).unwrap();
        let user_tree = store.open_tree::<User>();

        let users = vec![
            User {
                id: 1,
                username: "alice".to_string(),
                email: "alice@example.com".to_string(),
                age: 30,
            },
            User {
                id: 2,
                username: "bob".to_string(),
                email: "bob@example.com".to_string(),
                age: 30,
            },
            User {
                id: 3,
                username: "carol".to_string(),
                email: "carol@example.com".to_string(),
                age: 25,
            },
        ];

        for user in &users {
            user_tree.insert(user.clone()).unwrap();
        }

        // Query by age secondary key (should find 2 users with age 30)
        let results = user_tree
            .get_by_secondary_key(UserSecondaryKeys::Age(UserAgeSecondaryKey(30)))
            .unwrap();

        assert_eq!(2, results.len(), "Should find 2 users with age 30");
        assert!(results.contains(&users[0]), "Should include Alice");
        assert!(results.contains(&users[1]), "Should include Bob");
    }

    #[test]
    fn test_redb_multiple_models() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_multi_models.redb");
        let store = RedbStore::<TestDefinition>::new(db_path.to_str().unwrap()).unwrap();

        // Test User model
        let user_tree = store.open_tree::<User>();
        let alice = User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 30,
        };
        user_tree.insert(alice.clone()).unwrap();

        // Test Product model
        let product_tree = store.open_tree::<Product>();
        let laptop = Product {
            id: "LAPTOP-001".to_string(),
            name: "ThinkPad X1".to_string(),
            price: 1299,
            category: "Electronics".to_string(),
            in_stock: true,
        };
        product_tree.insert(laptop.clone()).unwrap();

        // Verify both models are stored correctly
        assert_eq!(Some(alice), user_tree.get(UserKey::Primary(UserPrimaryKey(1))).unwrap());
        assert_eq!(
            Some(laptop),
            product_tree.get(ProductKey::Primary(ProductPrimaryKey("LAPTOP-001".to_string()))).unwrap()
        );
    }

    #[test]
    fn test_redb_iteration() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_iter.redb");
        let store = RedbStore::<TestDefinition>::new(db_path.to_str().unwrap()).unwrap();
        let user_tree = store.open_tree::<User>();

        let users = vec![
            User {
                id: 1,
                username: "alice".to_string(),
                email: "alice@example.com".to_string(),
                age: 30,
            },
            User {
                id: 2,
                username: "bob".to_string(),
                email: "bob@example.com".to_string(),
                age: 25,
            },
            User {
                id: 3,
                username: "carol".to_string(),
                email: "carol@example.com".to_string(),
                age: 35,
            },
        ];

        for user in &users {
            user_tree.insert(user.clone()).unwrap();
        }

        let results = user_tree.iter().unwrap();
        let mut retrieved = Vec::new();
        for (_, user) in results {
            retrieved.push(user);
        }

        assert_eq!(3, retrieved.len(), "Should retrieve all 3 users");
        for user in &users {
            assert!(retrieved.contains(user), "Should contain user {:?}", user);
        }
    }

    #[test]
    fn test_redb_clear_and_len() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_clear.redb");
        let store = RedbStore::<TestDefinition>::new(db_path.to_str().unwrap()).unwrap();
        let user_tree = store.open_tree::<User>();

        assert!(user_tree.is_empty().unwrap(), "Tree should be empty initially");
        assert_eq!(0, user_tree.len().unwrap(), "Length should be 0");

        let alice = User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 30,
        };
        user_tree.insert(alice).unwrap();

        assert!(!user_tree.is_empty().unwrap(), "Tree should not be empty");
        assert_eq!(1, user_tree.len().unwrap(), "Length should be 1");

        user_tree.clear().unwrap();
        assert!(user_tree.is_empty().unwrap(), "Tree should be empty after clear");
    }

    #[test]
    fn test_redb_string_primary_key() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_string_pk.redb");
        let store = RedbStore::<TestDefinition>::new(db_path.to_str().unwrap()).unwrap();
        let product_tree = store.open_tree::<Product>();

        let laptop = Product {
            id: "LAPTOP-001".to_string(),
            name: "ThinkPad X1".to_string(),
            price: 1299,
            category: "Electronics".to_string(),
            in_stock: true,
        };

        product_tree.insert(laptop.clone()).unwrap();

        let retrieved = product_tree
            .get(ProductKey::Primary(ProductPrimaryKey("LAPTOP-001".to_string())))
            .unwrap();
        assert_eq!(Some(laptop), retrieved);
    }

    #[test]
    fn test_redb_secondary_key_with_bool() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_bool_sec.redb");
        let store = RedbStore::<TestDefinition>::new(db_path.to_str().unwrap()).unwrap();
        let product_tree = store.open_tree::<Product>();

        let products = vec![
            Product {
                id: "PROD-1".to_string(),
                name: "Product 1".to_string(),
                price: 100,
                category: "Electronics".to_string(),
                in_stock: true,
            },
            Product {
                id: "PROD-2".to_string(),
                name: "Product 2".to_string(),
                price: 200,
                category: "Electronics".to_string(),
                in_stock: true,
            },
            Product {
                id: "PROD-3".to_string(),
                name: "Product 3".to_string(),
                price: 300,
                category: "Books".to_string(),
                in_stock: false,
            },
        ];

        for product in &products {
            product_tree.insert(product.clone()).unwrap();
        }

        // Query by in_stock = true
        let results = product_tree
            .get_by_secondary_key(ProductSecondaryKeys::InStock(ProductInStockSecondaryKey(true)))
            .unwrap();

        assert_eq!(2, results.len(), "Should find 2 in-stock products");
        assert!(results.contains(&products[0]));
        assert!(results.contains(&products[1]));
    }
}

// ============================================================================
// INDEXEDDB STORE TESTS (WASM only)
// ============================================================================

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
mod indexeddb_tests {
    use super::*;
    use netabase_store::databases::indexeddb_store::IndexedDBStore;
    use netabase_store::model::NetabaseModelTrait;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn test_indexeddb_create_store() {
        let store = IndexedDBStore::<TestDefinition>::new("test_db").await;
        assert!(store.is_ok(), "Failed to create IndexedDBStore");
    }

    #[wasm_bindgen_test]
    async fn test_indexeddb_crud_operations() {
        let store = IndexedDBStore::<TestDefinition>::new("test_crud_db").await.unwrap();
        let user_tree = store.open_tree::<User>();

        // CREATE
        let alice = User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 30,
        };
        assert!(user_tree.put(alice.clone()).await.is_ok(), "Failed to insert user");

        // READ
        let retrieved = user_tree.get(UserPrimaryKey(1)).await.unwrap();
        assert_eq!(Some(alice.clone()), retrieved, "Failed to retrieve user");

        // UPDATE
        let updated_alice = User {
            id: 1,
            username: "alice_updated".to_string(),
            email: "alice_new@example.com".to_string(),
            age: 31,
        };
        assert!(user_tree.put(updated_alice.clone()).await.is_ok(), "Failed to update user");

        let retrieved = user_tree.get(UserPrimaryKey(1)).await.unwrap();
        assert_eq!(Some(updated_alice), retrieved, "Updated user doesn't match");

        // DELETE
        let removed = user_tree.remove(UserPrimaryKey(1)).await.unwrap();
        assert!(removed.is_some(), "Failed to remove user");

        let retrieved = user_tree.get(UserPrimaryKey(1)).await.unwrap();
        assert_eq!(None, retrieved, "User should be deleted");
    }

    #[wasm_bindgen_test]
    async fn test_indexeddb_secondary_key_single_result() {
        let store = IndexedDBStore::<TestDefinition>::new("test_sec_single_db").await.unwrap();
        let user_tree = store.open_tree::<User>();

        let alice = User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 30,
        };
        let bob = User {
            id: 2,
            username: "bob".to_string(),
            email: "bob@example.com".to_string(),
            age: 25,
        };

        user_tree.put(alice.clone()).await.unwrap();
        user_tree.put(bob.clone()).await.unwrap();

        // Query by email secondary key
        let results = user_tree
            .get_by_secondary_key(UserSecondaryKeys::Email(UserEmailSecondaryKey(
                "alice@example.com".to_string(),
            )))
            .await
            .unwrap();

        assert_eq!(1, results.len(), "Should find exactly one user");
        assert_eq!(alice, results[0], "Should find Alice");
    }

    #[wasm_bindgen_test]
    async fn test_indexeddb_secondary_key_multiple_results() {
        let store = IndexedDBStore::<TestDefinition>::new("test_sec_multi_db").await.unwrap();
        let user_tree = store.open_tree::<User>();

        let users = vec![
            User {
                id: 1,
                username: "alice".to_string(),
                email: "alice@example.com".to_string(),
                age: 30,
            },
            User {
                id: 2,
                username: "bob".to_string(),
                email: "bob@example.com".to_string(),
                age: 30,
            },
            User {
                id: 3,
                username: "carol".to_string(),
                email: "carol@example.com".to_string(),
                age: 25,
            },
        ];

        for user in &users {
            user_tree.put(user.clone()).await.unwrap();
        }

        // Query by age secondary key (should find 2 users with age 30)
        let results = user_tree
            .get_by_secondary_key(UserSecondaryKeys::Age(UserAgeSecondaryKey(30)))
            .await
            .unwrap();

        assert_eq!(2, results.len(), "Should find 2 users with age 30");
        assert!(results.contains(&users[0]), "Should include Alice");
        assert!(results.contains(&users[1]), "Should include Bob");
    }

    #[wasm_bindgen_test]
    async fn test_indexeddb_multiple_models() {
        let store = IndexedDBStore::<TestDefinition>::new("test_multi_models_db").await.unwrap();

        // Test User model
        let user_tree = store.open_tree::<User>();
        let alice = User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 30,
        };
        user_tree.put(alice.clone()).await.unwrap();

        // Test Product model
        let product_tree = store.open_tree::<Product>();
        let laptop = Product {
            id: "LAPTOP-001".to_string(),
            name: "ThinkPad X1".to_string(),
            price: 1299,
            category: "Electronics".to_string(),
            in_stock: true,
        };
        product_tree.put(laptop.clone()).await.unwrap();

        // Verify both models are stored correctly
        assert_eq!(Some(alice), user_tree.get(UserPrimaryKey(1)).await.unwrap());
        assert_eq!(
            Some(laptop),
            product_tree.get(ProductPrimaryKey("LAPTOP-001".to_string())).await.unwrap()
        );
    }

    #[wasm_bindgen_test]
    async fn test_indexeddb_iteration() {
        let store = IndexedDBStore::<TestDefinition>::new("test_iter_db").await.unwrap();
        let user_tree = store.open_tree::<User>();

        let users = vec![
            User {
                id: 1,
                username: "alice".to_string(),
                email: "alice@example.com".to_string(),
                age: 30,
            },
            User {
                id: 2,
                username: "bob".to_string(),
                email: "bob@example.com".to_string(),
                age: 25,
            },
            User {
                id: 3,
                username: "carol".to_string(),
                email: "carol@example.com".to_string(),
                age: 35,
            },
        ];

        for user in &users {
            user_tree.put(user.clone()).await.unwrap();
        }

        let mut retrieved = Vec::new();
        for (_, user) in user_tree.iter().await.unwrap() {
            retrieved.push(user);
        }

        assert_eq!(3, retrieved.len(), "Should retrieve all 3 users");
        for user in &users {
            assert!(retrieved.contains(user), "Should contain user {:?}", user);
        }
    }

    #[wasm_bindgen_test]
    async fn test_indexeddb_clear_and_len() {
        let store = IndexedDBStore::<TestDefinition>::new("test_clear_db").await.unwrap();
        let user_tree = store.open_tree::<User>();

        assert!(user_tree.is_empty().await.unwrap(), "Tree should be empty initially");
        assert_eq!(0, user_tree.len().await.unwrap(), "Length should be 0");

        let alice = User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            age: 30,
        };
        user_tree.put(alice).await.unwrap();

        assert!(!user_tree.is_empty().await.unwrap(), "Tree should not be empty");
        assert_eq!(1, user_tree.len().await.unwrap(), "Length should be 1");

        user_tree.clear().await.unwrap();
        assert!(user_tree.is_empty().await.unwrap(), "Tree should be empty after clear");
    }

    #[wasm_bindgen_test]
    async fn test_indexeddb_string_primary_key() {
        let store = IndexedDBStore::<TestDefinition>::new("test_string_pk_db").await.unwrap();
        let product_tree = store.open_tree::<Product>();

        let laptop = Product {
            id: "LAPTOP-001".to_string(),
            name: "ThinkPad X1".to_string(),
            price: 1299,
            category: "Electronics".to_string(),
            in_stock: true,
        };

        product_tree.put(laptop.clone()).await.unwrap();

        let retrieved = product_tree
            .get(ProductPrimaryKey("LAPTOP-001".to_string()))
            .await
            .unwrap();
        assert_eq!(Some(laptop), retrieved);
    }

    #[wasm_bindgen_test]
    async fn test_indexeddb_secondary_key_with_bool() {
        let store = IndexedDBStore::<TestDefinition>::new("test_bool_sec_db").await.unwrap();
        let product_tree = store.open_tree::<Product>();

        let products = vec![
            Product {
                id: "PROD-1".to_string(),
                name: "Product 1".to_string(),
                price: 100,
                category: "Electronics".to_string(),
                in_stock: true,
            },
            Product {
                id: "PROD-2".to_string(),
                name: "Product 2".to_string(),
                price: 200,
                category: "Electronics".to_string(),
                in_stock: true,
            },
            Product {
                id: "PROD-3".to_string(),
                name: "Product 3".to_string(),
                price: 300,
                category: "Books".to_string(),
                in_stock: false,
            },
        ];

        for product in &products {
            product_tree.put(product.clone()).await.unwrap();
        }

        // Query by in_stock = true
        let results = product_tree
            .get_by_secondary_key(ProductSecondaryKeys::InStock(ProductInStockSecondaryKey(true)))
            .await
            .unwrap();

        assert_eq!(2, results.len(), "Should find 2 in-stock products");
        assert!(results.contains(&products[0]));
        assert!(results.contains(&products[1]));
    }
}
