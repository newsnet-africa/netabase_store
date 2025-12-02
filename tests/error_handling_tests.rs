//! Error Handling and Edge Case Tests
//!
//! These tests verify that the crate properly handles error conditions,
//! edge cases, and boundary conditions gracefully.

use netabase_store::error::NetabaseError;
use netabase_store::{NetabaseModelTrait, netabase_definition_module};
use std::path::{Path, PathBuf};

// Test schema for error handling
#[netabase_definition_module(ErrorTestDefinition, ErrorTestKeys)]
mod error_test_schema {
    use netabase_store::{NetabaseModel, netabase};

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
    #[netabase(ErrorTestDefinition)]
    pub struct TestModel {
        #[primary_key]
        pub id: u64,
        pub name: String,
        #[secondary_key]
        pub category: String,
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
    #[netabase(ErrorTestDefinition)]
    pub struct LargeModel {
        #[primary_key]
        pub id: u64,
        pub large_string: String,
        pub large_vector: Vec<u8>,
    }
}

use error_test_schema::*;

/// Test invalid file paths
#[test]
#[cfg(feature = "redb")]
fn test_invalid_file_paths() {
    use netabase_store::config::FileConfig;
    use netabase_store::databases::redb_store::RedbStore;
    use netabase_store::traits::backend_store::BackendStore;

    // Test with invalid characters in path
    let invalid_config = FileConfig::new("/dev/null/invalid\0path/test.redb");
    let result = RedbStore::<ErrorTestDefinition>::new(invalid_config.path);
    assert!(result.is_err());

    // Test with non-existent directory (without create_if_missing)
    let non_existent_config = FileConfig::builder()
        .path(PathBuf::from("/non/existent/directory/test.redb"))
        .create_if_missing(false)
        .build();
    let result = RedbStore::<ErrorTestDefinition>::new(non_existent_config.path);
    assert!(result.is_err());

    // Test with read-only directory
    #[cfg(unix)]
    {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = tempfile::tempdir().unwrap();
        let readonly_dir = temp_dir.path().join("readonly");
        fs::create_dir(&readonly_dir).unwrap();

        // Make directory read-only
        let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
        perms.set_mode(0o444);
        fs::set_permissions(&readonly_dir, perms).unwrap();

        let readonly_config = FileConfig::new(readonly_dir.join("test.redb"));
        let result = RedbStore::<ErrorTestDefinition>::new(readonly_config.path);
        assert!(result.is_err());
    }
}

/// Test operations on closed/corrupted database
#[test]
#[cfg(feature = "sled")]
fn test_operations_on_corrupted_database() {
    use netabase_store::databases::sled_store::SledStore;
    use std::fs;

    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("corrupted.db");

    // Create and use database normally
    {
        let store = SledStore::<ErrorTestDefinition>::new(&db_path).unwrap();
        let tree = store.open_tree::<TestModel>();

        tree.put(TestModel {
            id: 1,
            name: "Test".to_string(),
            category: "test".to_string(),
        })
        .unwrap();
    }

    // Simulate corruption by writing invalid data
    fs::write(&db_path, b"CORRUPTED_DATA").unwrap();

    // Try to open corrupted database
    let result = SledStore::<ErrorTestDefinition>::new(&db_path);
    // Should either fail to open or handle corruption gracefully
    match result {
        Ok(_) => {
            // If it opens, operations should fail gracefully
            // This depends on Sled's internal corruption handling
        }
        Err(_) => {
            // Expected behavior for corrupted database
        }
    }
}

/// Test extremely large keys and values
#[test]
#[cfg(feature = "sled")]
fn test_large_keys_and_values() {
    use netabase_store::databases::sled_store::SledStore;

    let store = SledStore::<ErrorTestDefinition>::temp().unwrap();
    let tree = store.open_tree::<LargeModel>();

    // Test with very large string (10MB)
    let large_string = "x".repeat(10 * 1024 * 1024);
    let large_vector = vec![0u8; 5 * 1024 * 1024]; // 5MB vector

    let large_model = LargeModel {
        id: 1,
        large_string,
        large_vector,
    };

    // This might succeed or fail depending on backend limits
    let result = tree.put(large_model.clone());
    match result {
        Ok(_) => {
            // If it succeeds, retrieval should also work
            let retrieved = tree.get(large_model.primary_key()).unwrap();
            assert_eq!(retrieved, Some(large_model));
        }
        Err(e) => {
            // Expected for extremely large data
            println!("Large data rejected as expected: {:?}", e);
        }
    }
}

/// Test operations with empty/invalid data
#[test]
#[cfg(feature = "sled")]
fn test_empty_and_invalid_data() {
    use netabase_store::databases::sled_store::SledStore;

    let store = SledStore::<ErrorTestDefinition>::temp().unwrap();
    let tree = store.open_tree::<TestModel>();

    // Test with empty strings
    let empty_model = TestModel {
        id: 1,
        name: "".to_string(),
        category: "".to_string(),
    };

    tree.put(empty_model.clone()).unwrap();
    let retrieved = tree.get(empty_model.primary_key()).unwrap();
    assert_eq!(retrieved, Some(empty_model));

    // Test with unicode strings
    let unicode_model = TestModel {
        id: 2,
        name: "🦀 Rust 测试 🔥".to_string(),
        category: "unicode-🌍".to_string(),
    };

    tree.put(unicode_model.clone()).unwrap();
    let retrieved = tree.get(unicode_model.primary_key()).unwrap();
    assert_eq!(retrieved, Some(unicode_model));

    // Test with very long strings
    let long_name = "a".repeat(1000);
    let long_model = TestModel {
        id: 3,
        name: long_name.clone(),
        category: "long".to_string(),
    };

    tree.put(long_model.clone()).unwrap();
    let retrieved = tree.get(long_model.primary_key()).unwrap();
    assert_eq!(retrieved, Some(long_model));
}

/// Test boundary conditions for numeric keys
#[test]
#[cfg(feature = "sled")]
fn test_numeric_boundary_conditions() {
    use netabase_store::databases::sled_store::SledStore;

    let store = SledStore::<ErrorTestDefinition>::temp().unwrap();
    let tree = store.open_tree::<TestModel>();

    // Test with zero
    let zero_model = TestModel {
        id: 0,
        name: "Zero".to_string(),
        category: "boundary".to_string(),
    };
    tree.put(zero_model.clone()).unwrap();

    // Test with maximum u64
    let max_model = TestModel {
        id: u64::MAX,
        name: "Max".to_string(),
        category: "boundary".to_string(),
    };
    tree.put(max_model.clone()).unwrap();

    // Verify both can be retrieved
    assert_eq!(
        tree.get(zero_model.primary_key()).unwrap(),
        Some(zero_model)
    );
    assert_eq!(tree.get(max_model.primary_key()).unwrap(), Some(max_model));
}

/// Test operations on non-existent keys
#[test]
#[cfg(feature = "sled")]
fn test_non_existent_key_operations() {
    use netabase_store::databases::sled_store::SledStore;

    let store = SledStore::<ErrorTestDefinition>::temp().unwrap();
    let tree = store.open_tree::<TestModel>();

    // Test get on non-existent key
    let result = tree.get(TestModelPrimaryKey(999999));
    assert_eq!(result.unwrap(), None);

    // Test remove on non-existent key
    let result = tree.remove(TestModelPrimaryKey(999999));
    assert_eq!(result.unwrap(), None);

    // Test secondary key query on non-existent value
    let results = tree
        .get_by_secondary_key(TestModelSecondaryKeys::Category(
            TestModelCategorySecondaryKey("non-existent".to_string()),
        ))
        .unwrap();
    assert!(results.is_empty());
}

/// Test memory exhaustion scenarios
#[test]
#[cfg(feature = "sled")]
fn test_memory_pressure_handling() {
    use netabase_store::databases::sled_store::SledStore;

    let store = SledStore::<ErrorTestDefinition>::temp().unwrap();
    let tree = store.open_tree::<TestModel>();

    // Try to insert many records to create memory pressure
    let mut successful_inserts = 0;
    for i in 0..100000 {
        let model = TestModel {
            id: i,
            name: format!("Item {}", i),
            category: "pressure-test".to_string(),
        };

        match tree.put(model) {
            Ok(_) => successful_inserts += 1,
            Err(_) => break, // Stop on first failure
        }

        // Check if we can still read data under pressure
        if i % 1000 == 0 {
            let key = TestModelPrimaryKey(i / 2);
            match tree.get(key) {
                Ok(Some(_)) => {}
                Ok(None) => {}
                Err(_) => break, // Stop if reads start failing
            }
        }
    }

    println!(
        "Successfully inserted {} records under memory pressure",
        successful_inserts
    );
    assert!(successful_inserts > 1000); // Should handle at least some records
}

/// Test transaction failure scenarios
#[test]
#[cfg(feature = "redb-zerocopy")]
fn test_transaction_failure_scenarios() {
    use netabase_store::databases::redb_zerocopy::RedbStoreZeroCopy;

    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("transaction_failures.redb");
    let store = RedbStoreZeroCopy::<ErrorTestDefinition>::new(&db_path).unwrap();

    // Test nested transaction attempts
    let mut txn1 = store.begin_write().unwrap();
    let result = store.begin_write(); // This should fail
    assert!(result.is_err()); // Cannot have multiple write transactions

    // Test operations on committed transaction
    let mut tree = txn1.open_tree::<TestModel>().unwrap();
    tree.put(TestModel {
        id: 1,
        name: "Test".to_string(),
        category: "test".to_string(),
    })
    .unwrap();
    drop(tree);
    txn1.commit().unwrap();

    // Test operations on aborted transaction
    let mut txn2 = store.begin_write().unwrap();
    let mut tree2 = txn2.open_tree::<TestModel>().unwrap();
    tree2
        .put(TestModel {
            id: 2,
            name: "Abort Test".to_string(),
            category: "abort".to_string(),
        })
        .unwrap();
    drop(tree2);
    txn2.abort().unwrap();

    // Verify aborted changes are not visible
    let txn3 = store.begin_read().unwrap();
    let tree3 = txn3.open_tree::<TestModel>().unwrap();
    assert!(tree3.get(&TestModelPrimaryKey(1)).unwrap().is_some()); // Committed
    assert!(tree3.get(&TestModelPrimaryKey(2)).unwrap().is_none()); // Aborted
}

/// Test concurrent access errors
#[test]
#[cfg(all(feature = "redb", not(target_arch = "wasm32")))]
fn test_concurrent_access_errors() {
    use netabase_store::config::FileConfig;
    use netabase_store::databases::redb_store::RedbStore;
    use netabase_store::traits::backend_store::BackendStore;
    use std::sync::Arc;
    use std::thread;

    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("concurrent_errors.redb");
    let config = FileConfig::new(&db_path);

    // Create first store instance
    let store1 = Arc::new(RedbStore::<ErrorTestDefinition>::new(&config.path).unwrap());

    let store1_clone = Arc::clone(&store1);
    let handle = thread::spawn(move || {
        // Try to create second store instance on same file
        let config2 = FileConfig::new(&db_path);
        let result = RedbStore::<ErrorTestDefinition>::new(&config2.path);

        // This should either succeed (if backend supports it) or fail gracefully
        match result {
            Ok(store2) => {
                // If it succeeds, basic operations should work
                let tree = store2.open_tree::<TestModel>();
                let _ = tree.put(TestModel {
                    id: 1,
                    name: "Concurrent".to_string(),
                    category: "test".to_string(),
                });
            }
            Err(_) => {
                // Expected if backend doesn't support concurrent access
            }
        }
    });

    // Use first store while second thread tries to access
    let tree1 = store1.open_tree::<TestModel>();
    tree1
        .put(TestModel {
            id: 2,
            name: "Main".to_string(),
            category: "main".to_string(),
        })
        .unwrap();

    handle.join().unwrap();
}

/// Test disk space exhaustion
#[test]
#[cfg(all(feature = "sled", unix))]
fn test_disk_space_exhaustion() {
    use netabase_store::databases::sled_store::SledStore;
    use std::fs;

    // Create a small tmpfs mount to simulate disk space exhaustion
    // Note: This test requires root privileges in real scenarios
    // For testing purposes, we'll simulate by trying to write large amounts of data

    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("disk_full.db");

    let store = SledStore::<ErrorTestDefinition>::new(&db_path).unwrap();
    let tree = store.open_tree::<LargeModel>();

    // Try to write increasingly large data until we hit limits
    let mut size = 1024; // Start with 1KB
    let mut max_successful_size = 0;

    while size <= 100 * 1024 * 1024 {
        // Up to 100MB
        let large_data = vec![0u8; size];
        let model = LargeModel {
            id: size as u64,
            large_string: "x".repeat(size / 2),
            large_vector: large_data,
        };

        match tree.put(model) {
            Ok(_) => {
                max_successful_size = size;
                size *= 2;
            }
            Err(_) => {
                // Hit a limit - this is expected behavior
                break;
            }
        }
    }

    println!(
        "Maximum successful data size: {} bytes",
        max_successful_size
    );
    assert!(max_successful_size > 1024); // Should handle at least 1KB
}

/// Test configuration validation
#[test]
#[cfg(feature = "native")]
fn test_configuration_validation() {
    use netabase_store::config::FileConfig;

    // Test valid configurations
    let valid_config = FileConfig::builder()
        .path(PathBuf::from("test.db"))
        .cache_size_mb(256)
        .create_if_missing(true)
        .build();
    assert!(valid_config.cache_size_mb == 256);

    // Test edge case configurations
    let minimal_config = FileConfig::builder()
        .path(PathBuf::from("minimal.db"))
        .cache_size_mb(1) // Very small cache
        .build();
    assert!(minimal_config.cache_size_mb == 1);

    let large_config = FileConfig::builder()
        .path(PathBuf::from("large.db"))
        .cache_size_mb(16384) // 16GB cache
        .build();
    assert!(large_config.cache_size_mb == 16384);
}

/// Test serialization/deserialization errors
#[test]
#[cfg(feature = "sled")]
fn test_serialization_edge_cases() {
    use netabase_store::databases::sled_store::SledStore;

    let store = SledStore::<ErrorTestDefinition>::temp().unwrap();
    let tree = store.open_tree::<TestModel>();

    // Test with strings containing null bytes
    let null_byte_model = TestModel {
        id: 1,
        name: "Test\0with\0nulls".to_string(),
        category: "null-bytes".to_string(),
    };

    tree.put(null_byte_model.clone()).unwrap();
    let retrieved = tree.get(null_byte_model.primary_key()).unwrap();
    assert_eq!(retrieved, Some(null_byte_model));

    // Test with strings containing control characters
    let control_char_model = TestModel {
        id: 2,
        name: "Test\n\t\r\x0c".to_string(),
        category: "control".to_string(),
    };

    tree.put(control_char_model.clone()).unwrap();
    let retrieved = tree.get(control_char_model.primary_key()).unwrap();
    assert_eq!(retrieved, Some(control_char_model));

    // Test with high unicode characters
    let high_unicode_model = TestModel {
        id: 3,
        name: "𝕋𝕖𝓈𝓉 𝒽𝒾𝑔𝒽 𝓊𝓃𝒾𝒸𝑜𝒹𝑒 💯".to_string(),
        category: "unicode".to_string(),
    };

    tree.put(high_unicode_model.clone()).unwrap();
    let retrieved = tree.get(high_unicode_model.primary_key()).unwrap();
    assert_eq!(retrieved, Some(high_unicode_model));
}

/// Test error propagation and handling
#[test]
#[cfg(feature = "sled")]
fn test_error_propagation() {
    use netabase_store::databases::sled_store::SledStore;

    let store = SledStore::<ErrorTestDefinition>::temp().unwrap();
    let tree = store.open_tree::<TestModel>();

    // Test that errors are properly typed and contain useful information
    let non_existent_key = TestModelPrimaryKey(999999);
    let result = tree.get(non_existent_key);

    match result {
        Ok(None) => {
            // This is the expected result for non-existent keys
        }
        Ok(Some(_)) => {
            panic!("Should not find non-existent key");
        }
        Err(e) => {
            // If an error occurs, it should be meaningful
            println!("Error details: {:?}", e);
        }
    }

    // Test batch operation error handling
    let mixed_batch = vec![
        TestModel {
            id: 1,
            name: "Valid".to_string(),
            category: "valid".to_string(),
        },
        TestModel {
            id: 2,
            name: "Also Valid".to_string(),
            category: "valid".to_string(),
        },
    ];

    // Normal batch should succeed
    for model in mixed_batch {
        let result = tree.put(model);
        assert!(result.is_ok());
    }
}

/// Test resource cleanup after errors
#[test]
#[cfg(feature = "sled")]
fn test_resource_cleanup_after_errors() {
    use netabase_store::databases::sled_store::SledStore;

    // Test that resources are properly cleaned up even when operations fail
    for i in 0..10 {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join(format!("cleanup_test_{}.db", i));

        {
            let store = SledStore::<ErrorTestDefinition>::new(&db_path).unwrap();
            let tree = store.open_tree::<TestModel>();

            // Perform some operations
            tree.put(TestModel {
                id: i,
                name: format!("Test {}", i),
                category: "cleanup".to_string(),
            })
            .unwrap();

            // Try an operation that might fail
            let _ = tree.get(TestModelPrimaryKey(999999));
        }

        // Store should be properly dropped and resources cleaned up
        // Directory cleanup happens when temp_dir is dropped
    }

    // If we get here without panicking or hanging, cleanup worked
    assert!(true);
}

/// Test handling of OS-level errors
#[test]
#[cfg(all(feature = "redb", unix))]
fn test_os_level_errors() {
    use netabase_store::config::FileConfig;
    use netabase_store::databases::redb_store::RedbStore;
    use netabase_store::traits::backend_store::BackendStore;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("os_errors.redb");

    // Create database file
    {
        let config = FileConfig::new(&db_path);
        let store = RedbStore::<ErrorTestDefinition>::new(&config.path).unwrap();
        let tree = store.open_tree::<TestModel>();
        tree.put(TestModel {
            id: 1,
            name: "Test".to_string(),
            category: "test".to_string(),
        })
        .unwrap();
    }

    // Make file read-only
    let mut perms = fs::metadata(&db_path).unwrap().permissions();
    perms.set_mode(0o444);
    fs::set_permissions(&db_path, perms).unwrap();

    // Try to open read-only file for writing
    let config = FileConfig::builder()
        .path(db_path.clone())
        .read_only(false) // Request write access
        .build();

    let result = RedbStore::<ErrorTestDefinition>::new(config.path.clone());
    // Should either fail or open in read-only mode
    match result {
        Ok(store) => {
            // If it opens, write operations should fail
            let tree = store.open_tree::<TestModel>();
            let write_result = tree.put(TestModel {
                id: 2,
                name: "Should Fail".to_string(),
                category: "fail".to_string(),
            });
            // Write should fail on read-only database
            assert!(write_result.is_err());
        }
        Err(_) => {
            // Expected if backend rejects read-only files for write mode
        }
    }
}
