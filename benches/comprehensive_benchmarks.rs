//! Comprehensive Performance Benchmarks
//!
//! These benchmarks provide detailed performance measurements across different
//! backends, operations, and scenarios to identify performance bottlenecks
//! and validate performance claims.

#![cfg(not(target_arch = "wasm32"))] // Benchmarks are native-only

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use netabase_store::netabase_definition_module;
use std::time::Duration;

// Benchmark schema
#[netabase_definition_module(BenchmarkDefinition, BenchmarkKeys)]
mod benchmark_schema {
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
    #[netabase(BenchmarkDefinition)]
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
    #[netabase(BenchmarkDefinition)]
    pub struct MediumRecord {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,
        #[secondary_key]
        pub author_id: u64,
        #[secondary_key]
        pub published: bool,
        pub tags: Vec<String>,
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
    #[netabase(BenchmarkDefinition)]
    pub struct LargeRecord {
        #[primary_key]
        pub id: u64,
        pub data: Vec<u8>,
        pub metadata: String,
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
    #[netabase(BenchmarkDefinition)]
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
        pub data1: Vec<u8>,
        pub data2: Vec<u8>,
        pub data3: Vec<u8>,
    }
}

use benchmark_schema::*;

fn generate_small_record(id: u64) -> SmallRecord {
    SmallRecord {
        id,
        name: format!("Record_{:06}", id),
        category: format!("Category_{}", id % 100),
    }
}

fn generate_medium_record(id: u64) -> MediumRecord {
    MediumRecord {
        id,
        title: format!("Title for record {}", id),
        content: format!(
            "This is the content for record {}. It contains some reasonable amount of text to simulate a typical medium-sized record. The content includes multiple sentences and provides a realistic test case for performance measurement.",
            id
        ),
        author_id: id % 1000,
        published: id % 2 == 0,
        tags: vec![
            format!("tag_{}", id % 50),
            format!("tag_{}", (id + 1) % 50),
            format!("tag_{}", (id + 2) % 50),
        ],
    }
}

fn generate_large_record(id: u64, size: usize) -> LargeRecord {
    LargeRecord {
        id,
        data: vec![(id % 256) as u8; size],
        metadata: format!("Large record {} with {} bytes of data", id, size),
    }
}

fn generate_wide_record(id: u64) -> WideRecord {
    let data_size = 256;
    WideRecord {
        id,
        field1: format!("field1_{}", id % 100),
        field2: format!("field2_{}", id % 200),
        field3: format!("field3_{}", id % 300),
        field4: format!("field4_{}", id % 400),
        field5: format!("field5_{}", id % 500),
        data1: vec![(id % 256) as u8; data_size],
        data2: vec![((id + 1) % 256) as u8; data_size],
        data3: vec![((id + 2) % 256) as u8; data_size],
    }
}

// Sled Benchmarks
#[cfg(feature = "sled")]
mod sled_benchmarks {
    use super::*;
    use netabase_store::databases::sled_store::SledStore;

    pub fn bench_sled_small_record_operations(c: &mut Criterion) {
        let mut group = c.benchmark_group("sled_small_records");
        group.measurement_time(Duration::from_secs(10));

        for size in [100, 1000, 10000].iter() {
            group.throughput(Throughput::Elements(*size as u64));

            // Insert benchmark
            group.bench_with_input(BenchmarkId::new("insert", size), size, |b, &size| {
                b.iter_batched(
                    || {
                        let store = SledStore::<BenchmarkDefinition>::temp().unwrap();
                        let tree = store.open_tree::<SmallRecord>();
                        let records: Vec<SmallRecord> =
                            (0..size).map(|i| generate_small_record(i as u64)).collect();
                        (tree, records)
                    },
                    |(tree, records)| {
                        for record in records {
                            tree.put(record).unwrap();
                        }
                    },
                    criterion::BatchSize::SmallInput,
                );
            });

            // Read benchmark (after setup)
            group.bench_with_input(BenchmarkId::new("read", size), size, |b, &size| {
                let store = SledStore::<BenchmarkDefinition>::temp().unwrap();
                let tree = store.open_tree::<SmallRecord>();
                for i in 0..size {
                    tree.put(generate_small_record(i as u64)).unwrap();
                }

                b.iter(|| {
                    for i in 0..size {
                        criterion::black_box(tree.get(SmallRecordPrimaryKey(i as u64)).unwrap());
                    }
                });
            });

            // Batch insert benchmark
            group.bench_with_input(BenchmarkId::new("batch_insert", size), size, |b, &size| {
                b.iter_batched(
                    || {
                        let store = SledStore::<BenchmarkDefinition>::temp().unwrap();
                        let tree = store.open_tree::<SmallRecord>();
                        let records: Vec<SmallRecord> =
                            (0..size).map(|i| generate_small_record(i as u64)).collect();
                        (tree, records)
                    },
                    |(tree, records)| {
                        tree.put_many(records).unwrap();
                    },
                    criterion::BatchSize::SmallInput,
                );
            });
        }

        group.finish();
    }

