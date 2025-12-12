use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use netabase_macros::{netabase_definition_module, NetabaseModel};
use netabase_store::{
    databases::{redb_store::RedbStore, sled_store::SledStore},
    traits::store::store::StoreTrait,
    Root,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tempfile::TempDir;

// Define a comprehensive benchmark model using the macros
#[netabase_definition_module(BenchmarkDefinition, BenchmarkDefinitionKeys)]
pub mod benchmark_definition {
    use super::*;

    #[derive(NetabaseModel, Clone, Debug, PartialEq, Serialize, Deserialize)]
    pub struct User {
        #[primary_key]
        pub id: u64,
        #[secondary_key]
        pub email: String,
        #[secondary_key]
        pub username: String,
        pub name: String,
        pub age: u32,
        pub active: bool,
        pub metadata: HashMap<String, String>,
    }

    #[derive(NetabaseModel, Clone, Debug, PartialEq, Serialize, Deserialize)]
    pub struct Product {
        #[primary_key]
        pub id: u64,
        #[secondary_key]
        pub sku: String,
        #[secondary_key]
        pub category: String,
        pub name: String,
        pub price: f64,
        pub description: String,
        pub in_stock: bool,
        pub tags: Vec<String>,
    }

    #[derive(NetabaseModel, Clone, Debug, PartialEq, Serialize, Deserialize)]
    pub struct Order {
        #[primary_key]
        pub id: u64,
        #[secondary_key]
        pub user_id: u64,
        #[secondary_key]
        pub status: String,
        #[relational_key(User, id, user_id)]
        pub user: (),
        pub total: f64,
        pub items: Vec<OrderItem>,
        pub created_at: u64,
    }

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    pub struct OrderItem {
        pub product_id: u64,
        pub quantity: u32,
        pub price: f64,
    }
}

fn create_test_root() -> (Root, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let root = Root::new(temp_dir.path().to_path_buf()).unwrap();
    (root, temp_dir)
}

fn create_sample_user(id: u64) -> benchmark_definition::User {
    benchmark_definition::User {
        id,
        email: format!("user{}@example.com", id),
        username: format!("user{}", id),
        name: format!("User {}", id),
        age: 25 + (id as u32 % 50),
        active: id % 2 == 0,
        metadata: {
            let mut map = HashMap::new();
            map.insert("department".to_string(), format!("dept{}", id % 10));
            map.insert("role".to_string(), format!("role{}", id % 5));
            map
        },
    }
}

fn create_sample_product(id: u64) -> benchmark_definition::Product {
    benchmark_definition::Product {
        id,
        sku: format!("SKU-{:06}", id),
        category: format!("category{}", id % 10),
        name: format!("Product {}", id),
        price: 10.0 + (id as f64 * 0.99),
        description: format!("Description for product {}", id),
        in_stock: id % 3 != 0,
        tags: vec![
            format!("tag{}", id % 20),
            format!("tag{}", (id + 1) % 20),
        ],
    }
}

fn create_sample_order(id: u64, user_id: u64) -> benchmark_definition::Order {
    benchmark_definition::Order {
        id,
        user_id,
        status: if id % 4 == 0 { "completed" } else { "pending" }.to_string(),
        user: (),
        total: 100.0 + (id as f64 * 1.5),
        items: vec![
            benchmark_definition::OrderItem {
                product_id: id * 2,
                quantity: 1 + (id as u32 % 5),
                price: 25.0,
            },
            benchmark_definition::OrderItem {
                product_id: id * 2 + 1,
                quantity: 1,
                price: 15.0,
            },
        ],
        created_at: 1640000000 + id * 86400,
    }
}

fn bench_primary_key_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("primary_key_operations");
    
    for &store_type in &["redb", "sled"] {
        // Single insert benchmark
        group.bench_with_input(
            BenchmarkId::new("single_insert", store_type),
            &store_type,
            |b, &store_type| {
                b.iter_batched(
                    || {
                        let (root, _temp) = create_test_root();
                        let store: Box<dyn StoreTrait> = if store_type == "redb" {
                            Box::new(RedbStore::new(&root.path.join("test.redb")).unwrap())
                        } else {
                            Box::new(SledStore::new(&root.path.join("test.sled")).unwrap())
                        };
                        let definition_store = benchmark_definition::BenchmarkDefinitionStore::new(store);
                        (definition_store, create_sample_user(1), _temp)
                    },
                    |(mut store, user, _temp)| {
                        let _ = black_box(store.user_create(&user));
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );

        // Batch insert benchmark
        group.bench_with_input(
            BenchmarkId::new("batch_insert_100", store_type),
            &store_type,
            |b, &store_type| {
                b.iter_batched(
                    || {
                        let (root, _temp) = create_test_root();
                        let store: Box<dyn StoreTrait> = if store_type == "redb" {
                            Box::new(RedbStore::new(&root.path.join("test.redb")).unwrap())
                        } else {
                            Box::new(SledStore::new(&root.path.join("test.sled")).unwrap())
                        };
                        let definition_store = benchmark_definition::BenchmarkDefinitionStore::new(store);
                        let users: Vec<_> = (1..=100).map(create_sample_user).collect();
                        (definition_store, users, _temp)
                    },
                    |(mut store, users, _temp)| {
                        for user in users {
                            let _ = black_box(store.user_create(&user));
                        }
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );

        // Single read benchmark
        group.bench_with_input(
            BenchmarkId::new("single_read", store_type),
            &store_type,
            |b, &store_type| {
                b.iter_batched(
                    || {
                        let (root, _temp) = create_test_root();
                        let store: Box<dyn StoreTrait> = if store_type == "redb" {
                            Box::new(RedbStore::new(&root.path.join("test.redb")).unwrap())
                        } else {
                            Box::new(SledStore::new(&root.path.join("test.sled")).unwrap())
                        };
                        let mut definition_store = benchmark_definition::BenchmarkDefinitionStore::new(store);
                        let user = create_sample_user(1);
                        definition_store.user_create(&user).unwrap();
                        (definition_store, _temp)
                    },
                    |(store, _temp)| {
                        let _ = black_box(store.user_read(&1));
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );

        // Update benchmark
        group.bench_with_input(
            BenchmarkId::new("single_update", store_type),
            &store_type,
            |b, &store_type| {
                b.iter_batched(
                    || {
                        let (root, _temp) = create_test_root();
                        let store: Box<dyn StoreTrait> = if store_type == "redb" {
                            Box::new(RedbStore::new(&root.path.join("test.redb")).unwrap())
                        } else {
                            Box::new(SledStore::new(&root.path.join("test.sled")).unwrap())
                        };
                        let mut definition_store = benchmark_definition::BenchmarkDefinitionStore::new(store);
                        let mut user = create_sample_user(1);
                        definition_store.user_create(&user).unwrap();
                        user.name = "Updated Name".to_string();
                        (definition_store, user, _temp)
                    },
                    |(mut store, user, _temp)| {
                        let _ = black_box(store.user_update(&user));
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );

        // Delete benchmark
        group.bench_with_input(
            BenchmarkId::new("single_delete", store_type),
            &store_type,
            |b, &store_type| {
                b.iter_batched(
                    || {
                        let (root, _temp) = create_test_root();
                        let store: Box<dyn StoreTrait> = if store_type == "redb" {
                            Box::new(RedbStore::new(&root.path.join("test.redb")).unwrap())
                        } else {
                            Box::new(SledStore::new(&root.path.join("test.sled")).unwrap())
                        };
                        let mut definition_store = benchmark_definition::BenchmarkDefinitionStore::new(store);
                        let user = create_sample_user(1);
                        definition_store.user_create(&user).unwrap();
                        (definition_store, _temp)
                    },
                    |(mut store, _temp)| {
                        let _ = black_box(store.user_delete(&1));
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }
    
    group.finish();
}

fn bench_secondary_key_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("secondary_key_operations");
    
    for &store_type in &["redb", "sled"] {
        // Secondary key read benchmark
        group.bench_with_input(
            BenchmarkId::new("read_by_email", store_type),
            &store_type,
            |b, &store_type| {
                b.iter_batched(
                    || {
                        let (root, _temp) = create_test_root();
                        let store: Box<dyn StoreTrait> = if store_type == "redb" {
                            Box::new(RedbStore::new(&root.path.join("test.redb")).unwrap())
                        } else {
                            Box::new(SledStore::new(&root.path.join("test.sled")).unwrap())
                        };
                        let mut definition_store = benchmark_definition::BenchmarkDefinitionStore::new(store);
                        
                        // Create multiple users for realistic testing
                        for i in 1..=100 {
                            let user = create_sample_user(i);
                            definition_store.user_create(&user).unwrap();
                        }
                        (definition_store, _temp)
                    },
                    |(store, _temp)| {
                        let _ = black_box(store.user_read_by_email(&"user50@example.com".to_string()));
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );

        // Secondary key range query benchmark
        group.bench_with_input(
            BenchmarkId::new("list_by_category", store_type),
            &store_type,
            |b, &store_type| {
                b.iter_batched(
                    || {
                        let (root, _temp) = create_test_root();
                        let store: Box<dyn StoreTrait> = if store_type == "redb" {
                            Box::new(RedbStore::new(&root.path.join("test.redb")).unwrap())
                        } else {
                            Box::new(SledStore::new(&root.path.join("test.sled")).unwrap())
                        };
                        let mut definition_store = benchmark_definition::BenchmarkDefinitionStore::new(store);
                        
                        // Create products across different categories
                        for i in 1..=1000 {
                            let product = create_sample_product(i);
                            definition_store.product_create(&product).unwrap();
                        }
                        (definition_store, _temp)
                    },
                    |(store, _temp)| {
                        let _ = black_box(store.product_list_by_category(&"category5".to_string(), None, None));
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }
    
    group.finish();
}

fn bench_relational_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("relational_operations");
    
    for &store_type in &["redb", "sled"] {
        // Relational query benchmark
        group.bench_with_input(
            BenchmarkId::new("orders_by_user", store_type),
            &store_type,
            |b, &store_type| {
                b.iter_batched(
                    || {
                        let (root, _temp) = create_test_root();
                        let store: Box<dyn StoreTrait> = if store_type == "redb" {
                            Box::new(RedbStore::new(&root.path.join("test.redb")).unwrap())
                        } else {
                            Box::new(SledStore::new(&root.path.join("test.sled")).unwrap())
                        };
                        let mut definition_store = benchmark_definition::BenchmarkDefinitionStore::new(store);
                        
                        // Create users and orders
                        for i in 1..=50 {
                            let user = create_sample_user(i);
                            definition_store.user_create(&user).unwrap();
                        }
                        
                        for i in 1..=500 {
                            let user_id = (i % 50) + 1;
                            let order = create_sample_order(i, user_id);
                            definition_store.order_create(&order).unwrap();
                        }
                        (definition_store, _temp)
                    },
                    |(store, _temp)| {
                        let _ = black_box(store.order_list_by_user_id(&25, None, None));
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }
    
    group.finish();
}

fn bench_complex_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_operations");
    
    for &store_type in &["redb", "sled"] {
        // Complex transaction-like operation
        group.bench_with_input(
            BenchmarkId::new("create_user_with_orders", store_type),
            &store_type,
            |b, &store_type| {
                b.iter_batched(
                    || {
                        let (root, _temp) = create_test_root();
                        let store: Box<dyn StoreTrait> = if store_type == "redb" {
                            Box::new(RedbStore::new(&root.path.join("test.redb")).unwrap())
                        } else {
                            Box::new(SledStore::new(&root.path.join("test.sled")).unwrap())
                        };
                        let definition_store = benchmark_definition::BenchmarkDefinitionStore::new(store);
                        (definition_store, _temp)
                    },
                    |(mut store, _temp)| {
                        // Create a user
                        let user = create_sample_user(1000);
                        store.user_create(&user).unwrap();
                        
                        // Create products for the orders
                        for i in 2000..2010 {
                            let product = create_sample_product(i);
                            store.product_create(&product).unwrap();
                        }
                        
                        // Create multiple orders for the user
                        for i in 3000..3005 {
                            let order = create_sample_order(i, 1000);
                            store.order_create(&order).unwrap();
                        }
                        
                        // Query the user's orders
                        let _ = black_box(store.order_list_by_user_id(&1000, None, None));
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_primary_key_operations,
    bench_secondary_key_operations,
    bench_relational_operations,
    bench_complex_operations
);
criterion_main!(benches);