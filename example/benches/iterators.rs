//! Iterator vs Vector Collection Benchmarks
//!
//! Compares performance of iterator-based operations versus vector-based operations.
//! The iterator approach should be more memory efficient and faster for large datasets
//! when only a subset of results is needed.

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use netabase_store::databases::redb::transaction::{RedbModelCrud, CrudOptions};
use netabase_store::relational::RelationalLink;
use example::boilerplate_lib::definition::{User, UserID};
use example::boilerplate_lib::models::blob_types::{AnotherLargeUserFile, LargeUserFile};
use example::boilerplate_lib::{CategoryID, Definition};
use rand::prelude::*;
use std::hint::black_box;

mod common;
use common::*;

fn random_id(prefix: &str, rng: &mut impl Rng) -> String {
    let n: u64 = rng.random();
    format!("{}_{:016x}", prefix, n)
}

fn generate_random_user(rng: &mut impl Rng) -> User {
    let names = [
        "Alice", "Bob", "Carol", "Dave", "Eve", "Frank", "Grace", "Heidi", "Ivan", "Judy",
    ];
    let name = names.choose(rng).unwrap().to_string();
    let age: u8 = rng.random_range(1..=100);
    let user_id = UserID(random_id("user", rng));
    let category_id = CategoryID(random_id("cat", rng));

    let partner = RelationalLink::new_dehydrated(UserID("none".to_string()));
    let category = RelationalLink::new_dehydrated(category_id);

    // Smaller blob for faster benchmarks
    let bio = LargeUserFile {
        data: vec![0u8; 100],
        metadata: "meta".to_string(),
    };
    let another = AnotherLargeUserFile(vec![0u8; 50]);

    User {
        id: user_id,
        first_name: name,
        last_name: "Test".to_string(),
        age,
        partner,
        category,
        bio,
        another,
    }
}

/// Benchmark comparing iter_entries vs list_entries for full iteration
fn bench_full_iteration(c: &mut Criterion) {
    let sizes = [100, 1_000, 10_000];

    let mut group = c.benchmark_group("Iteration/Full");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(10));

    for size in sizes.iter() {
        // Vector-based list_entries - collects all into Vec then iterates
        group.bench_with_input(BenchmarkId::new("list_entries_vec", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let mut rng = rand::rng();
                    let users: Vec<User> = (0..size).map(|_| generate_random_user(&mut rng)).collect();
                    let name = format!("bench_iter_vec_{}_{}", size, rng.random::<u64>());
                    let store = create_test_db::<Definition>(&name).expect("Failed to create DB");

                    let txn = store.begin_write().expect("Failed to begin txn");
                    {
                        let mut tables = txn.prepare_model::<User>().expect("Failed to prepare model");
                        for user in &users {
                            user.create_entry(&mut tables).expect("Failed to create user");
                        }
                    }
                    txn.commit().expect("Failed to commit");

                    store
                },
                |store| {
                    let txn = store.begin_read().expect("Failed to begin txn");
                    let tables = txn.prepare_model::<User>().expect("Failed to prepare model");
                    
                    // Collect all into vector first, then iterate
                    let users = User::list_entries(&tables, CrudOptions::default())
                        .expect("Failed to list users");
                    
                    let mut count = 0;
                    for guard in users {
                        let user = guard.value();
                        black_box(&user);
                        count += 1;
                    }
                    black_box(count);
                },
                BatchSize::PerIteration,
            );
        });

        // Iterator-based iter_entries - streams directly without collecting
        group.bench_with_input(BenchmarkId::new("iter_entries_stream", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let mut rng = rand::rng();
                    let users: Vec<User> = (0..size).map(|_| generate_random_user(&mut rng)).collect();
                    let name = format!("bench_iter_stream_{}_{}", size, rng.random::<u64>());
                    let store = create_test_db::<Definition>(&name).expect("Failed to create DB");

                    let txn = store.begin_write().expect("Failed to begin txn");
                    {
                        let mut tables = txn.prepare_model::<User>().expect("Failed to prepare model");
                        for user in &users {
                            user.create_entry(&mut tables).expect("Failed to create user");
                        }
                    }
                    txn.commit().expect("Failed to commit");

                    store
                },
                |store| {
                    let txn = store.begin_read().expect("Failed to begin txn");
                    let tables = txn.prepare_model::<User>().expect("Failed to prepare model");
                    
                    // Stream directly via iterator without collecting
                    let iter = User::iter_entries(&tables).expect("Failed to get iterator");
                    
                    let mut count = 0;
                    for result in iter {
                        let (_key, value) = result.expect("Failed to read user");
                        let user = value.value();
                        black_box(&user);
                        count += 1;
                    }
                    black_box(count);
                },
                BatchSize::PerIteration,
            );
        });
    }

    group.finish();
}