    pub fn bench_sled_secondary_key_queries(c: &mut Criterion) {
        let mut group = c.benchmark_group("sled_secondary_keys");
        group.measurement_time(Duration::from_secs(15));

        let dataset_size = 10000;
        let store = SledStore::<BenchmarkDefinition>::temp().unwrap();
        let tree = store.open_tree::<SmallRecord>();

        // Setup data
        for i in 0..dataset_size {
            tree.put(generate_small_record(i)).unwrap();
        }

        group.bench_function("query_by_category", |b| {
            b.iter(|| {
                let category = format!("Category_{}", criterion::black_box(42));
                let results = tree
                    .get_by_secondary_key(SmallRecordSecondaryKeys::Category(
                        SmallRecordCategorySecondaryKey(category),
                    ))
                    .unwrap();
                criterion::black_box(results);
            });
        });

        group.finish();
    }

    pub fn bench_sled_large_records(c: &mut Criterion) {
        let mut group = c.benchmark_group("sled_large_records");
        group.measurement_time(Duration::from_secs(20));

        for size in [1024, 64 * 1024, 256 * 1024, 1024 * 1024].iter() {
            group.throughput(Throughput::Bytes(*size as u64));

            group.bench_with_input(BenchmarkId::new("insert", size), size, |b, &size| {
                b.iter_batched(
                    || {
                        let store = SledStore::<BenchmarkDefinition>::temp().unwrap();
                        let tree = store.open_tree::<LargeRecord>();
                        (tree, generate_large_record(1, size))
                    },
                    |(tree, record)| {
                        tree.put(record).unwrap();
                    },
                    criterion::BatchSize::SmallInput,
                );
            });

            group.bench_with_input(BenchmarkId::new("read", size), size, |b, &size| {
                let store = SledStore::<BenchmarkDefinition>::temp().unwrap();
                let tree = store.open_tree::<LargeRecord>();
                let record = generate_large_record(1, size);
                tree.put(record).unwrap();

                b.iter(|| {
                    let result = tree.get(LargeRecordPrimaryKey(1)).unwrap();
                    criterion::black_box(result);
                });
            });
        }

        group.finish();
    }

    pub fn bench_sled_wide_records(c: &mut Criterion) {
        let mut group = c.benchmark_group("sled_wide_records");
        group.measurement_time(Duration::from_secs(15));

        let num_records = 1000;

        group.bench_function("insert_wide_records", |b| {
            b.iter_batched(
                || {
                    let store = SledStore::<BenchmarkDefinition>::temp().unwrap();
                    let tree = store.open_tree::<WideRecord>();
                    let records: Vec<WideRecord> =
                        (0..num_records).map(|i| generate_wide_record(i)).collect();
                    (tree, records)
                },
                |(tree, records)| {
                    for record in records {
                        tree.put(record).unwrap();
                    }
                },
                criterion::BatchSize::SmallInput,
            );
        });

        group.bench_function("query_wide_records", |b| {
            let store = SledStore::<BenchmarkDefinition>::temp().unwrap();
            let tree = store.open_tree::<WideRecord>();
            for i in 0..num_records {
                tree.put(generate_wide_record(i)).unwrap();
            }

            b.iter(|| {
                let field_value = format!("field1_{}", criterion::black_box(50));
                let results = tree
                    .get_by_secondary_key(WideRecordSecondaryKeys::Field1(
                        WideRecordField1SecondaryKey(field_value),
                    ))
                    .unwrap();
                criterion::black_box(results);
            });
        });

        group.finish();
    }
}

// Redb Benchmarks
#[cfg(feature = "redb")]
mod redb_benchmarks {
    use super::*;
    use netabase_store::config::FileConfig;
    use netabase_store::databases::redb_store::RedbStore;
    use netabase_store::traits::backend_store::BackendStore;

