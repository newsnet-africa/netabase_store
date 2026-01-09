use netabase_store::databases::redb::libp2p::Libp2pRedbStore;
use netabase_store::traits::database::store::NBStore;
use netabase_store::traits::libp2p::libp2p_model::Libp2pMetadata;
use netabase_store::libp2p::kad::{Record, RecordKey};
use netabase_store::libp2p::kad::store::RecordStore;
use netabase_store::libp2p::PeerId;
use netabase_store_examples::boilerplate_lib::{Definition, User, UserID, DefinitionRecord};
use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use rand::prelude::*;
use std::borrow::Cow;

mod common;
use common::*;

fn random_id(prefix: &str, rng: &mut impl Rng) -> String {
    let n: u64 = rng.random();
    format!("{}_{:016x}", prefix, n)
}

fn generate_random_record() -> Record {
    let mut rng = rand::rng();
    let id = UserID(random_id("user", &mut rng));
    let user = User {
        id: id.clone(),
        first_name: "Test".to_string(),
        last_name: "User".to_string(),
        age: 30,
        partner: netabase_store::relational::RelationalLink::new_dehydrated(UserID("none".to_string())),
        category: netabase_store::relational::RelationalLink::new_dehydrated(netabase_store_examples::boilerplate_lib::CategoryID("none".to_string())),
        bio: Default::default(),
        another: Default::default(),
    };
    
    // Create wrapper and convert
    let meta = Libp2pMetadata::default();
    let wrapper = DefinitionRecord(Definition::User(user), meta);
    wrapper.into()
}

fn generate_random_content_addressed_record() -> Record {
    let mut rng = rand::rng();
    
    // random string
    let mut random_string = |len: usize| {
        (0..len)
            .map(|_| rng.random_range(b'a'..=b'z') as char)
            .collect::<String>()
    };

    let post = netabase_store_examples::ImmutablePost {
        author: random_string(10),
        content: random_string(100),
        timestamp: rng.random(),
    };
    
    let envelope = netabase_store_examples::ImmutablePostEnvelope::from(post);
    let meta = Libp2pMetadata::default();
    let wrapper = DefinitionRecord(Definition::ImmutablePost(envelope), meta);
    wrapper.into()
}

fn generate_random_content_addressed_fast_record() -> Record {
    let mut rng = rand::rng();
    
    // random string
    let mut random_string = |len: usize| {
        (0..len)
            .map(|_| rng.random_range(b'a'..=b'z') as char)
            .collect::<String>()
    };

    let post = netabase_store_examples::ImmutablePostFast {
        author: random_string(10),
        content: random_string(100),
        timestamp: rng.random(),
    };
    
    let envelope = netabase_store_examples::ImmutablePostFastEnvelope::from(post);
    let meta = Libp2pMetadata::default();
    let wrapper = DefinitionRecord(Definition::ImmutablePostFast(envelope), meta);
    wrapper.into()
}

fn generate_random_content_addressed_crypto_record() -> Record {
    let mut rng = rand::rng();
    
    // random string
    let mut random_string = |len: usize| {
        (0..len)
            .map(|_| rng.random_range(b'a'..=b'z') as char)
            .collect::<String>()
    };

    let post = netabase_store_examples::ImmutablePostCrypto {
        author: random_string(10),
        content: random_string(100),
        timestamp: rng.random(),
    };
    
    let envelope = netabase_store_examples::ImmutablePostCryptoEnvelope::from(post);
    let meta = Libp2pMetadata::default();
    let wrapper = DefinitionRecord(Definition::ImmutablePostCrypto(envelope), meta);
    wrapper.into()
}

