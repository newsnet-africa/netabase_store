use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use netabase_store::databases::redb::transaction::RedbModelCrud;
use netabase_store::relational::RelationalLink;
use netabase_store_examples::boilerplate_lib::{
    CategoryID, Definition, DefinitionSubscriptions,
    models::{
        heavy::{
            HeavyAttachment, HeavyCategory, HeavyCreator, HeavyID, HeavyModel, HeavyRelation,
            HeavyScore,
        },
        user::{AnotherLargeUserFile, LargeUserFile, User, UserID},
    },
};
use rand::prelude::*;
use std::hint::black_box;
use std::path::PathBuf;

// Include common test utils.
mod common;
use common::*;

// --- Helpers ---

fn random_id(prefix: &str, rng: &mut impl Rng) -> String {
    let n: u64 = rng.random();
    format!("{}_{:016x}", prefix, n)
}

fn generate_random_user(rng: &mut impl Rng) -> User {
    // sample names
    let names = [
        "Alice", "Bob", "Carol", "Dave", "Eve", "Frank", "Grace", "Heidi", "Ivan", "Judy",
        "Mallory", "Niaj", "Olivia", "Peggy", "Rupert", "Sybil", "Trent", "Victor", "Walter",
    ];
    let name = names.choose(rng).unwrap().to_string();

    // random age 1..=100
    let age: u8 = rng.random_range(1..=100);

    // random user id and category id
    let user_id = UserID(random_id("user", rng));
    let category_id = CategoryID(random_id("cat", rng));

    // random choice to either set partner to "none" or a random id
    let partner = if rng.random_bool(0.5) {
        RelationalLink::new_dehydrated(UserID("none".to_string()))
    } else {
        RelationalLink::new_dehydrated(UserID(random_id("user", rng)))
    };

    // category — sometimes none, sometimes random
    let category = if rng.random_bool(0.1) {
        RelationalLink::new_dehydrated(CategoryID("none".to_string()))
    } else {
        RelationalLink::new_dehydrated(category_id.clone())
    };

    // random non-empty subscriptions
    let all_subs = [
        DefinitionSubscriptions::Topic1,
        DefinitionSubscriptions::Topic2,
    ];
    // pick between 1 and all topics
    let mut subscriptions = Vec::new();
    let pick_count = rng.random_range(1..=all_subs.len());
    let mut indices: Vec<usize> = (0..all_subs.len()).collect();
    indices.shuffle(rng);
    for &i in indices.iter().take(pick_count) {
        subscriptions.push(all_subs[i].clone());
    }

    // random bio (1kb - 10kb)
    let bio_size = rng.random_range(1024..10240);
    let mut bio_data = vec![0u8; bio_size];
    rng.fill_bytes(&mut bio_data);

    let bio = LargeUserFile {
        data: bio_data,
        metadata: "meta".to_string(),
    };

    let another = AnotherLargeUserFile(vec![0u8; 100]);

    User {
        id: user_id,
        name,
        age,
        partner,
        category,
        subscriptions,
        bio,
        another,
    }
}

fn generate_random_heavy(
    rng: &mut impl Rng,
    existing_users: &[User],
    existing_heavies: &[HeavyModel],
) -> HeavyModel {
    let id = HeavyID(random_id("heavy", rng));

    let creator = if !existing_users.is_empty() {
        let user = existing_users.choose(rng).unwrap();
        RelationalLink::new_dehydrated(user.id.clone())
    } else {
        RelationalLink::new_dehydrated(UserID("none".to_string()))
    };

    let related = if !existing_heavies.is_empty() && rng.random_bool(0.7) {
        let h = existing_heavies.choose(rng).unwrap();
        RelationalLink::new_dehydrated(h.id.clone())
    } else {
        RelationalLink::new_dehydrated(id.clone()) // Self reference if none available
    };

    let blob_size = rng.random_range(1024..10 * 1024); // 1KB to 10KB blob
    let mut blob_data = vec![0u8; blob_size];
    rng.fill_bytes(&mut blob_data);

    let subscriptions = vec![
        DefinitionSubscriptions::Topic1,
        DefinitionSubscriptions::Topic3,
    ];

    HeavyModel {
        id,
        name: format!("Heavy Item {}", rng.random::<u32>()),
        title: format!("Title {}", rng.random::<u32>()),
        category_label: ["A", "B", "C", "D"].choose(rng).unwrap().to_string(),
        score: rng.random_range(0..1000),
        creator,
        related_heavy: related,
        subscriptions,
        attachment: HeavyAttachment {
            mime_type: "application/octet-stream".to_string(),
            data: blob_data,
        },
        matrix: (0..100).map(|_| rng.random()).collect(),
    }
}

struct CleanupGuard(PathBuf);
impl Drop for CleanupGuard {
    fn drop(&mut self) {
        std::fs::remove_file(&self.0).ok();
    }
}

// --- Benchmarks ---

