#![cfg(feature = "native")]
#![cfg(not(feature = "paxos"))]

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use netabase_macros::netabase_definition_module;
use netabase_store::databases::redb_store::RedbStore;
use netabase_store::databases::redb_zerocopy::{
    RedbStoreZeroCopy, with_read_transaction, with_write_transaction,
};
use pprof::criterion::PProfProfiler;

// Test schema
#[netabase_definition_module(BenchDefinition, BenchKeys)]
mod bench_schema {
    use netabase_deps::{bincode, serde};
    use netabase_macros::NetabaseModel;
    use netabase_store::netabase;

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
    #[netabase(BenchDefinition)]
    pub struct Article {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,
        #[secondary_key]
        pub author_id: u64,
    }
}

use bench_schema::*;

/// Benchmark: Insert operations
/// Compares standard redb (auto-commit per operation) vs zerocopy (explicit transactions)
fn bench_redb_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("redb_zerocopy_insert");

    for size in [100, 1000, 5000].iter() {
        // Standard redb API (auto-commit per insert)
        group.bench_with_input(
            BenchmarkId::new("standard_autocommit", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let temp_dir = tempfile::TempDir::new().unwrap();
                    let db_path = temp_dir.path().join("bench_standard.redb");
                    let store = RedbStore::<BenchDefinition>::new(&db_path).unwrap();
                    let article_tree = store.open_tree::<Article>();

                    for i in 0u64..size {
                        let article = Article {
                            id: i,
                            title: format!("Article {}", i),
                            content: format!("Content {}", i),
                            author_id: i % 10,
                        };
                        article_tree.put(article).unwrap();
                    }
                    black_box(article_tree.len().unwrap());
                });
            },
        );

        // Zerocopy API with single transaction for all inserts
        group.bench_with_input(
            BenchmarkId::new("zerocopy_single_txn", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let temp_dir = tempfile::TempDir::new().unwrap();
                    let db_path = temp_dir.path().join("bench_zerocopy.redb");
                    let store = RedbStoreZeroCopy::<BenchDefinition>::new(&db_path).unwrap();

                    let mut txn = store.begin_write().unwrap();
                    let mut tree = txn.open_tree::<Article>().unwrap();

                    for i in 0u64..size {
                        let article = Article {
                            id: i,
                            title: format!("Article {}", i),
                            content: format!("Content {}", i),
                            author_id: i % 10,
                        };
                        tree.put(article).unwrap();
                    }

                    let count = tree.len().unwrap();
                    drop(tree);
                    txn.commit().unwrap();
                    black_box(count);
                });
            },
        );

        // Zerocopy API with bulk insert (put_many)
        group.bench_with_input(BenchmarkId::new("zerocopy_bulk", size), size, |b, &size| {
            b.iter(|| {
                let temp_dir = tempfile::TempDir::new().unwrap();
                let db_path = temp_dir.path().join("bench_zerocopy_bulk.redb");
                let store = RedbStoreZeroCopy::<BenchDefinition>::new(&db_path).unwrap();

                let articles: Vec<Article> = (0u64..size)
                    .map(|i| Article {
                        id: i,
                        title: format!("Article {}", i),
                        content: format!("Content {}", i),
                        author_id: i % 10,
                    })
                    .collect();

                let count = with_write_transaction(&store, |txn| {
                    let mut tree = txn.open_tree::<Article>()?;
                    tree.put_many(articles)?;
                    tree.len()
                })
                .unwrap();

                black_box(count);
            });
        });
    }

    group.finish();
}