/// Benchmark comparing early termination (take N)
/// Iterator should be much faster when only taking a few elements
fn bench_early_termination(c: &mut Criterion) {
    let db_sizes = [1_000, 10_000];
    let take_counts = [10, 100, 500];

    let mut group = c.benchmark_group("Iteration/EarlyTermination");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(10));

    for db_size in db_sizes.iter() {
        for take_count in take_counts.iter() {
            let bench_id = format!("db{}_take{}", db_size, take_count);
            
            // Vector-based - must collect ALL, then take N
            group.bench_with_input(
                BenchmarkId::new("list_all_take", &bench_id),
                &(*db_size, *take_count),
                |b, &(db_size, take_count)| {
                    b.iter_batched(
                        || {
                            let mut rng = rand::rng();
                            let users: Vec<User> = (0..db_size).map(|_| generate_random_user(&mut rng)).collect();
                            let name = format!("bench_take_vec_{}_{}", db_size, rng.random::<u64>());
                            let store = create_test_db::<Definition>(&name).expect("Failed to create DB");

                            let txn = store.begin_write().expect("Failed to begin txn");
                            {
                                let mut tables = txn.prepare_model::<User>().expect("Failed to prepare model");
                                for user in &users {
                                    user.create_entry(&mut tables).expect("Failed to create user");
                                }
                            }
                            txn.commit().expect("Failed to commit");

                            (store, take_count)
                        },
                        |(store, take_count)| {
                            let txn = store.begin_read().expect("Failed to begin txn");
                            let tables = txn.prepare_model::<User>().expect("Failed to prepare model");
                            
                            // Must collect ALL into vector first
                            let users = User::list_entries(&tables, CrudOptions::default())
                                .expect("Failed to list users");
                            
                            // Then take only what we need
                            let mut count = 0;
                            for guard in users.into_iter().take(take_count) {
                                let user = guard.value();
                                black_box(&user);
                                count += 1;
                            }
                            black_box(count);
                        },
                        BatchSize::PerIteration,
                    );
                },
            );

            // Iterator-based - stops after N elements
            group.bench_with_input(
                BenchmarkId::new("iter_all_take", &bench_id),
                &(*db_size, *take_count),
                |b, &(db_size, take_count)| {
                    b.iter_batched(
                        || {
                            let mut rng = rand::rng();
                            let users: Vec<User> = (0..db_size).map(|_| generate_random_user(&mut rng)).collect();
                            let name = format!("bench_take_iter_{}_{}", db_size, rng.random::<u64>());
                            let store = create_test_db::<Definition>(&name).expect("Failed to create DB");

                            let txn = store.begin_write().expect("Failed to begin txn");
                            {
                                let mut tables = txn.prepare_model::<User>().expect("Failed to prepare model");
                                for user in &users {
                                    user.create_entry(&mut tables).expect("Failed to create user");
                                }
                            }
                            txn.commit().expect("Failed to commit");

                            (store, take_count)
                        },
                        |(store, take_count)| {
                            let txn = store.begin_read().expect("Failed to begin txn");
                            let tables = txn.prepare_model::<User>().expect("Failed to prepare model");
                            
                            // Iterator can stop after take_count elements
                            let iter = User::iter_entries(&tables).expect("Failed to get iterator");
                            
                            let mut count = 0;
                            for result in iter.take(take_count) {
                                let (_key, value) = result.expect("Failed to read user");
                                let user = value.value();
                                black_box(&user);
                                count += 1;
                            }
                            black_box(count);
                        },
                        BatchSize::PerIteration,
                    );
                },
            );
        }
    }

    group.finish();
}

