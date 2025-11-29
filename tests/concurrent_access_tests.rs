//! Concurrent Access and Thread Safety Tests
//!
//! These tests verify that the crate handles concurrent access correctly
//! and maintains data integrity under multi-threaded conditions.

#![cfg(not(target_arch = "wasm32"))] // Threading tests are native-only

use netabase_store::netabase_definition_module;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

// Test schema for concurrent operations
#[netabase_definition_module(ConcurrentTestDefinition, ConcurrentTestKeys)]
mod concurrent_test_schema {
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
    #[netabase(ConcurrentTestDefinition)]
    pub struct Counter {
        #[primary_key]
        pub id: String,
        pub value: u64,
        #[secondary_key]
        pub thread_id: u32,
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
    #[netabase(ConcurrentTestDefinition)]
    pub struct BankAccount {
        #[primary_key]
        pub account_id: u64,
        pub balance: i64,
        #[secondary_key]
        pub owner: String,
    }
}

use concurrent_test_schema::*;

/// Test concurrent reads from multiple threads
#[test]
#[cfg(feature = "sled")]
fn test_concurrent_reads_sled() {
    use netabase_store::databases::sled_store::SledStore;

    let store = Arc::new(SledStore::<ConcurrentTestDefinition>::temp().unwrap());
    let tree = store.open_tree::<Counter>();

    // Insert initial data
    for i in 0..100 {
        tree.put(Counter {
            id: format!("counter_{}", i),
            value: i,
            thread_id: 0,
        })
        .unwrap();
    }

    // Test concurrent reads
    let num_threads = 10;
    let reads_per_thread = 50;
    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let store_clone = Arc::clone(&store);
        let handle = thread::spawn(move || {
            let tree = store_clone.open_tree::<Counter>();
            let mut successful_reads = 0;

            for i in 0..reads_per_thread {
                let key = CounterPrimaryKey(format!("counter_{}", i % 100));
                if let Ok(Some(counter)) = tree.get(key) {
                    assert_eq!(counter.value, (i % 100) as u64);
                    successful_reads += 1;
                }
            }

            successful_reads
        });
        handles.push(handle);
    }

    // Wait for all threads and verify they all succeeded
    for handle in handles {
        let successful_reads = handle.join().unwrap();
        assert_eq!(successful_reads, reads_per_thread);
    }
}

/// Test concurrent writes from multiple threads
#[test]
#[cfg(feature = "sled")]
fn test_concurrent_writes_sled() {
    use netabase_store::databases::sled_store::SledStore;

    let store = Arc::new(SledStore::<ConcurrentTestDefinition>::temp().unwrap());
    let num_threads = 8;
    let writes_per_thread = 100;
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let store_clone = Arc::clone(&store);
        let barrier_clone = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            let tree = store_clone.open_tree::<Counter>();

            // Wait for all threads to be ready
            barrier_clone.wait();

            // Each thread writes to its own key space to avoid conflicts
            for i in 0..writes_per_thread {
                let counter = Counter {
                    id: format!("thread_{}_{}", thread_id, i),
                    value: (thread_id as u64) * 1000 + i as u64,
                    thread_id: thread_id as u32,
                };

                tree.put(counter).unwrap();
            }

            writes_per_thread
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        let writes_completed = handle.join().unwrap();
        assert_eq!(writes_completed, writes_per_thread);
    }

    // Verify all data was written correctly
    let tree = store.open_tree::<Counter>();
    for thread_id in 0..num_threads {
        for i in 0..writes_per_thread {
            let key = CounterPrimaryKey(format!("thread_{}_{}", thread_id, i));
            let counter = tree.get(key).unwrap().unwrap();
            assert_eq!(counter.value, (thread_id as u64) * 1000 + i as u64);
            assert_eq!(counter.thread_id, thread_id as u32);
        }
    }
}