/// Benchmark: Get operations
/// Compares read performance between standard and zerocopy APIs
fn bench_redb_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("redb_zerocopy_get");

    for size in [100, 1000, 5000].iter() {
        // Setup standard redb
        let temp_dir_std = tempfile::TempDir::new().unwrap();
        let db_path_std = temp_dir_std.path().join("bench_standard.redb");
        let store_std = RedbStore::<BenchDefinition>::new(&db_path_std).unwrap();
        let article_tree_std = store_std.open_tree::<Article>();

        for i in 0..*size {
            let article = Article {
                id: i,
                title: format!("Article {}", i),
                content: format!("Content {}", i),
                author_id: i % 10,
            };
            article_tree_std.put(article).unwrap();
        }

        // Setup zerocopy
        let temp_dir_zc = tempfile::TempDir::new().unwrap();
        let db_path_zc = temp_dir_zc.path().join("bench_zerocopy.redb");
        let store_zc = RedbStoreZeroCopy::<BenchDefinition>::new(&db_path_zc).unwrap();

        {
            let mut txn = store_zc.begin_write().unwrap();
            let mut tree = txn.open_tree::<Article>().unwrap();
            for i in 0..*size {
                let article = Article {
                    id: i,
                    title: format!("Article {}", i),
                    content: format!("Content {}", i),
                    author_id: i % 10,
                };
                tree.put(article).unwrap();
            }
            drop(tree);
            txn.commit().unwrap();
        }

        // Benchmark standard API (creates transaction per get)
        group.bench_with_input(
            BenchmarkId::new("standard_per_get", size),
            size,
            |b, &size| {
                b.iter(|| {
                    for i in 0u64..size {
                        let article = article_tree_std
                            .get(ArticleKey::Primary(ArticlePrimaryKey(i)))
                            .unwrap();
                        black_box(article);
                    }
                });
            },
        );

        // Benchmark zerocopy with single read transaction
        group.bench_with_input(
            BenchmarkId::new("zerocopy_single_txn", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let txn = store_zc.begin_read().unwrap();
                    let tree = txn.open_tree::<Article>().unwrap();

                    for i in 0u64..size {
                        let article = tree.get(&ArticlePrimaryKey(i)).unwrap();
                        black_box(article);
                    }
                });
            },
        );

        // Benchmark zerocopy with helper function
        group.bench_with_input(
            BenchmarkId::new("zerocopy_helper", size),
            size,
            |b, &size| {
                b.iter(|| {
                    with_read_transaction(&store_zc, |txn| {
                        let tree = txn.open_tree::<Article>()?;

                        for i in 0u64..size {
                            let article = tree.get(&ArticlePrimaryKey(i))?;
                            black_box(article);
                        }
                        Ok(())
                    })
                    .unwrap();
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Bulk remove operations
fn bench_redb_bulk_remove(c: &mut Criterion) {
    let mut group = c.benchmark_group("redb_zerocopy_bulk_remove");

    for size in [100, 1000, 5000].iter() {
        // Standard API (one transaction per remove)
        group.bench_with_input(BenchmarkId::new("standard_loop", size), size, |b, &size| {
            b.iter(|| {
                let temp_dir = tempfile::TempDir::new().unwrap();
                let db_path = temp_dir.path().join("bench.redb");
                let store = RedbStore::<BenchDefinition>::new(&db_path).unwrap();
                let article_tree = store.open_tree::<Article>();

                // Insert data
                for i in 0u64..size {
                    article_tree
                        .put(Article {
                            id: i,
                            title: format!("Article {}", i),
                            content: format!("Content {}", i),
                            author_id: i % 10,
                        })
                        .unwrap();
                }

                // Remove half
                for i in 0u64..(size / 2) {
                    article_tree
                        .remove(ArticleKey::Primary(ArticlePrimaryKey(i)))
                        .unwrap();
                }

                black_box(article_tree.len().unwrap());
            });
        });

        // Zerocopy API with bulk remove
        group.bench_with_input(BenchmarkId::new("zerocopy_bulk", size), size, |b, &size| {
            b.iter(|| {
                let temp_dir = tempfile::TempDir::new().unwrap();
                let db_path = temp_dir.path().join("bench.redb");
                let store = RedbStoreZeroCopy::<BenchDefinition>::new(&db_path).unwrap();

                // Insert data
                with_write_transaction(&store, |txn| {
                    let mut tree = txn.open_tree::<Article>()?;
                    for i in 0u64..size {
                        tree.put(Article {
                            id: i,
                            title: format!("Article {}", i),
                            content: format!("Content {}", i),
                            author_id: i % 10,
                        })?;
                    }
                    Ok(())
                })
                .unwrap();

                // Bulk remove half
                let count = with_write_transaction(&store, |txn| {
                    let mut tree = txn.open_tree::<Article>()?;
                    let keys: Vec<_> = (0u64..(size / 2)).map(ArticlePrimaryKey).collect();
                    tree.remove_many(keys)?;
                    tree.len()
                })
                .unwrap();

                black_box(count);
            });
        });
    }

    group.finish();
}

/// Benchmark: Transaction isolation and MVCC
fn bench_redb_mvcc(c: &mut Criterion) {
    let mut group = c.benchmark_group("redb_zerocopy_mvcc");

    // Only zerocopy supports true MVCC with long-lived read transactions
    let size = 1000u64;

    group.bench_function("zerocopy_concurrent_read_write", |b| {
        b.iter(|| {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("bench.redb");
            let store = RedbStoreZeroCopy::<BenchDefinition>::new(&db_path).unwrap();

            // Initial data
            with_write_transaction(&store, |txn| {
                let mut tree = txn.open_tree::<Article>()?;
                for i in 0..size {
                    tree.put(Article {
                        id: i,
                        title: format!("Article {}", i),
                        content: format!("Content {}", i),
                        author_id: i % 10,
                    })?;
                }
                Ok(())
            })
            .unwrap();

            // Start long-lived read transaction
            let read_txn = store.begin_read().unwrap();
            let read_tree = read_txn.open_tree::<Article>().unwrap();
            let initial_count = read_tree.len().unwrap();

            // Write transaction modifies data
            with_write_transaction(&store, |txn| {
                let mut tree = txn.open_tree::<Article>()?;
                for i in 0..(size / 2) {
                    tree.remove(ArticlePrimaryKey(i))?;
                }
                Ok(())
            })
            .unwrap();

            // Read transaction still sees old snapshot
            let snapshot_count = read_tree.len().unwrap();

            black_box((initial_count, snapshot_count));
        });
    });

    group.finish();
}

// Configure criterion with profiler support
fn configure_criterion() -> Criterion {
    Criterion::default().with_profiler(PProfProfiler::new(
        100,
        pprof::criterion::Output::Flamegraph(None),
    ))
}

criterion_group! {
    name = benches;
    config = configure_criterion();
    targets = bench_redb_insert, bench_redb_get, bench_redb_bulk_remove, bench_redb_mvcc
}
criterion_main!(benches);
