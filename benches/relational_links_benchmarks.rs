//! Comprehensive benchmarks for RelationalLink functionality
//!
//! This benchmark suite measures the performance characteristics of RelationalLink
//! operations across different scenarios to assess overhead and identify
//! performance characteristics.

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use netabase_store::links::RelationalLink;
use netabase_store::*;
use std::hint::black_box as std_black_box;
use tempfile::TempDir;

// Test schema for benchmarks
#[netabase_definition_module(BenchmarkDefinition, BenchmarkDefinitionKey)]
mod benchmark_schema {
    use super::*;

    #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode, PartialEq)]
    #[netabase(BenchmarkDefinition)]
    pub struct SmallEntity {
        #[primary_key]
        pub id: u64,
        pub value: u32,
    }

    #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode, PartialEq)]
    #[netabase(BenchmarkDefinition)]
    pub struct MediumEntity {
        #[primary_key]
        pub id: u64,
        pub name: String,
        pub description: String,
        pub tags: Vec<String>,
        pub metadata: std::collections::HashMap<String, String>,
    }

    #[derive(NetabaseModel, Clone, Debug, bincode::Encode, bincode::Decode, PartialEq)]
    #[netabase(BenchmarkDefinition)]
    pub struct LargeEntity {
        #[primary_key]
        pub id: u64,
        pub title: String,
        pub content: String,
        pub attachments: Vec<String>,
        pub properties: std::collections::HashMap<String, serde_json::Value>,
        pub created_at: u64,
        pub updated_at: u64,
        pub version: u32,
    }

    // Entity with multiple RelationalLink fields
    #[derive(Clone, Debug, bincode::Encode, bincode::Decode, PartialEq)]
    pub struct ComplexEntity {
        pub id: u64,
        pub name: String,
        pub small_ref: RelationalLink<BenchmarkDefinition, SmallEntity>,
        pub medium_ref: RelationalLink<BenchmarkDefinition, MediumEntity>,
        pub large_ref: RelationalLink<BenchmarkDefinition, LargeEntity>,
        pub small_collection: Vec<RelationalLink<BenchmarkDefinition, SmallEntity>>,
        pub medium_collection: Vec<RelationalLink<BenchmarkDefinition, MediumEntity>>,
    }
}

use benchmark_schema::*;

// Helper function to create test entities
fn create_small_entity(id: u64) -> SmallEntity {
    SmallEntity {
        id,
        value: (id * 42) as u32,
    }
}

fn create_medium_entity(id: u64) -> MediumEntity {
    MediumEntity {
        id,
        name: format!("Medium Entity {}", id),
        description: format!("This is a medium-sized entity with ID {}", id),
        tags: vec![
            "tag1".to_string(),
            "tag2".to_string(),
            format!("tag-{}", id),
        ],
        metadata: {
            let mut map = std::collections::HashMap::new();
            map.insert("key1".to_string(), "value1".to_string());
            map.insert("key2".to_string(), "value2".to_string());
            map.insert(format!("key-{}", id), format!("value-{}", id));
            map
        },
    }
}

fn create_large_entity(id: u64) -> LargeEntity {
    LargeEntity {
        id,
        title: format!(
            "Large Entity {} - Very Long Title That Takes More Space",
            id
        ),
        content: format!(
            "This is a large entity with ID {}. It contains a lot of text content that would \
             represent something like a blog post or article. The content is intentionally \
             verbose to simulate real-world usage where entities contain substantial amounts \
             of data that need to be serialized and stored. This helps us measure the \
             performance impact of using Entity vs Reference variants in RelationalLink.",
            id
        ),
        attachments: (0..10)
            .map(|i| format!("attachment-{}-{}.txt", id, i))
            .collect(),
        properties: {
            let mut map = std::collections::HashMap::new();
            for i in 0..20 {
                map.insert(
                    format!("property-{}", i),
                    serde_json::json!({
                        "id": id,
                        "index": i,
                        "description": format!("Property {} for entity {}", i, id)
                    }),
                );
            }
            map
        },
        created_at: 1640995200 + id * 3600, // Incremental timestamps
        updated_at: 1640995200 + id * 3600 + 1800,
        version: (id % 100) as u32,
    }
}

