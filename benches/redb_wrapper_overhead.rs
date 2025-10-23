#![cfg(feature = "native")]

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use netabase_macros::netabase_definition_module;
use netabase_store::databases::redb_store::RedbStore;

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
    }

    group.finish();
}

fn bench_redb_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("redb_get");

    for size in [100, 1000, 5000].iter() {
        // Setup wrapper
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("bench.redb");
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
    }

    group.finish();
}

fn bench_redb_iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("redb_iteration");

    for size in [100, 1000, 5000].iter() {
        // Setup wrapper
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("bench.redb");
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
    }

    group.finish();
}

fn bench_redb_secondary_key_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("redb_secondary_key");

    for size in [100, 1000, 5000].iter() {
        // Setup wrapper
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("bench.redb");
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
                    .get_by_secondary_key(ArticleSecondaryKeys::AuthorId(AuthorIdSecondaryKey(5)))
                    .unwrap();
                black_box(results.len());
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_redb_insert,
    bench_redb_get,
    bench_redb_iteration,
    bench_redb_secondary_key_lookup
);
criterion_main!(benches);