/// Benchmark range iteration
fn bench_range_iteration(c: &mut Criterion) {
    let db_sizes = [1_000, 10_000];

    let mut group = c.benchmark_group("Iteration/Range");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(10));

    for db_size in db_sizes.iter() {
        // Vector-based list_range
        group.bench_with_input(
            BenchmarkId::new("list_range_vec", db_size),
            db_size,
            |b, &db_size| {
                b.iter_batched(
                    || {
                        let mut rng = rand::rng();
                        // Generate users with predictable IDs for range queries
                        let users: Vec<User> = (0..db_size).map(|i| {
                            let mut user = generate_random_user(&mut rng);
                            user.id = UserID(format!("user_{:08}", i));
                            user
                        }).collect();
                        
                        let name = format!("bench_range_vec_{}_{}", db_size, rng.random::<u64>());
                        let store = create_test_db::<Definition>(&name).expect("Failed to create DB");

                        let txn = store.begin_write().expect("Failed to begin txn");
                        {
                            let mut tables = txn.prepare_model::<User>().expect("Failed to prepare model");
                            for user in &users {
                                user.create_entry(&mut tables).expect("Failed to create user");
                            }
                        }
                        txn.commit().expect("Failed to commit");

                        store
                    },
                    |store| {
                        let txn = store.begin_read().expect("Failed to begin txn");
                        let tables = txn.prepare_model::<User>().expect("Failed to prepare model");
                        
                        // Get a range in the middle
                        let start = UserID(format!("user_{:08}", 100));
                        let end = UserID(format!("user_{:08}", 200));
                        
                        let users = User::list_range(&tables, start..end, CrudOptions::default())
                            .expect("Failed to list range");
                        
                        let mut count = 0;
                        for guard in users {
                            let user = guard.value();
                            black_box(&user);
                            count += 1;
                        }
                        black_box(count);
                    },
                    BatchSize::PerIteration,
                );
            },
        );

        // Iterator-based iter_range
        group.bench_with_input(
            BenchmarkId::new("iter_range_stream", db_size),
            db_size,
            |b, &db_size| {
                b.iter_batched(
                    || {
                        let mut rng = rand::rng();
                        let users: Vec<User> = (0..db_size).map(|i| {
                            let mut user = generate_random_user(&mut rng);
                            user.id = UserID(format!("user_{:08}", i));
                            user
                        }).collect();
                        
                        let name = format!("bench_range_iter_{}_{}", db_size, rng.random::<u64>());
                        let store = create_test_db::<Definition>(&name).expect("Failed to create DB");

                        let txn = store.begin_write().expect("Failed to begin txn");
                        {
                            let mut tables = txn.prepare_model::<User>().expect("Failed to prepare model");
                            for user in &users {
                                user.create_entry(&mut tables).expect("Failed to create user");
                            }
                        }
                        txn.commit().expect("Failed to commit");

                        store
                    },
                    |store| {
                        let txn = store.begin_read().expect("Failed to begin txn");
                        let tables = txn.prepare_model::<User>().expect("Failed to prepare model");
                        
                        let start = UserID(format!("user_{:08}", 100));
                        let end = UserID(format!("user_{:08}", 200));
                        
                        let iter = User::iter_range(&tables, start..end)
                            .expect("Failed to get iterator");
                        
                        let mut count = 0;
                        for result in iter {
                            let (_key, value) = result.expect("Failed to read user");
                            let user = value.value();
                            black_box(&user);
                            count += 1;
                        }
                        black_box(count);
                    },
                    BatchSize::PerIteration,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark memory efficiency - measure peak memory during iteration
/// (This is more of a qualitative benchmark as criterion doesn't measure memory directly)
fn bench_filtering(c: &mut Criterion) {
    let db_sizes = [1_000, 10_000];

    let mut group = c.benchmark_group("Iteration/Filter");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(10));

    for db_size in db_sizes.iter() {
        // Vector-based - collect all, then filter
        group.bench_with_input(
            BenchmarkId::new("list_filter_vec", db_size),
            db_size,
            |b, &db_size| {
                b.iter_batched(
                    || {
                        let mut rng = rand::rng();
                        let users: Vec<User> = (0..db_size).map(|_| generate_random_user(&mut rng)).collect();
                        let name = format!("bench_filter_vec_{}_{}", db_size, rng.random::<u64>());
                        let store = create_test_db::<Definition>(&name).expect("Failed to create DB");

                        let txn = store.begin_write().expect("Failed to begin txn");
                        {
                            let mut tables = txn.prepare_model::<User>().expect("Failed to prepare model");
                            for user in &users {
                                user.create_entry(&mut tables).expect("Failed to create user");
                            }
                        }
                        txn.commit().expect("Failed to commit");

                        store
                    },
                    |store| {
                        let txn = store.begin_read().expect("Failed to begin txn");
                        let tables = txn.prepare_model::<User>().expect("Failed to prepare model");
                        
                        // Collect ALL users first
                        let users = User::list_entries(&tables, CrudOptions::default())
                            .expect("Failed to list users");
                        
                        // Then filter
                        let filtered: Vec<_> = users.into_iter()
                            .map(|g| g.value())
                            .filter(|u| u.age > 50)
                            .collect();
                        
                        black_box(filtered.len());
                    },
                    BatchSize::PerIteration,
                );
            },
        );

        // Iterator-based - filter during iteration, no intermediate allocation
        group.bench_with_input(
            BenchmarkId::new("iter_filter_stream", db_size),
            db_size,
            |b, &db_size| {
                b.iter_batched(
                    || {
                        let mut rng = rand::rng();
                        let users: Vec<User> = (0..db_size).map(|_| generate_random_user(&mut rng)).collect();
                        let name = format!("bench_filter_iter_{}_{}", db_size, rng.random::<u64>());
                        let store = create_test_db::<Definition>(&name).expect("Failed to create DB");

                        let txn = store.begin_write().expect("Failed to begin txn");
                        {
                            let mut tables = txn.prepare_model::<User>().expect("Failed to prepare model");
                            for user in &users {
                                user.create_entry(&mut tables).expect("Failed to create user");
                            }
                        }
                        txn.commit().expect("Failed to commit");

                        store
                    },
                    |store| {
                        let txn = store.begin_read().expect("Failed to begin txn");
                        let tables = txn.prepare_model::<User>().expect("Failed to prepare model");
                        
                        // Stream and filter without collecting all
                        let iter = User::iter_entries(&tables).expect("Failed to get iterator");
                        
                        let filtered: Vec<_> = iter
                            .filter_map(|r| r.ok())
                            .map(|(_key, value)| value.value())
                            .filter(|u| u.age > 50)
                            .collect();
                        
                        black_box(filtered.len());
                    },
                    BatchSize::PerIteration,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_full_iteration,
    bench_early_termination,
    bench_range_iteration,
    bench_filtering
);
criterion_main!(benches);
