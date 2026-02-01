# Benchmark Architecture - Composable Feature Benchmarking

## Overview

Benchmarks should be:
- **Modular** - Measure specific operations
- **Feature-gated** - Only compile with required features
- **Comparable** - Share baseline for comparison
- **Realistic** - Test real-world scenarios

## Structure

```
benches/
├── common/
│   ├── mod.rs              # Shared utilities
│   ├── fixtures.rs         # Test data generation
│   └── baseline.rs         # Baseline measurements
│
├── core/
│   ├── crud.rs             # Basic CRUD (no features)
│   ├── serialization.rs    # Postcard performance
│   └── transaction.rs      # Transaction overhead
│
├── features/
│   ├── secondary_keys.rs   # #[cfg(feature = "secondary_keys")]
│   ├── relational.rs       # #[cfg(feature = "relational_keys")]
│   ├── blobs.rs            # #[cfg(feature = "blobs")]
│   ├── subscriptions.rs    # #[cfg(feature = "subscriptions")]
│   └── migration.rs        # #[cfg(feature = "migration")]
│
├── combinations/
│   ├── full_featured.rs    # All features enabled
│   └── common_stack.rs     # Most common feature set
│
└── stress/
    ├── growth.rs           # Database growth patterns
    ├── concurrent.rs       # Concurrent operations
    └── large_scale.rs      # Large datasets
```

## Example: Base CRUD Benchmark

```rust
// benches/core/crud.rs
//! Core CRUD operations benchmark - no optional features
//!
//! Run with: cargo bench --bench crud --no-default-features

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use netabase_store::prelude::*;
use netabase_store::traits::database::store::NBStore;
use serde::{Serialize, Deserialize};

#[netabase_macros::netabase_definition(BenchDef)]
mod bench_models {
    use super::*;

    #[derive(
        netabase_macros::NetabaseModel,
        Debug, Clone, Serialize, Deserialize,
        PartialEq, Eq, Hash, PartialOrd, Ord
    )]
    pub struct Record {
        #[primary_key]
        pub id: u64,
        pub data: String,
    }
}

fn bench_create(c: &mut Criterion) {
    use bench_models::*;
    
    let (store, _temp) = RedbStore::<BenchDef>::new_temporary().unwrap();
    
    c.bench_function("crud_create", |b| {
        let mut counter = 0u64;
        b.iter(|| {
            let txn = store.begin_write().unwrap();
            txn.create(&Record {
                id: RecordID(counter),
                data: format!("data_{}", counter),
            }).unwrap();
            txn.commit().unwrap();
            counter += 1;
        });
    });
}

fn bench_read(c: &mut Criterion) {
    use bench_models::*;
    
    let (store, _temp) = RedbStore::<BenchDef>::new_temporary().unwrap();
    
    // Setup: create records
    let txn = store.begin_write().unwrap();
    for i in 0..1000 {
        txn.create(&Record {
            id: RecordID(i),
            data: format!("data_{}", i),
        }).unwrap();
    }
    txn.commit().unwrap();
    
    c.bench_function("crud_read", |b| {
        b.iter(|| {
            let txn = store.begin_read().unwrap();
            let _record: Option<Record> = txn.read(&RecordID(black_box(500))).unwrap();
        });
    });
}

fn bench_batch_create(c: &mut Criterion) {
    use bench_models::*;
    
    let mut group = c.benchmark_group("batch_create");
    
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let (store, _temp) = RedbStore::<BenchDef>::new_temporary().unwrap();
            
            b.iter(|| {
                let txn = store.begin_write().unwrap();
                for i in 0..size {
                    txn.create(&Record {
                        id: RecordID(i),
                        data: format!("data_{}", i),
                    }).unwrap();
                }
                txn.commit().unwrap();
            });
        });
    }
    
    group.finish();
}

criterion_group!(benches, bench_create, bench_read, bench_batch_create);
criterion_main!(benches);
```

## Example: Feature-Gated Benchmark

