/// Minimal Benchmark - Bare Bones Single Table
/// 
/// Compares the absolute minimum overhead:
/// - Single model with only primary key
/// - No secondary indexes
/// - No relations
/// - No subscriptions
/// - No blobs
///
/// Abstracted vs Raw redb implementation

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use netabase_macros::NetabaseModel;
use netabase_store::databases::redb::RedbStore;
use netabase_store::traits::database::store::NBStore;
use netabase_store::databases::redb::transaction::RedbModelCrud;
use redb::{Database, ReadableDatabase, TableDefinition};
use serde::{Deserialize, Serialize};
use std::hint::black_box;


// Minimal Definition
#[netabase_macros::netabase_definition(MinimalDefinition)]
pub mod minimal_def {
    use super::*;

    #[derive(NetabaseModel, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct Item {
        #[primary_key]
        pub id: String,
        pub value: u64,
    }
}

use minimal_def::{Item, ItemID, MinimalDefinition};

fn bench_minimal_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("Minimal/Insert");
    
    for count in [0, 100, 1000, 10000].iter() {
        let items: Vec<Item> = (0..*count)
            .map(|i| Item {
                id: ItemID(format!("item_{:06}", i)),
                value: i as u64,
            })
            .collect();
        
        // Abstracted implementation (Naive)
        group.bench_with_input(
            BenchmarkId::new("Abstracted (Naive)", count),
            count,
            |b, _| {
                b.iter_batched(
                    || {
                        let store = RedbStore::<MinimalDefinition>::new_in_memory()
                            .expect("Failed to create store");
                        (store, items.clone())
                    },
                    |(store, items)| {
                        let txn = store.begin_write().expect("Failed to begin write");
                        for item in &items {
                            txn.create::<Item>(item).expect("Failed to insert");
                        }
                        txn.commit().expect("Failed to commit");
                        black_box(());
                    },
                    criterion::BatchSize::PerIteration,
                );
            },
        );

        // Abstracted implementation (Batch/Optimized)
        group.bench_with_input(
            BenchmarkId::new("Abstracted (Batch)", count),
            count,
            |b, _| {
                b.iter_batched(
                    || {
                        let store = RedbStore::<MinimalDefinition>::new_in_memory()
                            .expect("Failed to create store");
                        (store, items.clone())
                    },
                    |(store, items)| {
                        let txn = store.begin_write().expect("Failed to begin write");
                        {
                            let mut tables = txn.prepare_model::<Item>().expect("Failed to prepare tables");
                            for item in &items {
                                item.create_entry(&mut tables).expect("Failed to insert");
                            }
                        }
                        txn.commit().expect("Failed to commit");
                        black_box(());
                    },
                    criterion::BatchSize::PerIteration,
                );
            },
        );
        
        // Raw redb implementation
        group.bench_with_input(
            BenchmarkId::new("Raw", count),
            count,
            |b, _| {
                b.iter_batched(
                    || {
                        let db = Database::builder()
                            .create_with_backend(redb::backends::InMemoryBackend::new())
                            .expect("Failed to create raw DB");
                        (db, items.clone())
                    },
                    |(db, items)| {
                        const TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("items");
                        let write_txn = db.begin_write().expect("Failed to begin write");
                        {
                            let mut table = write_txn.open_table(TABLE).expect("Failed to open table");
                            for item in &items {
                                let key = item.id.0.as_str();
                                let value = postcard::to_allocvec(&item).expect("Failed to serialize");
                                table.insert(key, value.as_slice()).expect("Failed to insert");
                            }
                        }
                        write_txn.commit().expect("Failed to commit");
                        black_box(());
                    },
                    criterion::BatchSize::PerIteration,
                );
            },
        );
    }
    
    group.finish();
}

fn bench_minimal_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("Minimal/Read");
    
    for count in [100, 1000, 10000].iter() {
        let items: Vec<Item> = (0..*count)
            .map(|i| Item {
                id: ItemID(format!("item_{:06}", i)),
                value: i as u64,
            })
            .collect();
        
        // Abstracted implementation (Naive)
        group.bench_with_input(
            BenchmarkId::new("Abstracted (Naive)", count),
            count,
            |b, _| {
                b.iter_batched(
                    || {
                        let store = RedbStore::<MinimalDefinition>::new_in_memory()
                            .expect("Failed to create store");
                        
                        // Insert data
                        let txn = store.begin_write().expect("Failed to begin write");
                        {
                            let mut tables = txn.prepare_model::<Item>().expect("Failed to prepare");
                            for item in &items {
                                item.create_entry(&mut tables).expect("Failed to insert");
                            }
                        }
                        txn.commit().expect("Failed to commit");
                        
                        store
                    },
                    |store| {
                        let txn = store.begin_read().expect("Failed to begin read");
                        for i in 0..*count {
                            let id = ItemID(format!("item_{:06}", i));
                            let _ = black_box(txn.read::<Item>(&id).expect("Failed to read"));
                        }
                        black_box(());
                    },
                    criterion::BatchSize::PerIteration,
                );
            },
        );

        // Abstracted implementation (Batch/Optimized)
        group.bench_with_input(
            BenchmarkId::new("Abstracted (Batch)", count),
            count,
            |b, _| {
                b.iter_batched(
                    || {
                        let store = RedbStore::<MinimalDefinition>::new_in_memory()
                            .expect("Failed to create store");
                        
                        // Insert data
                        let txn = store.begin_write().expect("Failed to begin write");
                        {
                            let mut tables = txn.prepare_model::<Item>().expect("Failed to prepare");
                            for item in &items {
                                item.create_entry(&mut tables).expect("Failed to insert");
                            }
                        }
                        txn.commit().expect("Failed to commit");
                        
                        store
                    },
                    |store| {
                        let txn = store.begin_read().expect("Failed to begin read");
                        {
                            let tables = txn.prepare_model::<Item>().expect("Failed to prepare");
                            for i in 0..*count {
                                let id = ItemID(format!("item_{:06}", i));
                                let _ = black_box(Item::read_default(&id, &tables).expect("Failed to read"));
                            }
                        }
                        black_box(());
                    },
                    criterion::BatchSize::PerIteration,
                );
            },
        );
        
        // Raw redb implementation
        group.bench_with_input(
            BenchmarkId::new("Raw", count),
            count,
            |b, _| {
                b.iter_batched(
                    || {
                        let db = Database::builder()
                            .create_with_backend(redb::backends::InMemoryBackend::new())
                            .expect("Failed to create raw DB");
                        
                        // Insert data
                        const TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("items");
                        let write_txn = db.begin_write().expect("Failed to begin write");
                        {
                            let mut table = write_txn.open_table(TABLE).expect("Failed to open table");
                            for item in &items {
                                let key = item.id.0.as_str();
                                let value = postcard::to_allocvec(&item).expect("Failed to serialize");
                                table.insert(key, value.as_slice()).expect("Failed to insert");
                            }
                        }
                        write_txn.commit().expect("Failed to commit");
                        
                        db
                    },
                    |db| {
                        const TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("items");
                        let read_txn = db.begin_read().expect("Failed to begin read");
                        let table = read_txn.open_table(TABLE).expect("Failed to open table");
                        
                        for i in 0..*count {
                            let key = format!("item_{:06}", i);
                            let value = table.get(key.as_str()).expect("Failed to get").expect("Not found");
                            let _item: Item = postcard::from_bytes(value.value())
                                .expect("Failed to deserialize");
                            black_box(_item);
                        }
                        black_box(());
                    },
                    criterion::BatchSize::PerIteration,
                );
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, bench_minimal_insert, bench_minimal_read);
criterion_main!(benches);
