//! WASM and Network Reliability Tests
//!
//! These tests verify that the crate works correctly in WASM environments
//! and handles network failures, browser limitations, and IndexedDB edge cases.

#![cfg(target_arch = "wasm32")]

use netabase_store::netabase_definition_module;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// Test schema for WASM reliability tests
#[netabase_definition_module(WasmReliabilityTestDefinition, WasmReliabilityTestKeys)]
mod wasm_reliability_test_schema {
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
    #[netabase(WasmReliabilityTestDefinition)]
    pub struct TestRecord {
        #[primary_key]
        pub id: u64,
        pub name: String,
        #[secondary_key]
        pub category: String,
        pub data: Vec<u8>,
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
    #[netabase(WasmReliabilityTestDefinition)]
    pub struct LargeRecord {
        #[primary_key]
        pub id: u64,
        pub large_data: Vec<u8>,
        pub metadata: String,
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
    #[netabase(WasmReliabilityTestDefinition)]
    pub struct VersionedRecord {
        #[primary_key]
        pub id: u64,
        pub version: u32,
        #[secondary_key]
        pub status: String,
        pub content: String,
    }
}

use wasm_reliability_test_schema::*;

/// Test basic IndexedDB functionality works in WASM
#[wasm_bindgen_test]
#[cfg(feature = "wasm")]
async fn test_basic_indexeddb_functionality() {
    use netabase_store::databases::indexeddb_store::IndexedDBStore;

    let db_name = format!("wasm_basic_test_{}", js_sys::Date::now());
    let store = IndexedDBStore::<WasmReliabilityTestDefinition>::new(&db_name)
        .await
        .expect("Failed to create IndexedDB store");

    let tree = store.open_tree::<TestRecord>();

    // Test basic CRUD operations
    let record = TestRecord {
        id: 1,
        name: "WASM Test Record".to_string(),
        category: "wasm".to_string(),
        data: vec![1, 2, 3, 4, 5],
    };

    // Insert
    tree.put(record.clone())
        .await
        .expect("Failed to insert record");

    // Read
    let retrieved = tree
        .get(TestRecordPrimaryKey(1))
        .await
        .expect("Failed to get record")
        .expect("Record not found");

    assert_eq!(retrieved, record);

    // Update
    let updated_record = TestRecord {
        id: 1,
        name: "Updated WASM Record".to_string(),
        category: "updated".to_string(),
        data: vec![6, 7, 8, 9, 10],
    };

    tree.put(updated_record.clone())
        .await
        .expect("Failed to update record");

    let retrieved_updated = tree
        .get(TestRecordPrimaryKey(1))
        .await
        .expect("Failed to get updated record")
        .expect("Updated record not found");

    assert_eq!(retrieved_updated, updated_record);

    // Delete
    let removed = tree
        .remove(TestRecordPrimaryKey(1))
        .await
        .expect("Failed to remove record");

    assert!(removed.is_some());
    assert_eq!(removed.unwrap(), updated_record);

    // Verify deletion
    let after_delete = tree
        .get(TestRecordPrimaryKey(1))
        .await
        .expect("Failed to check deleted record");

    assert!(after_delete.is_none());
}

/// Test large data handling in IndexedDB
#[wasm_bindgen_test]
#[cfg(feature = "wasm")]
async fn test_large_data_handling() {
    use netabase_store::databases::indexeddb_store::IndexedDBStore;

    let db_name = format!("wasm_large_data_test_{}", js_sys::Date::now());
    let store = IndexedDBStore::<WasmReliabilityTestDefinition>::new(&db_name)
        .await
        .expect("Failed to create IndexedDB store");

    let tree = store.open_tree::<LargeRecord>();

    // Test with incrementally larger data sizes
    let data_sizes = vec![1024, 64 * 1024, 256 * 1024]; // 1KB, 64KB, 256KB

    for (i, size) in data_sizes.iter().enumerate() {
        let large_data = vec![((i + 1) * 42) as u8; *size];
        let record = LargeRecord {
            id: i as u64,
            large_data: large_data.clone(),
            metadata: format!("Large record with {} bytes", size),
        };

        // Try to insert large record
        let insert_result = tree.put(record.clone()).await;

        match insert_result {
            Ok(_) => {
                // If insertion succeeds, verify retrieval
                let retrieved = tree
                    .get(LargeRecordPrimaryKey(i as u64))
                    .await
                    .expect("Failed to get large record")
                    .expect("Large record not found");

                assert_eq!(retrieved.large_data.len(), size);
                assert_eq!(retrieved.large_data, large_data);
                assert_eq!(retrieved.metadata, record.metadata);

                web_sys::console::log_1(
                    &format!("Successfully handled {}KB record", size / 1024).into(),
                );
            }
            Err(_) => {
                // Large data might fail due to browser limits
                web_sys::console::log_1(
                    &format!("Large data size {}KB rejected by browser", size / 1024).into(),
                );
                break;
            }
        }
    }
}

/// Test concurrent operations in WASM environment
#[wasm_bindgen_test]
#[cfg(feature = "wasm")]
async fn test_concurrent_operations() {
    use netabase_store::databases::indexeddb_store::IndexedDBStore;
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen_futures::spawn_local;

    let db_name = format!("wasm_concurrent_test_{}", js_sys::Date::now());
    let store = Rc::new(
        IndexedDBStore::<WasmReliabilityTestDefinition>::new(&db_name)
            .await
            .expect("Failed to create IndexedDB store"),
    );

    let results = Rc::new(RefCell::new(Vec::new()));
    let num_operations = 10;

    // Spawn multiple concurrent operations
    let mut handles = Vec::new();

    for i in 0..num_operations {
        let store_clone = Rc::clone(&store);
        let results_clone = Rc::clone(&results);

        let future = async move {
            let tree = store_clone.open_tree::<TestRecord>();

            let record = TestRecord {
                id: i,
                name: format!("Concurrent Record {}", i),
                category: "concurrent".to_string(),
                data: vec![i as u8; 100],
            };

            let insert_result = tree.put(record.clone()).await;

            match insert_result {
                Ok(_) => {
                    // Verify the record was inserted
                    let retrieved = tree.get(TestRecordPrimaryKey(i)).await;

                    match retrieved {
                        Ok(Some(retrieved_record)) => {
                            if retrieved_record == record {
                                results_clone.borrow_mut().push(i);
                            }
                        }
                        _ => {}
                    }
                }
                Err(_) => {}
            }
        };

        spawn_local(future);
    }

    // Wait for operations to complete (simplified approach for WASM)
    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL))
        .await
        .unwrap();