fn bench_stress_operations(c: &mut Criterion) {
    // Test with larger dataset sizes for stress
    let sizes = [100, 1_000, 10_000, 500_000]; // Reduced max size for CI/quick check, increase for real stress

    let mut group = c.benchmark_group("Stress/HeavyModel");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(15)); // Longer time for heavy ops

    // 1. Insert Heavy Models
    for size in sizes.iter() {
        group.bench_with_input(BenchmarkId::new("Insert", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let mut rng = rand::rng();
                    // Pre-generate data to measure only insertion time
                    // We need some users for relationships
                    let users: Vec<User> =
                        (0..100).map(|_| generate_random_user(&mut rng)).collect();
                    let mut heavies = Vec::with_capacity(size);
                    for _ in 0..size {
                        heavies.push(generate_random_heavy(&mut rng, &users, &heavies));
                    }

                    let name = format!("stress_insert_{}_{}", size, rng.random::<u64>());
                    let (store, path) =
                        create_test_db::<Definition>(&name).expect("Failed to create DB");

                    // Insert users first so foreign keys technically exist (though not enforced by DB strictly yet)
                    let txn = store.begin_transaction().expect("Failed to begin txn");
                    {
                        let mut tables = txn.prepare_model::<User>().unwrap();
                        for user in &users {
                            user.create_entry(&mut tables).unwrap();
                        }
                    }
                    txn.commit().unwrap();

                    (store, heavies, CleanupGuard(path))
                },
                |(store, heavies, _guard)| {
                    let txn = store.begin_transaction().expect("Failed to begin txn");
                    {
                        let mut tables = txn.prepare_model::<HeavyModel>().unwrap();
                        for item in &heavies {
                            item.create_entry(&mut tables).unwrap();
                        }
                    }
                    txn.commit().unwrap();
                },
                BatchSize::PerIteration,
            );
        });
    }

    // 2. Read by Primary Key
    for size in sizes.iter() {
        group.bench_with_input(BenchmarkId::new("Read_PK", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let mut rng = rand::rng();
                    let users: Vec<User> =
                        (0..100).map(|_| generate_random_user(&mut rng)).collect();
                    let mut heavies = Vec::with_capacity(size);
                    for _ in 0..size {
                        heavies.push(generate_random_heavy(&mut rng, &users, &heavies));
                    }

                    let name = format!("stress_read_pk_{}_{}", size, rng.random::<u64>());
                    let (store, path) =
                        create_test_db::<Definition>(&name).expect("Failed to create DB");

                    let txn = store.begin_transaction().unwrap();
                    {
                        let mut user_tables = txn.prepare_model::<User>().unwrap();
                        for u in &users {
                            u.create_entry(&mut user_tables).unwrap();
                        }
                        let mut heavy_tables = txn.prepare_model::<HeavyModel>().unwrap();
                        for h in &heavies {
                            h.create_entry(&mut heavy_tables).unwrap();
                        }
                    }
                    txn.commit().unwrap();

                    (store, heavies, CleanupGuard(path))
                },
                |(store, heavies, _guard)| {
                    let txn = store.begin_transaction().unwrap();
                    let tables = txn.prepare_model::<HeavyModel>().unwrap();
                    for item in &heavies {
                        black_box(HeavyModel::read_entry(&item.id, &tables)).unwrap();
                    }
                },
                BatchSize::PerIteration,
            );
        });
    }

    // 3. Hydrated Read (Read Heavy + Read Related User)
    for size in sizes.iter() {
        group.bench_with_input(BenchmarkId::new("Read_Hydrated", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let mut rng = rand::rng();
                    let users: Vec<User> =
                        (0..100).map(|_| generate_random_user(&mut rng)).collect();
                    let mut heavies = Vec::with_capacity(size);
                    for _ in 0..size {
                        heavies.push(generate_random_heavy(&mut rng, &users, &heavies));
                    }

                    let name = format!("stress_read_hydra_{}_{}", size, rng.random::<u64>());
                    let (store, path) =
                        create_test_db::<Definition>(&name).expect("Failed to create DB");

                    let txn = store.begin_transaction().unwrap();
                    {
                        let mut user_tables = txn.prepare_model::<User>().unwrap();
                        for u in &users {
                            u.create_entry(&mut user_tables).unwrap();
                        }
                        let mut heavy_tables = txn.prepare_model::<HeavyModel>().unwrap();
                        for h in &heavies {
                            h.create_entry(&mut heavy_tables).unwrap();
                        }
                    }
                    txn.commit().unwrap();

                    (store, heavies, CleanupGuard(path))
                },
                |(store, heavies, _guard)| {
                    let txn = store.begin_transaction().unwrap();
                    let heavy_tables = txn.prepare_model::<HeavyModel>().unwrap();
                    let user_tables = txn.prepare_model::<User>().unwrap();

                    for item in &heavies {
                        // 1. Read Heavy
                        let heavy = HeavyModel::read_entry(&item.id, &heavy_tables)
                            .unwrap()
                            .unwrap();

                        // 2. Hydrate Creator
                        let creator_id = heavy.creator.get_primary_key();
                        black_box(User::read_entry(creator_id, &user_tables)).unwrap();
                    }
                },
                BatchSize::PerIteration,
            );
        });
    }

    group.finish();
}

criterion_group!(benches, bench_stress_operations);
criterion_main!(benches);
