use criterion::{black_box, criterion_group, criterion_main, Criterion};
use netabase_store_examples::boilerplate_lib::models::user::{
    User, UserFile, UserFileEnum, UserID,
};
use netabase_store_examples::boilerplate_lib::Definition;
use netabase_store::databases::redb::transaction::RedbModelCrud;
use netabase_store::traits::registery::models::model::NetabaseModel;
use strum::IntoEnumIterator;

mod common;
use common::*;

fn bench_just_abstraction_call(c: &mut Criterion) {
    c.bench_function("abstraction_function_call_overhead", |b| {
        let user = generate_random_user();
        let (store, path) = create_test_db::<Definition>("overhead_test")
            .expect("Failed to create DB");

        b.iter(|| {
            let txn = store.begin_transaction().expect("Failed to begin txn");
            let mut tables = txn.prepare_model::<User>().expect("Failed to prepare");

            // Just measure the function call overhead
            black_box(user.get_secondary_keys());
            black_box(user.get_relational_keys());
            black_box(user.get_subscription_keys());
        });

        std::fs::remove_file(path).ok();
    });
}

fn bench_table_opening_overhead(c: &mut Criterion) {
    c.bench_function("table_opening_overhead", |b| {
        let (store, path) = create_test_db::<Definition>("table_overhead_test")
            .expect("Failed to create DB");

        b.iter(|| {
            let txn = store.begin_transaction().expect("Failed to begin txn");
            // Measure JUST the table opening
            black_box(txn.prepare_model::<User>()).expect("Failed to prepare");
        });

        std::fs::remove_file(path).ok();
    });
}

// Isolated test: just the subscription matching logic
fn bench_subscription_matching(c: &mut Criterion) {
    use netabase_store_examples::boilerplate_lib::DefinitionSubscriptions;

    let subscriptions = vec![
        DefinitionSubscriptions::Topic1,
        DefinitionSubscriptions::Topic2,
        DefinitionSubscriptions::Topic1,
    ];

    c.bench_function("raw_subscription_match", |b| {
        b.iter(|| {
            let mut count1 = 0;
            let mut count2 = 0;
            for sub in black_box(&subscriptions) {
                match sub {
                    DefinitionSubscriptions::Topic1 => count1 += 1,
                    DefinitionSubscriptions::Topic2 => count2 += 1,
                    _ => {}
                }
            }
            black_box((count1, count2))
        });
    });

    c.bench_function("discriminant_subscription_match", |b| {
        b.iter(|| {
            let mut count1 = 0;
            let mut count2 = 0;
            for sub in black_box(&subscriptions) {
                let disc = std::mem::discriminant(sub);
                if disc == std::mem::discriminant(&DefinitionSubscriptions::Topic1) {
                    count1 += 1;
                } else if disc == std::mem::discriminant(&DefinitionSubscriptions::Topic2) {
                    count2 += 1;
                }
            }
            black_box((count1, count2))
        });
    });
}

pub fn generate_random_user() -> User {
    use netabase_store_examples::boilerplate_lib::{CategoryID, DefinitionSubscriptions};
    use netabase_store::relational::RelationalLink;
    use rand::prelude::*;

    let mut rng = rand::rng();
    let names = ["Alice", "Bob", "Carol"];
    let name = names.choose(&mut rng).unwrap().to_string();
    let age: u8 = rng.random_range(1..=100);
    let user_id = UserID(format!("user_{:016x}", rng.random::<u64>()));
    let category_id = CategoryID(format!("cat_{:016x}", rng.random::<u64>()));

    let partner = if rng.random_bool(0.5) {
        RelationalLink::new_dehydrated(UserID("none".to_string()))
    } else {
        RelationalLink::new_dehydrated(UserID(format!("user_{:016x}", rng.random::<u64>())))
    };

    let category = RelationalLink::new_dehydrated(category_id);

    let subscriptions = vec![
        DefinitionSubscriptions::Topic1,
        DefinitionSubscriptions::Topic2,
    ];

    User {
        id: user_id,
        name,
        age,
        partner,
        category,
        subscriptions,
        user_file: UserFileEnum::Complete(UserFile {
            filename: "bench.tmp".to_string(),
            mime_type: "application/octet-stream".to_string(),
        }),
    }
}

criterion_group!(benches, bench_just_abstraction_call, bench_table_opening_overhead, bench_subscription_matching);
criterion_main!(benches);
