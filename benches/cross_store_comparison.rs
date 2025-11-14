#![cfg(feature = "native")]
#![cfg(not(feature = "paxos"))]

//! Cross-Store Comparison Benchmark
//!
//! This benchmark compares all available storage implementations:
//! - Raw Sled
//! - Wrapper Sled
//! - Raw Redb
//! - Wrapper Redb (standard API with auto-commit)
//! - Zerocopy Redb (explicit transaction API)
//!
//! Each implementation is tested on the same operations:
//! - Insert (sequential writes)
//! - Get (sequential reads)
//! - Bulk operations
//! - Secondary key lookups

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use netabase_macros::netabase_definition_module;
use netabase_store::databases::redb_store::RedbStore;
use netabase_store::databases::redb_zerocopy::{
    RedbStoreZeroCopy, with_read_transaction, with_write_transaction,
};
use netabase_store::databases::sled_store::SledStore;
use pprof::criterion::PProfProfiler;
use redb::{ReadableDatabase, ReadableTable, ReadableTableMetadata};

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

// Raw database table definitions
const SLED_ARTICLES_TREE: &str = "articles";
const SLED_AUTHOR_INDEX_TREE: &str = "author_index";

/// Benchmark: Sequential Insert Operations
/// Compares insert performance across all implementations
fn bench_cross_store_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("cross_store_insert");

    let sizes = [10, 100, 500, 1000, 5000];

    for size in sizes.iter() {
        // 1. Raw Sled (baseline - manual index management, per-item insert)
        group.bench_with_input(BenchmarkId::new("sled_raw_loop", size), size, |b, &size| {
            b.iter(|| {
                let temp_dir = tempfile::TempDir::new().unwrap();
                let db = sled::open(temp_dir.path()).unwrap();
                let articles_tree = db.open_tree(SLED_ARTICLES_TREE).unwrap();
                let author_index = db.open_tree(SLED_AUTHOR_INDEX_TREE).unwrap();

                for i in 0u64..size {
                    let article = Article {
                        id: i,
                        title: format!("Article {}", i),
                        content: format!("Content for article {}", i),
                        author_id: i % 10,
                    };
                    let encoded =
                        bincode::encode_to_vec(&article, bincode::config::standard()).unwrap();
                    articles_tree
                        .insert(&i.to_be_bytes(), encoded.as_slice())
                        .unwrap();

                    // Secondary index
                    let index_key = format!("{}:{}", article.author_id, i);
                    author_index.insert(index_key.as_bytes(), &[]).unwrap();
                }

                articles_tree.flush().unwrap();
                black_box(articles_tree.len());
            });
        });

        // 2. Wrapper Sled (type-safe API, auto-index, per-item insert = N transactions)
        group.bench_with_input(
            BenchmarkId::new("sled_wrapper_loop", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let temp_dir = tempfile::TempDir::new().unwrap();
                    let store = SledStore::<BenchDefinition>::new(temp_dir.path()).unwrap();
                    let article_tree = store.open_tree::<Article>();

                    for i in 0u64..size {
                        let article = Article {
                            id: i,
                            title: format!("Article {}", i),
                            content: format!("Content for article {}", i),
                            author_id: i % 10,
                        };
                        article_tree.put(article).unwrap();
                    }

                    black_box(article_tree.len());
                });
            },
        );

        // 3. Wrapper Sled with Transaction (type-safe API, single transaction for all inserts)
        group.bench_with_input(
            BenchmarkId::new("sled_wrapper_txn", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let temp_dir = tempfile::TempDir::new().unwrap();
                    let store = SledStore::<BenchDefinition>::new(temp_dir.path()).unwrap();

                    store
                        .transaction::<Article, _, _>(|txn_tree| {
                            for i in 0u64..size {
                                let article = Article {
                                    id: i,
                                    title: format!("Article {}", i),
                                    content: format!("Content for article {}", i),
                                    author_id: i % 10,
                                };
                                txn_tree.put(article)?;
                            }
                            Ok(())
                        })
                        .unwrap();

                    let article_tree = store.open_tree::<Article>();
                    black_box(article_tree.len());
                });
            },
        );

        // 3. Raw Redb (baseline - manual index, single transaction)
        group.bench_with_input(BenchmarkId::new("redb_raw_txn", size), size, |b, &size| {
            b.iter(|| {
                let temp_dir = tempfile::TempDir::new().unwrap();
                let db_path = temp_dir.path().join("bench.redb");
                let db = redb::Database::create(&db_path).unwrap();

                let articles_table: redb::TableDefinition<u64, &[u8]> =
                    redb::TableDefinition::new("articles");
                let author_index_table: redb::TableDefinition<(u64, u64), ()> =
                    redb::TableDefinition::new("author_index");

                let write_txn = db.begin_write().unwrap();
                {
                    let mut table = write_txn.open_table(articles_table).unwrap();
                    let mut index = write_txn.open_table(author_index_table).unwrap();

                    for i in 0u64..size {
                        let article = Article {
                            id: i,
                            title: format!("Article {}", i),
                            content: format!("Content for article {}", i),
                            author_id: i % 10,
                        };
                        let encoded =
                            bincode::encode_to_vec(&article, bincode::config::standard()).unwrap();
                        table.insert(i, encoded.as_slice()).unwrap();
                        index.insert((article.author_id, i), ()).unwrap();
                    }
                }
                write_txn.commit().unwrap();

                let read_txn = db.begin_read().unwrap();
                let table = read_txn.open_table(articles_table).unwrap();
                black_box(table.len().unwrap());
            });
        });

        // 4. Wrapper Redb Loop (type-safe, auto-index, per-item = N transactions)
        group.bench_with_input(
            BenchmarkId::new("redb_wrapper_loop", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let temp_dir = tempfile::TempDir::new().unwrap();
                    let db_path = temp_dir.path().join("bench.redb");
                    let store = RedbStore::<BenchDefinition>::new(&db_path).unwrap();
                    let article_tree = store.open_tree::<Article>();

                    for i in 0u64..size {
                        let article = Article {
                            id: i,
                            title: format!("Article {}", i),
                            content: format!("Content for article {}", i),
                            author_id: i % 10,
                        };
                        article_tree.put(article).unwrap();
                    }

                    black_box(article_tree.len().unwrap());
                });
            },
        );

        // 5. Wrapper Redb Bulk (type-safe, auto-index, put_many = 1 transaction)
        group.bench_with_input(
            BenchmarkId::new("redb_wrapper_bulk", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let temp_dir = tempfile::TempDir::new().unwrap();
                    let db_path = temp_dir.path().join("bench.redb");
                    let store = RedbStore::<BenchDefinition>::new(&db_path).unwrap();
                    let article_tree = store.open_tree::<Article>();

                    let articles: Vec<Article> = (0u64..size)
                        .map(|i| Article {
                            id: i,
                            title: format!("Article {}", i),
                            content: format!("Content for article {}", i),
                            author_id: i % 10,
                        })
                        .collect();

                    article_tree.put_many(articles).unwrap();

                    black_box(article_tree.len().unwrap());
                });
            },
        );

        // 5. Zerocopy Redb (explicit transaction API - single transaction)
        group.bench_with_input(
            BenchmarkId::new("redb_zerocopy_loop", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let temp_dir = tempfile::TempDir::new().unwrap();
                    let db_path = temp_dir.path().join("bench.redb");
                    let store = RedbStoreZeroCopy::<BenchDefinition>::new(&db_path).unwrap();

                    let count = with_write_transaction(&store, |txn| {
                        let mut tree = txn.open_tree::<Article>()?;

                        for i in 0u64..size {
                            let article = Article {
                                id: i,
                                title: format!("Article {}", i),
                                content: format!("Content for article {}", i),
                                author_id: i % 10,
                            };
                            tree.put(article)?;
                        }

                        tree.len()
                    })
                    .unwrap();

                    black_box(count);
                });
            },
        );

        // 6. Zerocopy Redb with bulk insert
        group.bench_with_input(
            BenchmarkId::new("redb_zerocopy_bulk", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let temp_dir = tempfile::TempDir::new().unwrap();
                    let db_path = temp_dir.path().join("bench.redb");
                    let store = RedbStoreZeroCopy::<BenchDefinition>::new(&db_path).unwrap();

                    let articles: Vec<Article> = (0u64..size)
                        .map(|i| Article {
                            id: i,
                            title: format!("Article {}", i),
                            content: format!("Content for article {}", i),
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
            },
        );
    }

    group.finish();
}