    pub fn bench_redb_small_record_operations(c: &mut Criterion) {
        let mut group = c.benchmark_group("redb_small_records");
        group.measurement_time(Duration::from_secs(10));

        for size in [100, 1000, 10000].iter() {
            group.throughput(Throughput::Elements(*size as u64));

            group.bench_with_input(BenchmarkId::new("insert", size), size, |b, &size| {
                b.iter_batched(
                    || {
                        let temp_dir = tempfile::tempdir().unwrap();
                        let config = FileConfig::new(temp_dir.path().join("bench.redb"));
                        let store = RedbStore::<BenchmarkDefinition>::new(config).unwrap();
                        let tree = store.open_tree::<SmallRecord>();
                        let records: Vec<SmallRecord> =
                            (0..size).map(|i| generate_small_record(i as u64)).collect();
                        (tree, records)
                    },
                    |(tree, records)| {
                        for record in records {
                            tree.put(record).unwrap();
                        }
                    },
                    criterion::BatchSize::SmallInput,
                );
            });

            group.bench_with_input(BenchmarkId::new("read", size), size, |b, &size| {
                let temp_dir = tempfile::tempdir().unwrap();
                let config = FileConfig::new(temp_dir.path().join("bench.redb"));
                let store = RedbStore::<BenchmarkDefinition>::new(config).unwrap();
                let tree = store.open_tree::<SmallRecord>();
                for i in 0..size {
                    tree.put(generate_small_record(i as u64)).unwrap();
                }

                b.iter(|| {
                    for i in 0..size {
                        let key = SmallRecordKey::Primary(SmallRecordPrimaryKey(i as u64));
                        criterion::black_box(tree.get(key).unwrap());
                    }
                });
            });
        }

        group.finish();
    }

    pub fn bench_redb_secondary_key_queries(c: &mut Criterion) {
        let mut group = c.benchmark_group("redb_secondary_keys");
        group.measurement_time(Duration::from_secs(15));

        let dataset_size = 10000;
        let temp_dir = tempfile::tempdir().unwrap();
        let config = FileConfig::new(temp_dir.path().join("bench.redb"));
        let store = RedbStore::<BenchmarkDefinition>::new(config).unwrap();
        let tree = store.open_tree::<SmallRecord>();

        // Setup data
        for i in 0..dataset_size {
            tree.put(generate_small_record(i)).unwrap();
        }

        group.bench_function("query_by_category", |b| {
            b.iter(|| {
                let category = format!("Category_{}", criterion::black_box(42));
                let key = SmallRecordKey::Secondary(SmallRecordSecondaryKeys::Category(
                    SmallRecordCategorySecondaryKey(category),
                ));
                let results = tree.get_by_secondary_key_redb(key).unwrap();
                criterion::black_box(results);
            });
        });

        group.finish();
    }
}

// Redb Zero-Copy Benchmarks
#[cfg(feature = "redb-zerocopy")]
mod redb_zerocopy_benchmarks {
    use super::*;
    use netabase_store::databases::redb_zerocopy::RedbStoreZeroCopy;