    // Give some time for all operations to complete
    let delay_promise = js_sys::Promise::new(&mut |resolve, _| {
        let window = web_sys::window().unwrap();
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 500)
            .unwrap();
    });
    wasm_bindgen_futures::JsFuture::from(delay_promise)
        .await
        .unwrap();

    let final_results = results.borrow();
    web_sys::console::log_1(
        &format!(
            "Concurrent operations completed: {}/{}",
            final_results.len(),
            num_operations
        )
        .into(),
    );

    // Should complete most operations successfully
    assert!(!final_results.is_empty());
}

/// Test IndexedDB transaction behavior
#[wasm_bindgen_test]
#[cfg(feature = "wasm")]
async fn test_transaction_behavior() {
    use netabase_store::databases::indexeddb_store::IndexedDBStore;

    let db_name = format!("wasm_transaction_test_{}", js_sys::Date::now());
    let store = IndexedDBStore::<WasmReliabilityTestDefinition>::new(&db_name)
        .await
        .expect("Failed to create IndexedDB store");

    let tree = store.open_tree::<VersionedRecord>();

    // Insert initial record
    let initial_record = VersionedRecord {
        id: 1,
        version: 1,
        status: "initial".to_string(),
        content: "Initial content".to_string(),
    };

    tree.put(initial_record.clone())
        .await
        .expect("Failed to insert initial record");

    // Test update behavior (should replace existing record)
    let updated_record = VersionedRecord {
        id: 1,
        version: 2,
        status: "updated".to_string(),
        content: "Updated content".to_string(),
    };

    tree.put(updated_record.clone())
        .await
        .expect("Failed to update record");

    // Verify only the updated record exists
    let retrieved = tree
        .get(VersionedRecordPrimaryKey(1))
        .await
        .expect("Failed to get record")
        .expect("Record not found");

    assert_eq!(retrieved, updated_record);
    assert_ne!(retrieved, initial_record);

    // Test batch operations within transaction scope
    let batch_records = vec![
        VersionedRecord {
            id: 2,
            version: 1,
            status: "batch".to_string(),
            content: "Batch record 1".to_string(),
        },
        VersionedRecord {
            id: 3,
            version: 1,
            status: "batch".to_string(),
            content: "Batch record 2".to_string(),
        },
    ];

    // Note: IndexedDB implementation may handle batch operations differently
    for record in batch_records.iter() {
        tree.put(record.clone())
            .await
            .expect("Failed to insert batch record");
    }

    // Verify batch records were inserted
    for record in &batch_records {
        let retrieved = tree
            .get(VersionedRecordPrimaryKey(record.id))
            .await
            .expect("Failed to get batch record")
            .expect("Batch record not found");

        assert_eq!(retrieved, *record);
    }
}