/// Benchmark: Sequential Get Operations
/// Compares read performance across all implementations
fn bench_cross_store_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("cross_store_get");

    let size = 1000u64;

    // Setup Raw Sled
    let temp_dir_raw_sled = tempfile::TempDir::new().unwrap();
    let db_sled = sled::open(temp_dir_raw_sled.path()).unwrap();
    let articles_tree_sled = db_sled.open_tree(SLED_ARTICLES_TREE).unwrap();
    let author_index_sled = db_sled.open_tree(SLED_AUTHOR_INDEX_TREE).unwrap();

    for i in 0..size {
        let article = Article {
            id: i,
            title: format!("Article {}", i),
            content: format!("Content for article {}", i),
            author_id: i % 10,
        };
        let encoded = bincode::encode_to_vec(&article, bincode::config::standard()).unwrap();
        articles_tree_sled
            .insert(&i.to_be_bytes(), encoded.as_slice())
            .unwrap();
        let index_key = format!("{}:{}", article.author_id, i);
        author_index_sled.insert(index_key.as_bytes(), &[]).unwrap();
    }
    articles_tree_sled.flush().unwrap();

    // Setup Wrapper Sled
    let temp_dir_wrapper_sled = tempfile::TempDir::new().unwrap();
    let store_sled = SledStore::<BenchDefinition>::new(temp_dir_wrapper_sled.path()).unwrap();
    let article_tree_sled_wrapper = store_sled.open_tree::<Article>();

    for i in 0..size {
        article_tree_sled_wrapper
            .put(Article {
                id: i,
                title: format!("Article {}", i),
                content: format!("Content for article {}", i),
                author_id: i % 10,
            })
            .unwrap();
    }

    // Setup Raw Redb
    let temp_dir_raw_redb = tempfile::TempDir::new().unwrap();
    let db_path_raw_redb = temp_dir_raw_redb.path().join("bench.redb");
    let db_redb = redb::Database::create(&db_path_raw_redb).unwrap();

    let articles_table: redb::TableDefinition<u64, &[u8]> = redb::TableDefinition::new("articles");
    let author_index_table: redb::TableDefinition<(u64, u64), ()> =
        redb::TableDefinition::new("author_index");

    {
        let write_txn = db_redb.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(articles_table).unwrap();
            let mut index = write_txn.open_table(author_index_table).unwrap();

            for i in 0..size {
                let article = Article {
                    id: i,
                    title: format!("Article {}", i),
                    content: format!("Content for article {}", i),
                    author_id: i % 10,
                };
                let encoded =
                    bincode::encode_to_vec(&article, bincode::config::standard()).unwrap();
                table.insert(i, encoded.as_slice()).unwrap();
                index.insert((article.author_id, i), ()).unwrap();
            }
        }
        write_txn.commit().unwrap();
    }

    // Setup Wrapper Redb
    let temp_dir_wrapper_redb = tempfile::TempDir::new().unwrap();
    let db_path_wrapper_redb = temp_dir_wrapper_redb.path().join("bench.redb");
    let store_redb = RedbStore::<BenchDefinition>::new(&db_path_wrapper_redb).unwrap();
    let article_tree_redb = store_redb.open_tree::<Article>();

    for i in 0..size {
        article_tree_redb
            .put(Article {
                id: i,
                title: format!("Article {}", i),
                content: format!("Content for article {}", i),
                author_id: i % 10,
            })
            .unwrap();
    }

    // Setup Zerocopy Redb
    let temp_dir_zerocopy = tempfile::TempDir::new().unwrap();
    let db_path_zerocopy = temp_dir_zerocopy.path().join("bench.redb");
    let store_zerocopy = RedbStoreZeroCopy::<BenchDefinition>::new(&db_path_zerocopy).unwrap();

    with_write_transaction(&store_zerocopy, |txn| {
        let mut tree = txn.open_tree::<Article>()?;
        for i in 0..size {
            tree.put(Article {
                id: i,
                title: format!("Article {}", i),
                content: format!("Content for article {}", i),
                author_id: i % 10,
            })?;
        }
        Ok(())
    })
    .unwrap();

    // 1. Raw Sled
    group.bench_function("sled_raw", |b| {
        b.iter(|| {
            for i in 0..size {
                let bytes = articles_tree_sled.get(&i.to_be_bytes()).unwrap().unwrap();
                let article: Article =
                    bincode::decode_from_slice(&bytes, bincode::config::standard())
                        .unwrap()
                        .0;
                black_box(article);
            }
        });
    });

    // 2. Wrapper Sled (loop - N transactions)
    group.bench_function("sled_wrapper_loop", |b| {
        b.iter(|| {
            for i in 0..size {
                let article = article_tree_sled_wrapper.get(ArticlePrimaryKey(i)).unwrap();
                black_box(article);
            }
        });
    });

    // 3. Wrapper Sled (transaction - single transaction for all reads)
    group.bench_function("sled_wrapper_txn", |b| {
        b.iter(|| {
            store_sled
                .transaction::<Article, _, _>(|txn_tree| {
                    for i in 0..size {
                        let article = txn_tree.get(ArticlePrimaryKey(i))?;
                        black_box(article);
                    }
                    Ok(())
                })
                .unwrap();
        });
    });

    // 3. Raw Redb
    group.bench_function("redb_raw", |b| {
        b.iter(|| {
            let read_txn = db_redb.begin_read().unwrap();
            let table = read_txn.open_table(articles_table).unwrap();

            for i in 0..size {
                let encoded = table.get(i).unwrap().unwrap();
                let article: Article =
                    bincode::decode_from_slice(encoded.value(), bincode::config::standard())
                        .unwrap()
                        .0;
                black_box(article);
            }
        });
    });

    // 4. Wrapper Redb (loop - creates transaction per get)
    group.bench_function("redb_wrapper_loop", |b| {
        b.iter(|| {
            for i in 0..size {
                let article = article_tree_redb
                    .get(ArticleKey::Primary(ArticlePrimaryKey(i)))
                    .unwrap();
                black_box(article);
            }
        });
    });

    // 4b. Wrapper Redb with get_many (single transaction)
    group.bench_function("redb_wrapper_bulk", |b| {
        b.iter(|| {
            let keys: Vec<ArticleKey> = (0..size)
                .map(|i| ArticleKey::Primary(ArticlePrimaryKey(i)))
                .collect();
            let articles = article_tree_redb.get_many(keys).unwrap();
            black_box(articles);
        });
    });

    // 5. ZeroCopy Redb (explicit transaction, single txn for all reads)
    group.bench_function("redb_zerocopy_loop", |b| {
        b.iter(|| {
            with_read_transaction(&store_zerocopy, |txn| {
                let tree = txn.open_tree::<Article>()?;

                for i in 0..size {
                    let article = tree.get(&ArticlePrimaryKey(i))?;
                    black_box(article);
                }
                Ok(())
            })
            .unwrap();
        });
    });

    group.finish();
}

