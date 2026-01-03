use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use netabase_store::databases::redb::transaction::RedbModelCrud;
use netabase_store::relational::RelationalLink;
use netabase_store_examples::boilerplate_lib::definition::{
    User, UserAge, UserBlobItem, UserBlobKeys, UserCategory, UserID, UserName, UserPartner,
    UserRelationalKeys, UserSecondaryKeys,
};
use netabase_store_examples::boilerplate_lib::models::blob_types::{
    AnotherLargeUserFile, LargeUserFile,
};
use netabase_store_examples::boilerplate_lib::{CategoryID, Definition, DefinitionSubscriptions};
use rand::prelude::*;
use redb::{MultimapTableDefinition, ReadableDatabase, ReadableTable, TableDefinition};
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

/// Helper to split blob data like the abstracted version does
/// Mimics NetabaseBlobItem::split_into_blobs() behavior
fn split_blob_into_chunks<T: bincode::Encode>(item: &T) -> Vec<(u8, Vec<u8>)> {
    let serialized = bincode::encode_to_vec(item, bincode::config::standard()).unwrap();

    if serialized.is_empty() {
        return Vec::new();
    }

    serialized
        .chunks(60000) // 60KB chunks
        .enumerate()
        .map(|(i, chunk)| (i as u8, chunk.to_vec()))
        .collect()
}