fn bench_record_store(c: &mut Criterion) {
    let sizes = [100, 1000];
    let local_id = PeerId::random();

    let mut group = c.benchmark_group("RecordStore");
    group.sample_size(10);

    for size in sizes.iter() {
        group.bench_with_input(BenchmarkId::new("Put", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let records: Vec<Record> = (0..size).map(|_| generate_random_record()).collect();
                    let name = format!("bench_record_put_{}_{}", size, rand::random::<u64>());
                    let store = create_test_db::<Definition>(&name).expect("Failed to create DB");
                    let record_store = Libp2pRedbStore::new(store, local_id);
                    (record_store, records)
                },
                |(mut record_store, records)| {
                    for r in records {
                        record_store.put(r).expect("Put failed");
                    }
                },
                BatchSize::PerIteration,
            );
        });
        
        group.bench_with_input(BenchmarkId::new("Get", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let records: Vec<Record> = (0..size).map(|_| generate_random_record()).collect();
                    let name = format!("bench_record_get_{}_{}", size, rand::random::<u64>());
                    let store = create_test_db::<Definition>(&name).expect("Failed to create DB");
                    let mut record_store = Libp2pRedbStore::new(store, local_id);
                    
                    for r in &records {
                        record_store.put(r.clone()).expect("Setup put failed");
                    }
                    
                    (record_store, records)
                },
                |(record_store, records)| {
                    for r in records {
                        let _ = record_store.get(&r.key); // Expect removed to avoid allocation in loop check
                    }
                },
                BatchSize::PerIteration,
            );
        });

        // Content Addressed Benchmarks
        group.bench_with_input(BenchmarkId::new("Put_ContentAddressed", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let records: Vec<Record> = (0..size).map(|_| generate_random_content_addressed_record()).collect();
                    let name = format!("bench_ca_record_put_{}_{}", size, rand::random::<u64>());
                    let store = create_test_db::<Definition>(&name).expect("Failed to create DB");
                    let record_store = Libp2pRedbStore::new(store, local_id);
                    (record_store, records)
                },
                |(mut record_store, records)| {
                    for r in records {
                        record_store.put(r).expect("Put failed");
                    }
                },
                BatchSize::PerIteration,
            );
        });
        
        group.bench_with_input(BenchmarkId::new("Get_ContentAddressed", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let records: Vec<Record> = (0..size).map(|_| generate_random_content_addressed_record()).collect();
                    let name = format!("bench_ca_record_get_{}_{}", size, rand::random::<u64>());
                    let store = create_test_db::<Definition>(&name).expect("Failed to create DB");
                    let mut record_store = Libp2pRedbStore::new(store, local_id);
                    
                    for r in &records {
                        record_store.put(r.clone()).expect("Setup put failed");
                    }
                    
                    (record_store, records)
                },
                |(record_store, records)| {
                    for r in records {
                        let _ = record_store.get(&r.key); 
                    }
                },
                BatchSize::PerIteration,
            );
        });

        group.bench_with_input(BenchmarkId::new("Put_ContentAddressed_Fast", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let records: Vec<Record> = (0..size).map(|_| generate_random_content_addressed_fast_record()).collect();
                    let name = format!("bench_ca_fast_record_put_{}_{}", size, rand::random::<u64>());
                    let store = create_test_db::<Definition>(&name).expect("Failed to create DB");
                    let record_store = Libp2pRedbStore::new(store, local_id);
                    (record_store, records)
                },
                |(mut record_store, records)| {
                    for r in records {
                        record_store.put(r).expect("Put failed");
                    }
                },
                BatchSize::PerIteration,
            );
        });
        
        group.bench_with_input(BenchmarkId::new("Get_ContentAddressed_Fast", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let records: Vec<Record> = (0..size).map(|_| generate_random_content_addressed_fast_record()).collect();
                    let name = format!("bench_ca_fast_record_get_{}_{}", size, rand::random::<u64>());
                    let store = create_test_db::<Definition>(&name).expect("Failed to create DB");
                    let mut record_store = Libp2pRedbStore::new(store, local_id);
                    
                    for r in &records {
                        record_store.put(r.clone()).expect("Setup put failed");
                    }
                    
                    (record_store, records)
                },
                |(record_store, records)| {
                    for r in records {
                        let _ = record_store.get(&r.key); 
                    }
                },
                BatchSize::PerIteration,
            );
        });

        group.bench_with_input(BenchmarkId::new("Put_ContentAddressed_Crypto", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let records: Vec<Record> = (0..size).map(|_| generate_random_content_addressed_crypto_record()).collect();
                    let name = format!("bench_ca_crypto_record_put_{}_{}", size, rand::random::<u64>());
                    let store = create_test_db::<Definition>(&name).expect("Failed to create DB");
                    let record_store = Libp2pRedbStore::new(store, local_id);
                    (record_store, records)
                },
                |(mut record_store, records)| {
                    for r in records {
                        record_store.put(r).expect("Put failed");
                    }
                },
                BatchSize::PerIteration,
            );
        });
        
        group.bench_with_input(BenchmarkId::new("Get_ContentAddressed_Crypto", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let records: Vec<Record> = (0..size).map(|_| generate_random_content_addressed_crypto_record()).collect();
                    let name = format!("bench_ca_crypto_record_get_{}_{}", size, rand::random::<u64>());
                    let store = create_test_db::<Definition>(&name).expect("Failed to create DB");
                    let mut record_store = Libp2pRedbStore::new(store, local_id);
                    
                    for r in &records {
                        record_store.put(r.clone()).expect("Setup put failed");
                    }
                    
                    (record_store, records)
                },
                |(record_store, records)| {
                    for r in records {
                        let _ = record_store.get(&r.key); 
                    }
                },
                BatchSize::PerIteration,
            );
        });
    }
    group.finish();
}

criterion_group!(benches, bench_record_store);
criterion_main!(benches);