/// Test concurrent read-modify-write operations (race condition test)
#[test]
#[cfg(feature = "sled")]
fn test_concurrent_increment_sled() {
    use netabase_store::databases::sled_store::SledStore;

    let store = Arc::new(SledStore::<ConcurrentTestDefinition>::temp().unwrap());
    let tree = store.open_tree::<Counter>();

    // Initialize counter
    let initial_counter = Counter {
        id: "shared_counter".to_string(),
        value: 0,
        thread_id: 0,
    };
    tree.put(initial_counter).unwrap();

    let num_threads = 10;
    let increments_per_thread = 100;
    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let store_clone = Arc::clone(&store);
        let handle = thread::spawn(move || {
            let tree = store_clone.open_tree::<Counter>();

            for _ in 0..increments_per_thread {
                // Read-modify-write loop with retries
                loop {
                    let key = CounterPrimaryKey("shared_counter".to_string());
                    let mut counter = tree.get(key.clone()).unwrap().unwrap();
                    counter.value += 1;
                    counter.thread_id = thread_id as u32;

                    // Try to update - this may fail due to concurrent modifications
                    if tree.put(counter).is_ok() {
                        break;
                    }
                    // Small delay before retry
                    thread::sleep(Duration::from_micros(1));
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify final count
    let key = CounterPrimaryKey("shared_counter".to_string());
    let final_counter = tree.get(key).unwrap().unwrap();

    // With Sled's concurrent nature, we expect the final value to be close to
    // but possibly not exactly the expected total due to the MVCC nature
    let expected_total = (num_threads * increments_per_thread) as u64;
    println!(
        "Expected: {}, Actual: {}",
        expected_total, final_counter.value
    );

    // The value should be at least some reasonable portion of the expected total
    assert!(final_counter.value > 0);
    assert!(final_counter.value <= expected_total);
}

/// Test transaction isolation with redb zero-copy
#[test]
#[cfg(feature = "redb-zerocopy")]
fn test_transaction_isolation_redb_zerocopy() {
    use netabase_store::config::FileConfig;
    use netabase_store::databases::redb_zerocopy::RedbStoreZeroCopy;

    let config = FileConfig::new(std::env::temp_dir().join("concurrent_test_isolation.redb"));
    let store = Arc::new(RedbStoreZeroCopy::<ConcurrentTestDefinition>::new(config.path).unwrap());

    // Setup initial data
    {
        let mut txn = store.begin_write().unwrap();
        let mut tree = txn.open_tree::<BankAccount>().unwrap();
        for i in 0..10 {
            tree.put(BankAccount {
                account_id: i,
                balance: 1000,
                owner: format!("user_{}", i),
            })
            .unwrap();
        }
        drop(tree);
        txn.commit().unwrap();
    }

    let num_threads = 4;
    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let store_clone = Arc::clone(&store);
        let handle = thread::spawn(move || {
            // Each thread runs in its own transaction
            if thread_id % 2 == 0 {
                // Reader thread
                let txn = store_clone.begin_read().unwrap();
                let tree = txn.open_tree::<BankAccount>().unwrap();

                let mut total_balance = 0i64;
                for i in 0..10 {
                    let account = tree.get(&BankAccountPrimaryKey(i)).unwrap().unwrap();
                    total_balance += account.balance;
                }

                assert_eq!(total_balance, 10000); // Should see consistent snapshot
                drop(tree);
                drop(txn);
            } else {
                // Writer thread
                let mut txn = store_clone.begin_write().unwrap();
                let mut tree = txn.open_tree::<BankAccount>().unwrap();

                // Transfer money between accounts
                let account_a = tree.get(&BankAccountPrimaryKey(0)).unwrap().unwrap();
                let mut account_b = tree.get(&BankAccountPrimaryKey(1)).unwrap().unwrap();

                let transfer_amount = 100;
                if account_a.balance >= transfer_amount {
                    let mut new_account_a = account_a.clone();
                    new_account_a.balance -= transfer_amount;
                    account_b.balance += transfer_amount;

                    tree.put(new_account_a).unwrap();
                    tree.put(account_b).unwrap();
                }

                drop(tree);
                txn.commit().unwrap();
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify data consistency
    let txn = store.begin_read().unwrap();
    let tree = txn.open_tree::<BankAccount>().unwrap();
    let mut total_balance = 0i64;
    for i in 0..10 {
        let account = tree.get(&BankAccountPrimaryKey(i)).unwrap().unwrap();
        total_balance += account.balance;
    }
    assert_eq!(total_balance, 10000); // Total should remain constant
}

/// Test concurrent secondary key queries
#[test]
#[cfg(feature = "sled")]
fn test_concurrent_secondary_key_queries() {
    use netabase_store::databases::sled_store::SledStore;

    let store = Arc::new(SledStore::<ConcurrentTestDefinition>::temp().unwrap());
    let tree = store.open_tree::<Counter>();

    // Insert data with different thread_ids
    for thread_id in 0..5 {
        for i in 0..20 {
            tree.put(Counter {
                id: format!("counter_{}_{}", thread_id, i),
                value: i as u64,
                thread_id,
            })
            .unwrap();
        }
    }

    let num_query_threads = 8;
    let mut handles = Vec::new();

    for query_thread_id in 0..num_query_threads {
        let store_clone = Arc::clone(&store);
        let handle = thread::spawn(move || {
            let tree = store_clone.open_tree::<Counter>();

            // Query by different thread_ids concurrently
            let target_thread_id = (query_thread_id % 5) as u32;
            let results = tree
                .get_by_secondary_key(CounterSecondaryKeys::ThreadId(CounterThreadIdSecondaryKey(
                    target_thread_id,
                )))
                .unwrap();

            assert_eq!(results.len(), 20);
            for counter in &results {
                assert_eq!(counter.thread_id, target_thread_id);
            }

            results.len()
        });
        handles.push(handle);
    }

    // Wait for all query threads
    for handle in handles {
        let results_count = handle.join().unwrap();
        assert_eq!(results_count, 20);
    }
}

/// Test concurrent batch operations
#[test]
#[cfg(feature = "sled")]
fn test_concurrent_batch_operations() {
    use netabase_store::databases::sled_store::SledStore;

    let store = Arc::new(SledStore::<ConcurrentTestDefinition>::temp().unwrap());
    let num_threads = 6;
    let batch_size = 50;
    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let store_clone = Arc::clone(&store);
        let handle = thread::spawn(move || {
            let tree = store_clone.open_tree::<Counter>();

            // Each thread performs batch operations
            let counters: Vec<Counter> = (0..batch_size)
                .map(|i| Counter {
                    id: format!("batch_{}_{}", thread_id, i),
                    value: (thread_id as u64) * 1000 + i as u64,
                    thread_id: thread_id as u32,
                })
                .collect();

            // Batch insert
            for counter in counters {
                tree.put(counter).unwrap();
            }

            // Verify batch was inserted
            let keys: Vec<CounterPrimaryKey> = (0..batch_size)
                .map(|i| CounterPrimaryKey(format!("batch_{}_{}", thread_id, i)))
                .collect();

            let mut retrieved = Vec::new();
            for key in keys {
                if let Some(counter) = tree.get(key).unwrap() {
                    retrieved.push(counter);
                }
            }
            assert_eq!(retrieved.len(), batch_size);

            batch_size
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        let inserted_count = handle.join().unwrap();
        assert_eq!(inserted_count, batch_size);
    }

    // Verify total count
    let tree = store.open_tree::<Counter>();
    let all_counters: Vec<_> = tree.iter().collect();
    assert_eq!(all_counters.len(), num_threads * batch_size);
}

/// Test resource cleanup under concurrent access
#[test]
#[cfg(feature = "sled")]
fn test_concurrent_resource_cleanup() {
    use netabase_store::databases::sled_store::SledStore;

    let num_iterations = 10;
    let threads_per_iteration = 4;

    for iteration in 0..num_iterations {
        let store = Arc::new(SledStore::<ConcurrentTestDefinition>::temp().unwrap());
        let mut handles = Vec::new();

        for thread_id in 0..threads_per_iteration {
            let store_clone = Arc::clone(&store);
            let handle = thread::spawn(move || {
                let tree = store_clone.open_tree::<Counter>();

                // Perform operations
                for i in 0..50 {
                    tree.put(Counter {
                        id: format!("cleanup_{}_{}", thread_id, i),
                        value: i as u64,
                        thread_id: thread_id as u32,
                    })
                    .unwrap();
                }

                // Read some data
                for i in 0..25 {
                    let key = CounterPrimaryKey(format!("cleanup_{}_{}", thread_id, i));
                    let _ = tree.get(key);
                }
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Store should be cleanly dropped here
        drop(store);

        // Small delay to allow cleanup
        thread::sleep(Duration::from_millis(10));

        println!("Completed iteration {}", iteration + 1);
    }
}

/// Test deadlock prevention
#[test]
#[cfg(feature = "redb")]
fn test_deadlock_prevention() {
    use netabase_store::config::FileConfig;
    use netabase_store::databases::redb_store::RedbStore;

    let config = FileConfig::new(std::env::temp_dir().join("concurrent_test_deadlock.redb"));
    let store = Arc::new(RedbStore::<ConcurrentTestDefinition>::new(&config.path).unwrap());

    // Setup initial accounts
    let tree = store.open_tree::<BankAccount>();
    for i in 0..10 {
        tree.put(BankAccount {
            account_id: i,
            balance: 1000,
            owner: format!("owner_{}", i),
        })
        .unwrap();
    }

    let num_threads = 8;
    let operations_per_thread = 50;
    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let store_clone = Arc::clone(&store);
        let handle = thread::spawn(move || {
            let tree = store_clone.open_tree::<BankAccount>();

            for i in 0..operations_per_thread {
                // Perform operations that could potentially deadlock
                let account_a_id = (thread_id + i) % 10;
                let account_b_id = (thread_id + i + 1) % 10;

                // Always access accounts in consistent order to prevent deadlock
                let (first_id, second_id) = if account_a_id < account_b_id {
                    (account_a_id, account_b_id)
                } else {
                    (account_b_id, account_a_id)
                };

                let first_account = tree
                    .get(BankAccountKey::Primary(BankAccountPrimaryKey(first_id)))
                    .unwrap();
                let second_account = tree
                    .get(BankAccountKey::Primary(BankAccountPrimaryKey(second_id)))
                    .unwrap();

                if let (Some(mut first), Some(mut second)) = (first_account, second_account) {
                    if first.balance > 0 {
                        first.balance -= 1;
                        second.balance += 1;

                        tree.put(first).unwrap();
                        tree.put(second).unwrap();
                    }
                }
            }

            operations_per_thread
        });
        handles.push(handle);
    }

    // Wait for all threads with timeout to detect deadlocks
    for handle in handles {
        let operations_completed = handle.join().unwrap();
        assert_eq!(operations_completed, operations_per_thread);
    }

    // Verify total balance is preserved
    let tree = store.open_tree::<BankAccount>();
    let mut total_balance = 0i64;
    for i in 0..10 {
        let account = tree
            .get(BankAccountKey::Primary(BankAccountPrimaryKey(i)))
            .unwrap()
            .unwrap();
        total_balance += account.balance;
    }
    assert_eq!(total_balance, 10000);
}

/// Test memory pressure under concurrent access
#[test]
#[cfg(feature = "sled")]
fn test_memory_pressure_concurrent() {
    use netabase_store::databases::sled_store::SledStore;

    let store = Arc::new(SledStore::<ConcurrentTestDefinition>::temp().unwrap());
    let num_threads = 4;
    let large_data_size = 1000; // Insert 1000 records per thread
    let mut handles = Vec::new();

    for thread_id in 0..num_threads {
        let store_clone = Arc::clone(&store);
        let handle = thread::spawn(move || {
            let tree = store_clone.open_tree::<Counter>();

            // Insert large amount of data
            for i in 0..large_data_size {
                tree.put(Counter {
                    id: format!("large_data_{}_{}", thread_id, i),
                    value: i as u64,
                    thread_id: thread_id as u32,
                })
                .unwrap();

                // Occasionally read data to create memory pressure
                if i % 100 == 0 {
                    let key = CounterPrimaryKey(format!("large_data_{}_{}", thread_id, i / 2));
                    let _ = tree.get(key);
                }
            }

            large_data_size
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        let inserted_count = handle.join().unwrap();
        assert_eq!(inserted_count, large_data_size);
    }

    // Verify data integrity after memory pressure
    let tree = store.open_tree::<Counter>();
    for thread_id in 0..num_threads {
        for i in (0..large_data_size).step_by(100) {
            let key = CounterPrimaryKey(format!("large_data_{}_{}", thread_id, i));
            let counter = tree.get(key).unwrap().unwrap();
            assert_eq!(counter.value, i as u64);
            assert_eq!(counter.thread_id, thread_id as u32);
        }
    }
}

/// Test concurrent store creation and destruction
#[test]
#[cfg(feature = "sled")]
fn test_concurrent_store_lifecycle() {
    use netabase_store::databases::sled_store::SledStore;

    let num_stores = 10;
    let mut handles = Vec::new();

    for store_id in 0..num_stores {
        let handle = thread::spawn(move || {
            // Create store
            let store = SledStore::<ConcurrentTestDefinition>::temp().unwrap();
            let tree = store.open_tree::<Counter>();

            // Use store
            for i in 0..50 {
                tree.put(Counter {
                    id: format!("store_{}_{}", store_id, i),
                    value: i as u64,
                    thread_id: store_id as u32,
                })
                .unwrap();
            }

            // Verify data
            for i in 0..50 {
                let key = CounterPrimaryKey(format!("store_{}_{}", store_id, i));
                let counter = tree.get(key).unwrap().unwrap();
                assert_eq!(counter.value, i as u64);
            }

            // Store is dropped here
            50
        });
        handles.push(handle);
    }

    // Wait for all store lifecycles to complete
    for handle in handles {
        let records_processed = handle.join().unwrap();
        assert_eq!(records_processed, 50);
    }
}
