//! Benchmarks for relational links overhead
//!
//! Measures the performance impact of using relational links compared to plain models

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use netabase_store::{
    NetabaseModel, NetabaseStore,
    links::RelationalLink,
    netabase_definition_module,
    traits::store_ops::StoreOps,
};

// Schema without relations (baseline)
#[netabase_definition_module(PlainDef, PlainKeys)]
mod plain_schema {
    use super::*;
    use netabase_store::netabase;

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(PlainDef)]
    pub struct PlainUser {
        #[primary_key]
        pub id: u64,
        pub name: String,
        pub email: String,
    }

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(PlainDef)]
    pub struct PlainPost {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,
        pub author_id: u64, // Manual foreign key
    }
}

// Schema with relations
#[netabase_definition_module(RelationalDef, RelationalKeys)]
mod relational_schema {
    use super::*;
    use netabase_store::netabase;

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(RelationalDef)]
    pub struct RelationalUser {
        #[primary_key]
        pub id: u64,
        pub name: String,
        pub email: String,
    }

    #[derive(
        NetabaseModel,
        Clone,
        Debug,
        PartialEq,
        bincode::Encode,
        bincode::Decode,
        serde::Serialize,
        serde::Deserialize,
    )]
    #[netabase(RelationalDef)]
    pub struct RelationalPost {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,

        #[relation(author)]
        pub author: RelationalLink<RelationalDef, RelationalUser>,
    }
}

use plain_schema::*;
use relational_schema::*;

fn bench_plain_insertion(c: &mut Criterion) {
    let mut group = c.benchmark_group("insertion_plain");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_batched(
                || NetabaseStore::temp().unwrap(),
                |store| {
                    let user_tree = store.open_tree::<PlainUser>();
                    let post_tree = store.open_tree::<PlainPost>();

                    for i in 0..size {
                        let user = PlainUser {
                            id: i,
                            name: format!("User {}", i),
                            email: format!("user{}@example.com", i),
                        };
                        user_tree.put(black_box(user)).unwrap();

                        let post = PlainPost {
                            id: i,
                            title: format!("Post {}", i),
                            content: format!("Content {}", i),
                            author_id: i,
                        };
                        post_tree.put(black_box(post)).unwrap();
                    }
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_relational_insertion_with_references(c: &mut Criterion) {
    let mut group = c.benchmark_group("insertion_relational_references");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_batched(
                || NetabaseStore::temp().unwrap(),
                |store| {
                    let user_tree = store.open_tree::<RelationalUser>();
                    let post_tree = store.open_tree::<RelationalPost>();

                    for i in 0..size {
                        let user = RelationalUser {
                            id: i,
                            name: format!("User {}", i),
                            email: format!("user{}@example.com", i),
                        };
                        user_tree.put(black_box(user)).unwrap();

                        let post = RelationalPost {
                            id: i,
                            title: format!("Post {}", i),
                            content: format!("Content {}", i),
                            author: RelationalLink::Reference(RelationalUserPrimaryKey(i)),
                        };
                        post_tree.put(black_box(post)).unwrap();
                    }
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_relational_insertion_with_entities(c: &mut Criterion) {
    let mut group = c.benchmark_group("insertion_relational_entities");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_batched(
                || NetabaseStore::temp().unwrap(),
                |store| {
                    for i in 0..size {
                        let user = RelationalUser {
                            id: i,
                            name: format!("User {}", i),
                            email: format!("user{}@example.com", i),
                        };

                        let post = RelationalPost {
                            id: i,
                            title: format!("Post {}", i),
                            content: format!("Content {}", i),
                            author: RelationalLink::Entity(user),
                        };

                        // Use insert_with_relations to insert both post and embedded author
                        post.insert_with_relations(black_box(&store)).unwrap();
                    }
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_hydration(c: &mut Criterion) {
    let mut group = c.benchmark_group("hydration");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_batched(
                || {
                    let store = NetabaseStore::temp().unwrap();

                    // Pre-populate the store
                    let user_tree = store.open_tree::<RelationalUser>();
                    let post_tree = store.open_tree::<RelationalPost>();

                    for i in 0..size {
                        let user = RelationalUser {
                            id: i,
                            name: format!("User {}", i),
                            email: format!("user{}@example.com", i),
                        };
                        user_tree.put(user).unwrap();

                        let post = RelationalPost {
                            id: i,
                            title: format!("Post {}", i),
                            content: format!("Content {}", i),
                            author: RelationalLink::Reference(RelationalUserPrimaryKey(i)),
                        };
                        post_tree.put(post).unwrap();
                    }

                    store
                },
                |store| {
                    let post_tree = store.open_tree::<RelationalPost>();
                    let user_tree = store.open_tree::<RelationalUser>();

                    for i in 0..size {
                        let post = post_tree.get(RelationalPostPrimaryKey(i)).unwrap().unwrap();
                        let _author = post.author.hydrate(black_box(&user_tree)).unwrap();
                    }
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_serialization_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");

    let plain_user = PlainUser {
        id: 1,
        name: "Test User".to_string(),
        email: "test@example.com".to_string(),
    };

    let plain_post = PlainPost {
        id: 1,
        title: "Test Post".to_string(),
        content: "Test Content".to_string(),
        author_id: 1,
    };

    let relational_user = RelationalUser {
        id: 1,
        name: "Test User".to_string(),
        email: "test@example.com".to_string(),
    };

    let relational_post_ref = RelationalPost {
        id: 1,
        title: "Test Post".to_string(),
        content: "Test Content".to_string(),
        author: RelationalLink::Reference(RelationalUserPrimaryKey(1)),
    };

    let relational_post_entity = RelationalPost {
        id: 1,
        title: "Test Post".to_string(),
        content: "Test Content".to_string(),
        author: RelationalLink::Entity(relational_user.clone()),
    };

    group.bench_function("plain_post", |b| {
        b.iter(|| {
            bincode::encode_to_vec(black_box(&plain_post), bincode::config::standard()).unwrap()
        });
    });

    group.bench_function("relational_post_reference", |b| {
        b.iter(|| {
            bincode::encode_to_vec(black_box(&relational_post_ref), bincode::config::standard()).unwrap()
        });
    });

    group.bench_function("relational_post_entity", |b| {
        b.iter(|| {
            bincode::encode_to_vec(black_box(&relational_post_entity), bincode::config::standard()).unwrap()
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_plain_insertion,
    bench_relational_insertion_with_references,
    bench_relational_insertion_with_entities,
    bench_hydration,
    bench_serialization_overhead,
);

criterion_main!(benches);