    pub fn bench_redb_zerocopy_operations(c: &mut Criterion) {
        let mut group = c.benchmark_group("redb_zerocopy");
        group.measurement_time(Duration::from_secs(10));

        for size in [100, 1000, 10000].iter() {
            group.throughput(Throughput::Elements(*size as u64));

            group.bench_with_input(
                BenchmarkId::new("transactional_insert", size),
                size,
                |b, &size| {
                    b.iter_batched(
                        || {
                            let temp_dir = tempfile::tempdir().unwrap();
                            let store = RedbStoreZeroCopy::<BenchmarkDefinition>::new(
                                temp_dir.path().join("bench.redb"),
                            )
                            .unwrap();
                            let records: Vec<SmallRecord> =
                                (0..size).map(|i| generate_small_record(i as u64)).collect();
                            (store, records)
                        },
                        |(store, records)| {
                            let mut txn = store.begin_write().unwrap();
                            let mut tree = txn.open_tree::<SmallRecord>().unwrap();
                            for record in records {
                                tree.put(record).unwrap();
                            }
                            drop(tree);
                            txn.commit().unwrap();
                        },
                        criterion::BatchSize::SmallInput,
                    );
                },
            );

            group.bench_with_input(BenchmarkId::new("bulk_insert", size), size, |b, &size| {
                b.iter_batched(
                    || {
                        let temp_dir = tempfile::tempdir().unwrap();
                        let store = RedbStoreZeroCopy::<BenchmarkDefinition>::new(
                            temp_dir.path().join("bench.redb"),
                        )
                        .unwrap();
                        let records: Vec<SmallRecord> =
                            (0..size).map(|i| generate_small_record(i as u64)).collect();
                        (store, records)
                    },
                    |(store, records)| {
                        let mut txn = store.begin_write().unwrap();
                        let mut tree = txn.open_tree::<SmallRecord>().unwrap();
                        tree.put_many(records).unwrap();
                        drop(tree);
                        txn.commit().unwrap();
                    },
                    criterion::BatchSize::SmallInput,
                );
            });
        }

        group.finish();
    }

    pub fn bench_redb_zerocopy_transaction_overhead(c: &mut Criterion) {
        let mut group = c.benchmark_group("redb_zerocopy_transactions");
        group.measurement_time(Duration::from_secs(10));

        let temp_dir = tempfile::tempdir().unwrap();
        let store =
            RedbStoreZeroCopy::<BenchmarkDefinition>::new(temp_dir.path().join("bench.redb"))
                .unwrap();

        group.bench_function("transaction_creation", |b| {
            b.iter(|| {
                let txn = store.begin_write().unwrap();
                drop(txn);
            });
        });

        group.bench_function("read_transaction_with_query", |b| {
            // Setup some data
            {
                let mut txn = store.begin_write().unwrap();
                let mut tree = txn.open_tree::<SmallRecord>().unwrap();
                for i in 0..100 {
                    tree.put(generate_small_record(i)).unwrap();
                }
                drop(tree);
                txn.commit().unwrap();
            }

            b.iter(|| {
                let txn = store.begin_read().unwrap();
                let tree = txn.open_tree::<SmallRecord>().unwrap();
                for i in 0..10 {
                    let result = tree.get(&SmallRecordPrimaryKey(i)).unwrap();
                    criterion::black_box(result);
                }
                drop(tree);
                drop(txn);
            });
        });

        group.finish();
    }
}

// Cross-Backend Comparison
fn bench_cross_backend_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("cross_backend_comparison");
    group.measurement_time(Duration::from_secs(15));

    let dataset_size = 1000;

    #[cfg(feature = "sled")]
    group.bench_function("sled_mixed_workload", |b| {
        use netabase_store::databases::sled_store::SledStore;

        b.iter_batched(
            || {
                let store = SledStore::<BenchmarkDefinition>::temp().unwrap();
                let tree = store.open_tree::<MediumRecord>();
                // Pre-populate with some data
                for i in 0..dataset_size / 2 {
                    tree.put(generate_medium_record(i)).unwrap();
                }
                tree
            },
            |tree| {
                // Mixed workload: inserts, reads, updates, queries
                for i in 0..100 {
                    match i % 4 {
                        0 => {
                            // Insert
                            tree.put(generate_medium_record(dataset_size / 2 + i))
                                .unwrap();
                        }
                        1 => {
                            // Read
                            let key = MediumRecordPrimaryKey(i % (dataset_size / 2));
                            criterion::black_box(tree.get(key).unwrap());
                        }
                        2 => {
                            // Update
                            let mut record = generate_medium_record(i % (dataset_size / 2));
                            record.content = format!("Updated content for {}", i);
                            tree.put(record).unwrap();
                        }
                        3 => {
                            // Secondary key query
                            let key = MediumRecordSecondaryKeys::AuthorId(
                                MediumRecordAuthorIdSecondaryKey(i % 100),
                            );
                            criterion::black_box(tree.get_by_secondary_key(key).unwrap());
                        }
                        _ => unreachable!(),
                    }
                }
            },
            criterion::BatchSize::SmallInput,
        );
    });