// Benchmark: RelationalLink creation performance
fn bench_relational_link_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("relational_link_creation");

    let small_entity = create_small_entity(1);
    let medium_entity = create_medium_entity(1);
    let large_entity = create_large_entity(1);

    // Small Entity benchmarks
    group.bench_function("small_entity_link", |b| {
        b.iter(|| {
            let entity_link = black_box(RelationalLink::Entity(small_entity.clone()));
            std_black_box(entity_link)
        })
    });

    group.bench_function("small_reference_link", |b| {
        b.iter(|| {
            let ref_link =
                black_box(RelationalLink::<BenchmarkDefinition, SmallEntity>::from_key(1));
            std_black_box(ref_link)
        })
    });

    // Medium Entity benchmarks
    group.bench_function("medium_entity_link", |b| {
        b.iter(|| {
            let entity_link = black_box(RelationalLink::Entity(medium_entity.clone()));
            std_black_box(entity_link)
        })
    });

    group.bench_function("medium_reference_link", |b| {
        b.iter(|| {
            let ref_link =
                black_box(RelationalLink::<BenchmarkDefinition, MediumEntity>::from_key(1));
            std_black_box(ref_link)
        })
    });

    // Large Entity benchmarks
    group.bench_function("large_entity_link", |b| {
        b.iter(|| {
            let entity_link = black_box(RelationalLink::Entity(large_entity.clone()));
            std_black_box(entity_link)
        })
    });

    group.bench_function("large_reference_link", |b| {
        b.iter(|| {
            let ref_link =
                black_box(RelationalLink::<BenchmarkDefinition, LargeEntity>::from_key(1));
            std_black_box(ref_link)
        })
    });

    group.finish();
}

// Benchmark: Serialization performance
fn bench_relational_link_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("relational_link_serialization");

    let small_entity = create_small_entity(1);
    let medium_entity = create_medium_entity(1);
    let large_entity = create_large_entity(1);

    let small_entity_link = RelationalLink::Entity(small_entity.clone());
    let small_ref_link = RelationalLink::<BenchmarkDefinition, SmallEntity>::from_key(1);
    let medium_entity_link = RelationalLink::Entity(medium_entity.clone());
    let medium_ref_link = RelationalLink::<BenchmarkDefinition, MediumEntity>::from_key(1);
    let large_entity_link = RelationalLink::Entity(large_entity.clone());
    let large_ref_link = RelationalLink::<BenchmarkDefinition, LargeEntity>::from_key(1);

    // Small entity serialization
    group.bench_function("small_entity_serialize", |b| {
        b.iter(|| {
            let encoded = black_box(
                bincode::encode_to_vec(&small_entity_link, bincode::config::standard()).unwrap(),
            );
            std_black_box(encoded)
        })
    });

    group.bench_function("small_reference_serialize", |b| {
        b.iter(|| {
            let encoded = black_box(
                bincode::encode_to_vec(&small_ref_link, bincode::config::standard()).unwrap(),
            );
            std_black_box(encoded)
        })
    });

    // Medium entity serialization
    group.bench_function("medium_entity_serialize", |b| {
        b.iter(|| {
            let encoded = black_box(
                bincode::encode_to_vec(&medium_entity_link, bincode::config::standard()).unwrap(),
            );
            std_black_box(encoded)
        })
    });

    group.bench_function("medium_reference_serialize", |b| {
        b.iter(|| {
            let encoded = black_box(
                bincode::encode_to_vec(&medium_ref_link, bincode::config::standard()).unwrap(),
            );
            std_black_box(encoded)
        })
    });

    // Large entity serialization
    group.bench_function("large_entity_serialize", |b| {
        b.iter(|| {
            let encoded = black_box(
                bincode::encode_to_vec(&large_entity_link, bincode::config::standard()).unwrap(),
            );
            std_black_box(encoded)
        })
    });

    group.bench_function("large_reference_serialize", |b| {
        b.iter(|| {
            let encoded = black_box(
                bincode::encode_to_vec(&large_ref_link, bincode::config::standard()).unwrap(),
            );
            std_black_box(encoded)
        })
    });

    group.finish();
}

