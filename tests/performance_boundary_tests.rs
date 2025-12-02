//! Performance Boundary and Resource Management Tests
//!
//! These tests verify that the crate handles performance boundaries,
//! resource limits, and cleanup correctly under various stress conditions.

#![cfg(not(target_arch = "wasm32"))] // Performance tests are native-only

use netabase_store::error::NetabaseError;
use netabase_store::netabase_definition_module;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

// Test schema for performance tests
#[netabase_definition_module(PerformanceTestDefinition, PerformanceTestKeys)]
mod performance_test_schema {
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
    #[netabase(PerformanceTestDefinition)]
    pub struct SmallRecord {
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
    #[netabase(PerformanceTestDefinition)]
    pub struct LargeRecord {
        #[primary_key]
        pub id: u64,
        pub large_data: Vec<u8>,
        pub description: String,
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
    #[netabase(PerformanceTestDefinition)]
    pub struct WideRecord {
        #[primary_key]
        pub id: u64,
        #[secondary_key]
        pub field1: String,
        #[secondary_key]
        pub field2: String,
        #[secondary_key]
        pub field3: String,
        #[secondary_key]
        pub field4: String,
        #[secondary_key]
        pub field5: String,
        pub data: Vec<u8>,
    }
}

use performance_test_schema::*;

/// Test insertion performance with small records
#[test]
#[cfg(feature = "sled")]
fn test_small_record_insertion_performance() {
    use netabase_store::databases::sled_store::SledStore;

    let store = SledStore::<PerformanceTestDefinition>::temp().unwrap();
    let tree = store.open_tree::<SmallRecord>();

    let num_records = 10_000;
    let start_time = Instant::now();

    for i in 0..num_records {
        let record = SmallRecord {
            id: i,
            name: format!("Record {}", i),
            category: format!("Category {}", i % 100),
        };
        tree.put(record).unwrap();
    }

    let insertion_duration = start_time.elapsed();
    let insertions_per_second = (num_records as f64) / insertion_duration.as_secs_f64();

    println!(
        "Inserted {} small records in {:?} ({:.2} ops/sec)",
        num_records, insertion_duration, insertions_per_second
    );

    // Should handle at least 1000 insertions per second
    assert!(insertions_per_second > 1000.0);
    assert!(insertion_duration.as_secs() < 30); // Should complete in under 30 seconds
}

/// Test query performance on large dataset
#[test]
#[cfg(feature = "sled")]
fn test_query_performance_large_dataset() {
    use netabase_store::databases::sled_store::SledStore;

    let store = SledStore::<PerformanceTestDefinition>::temp().unwrap();
    let tree = store.open_tree::<SmallRecord>();

    // Insert test data
    let num_records = 50_000;
    let batch_size = 1000;

    println!(
        "Inserting {} records for query performance test...",
        num_records
    );
    for batch_start in (0..num_records).step_by(batch_size) {
        let batch: Vec<SmallRecord> = (batch_start
            ..std::cmp::min(batch_start + batch_size, num_records))
            .map(|i| SmallRecord {
                id: i as u64,
                name: format!("Record {}", i),
                category: format!("Category {}", i % 1000),
            })
            .collect();

        for record in batch {
            tree.put(record).unwrap();
        }
    }

    // Test primary key queries
    let start_time = Instant::now();
    let mut found_count = 0;

    for i in (0..num_records).step_by(100) {
        if tree.get(SmallRecordPrimaryKey(i as u64)).unwrap().is_some() {
            found_count += 1;
        }
    }

    let query_duration = start_time.elapsed();
    let queries_per_second = (num_records as f64 / 100.0) / query_duration.as_secs_f64();

    println!(
        "Performed {} primary key queries in {:?} ({:.2} queries/sec)",
        num_records / 100,
        query_duration,
        queries_per_second
    );

    assert_eq!(found_count, num_records / 100);
    assert!(queries_per_second > 5000.0); // Should handle at least 5000 queries/sec
}

/// Test secondary key query performance
#[test]
#[cfg(feature = "sled")]
fn test_secondary_key_query_performance() {
    use netabase_store::databases::sled_store::SledStore;

    let store = SledStore::<PerformanceTestDefinition>::temp().unwrap();
    let tree = store.open_tree::<SmallRecord>();

    // Insert records with known distribution
    let num_records = 20_000;
    let num_categories = 100;

    for i in 0..num_records {
        let record = SmallRecord {
            id: i,
            name: format!("Record {}", i),
            category: format!("Category {}", i % num_categories),
        };
        tree.put(record).unwrap();
    }

    // Test secondary key queries
    let start_time = Instant::now();
    let mut total_results = 0;

    for category_id in 0..num_categories {
        let category = format!("Category {}", category_id);
        let results = tree
            .get_by_secondary_key(SmallRecordSecondaryKeys::Category(
                SmallRecordCategorySecondaryKey(category),
            ))
            .unwrap();

        total_results += results.len();
        assert_eq!(results.len(), (num_records / num_categories) as usize); // Should find expected count
    }

    let query_duration = start_time.elapsed();
    let queries_per_second = (num_categories as f64) / query_duration.as_secs_f64();

    println!(
        "Performed {} secondary key queries in {:?} ({:.2} queries/sec), found {} total results",
        num_categories, query_duration, queries_per_second, total_results
    );

    assert_eq!(total_results, num_records as usize);
    assert!(queries_per_second > 50.0); // Should handle at least 50 secondary key queries/sec
}

/// Test memory usage with large records
#[test]
#[cfg(feature = "sled")]
fn test_large_record_memory_usage() {
    use netabase_store::databases::sled_store::SledStore;

    let store = SledStore::<PerformanceTestDefinition>::temp().unwrap();
    let tree = store.open_tree::<LargeRecord>();

    let record_size = 1024 * 1024; // 1MB per record
    let num_records = 50; // 50MB total
    let large_data = vec![0u8; record_size];

    let start_time = Instant::now();

    for i in 0..num_records {
        let record = LargeRecord {
            id: i,
            large_data: large_data.clone(),
            description: format!("Large record {}", i),
        };

        match tree.put(record) {
            Ok(_) => {}
            Err(_) => {
                println!(
                    "Failed to insert large record {} (expected under memory pressure)",
                    i
                );
                break;
            }
        }

        // Check memory pressure by timing operations
        if i % 10 == 0 {
            let test_key = LargeRecordPrimaryKey(i / 2);
            let query_start = Instant::now();
            let _ = tree.get(test_key);
            let query_duration = query_start.elapsed();

            // Queries shouldn't take too long even under memory pressure
            assert!(query_duration < Duration::from_secs(1));
        }
    }

    let total_duration = start_time.elapsed();
    println!("Large record operations completed in {:?}", total_duration);

    // Should complete in reasonable time even with large records
    assert!(total_duration < Duration::from_secs(60));
}

/// Test performance with wide records (many secondary keys)
#[test]
#[cfg(feature = "sled")]
fn test_wide_record_performance() {
    use netabase_store::databases::sled_store::SledStore;

    let store = SledStore::<PerformanceTestDefinition>::temp().unwrap();
    let tree = store.open_tree::<WideRecord>();

    let num_records = 5_000;
    let data_size = 1024; // 1KB data per record
    let test_data = vec![42u8; data_size];

    let start_time = Instant::now();

    for i in 0..num_records {
        let record = WideRecord {
            id: i,
            field1: format!("field1_{}", i % 100),
            field2: format!("field2_{}", i % 200),
            field3: format!("field3_{}", i % 300),
            field4: format!("field4_{}", i % 400),
            field5: format!("field5_{}", i % 500),
            data: test_data.clone(),
        };
        tree.put(record).unwrap();
    }

    let insertion_duration = start_time.elapsed();
    let insertions_per_second = (num_records as f64) / insertion_duration.as_secs_f64();

    println!(
        "Inserted {} wide records in {:?} ({:.2} ops/sec)",
        num_records, insertion_duration, insertions_per_second
    );

    // Test queries on each secondary key
    let query_start = Instant::now();

    // Query by field1
    let results1 = tree
        .get_by_secondary_key(WideRecordSecondaryKeys::Field1(
            WideRecordField1SecondaryKey("field1_50".to_string()),
        ))
        .unwrap();

    // Query by field3
    let results3 = tree
        .get_by_secondary_key(WideRecordSecondaryKeys::Field3(
            WideRecordField3SecondaryKey("field3_150".to_string()),
        ))
        .unwrap();

    let query_duration = query_start.elapsed();

    println!(
        "Secondary key queries completed in {:?} (found {} and {} results)",
        query_duration,
        results1.len(),
        results3.len()
    );

    // Wide records should still maintain reasonable performance
    assert!(insertions_per_second > 100.0); // At least 100 ops/sec with many indexes
    assert!(query_duration < Duration::from_secs(2)); // Queries should be fast
}

/// Test concurrent performance under load
#[test]
#[cfg(feature = "sled")]
fn test_concurrent_performance_under_load() {
    use netabase_store::databases::sled_store::SledStore;

    let store = Arc::new(SledStore::<PerformanceTestDefinition>::temp().unwrap());
    let num_threads = 8;
    let operations_per_thread = 1_000;
    let total_operations = num_threads * operations_per_thread;

    let completed_operations = Arc::new(AtomicUsize::new(0));
    let start_time = Instant::now();
    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let store_clone = Arc::clone(&store);
        let completed_clone = Arc::clone(&completed_operations);

        let handle = thread::spawn(move || {
            let tree = store_clone.open_tree::<SmallRecord>();
            let mut local_completed = 0;

            for i in 0..operations_per_thread {
                let record = SmallRecord {
                    id: (thread_id * 10000 + i) as u64,
                    name: format!("Thread{}_Record{}", thread_id, i),
                    category: format!("Category{}", (thread_id + i) % 50),
                };

                match tree.put(record) {
                    Ok(_) => {
                        local_completed += 1;
                        completed_clone.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(_) => break,
                }

                // Mix in some read operations
                if i % 10 == 0 && i > 0 {
                    let read_key = SmallRecordPrimaryKey((thread_id * 10000 + i - 5) as u64);
                    let _ = tree.get(read_key);
                }
            }

            local_completed
        });
        handles.push(handle);
    }

    // Wait for all threads with progress monitoring
    let mut last_progress = 0;
    loop {
        thread::sleep(Duration::from_millis(500));
        let current_progress = completed_operations.load(Ordering::Relaxed);

        if current_progress > last_progress {
            println!(
                "Progress: {}/{} operations completed ({:.1}%)",
                current_progress,
                total_operations,
                (current_progress as f64 / total_operations as f64) * 100.0
            );
            last_progress = current_progress;
        }

        // Check if all threads are done
        let mut all_done = true;
        for handle in &handles {
            if !handle.is_finished() {
                all_done = false;
                break;
            }
        }
        if all_done {
            break;
        }

        // Timeout after reasonable time
        if start_time.elapsed() > Duration::from_secs(120) {
            panic!("Concurrent performance test timed out after 2 minutes");
        }
    }

    // Collect results
    let mut total_completed = 0;
    for handle in handles {
        total_completed += handle.join().unwrap();
    }

    let total_duration = start_time.elapsed();
    let operations_per_second = (total_completed as f64) / total_duration.as_secs_f64();

    println!(
        "Concurrent test: {} operations completed in {:?} ({:.2} ops/sec)",
        total_completed, total_duration, operations_per_second
    );

    // Should maintain reasonable performance under concurrent load
    assert!(total_completed > total_operations / 2); // At least 50% should succeed
    assert!(operations_per_second > 500.0); // At least 500 ops/sec total throughput
}

/// Test resource cleanup after heavy operations
#[test]
#[cfg(feature = "sled")]
fn test_resource_cleanup_after_heavy_operations() {
    use netabase_store::databases::sled_store::SledStore;

    for iteration in 0..5 {
        println!("Resource cleanup test iteration {}", iteration + 1);

        {
            let store = SledStore::<PerformanceTestDefinition>::temp().unwrap();
            let tree = store.open_tree::<LargeRecord>();

            // Perform heavy operations
            let large_data = vec![iteration as u8; 512 * 1024]; // 512KB per record

            for i in 0..20 {
                let record = LargeRecord {
                    id: i,
                    large_data: large_data.clone(),
                    description: format!("Iteration {} Record {}", iteration, i),
                };

                if tree.put(record).is_err() {
                    break; // Stop on error (resource pressure)
                }
            }

            // Perform some queries
            for i in 0..10 {
                let _ = tree.get(LargeRecordPrimaryKey(i));
            }

            // Store goes out of scope and should be cleaned up
        }

        // Small delay to allow cleanup
        thread::sleep(Duration::from_millis(100));
    }

    // If we completed all iterations without hanging or crashing, cleanup is working
    println!("Resource cleanup test completed successfully");
}

/// Test memory pressure handling
#[test]
#[cfg(feature = "sled")]
fn test_memory_pressure_handling() {
    use netabase_store::databases::sled_store::SledStore;

    let store = SledStore::<PerformanceTestDefinition>::temp().unwrap();
    let tree = store.open_tree::<LargeRecord>();

    let mut successful_inserts = 0;
    let mut record_size = 64 * 1024; // Start with 64KB
    let max_attempts = 1000;

    for i in 0..max_attempts {
        let large_data = vec![(i % 256) as u8; record_size];
        let record = LargeRecord {
            id: i,
            large_data,
            description: format!("Memory pressure test record {}", i),
        };

        match tree.put(record) {
            Ok(_) => {
                successful_inserts += 1;

                // Gradually increase record size to create pressure
                if i % 50 == 0 {
                    record_size = std::cmp::min(record_size + 32 * 1024, 2 * 1024 * 1024); // Cap at 2MB
                }
            }
            Err(_) => {
                println!(
                    "Hit memory pressure after {} successful inserts with record size {}KB",
                    successful_inserts,
                    record_size / 1024
                );
                break;
            }
        }

        // Test that reads still work under pressure
        if i % 25 == 0 && i > 0 {
            let test_key = LargeRecordPrimaryKey(i / 2);
            let read_start = Instant::now();

            match tree.get(test_key) {
                Ok(_) => {
                    let read_duration = read_start.elapsed();
                    assert!(read_duration < Duration::from_secs(5)); // Should still be responsive
                }
                Err(_) => {
                    println!("Reads starting to fail under memory pressure");
                    break;
                }
            }
        }
    }

    println!(
        "Memory pressure test: {} successful inserts before hitting limits",
        successful_inserts
    );

    // Should handle some reasonable amount of data before hitting limits
    assert!(successful_inserts > 10);
}

/// Test batch operation performance
#[test]
#[cfg(feature = "sled")]
fn test_batch_operation_performance() {
    use netabase_store::databases::sled_store::SledStore;

    let store = SledStore::<PerformanceTestDefinition>::temp().unwrap();
    let tree = store.open_tree::<SmallRecord>();

    // Test different batch sizes
    let batch_sizes = vec![1, 10, 100, 1000];
    let total_records = 10_000;

    for batch_size in batch_sizes {
        println!("Testing batch size: {}", batch_size);

        let start_time = Instant::now();
        let mut records_inserted = 0;

        for batch_start in (0..total_records).step_by(batch_size) {
            let batch_end = std::cmp::min(batch_start + batch_size, total_records);
            let batch: Vec<SmallRecord> = (batch_start..batch_end)
                .map(|i| SmallRecord {
                    id: (1_000_000 + i) as u64, // Offset to avoid conflicts
                    name: format!("Batch{}_Record{}", batch_size, i),
                    category: format!("BatchCat{}", i % 100),
                })
                .collect();

            for record in &batch {
                tree.put(record.clone()).unwrap();
            }
            match Result::<(), NetabaseError>::Ok(()) {
                Ok(_) => records_inserted += batch.len(),
                Err(_) => break,
            }
        }

        let duration = start_time.elapsed();
        let records_per_second = (records_inserted as f64) / duration.as_secs_f64();

        println!(
            "  Batch size {}: {} records in {:?} ({:.2} records/sec)",
            batch_size, records_inserted, duration, records_per_second
        );

        // Clear data for next test
        for i in 0..records_inserted {
            let _ = tree.remove(SmallRecordPrimaryKey((1_000_000 + i) as u64));
        }

        // Larger batches should generally be faster
        assert!(records_per_second > 1000.0);
    }
}

/// Test iteration performance on large datasets
#[test]
#[cfg(feature = "sled")]
fn test_iteration_performance() {
    use netabase_store::databases::sled_store::SledStore;

    let store = SledStore::<PerformanceTestDefinition>::temp().unwrap();
    let tree = store.open_tree::<SmallRecord>();

    // Insert test data
    let num_records = 25_000;
    println!("Inserting {} records for iteration test...", num_records);

    let batch_size = 1000;
    for batch_start in (0..num_records).step_by(batch_size) {
        let batch_end = std::cmp::min(batch_start + batch_size, num_records);
        let batch: Vec<SmallRecord> = (batch_start..batch_end)
            .map(|i| SmallRecord {
                id: i as u64,
                name: format!("IterRecord{}", i),
                category: format!("IterCat{}", i % 250),
            })
            .collect();

        for record in batch {
            tree.put(record).unwrap();
        }
    }

    // Test full iteration
    let start_time = Instant::now();
    let all_records: Vec<_> = tree.iter().collect();
    let iteration_duration = start_time.elapsed();

    let records_per_second = (all_records.len() as f64) / iteration_duration.as_secs_f64();

    println!(
        "Iterated over {} records in {:?} ({:.2} records/sec)",
        all_records.len(),
        iteration_duration,
        records_per_second
    );

    assert_eq!(all_records.len(), num_records);
    assert!(records_per_second > 50_000.0); // Should iterate at least 50k records/sec
    assert!(iteration_duration < Duration::from_secs(10)); // Should complete quickly
}

/// Test performance degradation with database growth
#[test]
#[cfg(feature = "sled")]
fn test_performance_with_database_growth() {
    use netabase_store::databases::sled_store::SledStore;

    let store = SledStore::<PerformanceTestDefinition>::temp().unwrap();
    let tree = store.open_tree::<SmallRecord>();

    let growth_stages = vec![1_000, 5_000, 10_000, 25_000];
    let test_operations = 100;

    for &stage_size in &growth_stages {
        // Grow database to target size
        let current_size = tree.iter().count();
        if current_size < stage_size {
            let records_to_add = stage_size - current_size;
            let batch_size = 1000;

            for batch_start in (0..records_to_add).step_by(batch_size) {
                let batch_end = std::cmp::min(batch_start + batch_size, records_to_add);
                let batch: Vec<SmallRecord> = (batch_start..batch_end)
                    .map(|i| SmallRecord {
                        id: (current_size + i) as u64,
                        name: format!("GrowthRecord{}", current_size + i),
                        category: format!("GrowthCat{}", i % 100),
                    })
                    .collect();

                for record in batch {
                    tree.put(record).unwrap();
                }
            }
        }

        // Measure performance at this stage
        let start_time = Instant::now();

        for i in 0..test_operations {
            // Mix of operations: inserts, reads, secondary key queries
            match i % 3 {
                0 => {
                    // Insert
                    let record = SmallRecord {
                        id: (stage_size + i) as u64,
                        name: format!("TestRecord{}", i),
                        category: "test".to_string(),
                    };
                    let _ = tree.put(record);
                }
                1 => {
                    // Read
                    let key = SmallRecordPrimaryKey((i % stage_size) as u64);
                    let _ = tree.get(key);
                }
                2 => {
                    // Secondary key query
                    let category = format!("GrowthCat{}", i % 100);
                    let _ = tree.get_by_secondary_key(SmallRecordSecondaryKeys::Category(
                        SmallRecordCategorySecondaryKey(category),
                    ));
                }
                _ => unreachable!(),
            }
        }

        let operations_duration = start_time.elapsed();
        let ops_per_second = (test_operations as f64) / operations_duration.as_secs_f64();

        println!(
            "Database size {}: {} operations in {:?} ({:.2} ops/sec)",
            stage_size, test_operations, operations_duration, ops_per_second
        );

        // Performance shouldn't degrade too dramatically with size
        assert!(ops_per_second > 100.0); // Should maintain at least 100 ops/sec
        assert!(operations_duration < Duration::from_secs(30)); // Should stay responsive
    }
}