```rust
// benches/features/secondary_keys.rs
//! Secondary key lookup benchmark
//!
//! Run with: cargo bench --bench secondary_keys --features secondary_keys

#![cfg(feature = "secondary_keys")]

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use netabase_store::prelude::*;
use netabase_store::traits::database::store::NBStore;
use netabase_store::databases::redb::transaction::RedbModelCrud;
use serde::{Serialize, Deserialize};

#[netabase_macros::netabase_definition(IndexBenchDef)]
mod index_bench_models {
    use super::*;

    #[derive(
        netabase_macros::NetabaseModel,
        Debug, Clone, Serialize, Deserialize,
        PartialEq, Eq, Hash, PartialOrd, Ord
    )]
    pub struct User {
        #[primary_key]
        pub id: u64,
        
        #[secondary_key]
        pub email: String,
        
        pub name: String,
    }
}

fn bench_primary_lookup(c: &mut Criterion) {
    use index_bench_models::*;
    
    let (store, _temp) = RedbStore::<IndexBenchDef>::new_temporary().unwrap();
    
    // Setup
    let txn = store.begin_write().unwrap();
    for i in 0..10000 {
        txn.create(&User {
            id: UserID(i),
            email: format!("user{}@example.com", i),
            name: format!("User {}", i),
        }).unwrap();
    }
    txn.commit().unwrap();
    
    c.bench_function("primary_key_lookup", |b| {
        b.iter(|| {
            let txn = store.begin_read().unwrap();
            let _user: Option<User> = txn.read(&UserID(black_box(5000))).unwrap();
        });
    });
}

fn bench_secondary_lookup(c: &mut Criterion) {
    use index_bench_models::*;
    
    let (store, _temp) = RedbStore::<IndexBenchDef>::new_temporary().unwrap();
    
    // Setup
    let txn = store.begin_write().unwrap();
    for i in 0..10000 {
        txn.create(&User {
            id: UserID(i),
            email: format!("user{}@example.com", i),
            name: format!("User {}", i),
        }).unwrap();
    }
    txn.commit().unwrap();
    
    c.bench_function("secondary_key_lookup", |b| {
        b.iter(|| {
            let txn = store.begin_read().unwrap();
            let _users = txn.read_by_secondary_key(
                &UserEmail(black_box("user5000@example.com".to_string()))
            ).unwrap();
        });
    });
}

fn bench_lookup_comparison(c: &mut Criterion) {
    use index_bench_models::*;
    
    let mut group = c.benchmark_group("lookup_comparison");
    
    let (store, _temp) = RedbStore::<IndexBenchDef>::new_temporary().unwrap();
    
    // Setup
    let txn = store.begin_write().unwrap();
    for i in 0..10000 {
        txn.create(&User {
            id: UserID(i),
            email: format!("user{}@example.com", i),
            name: format!("User {}", i),
        }).unwrap();
    }
    txn.commit().unwrap();
    
    group.bench_function("primary", |b| {
        b.iter(|| {
            let txn = store.begin_read().unwrap();
            let _: Option<User> = txn.read(&UserID(5000)).unwrap();
        });
    });
    
    group.bench_function("secondary", |b| {
        b.iter(|| {
            let txn = store.begin_read().unwrap();
            let _ = txn.read_by_secondary_key(&UserEmail("user5000@example.com".into())).unwrap();
        });
    });
    
    group.finish();
}

criterion_group!(benches, bench_primary_lookup, bench_secondary_lookup, bench_lookup_comparison);
criterion_main!(benches);
```

## Example: Combination Benchmark