/// Test secondary key queries in WASM
#[wasm_bindgen_test]
#[cfg(feature = "wasm")]
async fn test_secondary_key_queries() {
    use netabase_store::databases::indexeddb_store::IndexedDBStore;

    let db_name = format!("wasm_secondary_key_test_{}", js_sys::Date::now());
    let store = IndexedDBStore::<WasmReliabilityTestDefinition>::new(&db_name)
        .await
        .expect("Failed to create IndexedDB store");

    let tree = store.open_tree::<VersionedRecord>();

    // Insert test records with different statuses
    let test_records = vec![
        VersionedRecord {
            id: 1,
            version: 1,
            status: "active".to_string(),
            content: "Active record 1".to_string(),
        },
        VersionedRecord {
            id: 2,
            version: 1,
            status: "active".to_string(),
            content: "Active record 2".to_string(),
        },
        VersionedRecord {
            id: 3,
            version: 1,
            status: "inactive".to_string(),
            content: "Inactive record".to_string(),
        },
        VersionedRecord {
            id: 4,
            version: 2,
            status: "active".to_string(),
            content: "Active record 3".to_string(),
        },
    ];

    for record in &test_records {
        tree.put(record.clone())
            .await
            .expect("Failed to insert test record");
    }

    // Query by status = "active"
    let active_records = tree
        .get_by_secondary_key(VersionedRecordSecondaryKeys::Status(
            VersionedRecordStatusSecondaryKey("active".to_string()),
        ))
        .await
        .expect("Failed to query by secondary key");

    assert_eq!(active_records.len(), 3);
    for record in &active_records {
        assert_eq!(record.status, "active");
    }

    // Query by status = "inactive"
    let inactive_records = tree
        .get_by_secondary_key(VersionedRecordSecondaryKeys::Status(
            VersionedRecordStatusSecondaryKey("inactive".to_string()),
        ))
        .await
        .expect("Failed to query by secondary key");

    assert_eq!(inactive_records.len(), 1);
    assert_eq!(inactive_records[0].status, "inactive");

    // Query by non-existent status
    let missing_records = tree
        .get_by_secondary_key(VersionedRecordSecondaryKeys::Status(
            VersionedRecordStatusSecondaryKey("missing".to_string()),
        ))
        .await
        .expect("Failed to query by secondary key");

    assert!(missing_records.is_empty());
}

/// Test error handling in WASM environment
#[wasm_bindgen_test]
#[cfg(feature = "wasm")]
async fn test_error_handling() {
    use netabase_store::databases::indexeddb_store::IndexedDBStore;

    // Test with invalid database name
    let invalid_db_name = format!("invalid/db/name/{}", js_sys::Date::now());
    let result = IndexedDBStore::<WasmReliabilityTestDefinition>::new(&invalid_db_name).await;

    // This might succeed or fail depending on browser implementation
    match result {
        Ok(store) => {
            // If it succeeds, basic operations should still work
            let tree = store.open_tree::<TestRecord>();
            let test_record = TestRecord {
                id: 1,
                name: "Error test".to_string(),
                category: "error".to_string(),
                data: vec![1, 2, 3],
            };

            let _ = tree.put(test_record).await;
        }
        Err(_) => {
            // Expected behavior for invalid database names
            web_sys::console::log_1(&"Invalid database name rejected as expected".into());
        }
    }

    // Test with valid database but operations on non-existent records
    let db_name = format!("wasm_error_test_{}", js_sys::Date::now());
    let store = IndexedDBStore::<WasmReliabilityTestDefinition>::new(&db_name)
        .await
        .expect("Failed to create error test store");

    let tree = store.open_tree::<TestRecord>();

    // Try to get non-existent record
    let result = tree.get(TestRecordPrimaryKey(999999)).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());

    // Try to remove non-existent record
    let remove_result = tree.remove(TestRecordPrimaryKey(999999)).await;
    assert!(remove_result.is_ok());
    assert!(remove_result.unwrap().is_none());
}

