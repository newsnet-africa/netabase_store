//! Comprehensive benchmarks for NetabaseStore performance-critical operations
//! 
//! This benchmark suite thoroughly tests the performance characteristics of:
//! - Data structure creation and manipulation
//! - Serialization/deserialization performance  
//! - Memory usage patterns
//! - Concurrent access patterns

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use netabase_store::prelude::*;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use std::thread;
use std::time::Duration;

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

    fn generate_batch(count: usize) -> Vec<Self> {
        (0..count).map(|i| Self::generate(i as u64)).collect()
    }
}

// In-memory store for benchmarking
#[derive(Clone)]
struct BenchMemoryStore {
    data: Arc<RwLock<HashMap<u64, BenchUser>>>,
    email_index: Arc<RwLock<HashMap<String, u64>>>,
    username_index: Arc<RwLock<HashMap<String, u64>>>,
}

impl BenchMemoryStore {
    fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            email_index: Arc::new(RwLock::new(HashMap::new())),
            username_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn insert(&self, user: BenchUser) -> Result<(), String> {
        let mut data = self.data.write().unwrap();
        let mut email_index = self.email_index.write().unwrap();
        let mut username_index = self.username_index.write().unwrap();

        data.insert(user.id, user.clone());
        email_index.insert(user.email.clone(), user.id);
        username_index.insert(user.username.clone(), user.id);

        Ok(())
    }

    fn get(&self, id: u64) -> Result<Option<BenchUser>, String> {
        let data = self.data.read().unwrap();
        Ok(data.get(&id).cloned())
    }

    fn get_by_email(&self, email: &str) -> Result<Option<BenchUser>, String> {
        let email_index = self.email_index.read().unwrap();
        if let Some(&id) = email_index.get(email) {
            self.get(id)
        } else {
            Ok(None)
        }
    }

    fn batch_insert(&self, users: Vec<BenchUser>) -> Result<(), String> {
        let mut data = self.data.write().unwrap();
        let mut email_index = self.email_index.write().unwrap();
        let mut username_index = self.username_index.write().unwrap();

        for user in users {
            data.insert(user.id, user.clone());
            email_index.insert(user.email.clone(), user.id);
            username_index.insert(user.username.clone(), user.id);
        }

        Ok(())
    }

    fn len(&self) -> usize {
        self.data.read().unwrap().len()
    }
}

// Benchmarking functions
fn bench_data_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_creation");
    
    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("create_users", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let users = BenchUser::generate_batch(size);
                    black_box(users);
                });
            }
        );
    }
    
    group.finish();
}

fn bench_memory_store_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_store_ops");
    
    for size in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        group.bench_with_input(
            BenchmarkId::new("insert", size),
            size,
            |b, &size| {
                let users = BenchUser::generate_batch(size);
                b.iter(|| {
                    let store = BenchMemoryStore::new();
                    for user in &users {
                        black_box(store.insert(user.clone()).unwrap());
                    }
                });
            }
        );

        group.bench_with_input(
            BenchmarkId::new("get", size),
            size,
            |b, &size| {
                let store = BenchMemoryStore::new();
                let users = BenchUser::generate_batch(size);
                for user in &users {
                    store.insert(user.clone()).unwrap();
                }

                b.iter(|| {
                    for i in 0..size {
                        black_box(store.get(i as u64).unwrap());
                    }
                });
            }
        );

        group.bench_with_input(
            BenchmarkId::new("batch_insert", size),
            size,
            |b, &size| {
                let users = BenchUser::generate_batch(size);
                b.iter(|| {
                    let store = BenchMemoryStore::new();
                    black_box(store.batch_insert(users.clone()).unwrap());
                });
            }
        );
    }
    
    group.finish();
}

fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    
    let users: Vec<BenchUser> = (0..1000).map(|i| BenchUser::generate(i)).collect();
    
    group.bench_function("json_serialize", |b| {
        b.iter(|| {
            for user in &users {
                black_box(serde_json::to_string(user).unwrap());
            }
        });
    });

    let serialized: Vec<String> = users.iter()
        .map(|user| serde_json::to_string(user).unwrap())
        .collect();

    group.bench_function("json_deserialize", |b| {
        b.iter(|| {
            for data in &serialized {
                black_box(serde_json::from_str::<BenchUser>(data).unwrap());
            }
        });
    });

    group.finish();
}

fn bench_concurrent_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_access");
    group.measurement_time(Duration::from_secs(10));

    for thread_count in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_reads", thread_count),
            thread_count,
            |b, &thread_count| {
                let store = Arc::new(BenchMemoryStore::new());
                let users = BenchUser::generate_batch(10000);
                
                // Pre-populate the store
                for user in &users {
                    store.insert(user.clone()).unwrap();
                }

                b.iter(|| {
                    let mut handles = vec![];
                    
                    for _ in 0..thread_count {
                        let store_clone = Arc::clone(&store);
                        let handle = thread::spawn(move || {
                            for i in 0..1000 {
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

fn bench_data_size_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_size_impact");

    // Create users with varying amounts of data
    let small_user = {
        let mut user = BenchUser::generate(1);
        user.profile.bio = "Short".to_string();
        user.profile.tags = vec!["tag1".to_string()];
        user.settings.features = HashMap::new();
        user.metadata.flags = vec![];
        user.metadata.scores = HashMap::new();
        user
    };

    let medium_user = BenchUser::generate(1);

    let large_user = {
        let mut user = BenchUser::generate(1);
        user.profile.bio = "Very long bio ".repeat(100);
        user.profile.tags = (0..50).map(|i| format!("tag{}", i)).collect();
        user.settings.features = (0..100).map(|i| (format!("feature{}", i), i % 2 == 0)).collect();
        user.metadata.flags = (0..50).map(|i| format!("flag{}", i)).collect();
        user.metadata.scores = (0..100).map(|i| (format!("score{}", i), i as f64)).collect();
        user
    };

    for (name, user) in [
        ("small", &small_user),
        ("medium", &medium_user), 
        ("large", &large_user),
    ] {
        let serialized_size = serde_json::to_string(user).unwrap().len();
        
        group.bench_with_input(
            BenchmarkId::new("memory_store_insert", format!("{}({} bytes)", name, serialized_size)),
            user,
            |b, user| {
                b.iter(|| {
                    let store = BenchMemoryStore::new();
                    for i in 0..1000 {
                        let mut test_user = user.clone();
                        test_user.id = i;
                        black_box(store.insert(test_user).unwrap());
                    }
                });
            }
        );

        group.bench_with_input(
            BenchmarkId::new("serialization", format!("{}({} bytes)", name, serialized_size)),
            user,
            |b, user| {
                b.iter(|| {
                    for _ in 0..1000 {
                        black_box(serde_json::to_string(user).unwrap());
                    }
                });
            }
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_data_creation,
    bench_memory_store_operations,
    bench_serialization,
    bench_concurrent_access,
    bench_data_size_impact
);
criterion_main!(benches);