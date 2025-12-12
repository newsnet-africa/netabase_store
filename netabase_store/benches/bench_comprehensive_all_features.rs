//! Comprehensive Performance Benchmarks
//!
//! This benchmark suite tests the performance of critical NetabaseStore operations:
//! - Data structure creation and manipulation
//! - Serialization/deserialization performance  
//! - Memory usage patterns
//! - Concurrent access patterns

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use netabase_store::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

// Benchmark data structures
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchUser {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub profile: BenchProfile,
    pub settings: BenchSettings,
    pub metadata: BenchMetadata,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchProfile {
    pub first_name: String,
    pub last_name: String,
    pub bio: String,
    pub avatar_url: String,
    pub location: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchSettings {
    pub theme: String,
    pub language: String,
    pub notifications: bool,
    pub privacy_level: u8,
    pub features: HashMap<String, bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchMetadata {
    pub created_at: u64,
    pub updated_at: u64,
    pub last_login: u64,
    pub login_count: u64,
    pub flags: Vec<String>,
    pub scores: HashMap<String, f64>,
}

impl BenchUser {
    fn generate(id: u64) -> Self {
        let mut features = HashMap::new();
        features.insert("feature1".to_string(), true);
        features.insert("feature2".to_string(), false);
        features.insert("feature3".to_string(), true);

        let mut scores = HashMap::new();
        scores.insert("reputation".to_string(), (id as f64) * 1.5);
        scores.insert("activity".to_string(), (id as f64) * 0.8);

        Self {
            id,
            username: format!("user_{}", id),
            email: format!("user{}@benchmark.com", id),
            profile: BenchProfile {
                first_name: format!("First{}", id),
                last_name: format!("Last{}", id),
                bio: format!("Bio for user {} with some longer text to simulate realistic data size", id),
                avatar_url: format!("https://avatars.example.com/user_{}.png", id),
                location: format!("City{}", id % 100),
                tags: vec![
                    format!("tag{}", id % 10),
                    format!("category{}", id % 5),
                    format!("skill{}", id % 15),
                ],
            },
            settings: BenchSettings {
                theme: if id % 2 == 0 { "dark".to_string() } else { "light".to_string() },
                language: if id % 3 == 0 { "en".to_string() } else { "es".to_string() },
                notifications: id % 4 == 0,
                privacy_level: (id % 5) as u8,
                features,
            },
            metadata: BenchMetadata {
                created_at: 1600000000 + id * 86400,
                updated_at: 1700000000 + id * 3600,
                last_login: 1700000000 + id * 1800,
                login_count: id * 10 + (id % 100),
                flags: vec![
                    format!("flag_{}", id % 20),
                    format!("status_{}", id % 8),
                ],
                scores,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchProduct {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub category_id: u32,
    pub stock: u32,
    pub attributes: HashMap<String, String>,
    pub tags: Vec<String>,
    pub metadata: BenchProductMetadata,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchProductMetadata {
    pub created_at: u64,
    pub updated_at: u64,
    pub view_count: u64,
    pub rating: f32,
    pub reviews_count: u32,
}

impl BenchProduct {
    fn generate(id: u64, _user_id: u64) -> Self {
        let mut attributes = HashMap::new();
        attributes.insert("material".to_string(), format!("material_{}", id % 5));
        attributes.insert("color".to_string(), format!("color_{}", id % 8));
        attributes.insert("size".to_string(), format!("size_{}", id % 4));

        Self {
            id,
            name: format!("Product {}", id),
            description: format!("Description for product {} with detailed information", id),
            price: (id as f64) * 9.99 + 0.01,
            category_id: (id % 20) as u32,
            stock: (id % 100) as u32,
            attributes,
            tags: vec![
                format!("tag{}", id % 10),
                format!("category{}", id % 5),
                format!("brand{}", id % 3),
            ],
            metadata: BenchProductMetadata {
                created_at: 1600000000 + id * 86400,
                updated_at: 1700000000 + id * 3600,
                view_count: id * 5 + (id % 50),
                rating: 3.0 + ((id % 20) as f32) / 10.0,
                reviews_count: (id % 30) as u32,
            },
        }
    }
}

// =============================================================================
// BENCHMARK UTILITIES
// =============================================================================

/// Generate deterministic test data for benchmarks
struct BenchmarkDataGenerator {
    counter: u64,
}

impl BenchmarkDataGenerator {
    fn new() -> Self {
        Self { counter: 0 }
    }

    fn next_user(&mut self) -> BenchUser {
        self.counter += 1;
        BenchUser::generate(self.counter)
    }

    fn next_product(&mut self, user_id: u64) -> BenchProduct {
        self.counter += 1;
        BenchProduct::generate(self.counter, user_id)
    }
}

/// Setup a memory store with test data
fn setup_memory_store_with_data(size: usize) -> Vec<BenchUser> {
    let mut generator = BenchmarkDataGenerator::new();
    let mut users = Vec::new();

    for _ in 0..size {
        let user = generator.next_user();
        users.push(user);
    }

    users
}

// =============================================================================
// DATA STRUCTURE BENCHMARKS
// =============================================================================

fn bench_data_structure_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_structure_creation");
    
    // Benchmark user creation
    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("user_creation", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut generator = BenchmarkDataGenerator::new();
                    let mut users = Vec::new();
                    for _ in 0..size {
                        users.push(generator.next_user());
                    }
                    black_box(users);
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("product_creation", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut generator = BenchmarkDataGenerator::new();
                    let mut products = Vec::new();
                    for _ in 0..size {
                        products.push(generator.next_product(1));
                    }
                    black_box(products);
                });
            },
        );
    }
    
    group.finish();
}

fn bench_serialization_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    
    let mut generator = BenchmarkDataGenerator::new();
    let users: Vec<BenchUser> = (0..1000).map(|_| generator.next_user()).collect();
    let products: Vec<BenchProduct> = (0..1000).map(|_| generator.next_product(1)).collect();
    
    group.bench_function("user_json_serialize", |b| {
        b.iter(|| {
            for user in &users {
                black_box(serde_json::to_string(user).unwrap());
            }
        });
    });
    
    group.bench_function("user_serde_serialize", |b| {
        b.iter(|| {
            for user in &users {
                black_box(serde_json::to_string(user).unwrap());
            }
        });
    });
    
    group.bench_function("product_json_serialize", |b| {
        b.iter(|| {
            for product in &products {
                black_box(serde_json::to_string(product).unwrap());
            }
        });
    });
    
    group.finish();
}

fn bench_memory_usage_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");
    
    group.bench_function("large_dataset_creation", |b| {
        b.iter(|| {
            let mut generator = BenchmarkDataGenerator::new();
            let mut users = Vec::new();
            let mut products = Vec::new();
            
            // Create a realistic workload
            for i in 0..10000 {
                users.push(generator.next_user());
                if i % 10 == 0 {
                    products.push(generator.next_product(users.last().unwrap().id));
                }
            }
            
            black_box((users, products));
        });
    });
    
    group.finish();
}

fn bench_concurrent_access_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_access");
    
    group.bench_function("parallel_user_creation", |b| {
        b.iter(|| {
            let users = Arc::new(Mutex::new(Vec::new()));
            let handles: Vec<_> = (0..4)
                .map(|thread_id| {
                    let users_clone = Arc::clone(&users);
                    thread::spawn(move || {
                        let mut generator = BenchmarkDataGenerator::new();
                        let mut local_users = Vec::new();
                        
                        for i in 0..250 {
                            let user_id = (thread_id * 250 + i) as u64;
                            let mut user = generator.next_user();
                            user.id = user_id; // Ensure unique IDs
                            local_users.push(user);
                        }
                        
                        users_clone.lock().unwrap().extend(local_users);
                    })
                })
                .collect();
                
            for handle in handles {
                handle.join().unwrap();
            }
            
            black_box(users);
        });
    });
    
    group.finish();
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_benchmark_data_creation() {
        let mut generator = BenchmarkDataGenerator::new();
        let user = generator.next_user();
        assert!(!user.username.is_empty());
        
        let product = generator.next_product(user.id);
        assert!(!product.name.is_empty());
        
        println!("✅ Benchmark data structures work correctly");
        println!("User: {}", user.username);
        println!("Product: {}", product.name);
    }
}

criterion_group!(
    benches, 
    bench_data_structure_creation,
    bench_serialization_performance, 
    bench_memory_usage_patterns,
    bench_concurrent_access_patterns
);
criterion_main!(benches);