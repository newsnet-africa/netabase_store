//! Concurrency-focused benchmarks for NetabaseStore
//! 
//! These benchmarks test performance under concurrent access patterns,
//! thread safety, and synchronization overhead.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use netabase_store::prelude::*;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

// Helper function to create test data for concurrency tests
fn create_concurrency_test_data(count: usize) -> Vec<serde_json::Value> {
    (0..count)
        .map(|i| {
            json!({
                "id": i,
                "data": format!("data_{}", i),
                "thread_id": 0,
                "metadata": {
                    "created_at": 1600000000 + i * 3600,
                    "flags": vec![format!("flag_{}", i % 10)]
                }
            })
        })
        .collect()
}

// Thread-safe store wrapper for benchmarking
struct ThreadSafeDataStore {
    data: Arc<RwLock<HashMap<u64, serde_json::Value>>>,
    index: Arc<RwLock<HashMap<String, u64>>>,
}

impl ThreadSafeDataStore {
    fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            index: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    fn insert(&self, id: u64, data: serde_json::Value) -> Result<(), String> {
        let mut data_store = self.data.write().unwrap();
        let mut index = self.index.write().unwrap();

        if let Some(data_str) = data.get("data").and_then(|v| v.as_str()) {
            index.insert(data_str.to_string(), id);
        }
        data_store.insert(id, data);

        Ok(())
    }
    
    fn get(&self, id: u64) -> Result<Option<serde_json::Value>, String> {
        let data_store = self.data.read().unwrap();
        Ok(data_store.get(&id).cloned())
    }
    
    fn get_by_data(&self, data_key: &str) -> Result<Option<serde_json::Value>, String> {
        let index = self.index.read().unwrap();
        if let Some(&id) = index.get(data_key) {
            drop(index);
            self.get(id)
        } else {
            Ok(None)
        }
    }
    
    fn update(&self, id: u64, data: serde_json::Value) -> Result<(), String> {
        let mut data_store = self.data.write().unwrap();
        data_store.insert(id, data);
        Ok(())
    }
    
    fn delete(&self, id: u64) -> Result<Option<serde_json::Value>, String> {
        let mut data_store = self.data.write().unwrap();
        let mut index = self.index.write().unwrap();
        
        if let Some(removed) = data_store.remove(&id) {
            if let Some(data_str) = removed.get("data").and_then(|v| v.as_str()) {
                index.remove(data_str);
            }
            Ok(Some(removed))
        } else {
            Ok(None)
        }
    }
}

fn bench_single_threaded_baseline(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_threaded_baseline");
    
    group.bench_function("sequential_operations", |b| {
        b.iter(|| {
            let store = ThreadSafeDataStore::new();
            
            // Insert
            for i in 0..1000 {
                let data = json!({
                    "id": i,
                    "data": format!("data_{}", i),
                    "thread_id": 0
                });
                black_box(store.insert(i, data).unwrap());
            }
            
            // Read
            for i in 0..1000 {
                black_box(store.get(i).unwrap());
            }
            
            // Update
            for i in 0..500 {
                let data = json!({
                    "id": i,
                    "data": format!("updated_{}", i),
                    "thread_id": 0
                });
                black_box(store.update(i, data).unwrap());
            }
            
            // Delete
            for i in 500..1000 {
                black_box(store.delete(i).unwrap());
            }
        });
    });
    
    group.finish();
}

