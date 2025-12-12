use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use netabase_store::prelude::*;
use serde_json::json;
use std::collections::HashMap;

// Helper function to create test data
fn create_test_data(count: usize) -> Vec<serde_json::Value> {
    (0..count)
        .map(|i| {
            json!({
                "id": i,
                "data": format!("data_item_{}", i),
                "metadata": {
                    "created_at": 1600000000 + i * 3600,
                    "tags": vec![format!("tag_{}", i % 10), format!("category_{}", i % 5)]
                }
            })
        })
        .collect()
}

fn bench_throughput_data_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput_data_creation");
    
    for batch_size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*batch_size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("json_creation_throughput", batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    let data = create_test_data(size);
                    black_box(data);
                });
            }
        );
    }
    
    group.finish();
}

fn bench_throughput_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput_serialization");
    
    for batch_size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*batch_size as u64));
        
        let test_data = create_test_data(*batch_size);
        
        group.bench_with_input(
            BenchmarkId::new("json_serialization_throughput", batch_size),
            &test_data,
            |b, data| {
                b.iter(|| {
                    for item in data {
                        let serialized = serde_json::to_string(item).unwrap();
                        black_box(serialized);
                    }
                });
            }
        );
    }
    
    group.finish();
}

criterion_group!(benches, bench_throughput_data_creation, bench_throughput_serialization);
criterion_main!(benches);