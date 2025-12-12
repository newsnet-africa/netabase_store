//! Memory-focused benchmarks for NetabaseStore
//! 
//! These benchmarks specifically test memory usage patterns, allocation behavior,
//! and garbage collection performance under various workloads.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use netabase_store::prelude::*;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

// Helper function to create test data for memory tests
fn create_memory_test_data(count: usize, data_size: usize) -> Vec<serde_json::Value> {
    (0..count)
        .map(|i| {
            json!({
                "id": i,
                "large_data": "x".repeat(data_size),
                "metadata": format!("metadata_{}", i)
            })
        })
        .collect()
}

fn bench_memory_allocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");
    
    // Test different data sizes to see memory allocation behavior
    for data_size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("large_object_creation", data_size),
            data_size,
            |b, &size| {
                b.iter(|| {
                    let test_data = create_memory_test_data(100, size);
                    black_box(test_data);
                });
            }
        );
    }
    
    group.finish();
}

fn bench_memory_growth_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_growth");
    
    group.bench_function("progressive_data_growth", |b| {
        b.iter(|| {
            let mut all_data = Vec::new();
            
            // Insert increasingly larger objects to test memory growth
            for i in 0..1000 {
                let data_size = i + 100; // Growing data size
                let data = json!({
                    "id": i,
                    "large_data": "x".repeat(data_size),
                    "metadata": format!("metadata_{}", i)
                });
                all_data.push(data);
            }
            black_box(all_data);
        });
    });
    
    group.bench_function("fragmentation_simulation", |b| {
        b.iter(|| {
            let mut data_map = HashMap::new();
            
            // Insert data
            for i in 0..500 {
                let data = json!({
                    "id": i,
                    "large_data": "data".repeat(100),
                    "metadata": format!("metadata_{}", i)
                });
                data_map.insert(i, data);
            }
            
            // Remove every other item (simulates fragmentation)
            for i in (0..500).step_by(2) {
                data_map.remove(&i);
            }
            
            // Re-insert with different data
            for i in (0..500).step_by(2) {
                let data = json!({
                    "id": i,
                    "large_data": "new_data".repeat(150),
                    "metadata": format!("new_metadata_{}", i)
                });
                data_map.insert(i, data);
            }
            
            black_box(data_map);
        });
    });
    
    group.finish();
}

fn bench_concurrent_memory_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_memory");
    
    for thread_count in [2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_data_creation", thread_count),
            thread_count,
            |b, &threads| {
                b.iter(|| {
                    let mut handles = vec![];
                    
                    for thread_id in 0..threads {
                        let handle = thread::spawn(move || {
                            let mut local_data = Vec::new();
                            for i in 0..100 {
                                let data = json!({
                                    "thread_id": thread_id,
                                    "id": i,
                                    "large_data": "data".repeat(100),
                                    "metadata": format!("metadata_{}_{}", thread_id, i)
                                });
                                local_data.push(data);
                            }
                            black_box(local_data);
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

fn bench_memory_vs_serialization_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_vs_serialization");
    
    let test_data = create_memory_test_data(1000, 500);
    
    group.bench_function("memory_data_storage", |b| {
        b.iter(|| {
            let mut storage = HashMap::new();
            for (i, data) in test_data.iter().enumerate() {
                storage.insert(i, data.clone());
            }
            black_box(storage);
        });
    });
    
    group.bench_function("serialized_data_storage", |b| {
        b.iter(|| {
            let mut storage = HashMap::new();
            for (i, data) in test_data.iter().enumerate() {
                let serialized = serde_json::to_string(data).unwrap();
                storage.insert(i, serialized);
            }
            black_box(storage);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_memory_allocation_patterns,
    bench_memory_growth_patterns,
    bench_concurrent_memory_access,
    bench_memory_vs_serialization_overhead
);
criterion_main!(benches);