```rust
// benches/combinations/full_featured.rs
//! Benchmark with all features enabled to measure overhead
//!
//! Run with: cargo bench --bench full_featured

#![cfg(all(
    feature = "secondary_keys",
    feature = "relational_keys",
    feature = "blobs",
    feature = "subscriptions"
))]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use netabase_store::prelude::*;

#[netabase_macros::netabase_definition(FullDef)]
mod full_models {
    use super::*;

    #[derive(
        netabase_macros::NetabaseModel,
        Debug, Clone, Serialize, Deserialize,
        PartialEq, Eq, Hash, PartialOrd, Ord
    )]
    pub struct ComplexModel {
        #[primary_key]
        pub id: u64,
        
        #[secondary_key]
        pub indexed_field: String,
        
        #[link(FullDef, RelatedModel)]
        pub relation: u64,
        
        #[blob]
        pub large_data: Vec<u8>,
        
        #[subscribe(topic)]
        pub topic: String,
    }
    
    #[derive(
        netabase_macros::NetabaseModel,
        Debug, Clone, Serialize, Deserialize,
        PartialEq, Eq, Hash, PartialOrd, Ord
    )]
    pub struct RelatedModel {
        #[primary_key]
        pub id: u64,
        pub data: String,
    }
}

fn bench_full_featured_create(c: &mut Criterion) {
    use full_models::*;
    
    let (store, _temp) = RedbStore::<FullDef>::new_temporary().unwrap();
    
    c.bench_function("full_featured_create", |b| {
        let mut counter = 0u64;
        b.iter(|| {
            let txn = store.begin_write().unwrap();
            
            // Create related model
            txn.create(&RelatedModel {
                id: RelatedModelID(counter),
                data: "related".into(),
            }).unwrap();
            
            // Create complex model with all features
            txn.create(&ComplexModel {
                id: ComplexModelID(counter),
                indexed_field: format!("indexed_{}", counter),
                relation: RelationalLink::new_dehydrated(RelatedModelID(counter)),
                large_data: vec![0u8; 100_000],
                topic: "test_topic".into(),
            }).unwrap();
            
            txn.commit().unwrap();
            counter += 1;
        });
    });
}

criterion_group!(benches, bench_full_featured_create);
criterion_main!(benches);
```

## Cargo.toml Configuration

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

# Base benchmarks (no features)
[[bench]]
name = "crud"
harness = false

[[bench]]
name = "serialization"
harness = false

# Feature-specific benchmarks
[[bench]]
name = "secondary_keys"
harness = false
required-features = ["secondary_keys"]

[[bench]]
name = "relational"
harness = false
required-features = ["relational_keys"]

[[bench]]
name = "blobs"
harness = false
required-features = ["blobs"]

[[bench]]
name = "subscriptions"
harness = false
required-features = ["subscriptions"]

# Combination benchmarks
[[bench]]
name = "full_featured"
harness = false
required-features = ["secondary_keys", "relational_keys", "blobs", "subscriptions"]
```

## Running Benchmarks

```bash
# Base benchmarks (no features)
cargo bench --bench crud --no-default-features

# Specific feature
cargo bench --bench secondary_keys --features secondary_keys

# Compare primary vs secondary lookup
cargo bench --bench secondary_keys --features secondary_keys -- lookup_comparison

# All benchmarks with default features
cargo bench

# Generate HTML reports
cargo bench --bench crud -- --save-baseline baseline
cargo bench --bench crud -- --baseline baseline
```

## Shared Benchmark Utilities

```rust
// benches/common/mod.rs
pub mod fixtures;
pub mod baseline;

// benches/common/fixtures.rs
use netabase_store::prelude::*;

pub fn generate_test_data(count: usize) -> Vec<(u64, String)> {
    (0..count)
        .map(|i| (i as u64, format!("data_{}", i)))
        .collect()
}

pub fn setup_populated_store<D>(count: usize) -> (RedbStore<D>, TempDir)
where
    D: NetabaseDefinition + Clone,
    // ... trait bounds
{
    let (store, temp) = RedbStore::<D>::new_temporary().unwrap();
    // Populate with test data
    (store, temp)
}

// benches/common/baseline.rs
//! Baseline measurements for comparison

pub struct Baseline {
    pub empty_transaction_ns: u64,
    pub serialization_ns: u64,
}

impl Baseline {
    pub fn measure() -> Self {
        // Measure overhead of empty operations
        todo!()
    }
}
```

## CI Benchmark Integration

```yaml
# .github/workflows/bench.yml
name: Benchmarks

on:
  push:
    branches: [main]
  pull_request:

jobs:
  benchmark:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v2
      
      - name: Run core benchmarks
        run: cargo bench --bench crud --no-default-features
      
      - name: Run feature benchmarks
        run: |
          cargo bench --bench secondary_keys --features secondary_keys
          cargo bench --bench blobs --features blobs
      
      - name: Store results
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'criterion'
          output-file-path: target/criterion/
```

## Benefits

1. **Feature Isolation** - Measure overhead of each feature
2. **Regression Detection** - Track performance over time
3. **Comparison** - Compare approaches side-by-side
4. **CI Integration** - Automated performance tracking
5. **Documentation** - Benchmarks show expected performance