// Benchmark: Deserialization performance
fn bench_relational_link_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("relational_link_deserialization");

    let small_entity = create_small_entity(1);
    let medium_entity = create_medium_entity(1);
    let large_entity = create_large_entity(1);

    let small_entity_link = RelationalLink::Entity(small_entity);
    let small_ref_link = RelationalLink::<BenchmarkDefinition, SmallEntity>::from_key(1);
    let medium_entity_link = RelationalLink::Entity(medium_entity);
    let medium_ref_link = RelationalLink::<BenchmarkDefinition, MediumEntity>::from_key(1);
    let large_entity_link = RelationalLink::Entity(large_entity);
    let large_ref_link = RelationalLink::<BenchmarkDefinition, LargeEntity>::from_key(1);

    // Pre-serialize for deserialization benchmarks
    let small_entity_encoded =
        bincode::encode_to_vec(&small_entity_link, bincode::config::standard()).unwrap();
    let small_ref_encoded =
        bincode::encode_to_vec(&small_ref_link, bincode::config::standard()).unwrap();
    let medium_entity_encoded =
        bincode::encode_to_vec(&medium_entity_link, bincode::config::standard()).unwrap();
    let medium_ref_encoded =
        bincode::encode_to_vec(&medium_ref_link, bincode::config::standard()).unwrap();
    let large_entity_encoded =
        bincode::encode_to_vec(&large_entity_link, bincode::config::standard()).unwrap();
    let large_ref_encoded =
        bincode::encode_to_vec(&large_ref_link, bincode::config::standard()).unwrap();

    group.bench_function("small_entity_deserialize", |b| {
        b.iter(|| {
            let decoded: RelationalLink<BenchmarkDefinition, SmallEntity> = black_box(
                bincode::decode_from_slice(&small_entity_encoded, bincode::config::standard())
                    .unwrap()
                    .0,
            );
            std_black_box(decoded)
        })
    });

    group.bench_function("small_reference_deserialize", |b| {
        b.iter(|| {
            let decoded: RelationalLink<BenchmarkDefinition, SmallEntity> = black_box(
                bincode::decode_from_slice(&small_ref_encoded, bincode::config::standard())
                    .unwrap()
                    .0,
            );
            std_black_box(decoded)
        })
    });

    group.bench_function("medium_entity_deserialize", |b| {
        b.iter(|| {
            let decoded: RelationalLink<BenchmarkDefinition, MediumEntity> = black_box(
                bincode::decode_from_slice(&medium_entity_encoded, bincode::config::standard())
                    .unwrap()
                    .0,
            );
            std_black_box(decoded)
        })
    });

    group.bench_function("medium_reference_deserialize", |b| {
        b.iter(|| {
            let decoded: RelationalLink<BenchmarkDefinition, MediumEntity> = black_box(
                bincode::decode_from_slice(&medium_ref_encoded, bincode::config::standard())
                    .unwrap()
                    .0,
            );
            std_black_box(decoded)
        })
    });

    group.bench_function("large_entity_deserialize", |b| {
        b.iter(|| {
            let decoded: RelationalLink<BenchmarkDefinition, LargeEntity> = black_box(
                bincode::decode_from_slice(&large_entity_encoded, bincode::config::standard())
                    .unwrap()
                    .0,
            );
            std_black_box(decoded)
        })
    });

    group.bench_function("large_reference_deserialize", |b| {
        b.iter(|| {
            let decoded: RelationalLink<BenchmarkDefinition, LargeEntity> = black_box(
                bincode::decode_from_slice(&large_ref_encoded, bincode::config::standard())
                    .unwrap()
                    .0,
            );
            std_black_box(decoded)
        })
    });

    group.finish();
}