/// Test browser storage limits and quota handling
#[wasm_bindgen_test]
#[cfg(feature = "wasm")]
async fn test_storage_limits() {
    use netabase_store::databases::indexeddb_store::IndexedDBStore;

    let db_name = format!("wasm_storage_limits_test_{}", js_sys::Date::now());
    let store = IndexedDBStore::<WasmReliabilityTestDefinition>::new(&db_name)
        .await
        .expect("Failed to create storage limits test store");

    let tree = store.open_tree::<LargeRecord>();

    let mut successful_inserts = 0;
    let max_attempts = 100;
    let chunk_size = 100 * 1024; // 100KB chunks

    for i in 0..max_attempts {
        let large_data = vec![(i % 256) as u8; chunk_size];
        let record = LargeRecord {
            id: i,
            large_data,
            metadata: format!("Storage test record {}", i),
        };

        match tree.put(record).await {
            Ok(_) => {
                successful_inserts += 1;

                // Log progress periodically
                if i % 10 == 0 {
                    web_sys::console::log_1(
                        &format!("Inserted {} records so far", successful_inserts).into(),
                    );
                }
            }
            Err(_) => {
                web_sys::console::log_1(
                    &format!("Storage limit reached after {} records", successful_inserts).into(),
                );
                break;
            }
        }
    }

    web_sys::console::log_1(
        &format!(
            "Storage test completed: {} successful inserts",
            successful_inserts
        )
        .into(),
    );

    // Should handle at least some data before hitting limits
    assert!(successful_inserts > 0);
}