/// Benchmark: Bulk Operations
/// Compares bulk write performance where available
fn bench_cross_store_bulk_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("cross_store_bulk");

    let size = 1000u64;

    // 1. Raw Sled (batch)
    group.bench_function("sled_raw_batch", |b| {
        b.iter(|| {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db = sled::open(temp_dir.path()).unwrap();
            let articles_tree = db.open_tree(SLED_ARTICLES_TREE).unwrap();
            let author_index = db.open_tree(SLED_AUTHOR_INDEX_TREE).unwrap();

            let mut batch = sled::Batch::default();

            for i in 0..size {
                let article = Article {
                    id: i,
                    title: format!("Article {}", i),
                    content: format!("Content for article {}", i),
                    author_id: i % 10,
                };
                let encoded =
                    bincode::encode_to_vec(&article, bincode::config::standard()).unwrap();
                batch.insert(&i.to_be_bytes(), encoded.as_slice());
            }

            articles_tree.apply_batch(batch).unwrap();

            // Author index still needs individual inserts
            for i in 0..size {
                let author_id = i % 10;
                let index_key = format!("{}:{}", author_id, i);
                author_index.insert(index_key.as_bytes(), &[]).unwrap();
            }

            black_box(articles_tree.len());
        });
    });

    // 2. Wrapper Sled (loop - N transactions)
    group.bench_function("sled_wrapper_loop", |b| {
        b.iter(|| {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let store = SledStore::<BenchDefinition>::new(temp_dir.path()).unwrap();
            let article_tree = store.open_tree::<Article>();

            for i in 0..size {
                article_tree
                    .put(Article {
                        id: i,
                        title: format!("Article {}", i),
                        content: format!("Content for article {}", i),
                        author_id: i % 10,
                    })
                    .unwrap();
            }

            black_box(article_tree.len());
        });
    });

    // 3. Wrapper Sled (transaction - single transaction)
    group.bench_function("sled_wrapper_txn", |b| {
        b.iter(|| {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let store = SledStore::<BenchDefinition>::new(temp_dir.path()).unwrap();

            store
                .transaction::<Article, _, _>(|txn_tree| {
                    for i in 0..size {
                        txn_tree.put(Article {
                            id: i,
                            title: format!("Article {}", i),
                            content: format!("Content for article {}", i),
                            author_id: i % 10,
                        })?;
                    }
                    Ok(())
                })
                .unwrap();

            let article_tree = store.open_tree::<Article>();
            black_box(article_tree.len());
        });
    });

    // 3. Raw Redb (single transaction)
    group.bench_function("redb_raw_txn", |b| {
        b.iter(|| {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("bench.redb");
            let db = redb::Database::create(&db_path).unwrap();

            let articles_table: redb::TableDefinition<u64, &[u8]> =
                redb::TableDefinition::new("articles");
            let author_index_table: redb::TableDefinition<(u64, u64), ()> =
                redb::TableDefinition::new("author_index");

            let write_txn = db.begin_write().unwrap();
            {
                let mut table = write_txn.open_table(articles_table).unwrap();
                let mut index = write_txn.open_table(author_index_table).unwrap();

                for i in 0..size {
                    let article = Article {
                        id: i,
                        title: format!("Article {}", i),
                        content: format!("Content for article {}", i),
                        author_id: i % 10,
                    };
                    let encoded =
                        bincode::encode_to_vec(&article, bincode::config::standard()).unwrap();
                    table.insert(i, encoded.as_slice()).unwrap();
                    index.insert((article.author_id, i), ()).unwrap();
                }
            }
            write_txn.commit().unwrap();

            let read_txn = db.begin_read().unwrap();
            let table = read_txn.open_table(articles_table).unwrap();
            black_box(table.len().unwrap());
        });
    });

    // 4. Wrapper Redb (auto-commit per operation)
    group.bench_function("redb_wrapper_loop", |b| {
        b.iter(|| {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("bench.redb");
            let store = RedbStore::<BenchDefinition>::new(&db_path).unwrap();
            let article_tree = store.open_tree::<Article>();

            for i in 0..size {
                article_tree
                    .put(Article {
                        id: i,
                        title: format!("Article {}", i),
                        content: format!("Content for article {}", i),
                        author_id: i % 10,
                    })
                    .unwrap();
            }

            black_box(article_tree.len().unwrap());
        });
    });

    // 5. Zerocopy Redb (single transaction loop)
    group.bench_function("redb_zerocopy_txn", |b| {
        b.iter(|| {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("bench.redb");
            let store = RedbStoreZeroCopy::<BenchDefinition>::new(&db_path).unwrap();

            let count = with_write_transaction(&store, |txn| {
                let mut tree = txn.open_tree::<Article>()?;

                for i in 0..size {
                    tree.put(Article {
                        id: i,
                        title: format!("Article {}", i),
                        content: format!("Content for article {}", i),
                        author_id: i % 10,
                    })?;
                }

                tree.len()
            })
            .unwrap();

            black_box(count);
        });
    });

    // 6. ZeroCopy Redb Bulk (explicit transaction, put_many in 1 txn)
    group.bench_function("redb_zerocopy_bulk", |b| {
        b.iter(|| {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let db_path = temp_dir.path().join("bench.redb");
            let store = RedbStoreZeroCopy::<BenchDefinition>::new(&db_path).unwrap();

            let articles: Vec<Article> = (0..size)
                .map(|i| Article {
                    id: i,
                    title: format!("Article {}", i),
                    content: format!("Content for article {}", i),
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

    group.finish();
}

/// Benchmark: Secondary Key Queries
/// Compares the performance of querying by secondary keys across implementations
fn bench_cross_store_secondary_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("cross_store_secondary_query");
    let size = 1000u64;
    let num_queries = 10u64; // Query for 10 different author_ids

    // Setup: Pre-populate stores with data
    // Each store will have 1000 articles with author_ids from 0-9 (100 articles per author)

    // 1. Raw Sled with manual secondary index
    let temp_dir_sled = tempfile::TempDir::new().unwrap();
    let db_sled = sled::open(temp_dir_sled.path()).unwrap();
    let articles_tree_sled = db_sled.open_tree(SLED_ARTICLES_TREE).unwrap();
    let author_index_sled = db_sled.open_tree(SLED_AUTHOR_INDEX_TREE).unwrap();

    for i in 0..size {
        let article = Article {
            id: i,
            title: format!("Article {}", i),
            content: format!("Content for article {}", i),
            author_id: i % 10,
        };
        let encoded = bincode::encode_to_vec(&article, bincode::config::standard()).unwrap();
        articles_tree_sled
            .insert(&i.to_be_bytes(), encoded.as_slice())
            .unwrap();
        let index_key = format!("{}:{}", article.author_id, i);
        author_index_sled.insert(index_key.as_bytes(), &[]).unwrap();
    }

    group.bench_function("sled_raw_loop", |b| {
        b.iter(|| {
            for author_id in 0..num_queries {
                let prefix = format!("{}:", author_id);
                let mut results = Vec::new();
                for item in author_index_sled.scan_prefix(prefix.as_bytes()) {
                    let (key, _) = item.unwrap();
                    let key_str = std::str::from_utf8(&key).unwrap();
                    let article_id: u64 = key_str.split(':').nth(1).unwrap().parse().unwrap();
                    if let Some(data) = articles_tree_sled.get(&article_id.to_be_bytes()).unwrap() {
                        let (article, _): (Article, _) =
                            bincode::decode_from_slice(&data, bincode::config::standard()).unwrap();
                        results.push(article);
                    }
                }
                black_box(results);
            }
        });
    });

    // 2. Wrapper Sled (loop with get_by_secondary_key - N transactions)
    let temp_dir_sled_wrapper = tempfile::TempDir::new().unwrap();
    let store_sled = SledStore::<BenchDefinition>::new(temp_dir_sled_wrapper.path()).unwrap();
    let article_tree_sled = store_sled.open_tree::<Article>();

    for i in 0..size {
        article_tree_sled
            .put(Article {
                id: i,
                title: format!("Article {}", i),
                content: format!("Content for article {}", i),
                author_id: i % 10,
            })
            .unwrap();
    }

    group.bench_function("sled_wrapper_loop", |b| {
        b.iter(|| {
            for author_id in 0..num_queries {
                let results = article_tree_sled
                    .get_by_secondary_key(ArticleSecondaryKeys::AuthorId(
                        ArticleAuthorIdSecondaryKey(author_id),
                    ))
                    .unwrap();
                black_box(results);
            }
        });
    });

    // 3. Wrapper Sled (transaction - single transaction for all queries)
    group.bench_function("sled_wrapper_txn", |b| {
        b.iter(|| {
            store_sled
                .transaction::<Article, _, _>(|txn_tree| {
                    for author_id in 0..num_queries {
                        // Note: We need to implement get_by_secondary_key for SledTransactionalTree
                        // For now, do individual gets as a placeholder
                        // This would need the secondary key query implementation
                        for i in 0..size {
                            if i % 10 == author_id {
                                let article = txn_tree.get(ArticlePrimaryKey(i))?;
                                black_box(article);
                            }
                        }
                    }
                    Ok(())
                })
                .unwrap();
        });
    });

    // 3. Raw Redb with manual secondary index
    let temp_dir_redb = tempfile::TempDir::new().unwrap();
    let db_path_redb = temp_dir_redb.path().join("bench.redb");
    let db_redb = redb::Database::create(&db_path_redb).unwrap();

    let articles_table: redb::TableDefinition<u64, &[u8]> = redb::TableDefinition::new("articles");
    let author_index_table: redb::TableDefinition<(u64, u64), ()> =
        redb::TableDefinition::new("author_index");

    {
        let write_txn = db_redb.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(articles_table).unwrap();
            let mut index = write_txn.open_table(author_index_table).unwrap();

            for i in 0..size {
                let article = Article {
                    id: i,
                    title: format!("Article {}", i),
                    content: format!("Content for article {}", i),
                    author_id: i % 10,
                };
                let encoded =
                    bincode::encode_to_vec(&article, bincode::config::standard()).unwrap();
                table.insert(i, encoded.as_slice()).unwrap();
                index.insert((article.author_id, i), ()).unwrap();
            }
        }
        write_txn.commit().unwrap();
    }

    group.bench_function("redb_raw_loop", |b| {
        b.iter(|| {
            for author_id in 0..num_queries {
                let read_txn = db_redb.begin_read().unwrap();
                let table = read_txn.open_table(articles_table).unwrap();
                let index = read_txn.open_table(author_index_table).unwrap();

                let mut results = Vec::new();
                let range = (author_id, 0u64)..=(author_id, u64::MAX);
                for item in index.range(range).unwrap() {
                    let (key_guard, _) = item.unwrap();
                    let (_author_id, article_id) = key_guard.value();
                    if let Some(data) = table.get(article_id).unwrap() {
                        let (article, _): (Article, _) =
                            bincode::decode_from_slice(data.value(), bincode::config::standard())
                                .unwrap();
                        results.push(article);
                    }
                }
                black_box(results);
            }
        });
    });

    // 4. Wrapper Redb (loop with get_by_secondary_key - creates N transactions)
    let temp_dir_redb_wrapper = tempfile::TempDir::new().unwrap();
    let db_path_redb_wrapper = temp_dir_redb_wrapper.path().join("bench.redb");
    let store_redb = RedbStore::<BenchDefinition>::new(&db_path_redb_wrapper).unwrap();
    let article_tree_redb = store_redb.open_tree::<Article>();

    for i in 0..size {
        article_tree_redb
            .put(Article {
                id: i,
                title: format!("Article {}", i),
                content: format!("Content for article {}", i),
                author_id: i % 10,
            })
            .unwrap();
    }

    group.bench_function("redb_wrapper_loop", |b| {
        b.iter(|| {
            for author_id in 0..num_queries {
                let results = article_tree_redb
                    .get_by_secondary_key(ArticleSecondaryKeys::AuthorId(
                        ArticleAuthorIdSecondaryKey(author_id),
                    ))
                    .unwrap();
                black_box(results);
            }
        });
    });

    // 4b. Wrapper Redb with get_many_by_secondary_keys (single transaction)
    group.bench_function("redb_wrapper_bulk", |b| {
        b.iter(|| {
            let keys: Vec<_> = (0..num_queries)
                .map(|author_id| {
                    ArticleSecondaryKeys::AuthorId(ArticleAuthorIdSecondaryKey(author_id))
                })
                .collect();
            let results = article_tree_redb.get_many_by_secondary_keys(keys).unwrap();
            black_box(results);
        });
    });

    // 5. Zerocopy Redb (single transaction for all queries)
    let temp_dir_zerocopy = tempfile::TempDir::new().unwrap();
    let db_path_zerocopy = temp_dir_zerocopy.path().join("bench.redb");
    let store_zerocopy = RedbStoreZeroCopy::<BenchDefinition>::new(&db_path_zerocopy).unwrap();

    {
        let mut txn = store_zerocopy.begin_write().unwrap();
        let mut tree = txn.open_tree::<Article>().unwrap();
        for i in 0..size {
            tree.put(Article {
                id: i,
                title: format!("Article {}", i),
                content: format!("Content for article {}", i),
                author_id: i % 10,
            })
            .unwrap();
        }
        drop(tree);
        txn.commit().unwrap();
    }

    group.bench_function("redb_zerocopy_txn", |b| {
        b.iter(|| {
            with_read_transaction(&store_zerocopy, |txn| {
                let tree = txn.open_tree::<Article>()?;
                for author_id in 0..num_queries {
                    let results = tree.get_by_secondary_key(&ArticleSecondaryKeys::AuthorId(
                        ArticleAuthorIdSecondaryKey(author_id),
                    ))?;
                    black_box(results);
                }
                Ok(())
            })
            .unwrap();
        });
    });

    group.finish();
}

/// Benchmark: Raw Redb vs ZeroCopy Redb Direct Comparison
/// Detailed comparison of raw redb API vs our zerocopy wrapper
fn bench_redb_raw_vs_zerocopy(c: &mut Criterion) {
    let mut group = c.benchmark_group("redb_raw_vs_zerocopy");

    let sizes = [10, 100, 500, 1000, 5000];

    for size in sizes.iter() {
        // Raw Redb - Insert with single transaction
        group.bench_with_input(
            BenchmarkId::new("redb_raw_insert", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let temp_dir = tempfile::TempDir::new().unwrap();
                    let db_path = temp_dir.path().join("bench.redb");
                    let db = redb::Database::create(&db_path).unwrap();

                    let articles_table: redb::TableDefinition<u64, &[u8]> =
                        redb::TableDefinition::new("articles");
                    let author_index_table: redb::TableDefinition<(u64, u64), ()> =
                        redb::TableDefinition::new("author_index");

                    let write_txn = db.begin_write().unwrap();
                    {
                        let mut table = write_txn.open_table(articles_table).unwrap();
                        let mut index = write_txn.open_table(author_index_table).unwrap();

                        for i in 0u64..size {
                            let article = Article {
                                id: i,
                                title: format!("Article {}", i),
                                content: format!("Content for article {}", i),
                                author_id: i % 10,
                            };
                            let encoded =
                                bincode::encode_to_vec(&article, bincode::config::standard())
                                    .unwrap();
                            table.insert(i, encoded.as_slice()).unwrap();
                            index.insert((article.author_id, i), ()).unwrap();
                        }
                    }
                    write_txn.commit().unwrap();

                    black_box(());
                });
            },
        );

        // ZeroCopy Redb - Insert with single transaction
        group.bench_with_input(
            BenchmarkId::new("redb_zerocopy_insert", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let temp_dir = tempfile::TempDir::new().unwrap();
                    let db_path = temp_dir.path().join("bench.redb");
                    let store = RedbStoreZeroCopy::<BenchDefinition>::new(&db_path).unwrap();

                    let mut txn = store.begin_write().unwrap();
                    let mut tree = txn.open_tree::<Article>().unwrap();

                    for i in 0u64..size {
                        let article = Article {
                            id: i,
                            title: format!("Article {}", i),
                            content: format!("Content for article {}", i),
                            author_id: i % 10,
                        };
                        tree.put(article).unwrap();
                    }

                    drop(tree);
                    txn.commit().unwrap();

                    black_box(());
                });
            },
        );

        // ZeroCopy Redb - Insert with bulk API
        group.bench_with_input(
            BenchmarkId::new("redb_zerocopy_bulk_insert", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let temp_dir = tempfile::TempDir::new().unwrap();
                    let db_path = temp_dir.path().join("bench.redb");
                    let store = RedbStoreZeroCopy::<BenchDefinition>::new(&db_path).unwrap();

                    let articles: Vec<Article> = (0u64..size)
                        .map(|i| Article {
                            id: i,
                            title: format!("Article {}", i),
                            content: format!("Content for article {}", i),
                            author_id: i % 10,
                        })
                        .collect();

                    with_write_transaction(&store, |txn| {
                        let mut tree = txn.open_tree::<Article>()?;
                        tree.put_many(articles)?;
                        Ok(())
                    })
                    .unwrap();

                    black_box(());
                });
            },
        );
    }

    // Read benchmarks
    for size in [100, 1000, 5000].iter() {
        // Setup raw redb
        let temp_dir_raw = tempfile::TempDir::new().unwrap();
        let db_path_raw = temp_dir_raw.path().join("bench.redb");
        let db_raw = redb::Database::create(&db_path_raw).unwrap();

        let articles_table: redb::TableDefinition<u64, &[u8]> =
            redb::TableDefinition::new("articles");

        {
            let write_txn = db_raw.begin_write().unwrap();
            {
                let mut table = write_txn.open_table(articles_table).unwrap();
                for i in 0u64..*size {
                    let article = Article {
                        id: i,
                        title: format!("Article {}", i),
                        content: format!("Content for article {}", i),
                        author_id: i % 10,
                    };
                    let encoded =
                        bincode::encode_to_vec(&article, bincode::config::standard()).unwrap();
                    table.insert(i, encoded.as_slice()).unwrap();
                }
            }
            write_txn.commit().unwrap();
        }

        // Setup zerocopy redb
        let temp_dir_zc = tempfile::TempDir::new().unwrap();
        let db_path_zc = temp_dir_zc.path().join("bench.redb");
        let store_zc = RedbStoreZeroCopy::<BenchDefinition>::new(&db_path_zc).unwrap();

        {
            let mut txn = store_zc.begin_write().unwrap();
            let mut tree = txn.open_tree::<Article>().unwrap();
            for i in 0u64..*size {
                tree.put(Article {
                    id: i,
                    title: format!("Article {}", i),
                    content: format!("Content for article {}", i),
                    author_id: i % 10,
                })
                .unwrap();
            }
            drop(tree);
            txn.commit().unwrap();
        }

        // Raw redb - read with new transaction per get
        group.bench_with_input(
            BenchmarkId::new("redb_raw_read_per_txn", size),
            size,
            |b, &size| {
                b.iter(|| {
                    for i in 0u64..size {
                        let read_txn = db_raw.begin_read().unwrap();
                        let table = read_txn.open_table(articles_table).unwrap();
                        if let Some(data) = table.get(i).unwrap() {
                            let (article, _): (Article, _) = bincode::decode_from_slice(
                                data.value(),
                                bincode::config::standard(),
                            )
                            .unwrap();
                            black_box(article);
                        }
                    }
                });
            },
        );

        // Raw redb - read with single transaction
        group.bench_with_input(
            BenchmarkId::new("redb_raw_read_single_txn", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let read_txn = db_raw.begin_read().unwrap();
                    let table = read_txn.open_table(articles_table).unwrap();
                    for i in 0u64..size {
                        if let Some(data) = table.get(i).unwrap() {
                            let (article, _): (Article, _) = bincode::decode_from_slice(
                                data.value(),
                                bincode::config::standard(),
                            )
                            .unwrap();
                            black_box(article);
                        }
                    }
                });
            },
        );

        // ZeroCopy redb - read with single transaction
        group.bench_with_input(
            BenchmarkId::new("redb_zerocopy_read", size),
            size,
            |b, &size| {
                b.iter(|| {
                    with_read_transaction(&store_zc, |txn| {
                        let tree = txn.open_tree::<Article>()?;
                        for i in 0u64..size {
                            if let Some(article) = tree.get(&ArticlePrimaryKey(i))? {
                                black_box(article);
                            }
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

// Configure criterion with profiler support
// Generates both flamegraphs (SVG) and protobuf files for detailed analysis
fn configure_criterion() -> Criterion {
    Criterion::default()
        .with_profiler(PProfProfiler::new(
            100,                                        // Sample frequency (Hz)
            pprof::criterion::Output::Flamegraph(None), // Flamegraph output
        ))
        .sample_size(50) // Reduce sample size for faster profiling
        .measurement_time(std::time::Duration::from_secs(10)) // 10 seconds per benchmark
}

criterion_group! {
    name = benches;
    config = configure_criterion();
    targets = bench_cross_store_insert, bench_cross_store_get, bench_cross_store_bulk_ops, bench_cross_store_secondary_query, bench_redb_raw_vs_zerocopy
}
criterion_main!(benches);