pub fn generate_random_user() -> User {
    let mut rng = rand::rng();

    // sample names
    let names = [
        "Alice", "Bob", "Carol", "Dave", "Eve", "Frank", "Grace", "Heidi", "Ivan", "Judy",
        "Mallory", "Niaj", "Olivia", "Peggy", "Rupert", "Sybil", "Trent", "Victor", "Walter",
    ];
    let name = names.choose(&mut rng).unwrap().to_string();

    // random age 1..=100
    let age: u8 = rng.random_range(1..=100);

    // random user id and category id
    let user_id = UserID(random_id("user", &mut rng));
    let category_id = CategoryID(random_id("cat", &mut rng));

    // random choice to either set partner to "none" or a random id
    let partner = if rng.random_bool(0.5) {
        RelationalLink::new_dehydrated(UserID("none".to_string()))
    } else {
        RelationalLink::new_dehydrated(UserID(random_id("user", &mut rng)))
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
    indices.shuffle(&mut rng);
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

// Helper struct to clean up the DB file when it goes out of scope
struct CleanupGuard(PathBuf);
impl Drop for CleanupGuard {
    fn drop(&mut self) {
        std::fs::remove_file(&self.0).ok();
    }
}

// --- Benchmarks ---

fn bench_crud_operations(c: &mut Criterion) {
    let sizes = [0, 100, 1_000, 10_000, 100_000];

    // Define table definitions matching User::TREE_NAMES for raw redb operations
    const MAIN: TableDefinition<UserID, User> = TableDefinition::new("User:User:Primary:Main");
    const SEC_NAME: MultimapTableDefinition<UserSecondaryKeys, UserID> =
        MultimapTableDefinition::new("Definition:User:Secondary:Name");
    const SEC_AGE: MultimapTableDefinition<UserSecondaryKeys, UserID> =
        MultimapTableDefinition::new("Definition:User:Secondary:Age");
    const REL_PARTNER: MultimapTableDefinition<UserRelationalKeys, UserID> =
        MultimapTableDefinition::new("Definition:User:Relational:Partner");
    const REL_CATEGORY: MultimapTableDefinition<UserRelationalKeys, UserID> =
        MultimapTableDefinition::new("Definition:User:Relational:Category");
    const SUB_TOPIC1: MultimapTableDefinition<DefinitionSubscriptions, UserID> =
        MultimapTableDefinition::new("Definition:Subscription:Topic1");
    const SUB_TOPIC2: MultimapTableDefinition<DefinitionSubscriptions, UserID> =
        MultimapTableDefinition::new("Definition:Subscription:Topic2");
    // Updated blob table name and type usage
    const BLOB_BIO: MultimapTableDefinition<UserBlobKeys, UserBlobItem> =
        MultimapTableDefinition::new("Definition:User:Blob:Bio");
    const BLOB_ANOTHER: MultimapTableDefinition<UserBlobKeys, UserBlobItem> =
        MultimapTableDefinition::new("Definition:User:Blob:Another");

    // --- Insert Benchmarks ---
    let mut insert_group = c.benchmark_group("CRUD/Insert");
    insert_group.sample_size(10);
    insert_group.measurement_time(std::time::Duration::from_secs(10));

    for size in sizes.iter() {
        // Abstracted Insert
        insert_group.bench_with_input(BenchmarkId::new("Abstracted", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let users: Vec<User> = (0..size).map(|_| generate_random_user()).collect();
                    let name = format!("bench_opt_insert_{}_{}", size, rand::random::<u64>());
                    let (store, path) =
                        create_test_db::<Definition>(&name).expect("Failed to create DB");
                    (store, users, CleanupGuard(path))
                },
                |(store, users, _guard)| {
                    // MEASURED: Transaction, table opening, and insert loop
                    let txn = store.begin_write().expect("Failed to begin txn");
                    {
                        let mut tables = txn
                            .prepare_model::<User>()
                            .expect("Failed to prepare model");
                        for user in &users {
                            let user: &User = black_box(user);
                            user.create_entry(&mut tables)
                                .expect("Failed to create user");
                        }
                    }
                    txn.commit().expect("Failed to commit");
                },
                BatchSize::PerIteration,
            );
        });

        // Raw Redb Insert
        insert_group.bench_with_input(BenchmarkId::new("Raw", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let users: Vec<User> = (0..size).map(|_| generate_random_user()).collect();
                    let name = format!("bench_raw_insert_{}_{}", size, rand::random::<u64>());
                    let path = PathBuf::from(format!("/tmp/netabase_test_{}.redb", name));
                    if path.exists() {
                        std::fs::remove_file(&path).ok();
                    }
                    let db = redb::Database::create(&path).expect("Failed to create raw DB");
                    (db, users, CleanupGuard(path))
                },
                |(db, users, _guard)| {
                    // MEASURED: Transaction, table opening, and insert loop
                    let txn = db.begin_write().expect("Failed to begin txn");
                    {
                        let mut main_table = txn.open_table(MAIN).expect("Failed to open main");
                        let mut sec_name = txn
                            .open_multimap_table(SEC_NAME)
                            .expect("Failed to open sec name");
                        let mut sec_age = txn
                            .open_multimap_table(SEC_AGE)
                            .expect("Failed to open sec age");
                        let mut rel_partner = txn
                            .open_multimap_table(REL_PARTNER)
                            .expect("Failed to open rel partner");
                        let mut rel_category = txn
                            .open_multimap_table(REL_CATEGORY)
                            .expect("Failed to open rel category");
                        let mut sub_topic1 = txn
                            .open_multimap_table(SUB_TOPIC1)
                            .expect("Failed to open sub topic1");
                        let mut sub_topic2 = txn
                            .open_multimap_table(SUB_TOPIC2)
                            .expect("Failed to open sub topic2");
                        let mut blob_bio_table = txn
                            .open_multimap_table(BLOB_BIO)
                            .expect("Failed to open blob bio table");
                        let mut blob_another_table = txn
                            .open_multimap_table(BLOB_ANOTHER)
                            .expect("Failed to open blob another table");

                        for user in &users {
                            let user = black_box(user);
                            let user_id = &user.id;

                            // Insert Main
                            main_table
                                .insert(user_id, user)
                                .expect("Failed to insert main");

                            // Insert Secondary
                            sec_name
                                .insert(
                                    &UserSecondaryKeys::Name(UserName(user.name.clone())),
                                    user_id,
                                )
                                .expect("Failed to insert sec name");
                            sec_age
                                .insert(&UserSecondaryKeys::Age(UserAge(user.age)), user_id)
                                .expect("Failed to insert sec age");

                            // Insert Relational
                            rel_partner
                                .insert(
                                    &UserRelationalKeys::Partner(UserPartner(
                                        user.partner.get_primary_key().clone(),
                                    )),
                                    user_id,
                                )
                                .expect("Failed to insert rel partner");
                            rel_category
                                .insert(
                                    &UserRelationalKeys::Category(UserCategory(
                                        user.category.get_primary_key().clone(),
                                    )),
                                    user_id,
                                )
                                .expect("Failed to insert rel category");

                            // Insert Subscriptions
                            for subscription in &user.subscriptions {
                                match subscription {
                                    DefinitionSubscriptions::Topic1 => {
                                        sub_topic1
                                            .insert(subscription, user_id)
                                            .expect("Failed to insert sub topic1");
                                    }
                                    DefinitionSubscriptions::Topic2 => {
                                        sub_topic2
                                            .insert(subscription, user_id)
                                            .expect("Failed to insert sub topic2");
                                    }
                                    _ => {}
                                }
                            }

                            // Insert Blobs - properly split like the abstracted version
                            // Bio field
                            let bio_chunks = split_blob_into_chunks(&user.bio);
                            for (index, chunk) in bio_chunks {
                                blob_bio_table
                                    .insert(
                                        &UserBlobKeys::Bio {
                                            owner: user_id.clone(),
                                        },
                                        &UserBlobItem::Bio {
                                            index,
                                            value: chunk,
                                        },
                                    )
                                    .expect("Failed to insert bio blob");
                            }

                            // Another field
                            let another_chunks = split_blob_into_chunks(&user.another);
                            for (index, chunk) in another_chunks {
                                blob_another_table
                                    .insert(
                                        &UserBlobKeys::Another {
                                            owner: user_id.clone(),
                                        },
                                        &UserBlobItem::Another {
                                            index,
                                            value: chunk,
                                        },
                                    )
                                    .expect("Failed to insert another blob");
                            }
                        }
                    }
                    txn.commit().expect("Failed to commit");
                },
                BatchSize::PerIteration,
            );
        });
    }
    insert_group.finish();

    // --- Read Benchmarks ---
    let mut read_group = c.benchmark_group("CRUD/Read");
    read_group.sample_size(10);
    read_group.measurement_time(std::time::Duration::from_secs(10));

    for size in sizes.iter() {
        // Abstracted Read
        read_group.bench_with_input(BenchmarkId::new("Abstracted", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let users: Vec<User> = (0..size).map(|_| generate_random_user()).collect();
                    let name = format!("bench_opt_read_{}_{}", size, rand::random::<u64>());
                    let (store, path) =
                        create_test_db::<Definition>(&name).expect("Failed to create DB");

                    // Insert data in setup
                    let txn = store.begin_write().expect("Failed to begin txn");
                    {
                        let mut tables = txn
                            .prepare_model::<User>()
                            .expect("Failed to prepare model");
                        for user in &users {
                            user.create_entry(&mut tables)
                                .expect("Failed to create user");
                        }
                    }
                    txn.commit().expect("Failed to commit");

                    (store, users, CleanupGuard(path))
                },
                |(store, users, _guard)| {
                    // MEASURED: Transaction, table opening, and read loop
                    let txn = store.begin_read().expect("Failed to begin txn");
                    {
                        let tables = txn
                            .prepare_model::<User>()
                            .expect("Failed to prepare model");
                        for user in &users {
                            black_box(User::read_default(black_box(&user.id), &tables))
                                .expect("Failed to read user");
                        }
                    }
                    txn.commit().expect("Failed to commit");
                },
                BatchSize::PerIteration,
            );
        });

        // Raw Redb Read
        read_group.bench_with_input(BenchmarkId::new("Raw", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let users: Vec<User> = (0..size).map(|_| generate_random_user()).collect();
                    let name = format!("bench_raw_read_{}_{}", size, rand::random::<u64>());
                    let path = PathBuf::from(format!("/tmp/netabase_test_{}.redb", name));
                    if path.exists() {
                        std::fs::remove_file(&path).ok();
                    }
                    let db = redb::Database::create(&path).expect("Failed to create raw DB");

                    // Insert data in setup
                    let txn = db.begin_write().expect("Failed to begin txn");
                    {
                        let mut main_table = txn.open_table(MAIN).expect("Failed to open main");
                        let mut sec_name = txn
                            .open_multimap_table(SEC_NAME)
                            .expect("Failed to open sec name");
                        let mut sec_age = txn
                            .open_multimap_table(SEC_AGE)
                            .expect("Failed to open sec age");
                        let mut rel_partner = txn
                            .open_multimap_table(REL_PARTNER)
                            .expect("Failed to open rel partner");
                        let mut rel_category = txn
                            .open_multimap_table(REL_CATEGORY)
                            .expect("Failed to open rel category");
                        let mut sub_topic1 = txn
                            .open_multimap_table(SUB_TOPIC1)
                            .expect("Failed to open sub topic1");
                        let mut sub_topic2 = txn
                            .open_multimap_table(SUB_TOPIC2)
                            .expect("Failed to open sub topic2");
                        let mut blob_bio_table = txn
                            .open_multimap_table(BLOB_BIO)
                            .expect("Failed to open blob bio table");
                        let mut blob_another_table = txn
                            .open_multimap_table(BLOB_ANOTHER)
                            .expect("Failed to open blob another table");

                        for user in &users {
                            let user_id = &user.id;
                            main_table.insert(user_id, user).unwrap();
                            sec_name
                                .insert(
                                    &UserSecondaryKeys::Name(UserName(user.name.clone())),
                                    user_id,
                                )
                                .unwrap();
                            sec_age
                                .insert(&UserSecondaryKeys::Age(UserAge(user.age)), user_id)
                                .unwrap();
                            rel_partner
                                .insert(
                                    &UserRelationalKeys::Partner(UserPartner(
                                        user.partner.get_primary_key().clone(),
                                    )),
                                    user_id,
                                )
                                .unwrap();
                            rel_category
                                .insert(
                                    &UserRelationalKeys::Category(UserCategory(
                                        user.category.get_primary_key().clone(),
                                    )),
                                    user_id,
                                )
                                .unwrap();

                            // Insert Subscriptions
                            for subscription in &user.subscriptions {
                                match subscription {
                                    DefinitionSubscriptions::Topic1 => {
                                        sub_topic1.insert(subscription, user_id).unwrap();
                                    }
                                    DefinitionSubscriptions::Topic2 => {
                                        sub_topic2.insert(subscription, user_id).unwrap();
                                    }
                                    _ => {}
                                }
                            }

                            // Insert Blobs - properly split like the abstracted version
                            let bio_chunks = split_blob_into_chunks(&user.bio);
                            for (index, chunk) in bio_chunks {
                                blob_bio_table
                                    .insert(
                                        &UserBlobKeys::Bio {
                                            owner: user_id.clone(),
                                        },
                                        &UserBlobItem::Bio {
                                            index,
                                            value: chunk,
                                        },
                                    )
                                    .unwrap();
                            }

                            let another_chunks = split_blob_into_chunks(&user.another);
                            for (index, chunk) in another_chunks {
                                blob_another_table
                                    .insert(
                                        &UserBlobKeys::Another {
                                            owner: user_id.clone(),
                                        },
                                        &UserBlobItem::Another {
                                            index,
                                            value: chunk,
                                        },
                                    )
                                    .unwrap();
                            }
                        }
                    }
                    txn.commit().unwrap();

                    (db, users, CleanupGuard(path))
                },
                |(db, users, _guard)| {
                    // MEASURED: Transaction, table opening, and read loop
                    let txn = db.begin_read().expect("Failed to begin txn");
                    let main_table = txn.open_table(MAIN).expect("Failed to open main");
                    for user in &users {
                        black_box(main_table.get(black_box(&user.id)))
                            .expect("Failed to get user")
                            .map(|g| g.value());
                    }
                },
                BatchSize::PerIteration,
            );
        });
    }
    read_group.finish();

    // --- Delete Benchmarks ---
    let mut delete_group = c.benchmark_group("CRUD/Delete");
    delete_group.sample_size(10);
    delete_group.measurement_time(std::time::Duration::from_secs(10));

    for size in sizes.iter() {
        // Abstracted Delete
        delete_group.bench_with_input(BenchmarkId::new("Abstracted", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let users: Vec<User> = (0..size).map(|_| generate_random_user()).collect();
                    let name = format!("bench_opt_del_{}_{}", size, rand::random::<u64>());
                    let (store, path) =
                        create_test_db::<Definition>(&name).expect("Failed to create DB");

                    // Insert data in setup
                    let txn = store.begin_write().expect("Failed to begin txn");
                    {
                        let mut tables = txn
                            .prepare_model::<User>()
                            .expect("Failed to prepare model");
                        for user in &users {
                            user.create_entry(&mut tables)
                                .expect("Failed to create user");
                        }
                    }
                    txn.commit().expect("Failed to commit");

                    (store, users, CleanupGuard(path))
                },
                |(store, users, _guard)| {
                    // MEASURED: Transaction, table opening, and delete loop
                    let txn = store.begin_write().expect("Failed to begin txn");
                    {
                        let mut tables = txn
                            .prepare_model::<User>()
                            .expect("Failed to prepare model");
                        for user in users {
                            let user = black_box(user);
                            User::delete_entry(&user.id, &mut tables)
                                .expect("Failed to delete user");
                        }
                    }
                    txn.commit().expect("Failed to commit");
                },
                BatchSize::PerIteration,
            );
        });

        // Raw Redb Delete
        delete_group.bench_with_input(BenchmarkId::new("Raw", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let users: Vec<User> = (0..size).map(|_| generate_random_user()).collect();
                    let name = format!("bench_raw_del_{}_{}", size, rand::random::<u64>());
                    let path = PathBuf::from(format!("/tmp/netabase_test_{}.redb", name));
                    if path.exists() {
                        std::fs::remove_file(&path).ok();
                    }
                    let db = redb::Database::create(&path).expect("Failed to create raw DB");

                    // Insert data in setup
                    let txn = db.begin_write().expect("Failed to begin txn");
                    {
                        let mut main_table = txn.open_table(MAIN).expect("Failed to open main");
                        let mut sec_name = txn
                            .open_multimap_table(SEC_NAME)
                            .expect("Failed to open sec name");
                        let mut sec_age = txn
                            .open_multimap_table(SEC_AGE)
                            .expect("Failed to open sec age");
                        let mut rel_partner = txn
                            .open_multimap_table(REL_PARTNER)
                            .expect("Failed to open rel partner");
                        let mut rel_category = txn
                            .open_multimap_table(REL_CATEGORY)
                            .expect("Failed to open rel category");
                        let mut sub_topic1 = txn
                            .open_multimap_table(SUB_TOPIC1)
                            .expect("Failed to open sub topic1");
                        let mut sub_topic2 = txn
                            .open_multimap_table(SUB_TOPIC2)
                            .expect("Failed to open sub topic2");
                        let mut blob_bio_table = txn
                            .open_multimap_table(BLOB_BIO)
                            .expect("Failed to open blob bio table");
                        let mut blob_another_table = txn
                            .open_multimap_table(BLOB_ANOTHER)
                            .expect("Failed to open blob another table");

                        for user in &users {
                            let user_id = &user.id;
                            main_table.insert(user_id, user).unwrap();
                            sec_name
                                .insert(
                                    &UserSecondaryKeys::Name(UserName(user.name.clone())),
                                    user_id,
                                )
                                .unwrap();
                            sec_age
                                .insert(&UserSecondaryKeys::Age(UserAge(user.age)), user_id)
                                .unwrap();
                            rel_partner
                                .insert(
                                    &UserRelationalKeys::Partner(UserPartner(
                                        user.partner.get_primary_key().clone(),
                                    )),
                                    user_id,
                                )
                                .unwrap();
                            rel_category
                                .insert(
                                    &UserRelationalKeys::Category(UserCategory(
                                        user.category.get_primary_key().clone(),
                                    )),
                                    user_id,
                                )
                                .unwrap();

                            // Insert Subscriptions
                            for subscription in &user.subscriptions {
                                match subscription {
                                    DefinitionSubscriptions::Topic1 => {
                                        sub_topic1.insert(subscription, user_id).unwrap();
                                    }
                                    DefinitionSubscriptions::Topic2 => {
                                        sub_topic2.insert(subscription, user_id).unwrap();
                                    }
                                    _ => {}
                                }
                            }

                            // Insert Blobs - properly split like the abstracted version
                            let bio_chunks = split_blob_into_chunks(&user.bio);
                            for (index, chunk) in bio_chunks {
                                blob_bio_table
                                    .insert(
                                        &UserBlobKeys::Bio {
                                            owner: user_id.clone(),
                                        },
                                        &UserBlobItem::Bio {
                                            index,
                                            value: chunk,
                                        },
                                    )
                                    .unwrap();
                            }

                            let another_chunks = split_blob_into_chunks(&user.another);
                            for (index, chunk) in another_chunks {
                                blob_another_table
                                    .insert(
                                        &UserBlobKeys::Another {
                                            owner: user_id.clone(),
                                        },
                                        &UserBlobItem::Another {
                                            index,
                                            value: chunk,
                                        },
                                    )
                                    .unwrap();
                            }
                        }
                    }
                    txn.commit().unwrap();

                    (db, users, CleanupGuard(path))
                },
                |(db, users, _guard)| {
                    // MEASURED: Transaction, table opening, and delete loop
                    let txn = db.begin_write().expect("Failed to begin txn");
                    {
                        let mut main_table = txn.open_table(MAIN).expect("Failed to open main");
                        let mut sec_name = txn
                            .open_multimap_table(SEC_NAME)
                            .expect("Failed to open sec name");
                        let mut sec_age = txn
                            .open_multimap_table(SEC_AGE)
                            .expect("Failed to open sec age");
                        let mut rel_partner = txn
                            .open_multimap_table(REL_PARTNER)
                            .expect("Failed to open rel partner");
                        let mut rel_category = txn
                            .open_multimap_table(REL_CATEGORY)
                            .expect("Failed to open rel category");
                        let mut sub_topic1 = txn
                            .open_multimap_table(SUB_TOPIC1)
                            .expect("Failed to open sub topic1");
                        let mut sub_topic2 = txn
                            .open_multimap_table(SUB_TOPIC2)
                            .expect("Failed to open sub topic2");
                        let mut blob_bio_table = txn
                            .open_multimap_table(BLOB_BIO)
                            .expect("Failed to open blob bio table");
                        let mut blob_another_table = txn
                            .open_multimap_table(BLOB_ANOTHER)
                            .expect("Failed to open blob another table");

                        for user in &users {
                            let user_id = &user.id;
                            let stored_user = black_box(main_table.get(user_id))
                                .expect("Failed to get user")
                                .expect("User not found")
                                .value();

                            main_table.remove(user_id).unwrap();

                            sec_name
                                .remove(
                                    &UserSecondaryKeys::Name(UserName(black_box(stored_user.name))),
                                    user_id,
                                )
                                .unwrap();
                            sec_age
                                .remove(
                                    &UserSecondaryKeys::Age(UserAge(black_box(stored_user.age))),
                                    user_id,
                                )
                                .unwrap();
                            rel_partner
                                .remove(
                                    &UserRelationalKeys::Partner(UserPartner(black_box(
                                        stored_user.partner.get_primary_key().clone(),
                                    ))),
                                    user_id,
                                )
                                .unwrap();
                            rel_category
                                .remove(
                                    &UserRelationalKeys::Category(UserCategory(black_box(
                                        stored_user.category.get_primary_key().clone(),
                                    ))),
                                    user_id,
                                )
                                .unwrap();

                            // Remove Subscriptions
                            for subscription in &stored_user.subscriptions {
                                match subscription {
                                    DefinitionSubscriptions::Topic1 => {
                                        sub_topic1.remove(subscription, user_id).unwrap();
                                    }
                                    DefinitionSubscriptions::Topic2 => {
                                        sub_topic2.remove(subscription, user_id).unwrap();
                                    }
                                    _ => {}
                                }
                            }

                            // Remove Blobs - properly remove all chunks like the abstracted version
                            let bio_chunks = split_blob_into_chunks(&stored_user.bio);
                            for (index, chunk) in bio_chunks {
                                blob_bio_table
                                    .remove(
                                        &UserBlobKeys::Bio {
                                            owner: user_id.clone(),
                                        },
                                        &UserBlobItem::Bio {
                                            index,
                                            value: chunk,
                                        },
                                    )
                                    .unwrap();
                            }

                            let another_chunks = split_blob_into_chunks(&stored_user.another);
                            for (index, chunk) in another_chunks {
                                blob_another_table
                                    .remove(
                                        &UserBlobKeys::Another {
                                            owner: user_id.clone(),
                                        },
                                        &UserBlobItem::Another {
                                            index,
                                            value: chunk,
                                        },
                                    )
                                    .unwrap();
                            }
                        }
                    }
                    txn.commit().expect("Failed to commit");
                },
                BatchSize::PerIteration,
            );
        });
    }
    delete_group.finish();
}

criterion_group!(benches, bench_crud_operations);
criterion_main!(benches);