    #[cfg(feature = "redb")]
    group.bench_function("redb_mixed_workload", |b| {
        use netabase_store::config::FileConfig;
        use netabase_store::databases::redb_store::RedbStore;
        use netabase_store::traits::backend_store::BackendStore;

        b.iter_batched(
            || {
                let temp_dir = tempfile::tempdir().unwrap();
                let config = FileConfig::new(temp_dir.path().join("bench.redb"));
                let store = RedbStore::<BenchmarkDefinition>::new(config).unwrap();
                let tree = store.open_tree::<MediumRecord>();
                // Pre-populate with some data
                for i in 0..dataset_size / 2 {
                    tree.put(generate_medium_record(i)).unwrap();
                }
                tree
            },
            |tree| {
                // Mixed workload: inserts, reads, updates, queries
                for i in 0..100 {
                    match i % 4 {
                        0 => {
                            // Insert
                            tree.put(generate_medium_record(dataset_size / 2 + i))
                                .unwrap();
                        }
                        1 => {
                            // Read
                            let key = MediumRecordKey::Primary(MediumRecordPrimaryKey(
                                i % (dataset_size / 2),
                            ));
                            criterion::black_box(tree.get(key).unwrap());
                        }
                        2 => {
                            // Update
                            let mut record = generate_medium_record(i % (dataset_size / 2));
                            record.content = format!("Updated content for {}", i);
                            tree.put(record).unwrap();
                        }
                        3 => {
                            // Secondary key query
                            let key =
                                MediumRecordKey::Secondary(MediumRecordSecondaryKeys::AuthorId(
                                    MediumRecordAuthorIdSecondaryKey(i % 100),
                                ));
                            criterion::black_box(tree.get_by_secondary_key_redb(key).unwrap());
                        }
                        _ => unreachable!(),
                    }
                }
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

// Memory Usage Benchmark
fn bench_memory_usage_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.measurement_time(Duration::from_secs(20));

    #[cfg(feature = "sled")]
    group.bench_function("sled_memory_pressure", |b| {
        use netabase_store::databases::sled_store::SledStore;

        b.iter(|| {
            let store = SledStore::<BenchmarkDefinition>::temp().unwrap();
            let tree = store.open_tree::<LargeRecord>();

            // Insert progressively larger records
            for i in 0..10 {
                let size = 1024 * (i + 1); // 1KB to 10KB
                let record = generate_large_record(i, size);
                tree.put(record).unwrap();
            }

            // Read all records back
            for i in 0..10 {
                let result = tree.get(LargeRecordPrimaryKey(i)).unwrap();
                criterion::black_box(result);
            }
        });
    });

    group.finish();
}

// Serialization Overhead Benchmark
fn bench_serialization_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization_overhead");
    group.measurement_time(Duration::from_secs(10));

    for size in [1024, 10 * 1024, 100 * 1024].iter() {
        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(BenchmarkId::new("encode", size), size, |b, &size| {
            let record = generate_large_record(1, size);
            b.iter(|| {
                let encoded = bincode::encode_to_vec(&record, bincode::config::standard()).unwrap();
                criterion::black_box(encoded);
            });
        });

        group.bench_with_input(BenchmarkId::new("decode", size), size, |b, &size| {
            let record = generate_large_record(1, size);
            let encoded = bincode::encode_to_vec(&record, bincode::config::standard()).unwrap();

            b.iter(|| {
                let decoded: LargeRecord =
                    bincode::decode_from_slice(&encoded, bincode::config::standard())
                        .unwrap()
                        .0;
                criterion::black_box(decoded);
            });
        });
    }

    group.finish();
}

// Configuration option benchmarks
fn bench_configuration_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("configuration_impact");
    group.measurement_time(Duration::from_secs(15));

    #[cfg(feature = "sled")]
    {
        use netabase_store::config::FileConfig;
        use netabase_store::databases::sled_store::SledStore;
        use netabase_store::traits::backend_store::BackendStore;

        let cache_sizes = vec![64, 256, 1024]; // MB

        for cache_size in cache_sizes {
            group.bench_with_input(
                BenchmarkId::new("sled_cache_size", cache_size),
                &cache_size,
                |b, &cache_size| {
                    b.iter_batched(
                        || {
                            let temp_dir = tempfile::tempdir().unwrap();
                            let config = FileConfig::builder()
                                .path(temp_dir.path().join("bench.db"))
                                .cache_size_mb(cache_size)
                                .build();
                            let store = SledStore::<BenchmarkDefinition>::new(config.path).unwrap();
                            let tree = store.open_tree::<MediumRecord>();
                            tree
                        },
                        |tree| {
                            // Perform operations that benefit from caching
                            for i in 0..500 {
                                tree.put(generate_medium_record(i)).unwrap();
                            }
                            for i in 0..500 {
                                let result = tree.get(MediumRecordPrimaryKey(i)).unwrap();
                                criterion::black_box(result);
                            }
                        },
                        criterion::BatchSize::SmallInput,
                    );
                },
            );
        }
    }

    group.finish();
}

// Register all benchmarks
criterion_group!(
    benches,
    bench_cross_backend_comparison,
    bench_memory_usage_patterns,
    bench_serialization_overhead,
    bench_configuration_impact,
);

#[cfg(feature = "sled")]
criterion_group!(
    sled_benches,
    sled_benchmarks::bench_sled_small_record_operations,
    sled_benchmarks::bench_sled_secondary_key_queries,
    sled_benchmarks::bench_sled_large_records,
    sled_benchmarks::bench_sled_wide_records,
);

#[cfg(feature = "redb")]
criterion_group!(
    redb_benches,
    redb_benchmarks::bench_redb_small_record_operations,
    redb_benchmarks::bench_redb_secondary_key_queries,
);

#[cfg(feature = "redb-zerocopy")]
criterion_group!(
    redb_zerocopy_benches,
    redb_zerocopy_benchmarks::bench_redb_zerocopy_operations,
    redb_zerocopy_benchmarks::bench_redb_zerocopy_transaction_overhead,
);

// Main benchmark runner
#[cfg(all(feature = "sled", feature = "redb", feature = "redb-zerocopy"))]
criterion_main!(benches, sled_benches, redb_benches, redb_zerocopy_benches);

#[cfg(all(feature = "sled", feature = "redb", not(feature = "redb-zerocopy")))]
criterion_main!(benches, sled_benches, redb_benches);

#[cfg(all(feature = "sled", not(feature = "redb"), feature = "redb-zerocopy"))]
criterion_main!(benches, sled_benches, redb_zerocopy_benches);

#[cfg(all(
    feature = "sled",
    not(feature = "redb"),
    not(feature = "redb-zerocopy")
))]
criterion_main!(benches, sled_benches);

#[cfg(all(not(feature = "sled"), feature = "redb", feature = "redb-zerocopy"))]
criterion_main!(benches, redb_benches, redb_zerocopy_benches);

#[cfg(all(
    not(feature = "sled"),
    feature = "redb",
    not(feature = "redb-zerocopy")
))]
criterion_main!(benches, redb_benches);

#[cfg(all(
    not(feature = "sled"),
    not(feature = "redb"),
    feature = "redb-zerocopy"
))]
criterion_main!(benches, redb_zerocopy_benches);

#[cfg(all(
    not(feature = "sled"),
    not(feature = "redb"),
    not(feature = "redb-zerocopy")
))]
criterion_main!(benches);
