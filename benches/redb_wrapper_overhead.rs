#![cfg(feature = "native")]
#![cfg(not(feature = "paxos"))]

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use netabase_macros::netabase_definition_module;
use netabase_store::databases::redb_store::RedbStore;
use redb::{Database, ReadableDatabase, ReadableTable, ReadableTableMetadata, TableDefinition};
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

// Define raw redb table
const ARTICLES_TABLE: TableDefinition<u64, &[u8]> = TableDefinition::new("articles");
const AUTHOR_INDEX_TABLE: TableDefinition<(u64, u64), ()> = TableDefinition::new("author_index");

fn bench_redb_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("redb_insert");

    for size in [100, 1000, 5000].iter() {
        group.bench_with_input(BenchmarkId::new("redb_wrapper", size), size, |b, &size| {
            b.iter(|| {
                let temp_dir = tempfile::TempDir::new().unwrap();
                let db_path = temp_dir.path().join("bench.redb");
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
        });

        group.bench_with_input(BenchmarkId::new("redb_raw", size), size, |b, &size| {
            b.iter(|| {
                let temp_dir = tempfile::TempDir::new().unwrap();
                let db_path = temp_dir.path().join("bench.redb");
                let db = Database::create(&db_path).unwrap();

                let write_txn = db.begin_write().unwrap();
                {
                    let mut table = write_txn.open_table(ARTICLES_TABLE).unwrap();
                    let mut index_table = write_txn.open_table(AUTHOR_INDEX_TABLE).unwrap();

                    for i in 0u64..size {
                        let article = Article {
                            id: i,
                            title: format!("Article {}", i),
                            content: format!("Content {}", i),
                            author_id: i % 10,
                        };
                        let encoded =
                            bincode::encode_to_vec(&article, bincode::config::standard()).unwrap();
                        table.insert(i, encoded.as_slice()).unwrap();
                        index_table.insert((article.author_id, i), ()).unwrap();
                    }
                }
                write_txn.commit().unwrap();

                let read_txn = db.begin_read().unwrap();
                let table = read_txn.open_table(ARTICLES_TABLE).unwrap();
                black_box(table.len().unwrap());
            });
        });
    }

    group.finish();
}

fn bench_redb_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("redb_get");

    for size in [100, 1000, 5000].iter() {
        // Setup wrapper
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("bench_wrapper.redb");
        let store = RedbStore::<BenchDefinition>::new(&db_path).unwrap();
        let article_tree = store.open_tree::<Article>();

        for i in 0..*size {
            let article = Article {
                id: i,
                title: format!("Article {}", i),
                content: format!("Content {}", i),
                author_id: i % 10,
            };
            article_tree.put(article).unwrap();
        }

        group.bench_with_input(BenchmarkId::new("redb_wrapper", size), size, |b, &size| {
            b.iter(|| {
                for i in 0u64..size {
                    let article = article_tree.get(ArticlePrimaryKey(i)).unwrap();
                    black_box(article);
                }
            });
        });

        // Setup raw redb
        let temp_dir_raw = tempfile::TempDir::new().unwrap();
        let db_path_raw = temp_dir_raw.path().join("bench_raw.redb");
        let db = Database::create(&db_path_raw).unwrap();

        {
            let write_txn = db.begin_write().unwrap();
            {
                let mut table = write_txn.open_table(ARTICLES_TABLE).unwrap();
                let mut index_table = write_txn.open_table(AUTHOR_INDEX_TABLE).unwrap();

                for i in 0..*size {
                    let article = Article {
                        id: i,
                        title: format!("Article {}", i),
                        content: format!("Content {}", i),
                        author_id: i % 10,
                    };
                    let encoded =
                        bincode::encode_to_vec(&article, bincode::config::standard()).unwrap();
                    table.insert(i, encoded.as_slice()).unwrap();
                    index_table.insert((article.author_id, i), ()).unwrap();
                }
            }
            write_txn.commit().unwrap();
        }

        group.bench_with_input(BenchmarkId::new("redb_raw", size), size, |b, &size| {
            b.iter(|| {
                let read_txn = db.begin_read().unwrap();
                let table = read_txn.open_table(ARTICLES_TABLE).unwrap();

                for i in 0u64..size {
                    let encoded = table.get(i).unwrap().unwrap();
                    let article: Article =
                        bincode::decode_from_slice(encoded.value(), bincode::config::standard())
                            .unwrap()
                            .0;
                    black_box(article);
                }
            });
        });
    }

    group.finish();
}

