use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use netabase_store::prelude::*;
use serde_json::json;
use tempfile::TempDir;

// Test data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchUser {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub data: String,
}

impl BenchUser {
    fn generate(id: u64) -> Self {
        Self {
            id,
            username: format!("user_{}", id),
            email: format!("user{}@benchmark.com", id),
            data: format!("test_data_{}", id),
        }
    }

    fn generate_batch(count: usize) -> Vec<Self> {
        (0..count).map(|i| Self::generate(i as u64)).collect()
    }
}

// Helper function to create test data
fn create_test_data(count: usize) -> Vec<serde_json::Value> {
    (0..count)
        .map(|i| {
            json!({
                "id": i,
                "name": format!("Name {}", i),
                "data": format!("data_item_{}", i)
            })
        })
        .collect()
}

fn bench_data_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_creation");
    
    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("json_creation", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let data = create_test_data(size);
                    black_box(data);
                });
            }
        );
        
        group.bench_with_input(
            BenchmarkId::new("struct_creation", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let data = BenchUser::generate_batch(size);
                    black_box(data);
                });
            }
        );
    }
    
    group.finish();
}

fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    
    let users = BenchUser::generate_batch(1000);
    let json_data = create_test_data(1000);
    
    group.bench_function("json_serialize", |b| {
        b.iter(|| {
            for data in &json_data {
                let serialized = serde_json::to_string(data).unwrap();
                black_box(serialized);
            }
        });
    });

    group.bench_function("struct_serialize", |b| {
        b.iter(|| {
            for user in &users {
                let serialized = serde_json::to_string(user).unwrap();
                black_box(serialized);
            }
        });
    });
    
    group.finish();
}

fn bench_basic_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("basic_operations");
    
    group.bench_function("json_manipulation", |b| {
        b.iter(|| {
            let mut data = json!({
                "id": 1,
                "name": "test",
                "items": []
            });
            
            // Add items to array
            for i in 0..100 {
                data["items"].as_array_mut().unwrap().push(json!({
                    "item_id": i,
                    "value": format!("item_{}", i)
                }));
            }
            
            black_box(data);
        });
    });
    
    group.finish();
}

criterion_group!(benches, bench_data_creation, bench_serialization, bench_basic_operations);
criterion_main!(benches);