// Benchmark: Hydration performance with different storage backends
#[cfg(feature = "native")]
fn bench_relational_link_hydration(c: &mut Criterion) {
    use netabase_store::databases::sled_store::SledStore;

    let mut group = c.benchmark_group("relational_link_hydration");

    // Setup test data
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let store =
        SledStore::<BenchmarkDefinition>::new(temp_dir.path()).expect("Failed to create store");

    // Create and insert test entities
    let small_entities: Vec<_> = (1..=1000).map(create_small_entity).collect();
    let medium_entities: Vec<_> = (1..=100).map(create_medium_entity).collect();
    let large_entities: Vec<_> = (1..=10).map(create_large_entity).collect();

    let small_tree = store.open_tree();
    for entity in &small_entities {
        small_tree.put_raw(entity.clone()).unwrap();
    }

    let medium_tree = store.open_tree();
    for entity in &medium_entities {
        medium_tree.put_raw(entity.clone()).unwrap();
    }

    let large_tree = store.open_tree();
    for entity in &large_entities {
        large_tree.put_raw(entity.clone()).unwrap();
    }

    // Create test links
    let small_entity_links: Vec<_> = small_entities
        .iter()
        .take(100)
        .map(|e| RelationalLink::Entity(e.clone()))
        .collect();
    let small_ref_links: Vec<_> = small_entities
        .iter()
        .take(100)
        .map(|e| RelationalLink::from_key(e.id))
        .collect();

    let medium_entity_links: Vec<_> = medium_entities
        .iter()
        .take(50)
        .map(|e| RelationalLink::Entity(e.clone()))
        .collect();
    let medium_ref_links: Vec<_> = medium_entities
        .iter()
        .take(50)
        .map(|e| RelationalLink::from_key(e.id))
        .collect();

    let large_entity_links: Vec<_> = large_entities
        .iter()
        .map(|e| RelationalLink::Entity(e.clone()))
        .collect();
    let large_ref_links: Vec<_> = large_entities
        .iter()
        .map(|e| RelationalLink::from_key(e.id))
        .collect();

    // Small entity hydration benchmarks
    group.throughput(Throughput::Elements(100));
    group.bench_function("small_entity_hydration", |b| {
        b.iter(|| {
            for link in &small_entity_links {
                let result = black_box(link.clone().hydrate(small_tree.clone()).unwrap());
                std_black_box(result);
            }
        })
    });

    group.bench_function("small_reference_hydration", |b| {
        b.iter(|| {
            for link in &small_ref_links {
                let result = black_box(link.clone().hydrate(small_tree.clone()).unwrap());
                std_black_box(result);
            }
        })
    });

    // Medium entity hydration benchmarks
    group.throughput(Throughput::Elements(50));
    group.bench_function("medium_entity_hydration", |b| {
        b.iter(|| {
            for link in &medium_entity_links {
                let result = black_box(link.clone().hydrate(medium_tree.clone()).unwrap());
                std_black_box(result);
            }
        })
    });

    group.bench_function("medium_reference_hydration", |b| {
        b.iter(|| {
            for link in &medium_ref_links {
                let result = black_box(link.clone().hydrate(medium_tree.clone()).unwrap());
                std_black_box(result);
            }
        })
    });

    // Large entity hydration benchmarks
    group.throughput(Throughput::Elements(10));
    group.bench_function("large_entity_hydration", |b| {
        b.iter(|| {
            for link in &large_entity_links {
                let result = black_box(link.clone().hydrate(large_tree.clone()).unwrap());
                std_black_box(result);
            }
        })
    });

    group.bench_function("large_reference_hydration", |b| {
        b.iter(|| {
            for link in &large_ref_links {
                let result = black_box(link.clone().hydrate(large_tree.clone()).unwrap());
                std_black_box(result);
            }
        })
    });

    group.finish();
}