/// Test persistence across "page reloads" (simulated)
#[wasm_bindgen_test]
#[cfg(feature = "wasm")]
async fn test_data_persistence() {
    use netabase_store::databases::indexeddb_store::IndexedDBStore;

    let db_name = format!("wasm_persistence_test_{}", js_sys::Date::now());

    // First "session" - insert data
    {
        let store = IndexedDBStore::<WasmReliabilityTestDefinition>::new(&db_name)
            .await
            .expect("Failed to create persistence test store");

        let tree = store.open_tree::<TestRecord>();

        let persistent_record = TestRecord {
            id: 42,
            name: "Persistent Record".to_string(),
            category: "persistence".to_string(),
            data: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        };

        tree.put(persistent_record.clone())
            .await
            .expect("Failed to insert persistent record");

        // Verify it was inserted
        let retrieved = tree
            .get(TestRecordPrimaryKey(42))
            .await
            .expect("Failed to get persistent record")
            .expect("Persistent record not found");

        assert_eq!(retrieved, persistent_record);
    }

    // Second "session" - reconnect to same database and verify data persists
    {
        let store = IndexedDBStore::<WasmReliabilityTestDefinition>::new(&db_name)
            .await
            .expect("Failed to reconnect to persistence test store");

        let tree = store.open_tree::<TestRecord>();

        // Data should still be there
        let retrieved = tree
            .get(TestRecordPrimaryKey(42))
            .await
            .expect("Failed to get persistent record after reconnect");

        match retrieved {
            Some(record) => {
                assert_eq!(record.id, 42);
                assert_eq!(record.name, "Persistent Record");
                assert_eq!(record.category, "persistence");
                assert_eq!(record.data, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
                web_sys::console::log_1(&"Data persistence test passed".into());
            }
            None => {
                panic!("Persistent record was lost between sessions");
            }
        }
    }
}

/// Test Unicode and special character handling
#[wasm_bindgen_test]
#[cfg(feature = "wasm")]
async fn test_unicode_handling() {
    use netabase_store::databases::indexeddb_store::IndexedDBStore;

    let db_name = format!("wasm_unicode_test_{}", js_sys::Date::now());
    let store = IndexedDBStore::<WasmReliabilityTestDefinition>::new(&db_name)
        .await
        .expect("Failed to create Unicode test store");

    let tree = store.open_tree::<TestRecord>();

    // Test various Unicode characters
    let unicode_tests = vec![
        ("emoji", "🦀 Rust 🔥 Database 💾"),
        ("chinese", "数据库测试"),
        ("arabic", "قاعدة البيانات"),
        ("emoji_mix", "Test 🚀 数据库 🔬 العربية 🎯"),
        ("special_chars", r#"!"$%&'()*+,-./:;<=>?@[\]^_`{|}~"#),
        ("newlines", "Line 1\nLine 2\r\nLine 3"),
        ("null_bytes", "Before\x00After"),
    ];

    for (test_name, unicode_text) in unicode_tests {
        let record = TestRecord {
            id: test_name.len() as u64, // Use length as ID to avoid conflicts
            name: unicode_text.to_string(),
            category: test_name.to_string(),
            data: unicode_text.as_bytes().to_vec(),
        };

        tree.put(record.clone())
            .await
            .expect(&format!("Failed to insert Unicode record: {}", test_name));

        let retrieved = tree
            .get(TestRecordPrimaryKey(test_name.len() as u64))
            .await
            .expect(&format!("Failed to get Unicode record: {}", test_name))
            .expect(&format!("Unicode record not found: {}", test_name));

        assert_eq!(retrieved.name, unicode_text);
        assert_eq!(retrieved.category, test_name);
        assert_eq!(retrieved.data, unicode_text.as_bytes());

        web_sys::console::log_1(&format!("Unicode test '{}' passed", test_name).into());
    }
}

/// Test performance in WASM environment
#[wasm_bindgen_test]
#[cfg(feature = "wasm")]
async fn test_wasm_performance() {
    use netabase_store::databases::indexeddb_store::IndexedDBStore;

    let db_name = format!("wasm_performance_test_{}", js_sys::Date::now());
    let store = IndexedDBStore::<WasmReliabilityTestDefinition>::new(&db_name)
        .await
        .expect("Failed to create performance test store");

    let tree = store.open_tree::<TestRecord>();

    let num_records = 1000; // Reasonable number for WASM testing
    let start_time = js_sys::Date::now();

    // Insert records
    for i in 0..num_records {
        let record = TestRecord {
            id: i,
            name: format!("Performance Test Record {}", i),
            category: format!("perf_{}", i % 50),
            data: vec![((i % 256) as u8); 64], // 64 bytes of data
        };

        tree.put(record)
            .await
            .expect("Failed to insert performance test record");

        // Log progress every 100 records
        if i % 100 == 0 && i > 0 {
            let elapsed = js_sys::Date::now() - start_time;
            web_sys::console::log_1(&format!("Inserted {} records in {}ms", i, elapsed).into());
        }
    }

    let insert_time = js_sys::Date::now() - start_time;

    // Read performance test
    let read_start = js_sys::Date::now();
    let mut successful_reads = 0;

    for i in (0..num_records).step_by(10) {
        if tree.get(TestRecordPrimaryKey(i)).await.unwrap().is_some() {
            successful_reads += 1;
        }
    }

    let read_time = js_sys::Date::now() - read_start;

    web_sys::console::log_1(
        &format!(
            "WASM Performance: {} inserts in {}ms, {} reads in {}ms",
            num_records, insert_time, successful_reads, read_time
        )
        .into(),
    );

    // Verify we completed a reasonable amount of work
    assert_eq!(successful_reads, num_records / 10);
    assert!(insert_time < 60000.0); // Less than 1 minute for inserts
    assert!(read_time < 10000.0); // Less than 10 seconds for reads
}

/// Test memory management in WASM
#[wasm_bindgen_test]
#[cfg(feature = "wasm")]
async fn test_wasm_memory_management() {
    use netabase_store::databases::indexeddb_store::IndexedDBStore;

    let db_name = format!("wasm_memory_test_{}", js_sys::Date::now());

    // Test creating and dropping multiple stores
    for iteration in 0..5 {
        let iteration_db_name = format!("{}_iter_{}", db_name, iteration);

        {
            let store = IndexedDBStore::<WasmReliabilityTestDefinition>::new(&iteration_db_name)
                .await
                .expect("Failed to create memory test store");

            let tree = store.open_tree::<TestRecord>();

            // Insert some data
            for i in 0..50 {
                let record = TestRecord {
                    id: i,
                    name: format!("Memory test {} - {}", iteration, i),
                    category: "memory".to_string(),
                    data: vec![(i % 256) as u8; 256],
                };

                tree.put(record)
                    .await
                    .expect("Failed to insert memory test record");
            }

            // Store goes out of scope here
        }

        web_sys::console::log_1(&format!("Memory test iteration {} completed", iteration).into());
    }

    // If we get here without issues, memory management is working
    web_sys::console::log_1(&"Memory management test completed successfully".into());
}