fn bench_concurrent_reads(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_reads");
    
    for thread_count in [2, 4, 8, 16].iter() {
        group.bench_with_input(
            BenchmarkId::new("read_heavy", thread_count),
            thread_count,
            |b, &threads| {
                b.iter(|| {
                    let store = Arc::new(ThreadSafeDataStore::new());
                    
                    // Pre-populate with data
                    for i in 0..1000 {
                        let data = json!({
                            "id": i,
                            "data": format!("data_{}", i),
                            "thread_id": 0
                        });
                        store.insert(i, data).unwrap();
                    }
                    
                    let mut handles = vec![];
                    
                    for thread_id in 0..threads {
                        let store_clone = Arc::clone(&store);
                        let handle = thread::spawn(move || {
                            for i in (thread_id * 250)..((thread_id + 1) * 250) {
                                black_box(store_clone.get(i as u64).unwrap());
                            }
                        });
                        handles.push(handle);
                    }
                    
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            }
        );
    }
    
    group.finish();
}

fn bench_concurrent_writes(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_writes");
    
    for thread_count in [2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::new("write_heavy", thread_count),
            thread_count,
            |b, &threads| {
                b.iter(|| {
                    let store = Arc::new(ThreadSafeDataStore::new());
                    let mut handles = vec![];
                    
                    for thread_id in 0..threads {
                        let store_clone = Arc::clone(&store);
                        let handle = thread::spawn(move || {
                            for i in 0..250 {
                                let id = thread_id * 250 + i;
                                let data = json!({
                                    "id": id,
                                    "data": format!("data_{}_{}", thread_id, i),
                                    "thread_id": thread_id
                                });
                                black_box(store_clone.insert(id as u64, data).unwrap());
                            }
                        });
                        handles.push(handle);
                    }
                    
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            }
        );
    }
    
    group.finish();
}

fn bench_mixed_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_concurrent");
    
    for thread_count in [2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::new("mixed_workload", thread_count),
            thread_count,
            |b, &threads| {
                b.iter(|| {
                    let store = Arc::new(ThreadSafeDataStore::new());
                    
                    // Pre-populate with some data
                    for i in 0..500 {
                        let data = json!({
                            "id": i,
                            "data": format!("initial_{}", i),
                            "thread_id": 0
                        });
                        store.insert(i, data).unwrap();
                    }
                    
                    let mut handles = vec![];
                    
                    for thread_id in 0..threads {
                        let store_clone = Arc::clone(&store);
                        let handle = thread::spawn(move || {
                            let base_id = thread_id * 100;
                            
                            for i in 0..50 {
                                let id = base_id + i;
                                
                                // Mix of operations
                                match i % 4 {
                                    0 => {
                                        // Insert new
                                        let data = json!({
                                            "id": id + 1000,
                                            "data": format!("new_{}_{}", thread_id, i),
                                            "thread_id": thread_id
                                        });
                                        black_box(store_clone.insert((id + 1000) as u64, data).unwrap());
                                    },
                                    1 => {
                                        // Read existing
                                        if id < 500 {
                                            black_box(store_clone.get(id as u64).unwrap());
                                        }
                                    },
                                    2 => {
                                        // Update existing
                                        if id < 500 {
                                            let data = json!({
                                                "id": id,
                                                "data": format!("updated_{}_{}", thread_id, i),
                                                "thread_id": thread_id
                                            });
                                            black_box(store_clone.update(id as u64, data).unwrap());
                                        }
                                    },
                                    3 => {
                                        // Delete (but only from our range)
                                        if id >= 400 && id < 450 {
                                            black_box(store_clone.delete(id as u64).unwrap());
                                        }
                                    },
                                    _ => unreachable!(),
                                }
                            }
                        });
                        handles.push(handle);
                    }
                    
                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            }
        );
    }
    
    group.finish();
}

fn bench_contention_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("contention_scenarios");
    
    group.bench_function("high_contention_same_keys", |b| {
        b.iter(|| {
            let store = Arc::new(ThreadSafeDataStore::new());
            
            // Pre-populate
            for i in 0..10 {
                let data = json!({
                    "id": i,
                    "data": format!("data_{}", i),
                    "thread_id": 0
                });
                store.insert(i, data).unwrap();
            }
            
            let mut handles = vec![];
            
            // Multiple threads accessing same small set of keys
            for thread_id in 0..8 {
                let store_clone = Arc::clone(&store);
                let handle = thread::spawn(move || {
                    for _ in 0..100 {
                        let key_id = thread_id % 10; // High contention on same keys
                        
                        // Read
                        black_box(store_clone.get(key_id).unwrap());
                        
                        // Update
                        let data = json!({
                            "id": key_id,
                            "data": format!("thread_{}_data", thread_id),
                            "thread_id": thread_id
                        });
                        black_box(store_clone.update(key_id, data).unwrap());
                    }
                });
                handles.push(handle);
            }
            
            for handle in handles {
                handle.join().unwrap();
            }
        });
    });
    
    group.bench_function("low_contention_different_keys", |b| {
        b.iter(|| {
            let store = Arc::new(ThreadSafeDataStore::new());
            let mut handles = vec![];
            
            // Each thread works on different key ranges (low contention)
            for thread_id in 0..8 {
                let store_clone = Arc::clone(&store);
                let handle = thread::spawn(move || {
                    let base = thread_id * 1000;
                    
                    for i in 0..100 {
                        let key_id = base + i;
                        
                        // Insert
                        let data = json!({
                            "id": key_id,
                            "data": format!("thread_{}_data_{}", thread_id, i),
                            "thread_id": thread_id
                        });
                        black_box(store_clone.insert(key_id as u64, data).unwrap());
                        
                        // Read
                        black_box(store_clone.get(key_id as u64).unwrap());
                    }
                });
                handles.push(handle);
            }
            
            for handle in handles {
                handle.join().unwrap();
            }
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_single_threaded_baseline,
    bench_concurrent_reads,
    bench_concurrent_writes,
    bench_mixed_concurrent_operations,
    bench_contention_scenarios
);
criterion_main!(benches);