// Benchmark: Collection operations with RelationalLinks
fn bench_relational_link_collections(c: &mut Criterion) {
    let mut group = c.benchmark_group("relational_link_collections");

    // Create test data
    let small_entities: Vec<_> = (1..=1000).map(create_small_entity).collect();
    let medium_entities: Vec<_> = (1..=100).map(create_medium_entity).collect();

    // Different collection sizes
    let sizes = [10, 50, 100, 500];

    for &size in &sizes {
        group.throughput(Throughput::Elements(size as u64));

        // Benchmark creating collections of Entity links
        group.bench_with_input(
            BenchmarkId::new("entity_collection_creation", size),
            &size,
            |b, &size| {
                let entities = &small_entities[..size];
                b.iter(|| {
                    let links: Vec<RelationalLink<BenchmarkDefinition, SmallEntity>> = black_box(
                        entities
                            .iter()
                            .map(|e| RelationalLink::Entity(e.clone()))
                            .collect(),
                    );
                    std_black_box(links)
                })
            },
        );

        // Benchmark creating collections of Reference links
        group.bench_with_input(
            BenchmarkId::new("reference_collection_creation", size),
            &size,
            |b, &size| {
                let entities = &small_entities[..size];
                b.iter(|| {
                    let links: Vec<RelationalLink<BenchmarkDefinition, SmallEntity>> = black_box(
                        entities
                            .iter()
                            .map(|e| RelationalLink::from_key(e.id))
                            .collect(),
                    );
                    std_black_box(links)
                })
            },
        );

        // Benchmark serializing collections
        let entity_links: Vec<RelationalLink<BenchmarkDefinition, SmallEntity>> = small_entities
            [..size]
            .iter()
            .map(|e| RelationalLink::Entity(e.clone()))
            .collect();
        let ref_links: Vec<RelationalLink<BenchmarkDefinition, SmallEntity>> = small_entities
            [..size]
            .iter()
            .map(|e| RelationalLink::from_key(e.id))
            .collect();

        group.bench_with_input(
            BenchmarkId::new("entity_collection_serialize", size),
            &entity_links,
            |b, links| {
                b.iter(|| {
                    let encoded = black_box(
                        bincode::encode_to_vec(links, bincode::config::standard()).unwrap(),
                    );
                    std_black_box(encoded)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("reference_collection_serialize", size),
            &ref_links,
            |b, links| {
                b.iter(|| {
                    let encoded = black_box(
                        bincode::encode_to_vec(links, bincode::config::standard()).unwrap(),
                    );
                    std_black_box(encoded)
                })
            },
        );
    }

    group.finish();
}

// Benchmark: Complex entity with multiple RelationalLink fields
fn bench_complex_entity_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_entity_operations");

    let small_entities: Vec<_> = (1..=100).map(create_small_entity).collect();
    let medium_entities: Vec<_> = (1..=50).map(create_medium_entity).collect();
    let large_entities: Vec<_> = (1..=10).map(create_large_entity).collect();

    // Create complex entities with different link configurations
    let complex_entity_with_entities = ComplexEntity {
        id: 1,
        name: "Complex Entity with Entities".to_string(),
        small_ref: RelationalLink::Entity(small_entities[0].clone()),
        medium_ref: RelationalLink::Entity(medium_entities[0].clone()),
        large_ref: RelationalLink::Entity(large_entities[0].clone()),
        small_collection: small_entities
            .iter()
            .take(10)
            .map(|e| RelationalLink::Entity(e.clone()))
            .collect(),
        medium_collection: medium_entities
            .iter()
            .take(5)
            .map(|e| RelationalLink::Entity(e.clone()))
            .collect(),
    };

    let complex_entity_with_refs = ComplexEntity {
        id: 2,
        name: "Complex Entity with References".to_string(),
        small_ref: RelationalLink::from_key(small_entities[0].id),
        medium_ref: RelationalLink::from_key(medium_entities[0].id),
        large_ref: RelationalLink::from_key(large_entities[0].id),
        small_collection: small_entities
            .iter()
            .take(10)
            .map(|e| RelationalLink::from_key(e.id))
            .collect(),
        medium_collection: medium_entities
            .iter()
            .take(5)
            .map(|e| RelationalLink::from_key(e.id))
            .collect(),
    };

    // Benchmark creation
    group.bench_function("complex_entity_with_entities_creation", |b| {
        b.iter(|| {
            let entity = black_box(ComplexEntity {
                id: 1,
                name: "Benchmark Entity".to_string(),
                small_ref: RelationalLink::Entity(small_entities[0].clone()),
                medium_ref: RelationalLink::Entity(medium_entities[0].clone()),
                large_ref: RelationalLink::Entity(large_entities[0].clone()),
                small_collection: small_entities
                    .iter()
                    .take(10)
                    .map(|e| RelationalLink::Entity(e.clone()))
                    .collect(),
                medium_collection: medium_entities
                    .iter()
                    .take(5)
                    .map(|e| RelationalLink::Entity(e.clone()))
                    .collect(),
            });
            std_black_box(entity)
        })
    });

    group.bench_function("complex_entity_with_refs_creation", |b| {
        b.iter(|| {
            let entity = black_box(ComplexEntity {
                id: 2,
                name: "Benchmark Entity".to_string(),
                small_ref: RelationalLink::from_key(small_entities[0].id),
                medium_ref: RelationalLink::from_key(medium_entities[0].id),
                large_ref: RelationalLink::from_key(large_entities[0].id),
                small_collection: small_entities
                    .iter()
                    .take(10)
                    .map(|e| RelationalLink::from_key(e.id))
                    .collect(),
                medium_collection: medium_entities
                    .iter()
                    .take(5)
                    .map(|e| RelationalLink::from_key(e.id))
                    .collect(),
            });
            std_black_box(entity)
        })
    });

    // Benchmark serialization
    group.bench_function("complex_entity_with_entities_serialize", |b| {
        b.iter(|| {
            let encoded = black_box(
                bincode::encode_to_vec(&complex_entity_with_entities, bincode::config::standard())
                    .unwrap(),
            );
            std_black_box(encoded)
        })
    });

    group.bench_function("complex_entity_with_refs_serialize", |b| {
        b.iter(|| {
            let encoded = black_box(
                bincode::encode_to_vec(&complex_entity_with_refs, bincode::config::standard())
                    .unwrap(),
            );
            std_black_box(encoded)
        })
    });

    // Benchmark cloning (important for understanding memory overhead)
    group.bench_function("complex_entity_with_entities_clone", |b| {
        b.iter(|| {
            let cloned = black_box(complex_entity_with_entities.clone());
            std_black_box(cloned)
        })
    });

    group.bench_function("complex_entity_with_refs_clone", |b| {
        b.iter(|| {
            let cloned = black_box(complex_entity_with_refs.clone());
            std_black_box(cloned)
        })
    });

    group.finish();
}

// Benchmark: Memory usage patterns
fn bench_memory_usage_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage_patterns");

    let large_entity = create_large_entity(1);

    // Compare memory allocation patterns
    group.bench_function("large_entity_link_allocation", |b| {
        b.iter(|| {
            // Simulates the memory allocation pattern of Entity links
            let entity_links: Vec<RelationalLink<BenchmarkDefinition, LargeEntity>> = black_box(
                (0..100)
                    .map(|_| RelationalLink::Entity(large_entity.clone()))
                    .collect(),
            );
            std_black_box(entity_links)
        })
    });

    group.bench_function("large_reference_link_allocation", |b| {
        b.iter(|| {
            // Simulates the memory allocation pattern of Reference links
            let ref_links: Vec<RelationalLink<BenchmarkDefinition, LargeEntity>> = black_box(
                (0..100)
                    .map(|i| RelationalLink::from_key(i as u64))
                    .collect(),
            );
            std_black_box(ref_links)
        })
    });

    group.finish();
}

// Configure benchmark groups
#[cfg(feature = "native")]
criterion_group!(
    benches,
    bench_relational_link_creation,
    bench_relational_link_serialization,
    bench_relational_link_deserialization,
    bench_relational_link_hydration,
    bench_relational_link_collections,
    bench_complex_entity_operations,
    bench_memory_usage_patterns
);

#[cfg(not(feature = "native"))]
criterion_group!(
    benches,
    bench_relational_link_creation,
    bench_relational_link_serialization,
    bench_relational_link_deserialization,
    bench_relational_link_collections,
    bench_complex_entity_operations,
    bench_memory_usage_patterns
);

criterion_main!(benches);