fn bench_redb_iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("redb_iteration");

    for size in [100, 1000, 5000].iter() {
        // Setup wrapper
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("bench_wrapper.redb");
        let store = RedbStore::<BenchDefinition>::new(&db_path).unwrap();
        let article_tree = store.open_tree::<Article>();

        for i in 0..*size {
            let article = Article {
                id: i,
                title: format!("Article {}", i),
                content: format!("Content {}", i),
                author_id: i % 10,
            };
            article_tree.put(article).unwrap();
        }

        group.bench_with_input(BenchmarkId::new("redb_wrapper", size), size, |b, _size| {
            b.iter(|| {
                let count = article_tree.iter().unwrap().len();
                black_box(count);
            });
        });

        // Setup raw redb
        let temp_dir_raw = tempfile::TempDir::new().unwrap();
        let db_path_raw = temp_dir_raw.path().join("bench_raw.redb");
        let db = Database::create(&db_path_raw).unwrap();

        {
            let write_txn = db.begin_write().unwrap();
            {
                let mut table = write_txn.open_table(ARTICLES_TABLE).unwrap();
                let mut index_table = write_txn.open_table(AUTHOR_INDEX_TABLE).unwrap();

                for i in 0..*size {
                    let article = Article {
                        id: i,
                        title: format!("Article {}", i),
                        content: format!("Content {}", i),
                        author_id: i % 10,
                    };
                    let encoded =
                        bincode::encode_to_vec(&article, bincode::config::standard()).unwrap();
                    table.insert(i, encoded.as_slice()).unwrap();
                    index_table.insert((article.author_id, i), ()).unwrap();
                }
            }
            write_txn.commit().unwrap();
        }

        group.bench_with_input(BenchmarkId::new("redb_raw", size), size, |b, _size| {
            b.iter(|| {
                let read_txn = db.begin_read().unwrap();
                let table = read_txn.open_table(ARTICLES_TABLE).unwrap();

                let mut count = 0;
                let iter = table.iter().unwrap();
                for result in iter {
                    let (_key, value): (redb::AccessGuard<u64>, redb::AccessGuard<&[u8]>) =
                        result.unwrap();
                    let _article: Article =
                        bincode::decode_from_slice(value.value(), bincode::config::standard())
                            .unwrap()
                            .0;
                    count += 1;
                }
                black_box(count);
            });
        });
    }

    group.finish();
}

fn bench_redb_secondary_key_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("redb_secondary_key");

    for size in [100, 1000, 5000].iter() {
        // Setup wrapper
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("bench_wrapper.redb");
        let store = RedbStore::<BenchDefinition>::new(&db_path).unwrap();
        let article_tree = store.open_tree::<Article>();

        for i in 0..*size {
            let article = Article {
                id: i,
                title: format!("Article {}", i),
                content: format!("Content {}", i),
                author_id: i % 10,
            };
            article_tree.put(article).unwrap();
        }

        group.bench_with_input(BenchmarkId::new("redb_wrapper", size), size, |b, _size| {
            b.iter(|| {
                // Query for author_id = 5 (should have ~10% of records)
                let results = article_tree
                    .get_by_secondary_key(ArticleSecondaryKeys::AuthorId(ArticleAuthorIdSecondaryKey(5)))
                    .unwrap();
                black_box(results.len());
            });
        });

        // Setup raw redb
        let temp_dir_raw = tempfile::TempDir::new().unwrap();
        let db_path_raw = temp_dir_raw.path().join("bench_raw.redb");
        let db = Database::create(&db_path_raw).unwrap();

        {
            let write_txn = db.begin_write().unwrap();
            {
                let mut table = write_txn.open_table(ARTICLES_TABLE).unwrap();
                let mut index_table = write_txn.open_table(AUTHOR_INDEX_TABLE).unwrap();

                for i in 0..*size {
                    let article = Article {
                        id: i,
                        title: format!("Article {}", i),
                        content: format!("Content {}", i),
                        author_id: i % 10,
                    };
                    let encoded =
                        bincode::encode_to_vec(&article, bincode::config::standard()).unwrap();
                    table.insert(i, encoded.as_slice()).unwrap();
                    index_table.insert((article.author_id, i), ()).unwrap();
                }
            }
            write_txn.commit().unwrap();
        }

        group.bench_with_input(BenchmarkId::new("redb_raw", size), size, |b, _size| {
            b.iter(|| {
                let read_txn = db.begin_read().unwrap();
                let table = read_txn.open_table(ARTICLES_TABLE).unwrap();
                let index_table = read_txn.open_table(AUTHOR_INDEX_TABLE).unwrap();

                // Query for author_id = 5 (should have ~10% of records)
                let mut results = Vec::new();
                let range = index_table.range((5u64, 0u64)..(5u64, u64::MAX)).unwrap();

                for result in range {
                    let (key, _): (redb::AccessGuard<(u64, u64)>, redb::AccessGuard<()>) =
                        result.unwrap();
                    let key_tuple = key.value();
                    if key_tuple.0 != 5 {
                        break;
                    }
                    let encoded = table.get(&key_tuple.1).unwrap().unwrap();
                    let article: Article =
                        bincode::decode_from_slice(encoded.value(), bincode::config::standard())
                            .unwrap()
                            .0;
                    results.push(article);
                }
                black_box(results.len());
            });
        });
    }

    group.finish();
}

// Configure criterion with profiler support
fn configure_criterion() -> Criterion {
    Criterion::default()
        .with_profiler(pprof::criterion::PProfProfiler::new(100, pprof::criterion::Output::Flamegraph(None)))
}

criterion_group! {
    name = benches;
    config = configure_criterion();
    targets = bench_redb_insert, bench_redb_get, bench_redb_iteration, bench_redb_secondary_key_lookup
}
criterion_main!(benches);
