use criterion::{criterion_group, criterion_main, Criterion};
use example::boilerplate_lib::definition::{LazyUser, LazyLargeFile, LazyUserBlobKeys};
use example::boilerplate_lib::definition::LazyUserID;
use example::boilerplate_lib::Definition;
use netabase_store::blob::BlobLink;
use netabase_store::databases::redb::transaction::crud::RedbModelCrud;
use netabase_store::databases::redb::RedbStore;
use netabase_store::traits::database::store::NBStore;

fn bench_blob_chunks(c: &mut Criterion) {
    let mut group = c.benchmark_group("blob_chunks");
    
    // Setup DB and data
    // 100 chunks of 60KB = 6MB total
    let chunk_size = 60_000;
    let num_chunks = 100;
    let data_size = chunk_size * num_chunks;
    let large_data = vec![1u8; data_size];
    
    // Use new_temporary which returns (store, temp_dir)
    let (store, _temp_dir) = RedbStore::<Definition>::new_temporary().unwrap();
    
    let user_id = LazyUserID("bench_user".to_string());
    let user = LazyUser {
        id: user_id.clone(),
        name: "Bench User".to_string(),
        bio: BlobLink::Complete(LazyLargeFile {
            data: large_data,
            metadata: "Benchmark Data".to_string(),
        }),
    };
    
    {
        let txn = store.begin_write().unwrap();
        txn.create(&user).unwrap();
        txn.commit().unwrap();
    }
    
    // Benchmark reading all chunks vs specific chunks
    let txn = store.begin_read().unwrap();
    let tables = txn.prepare_model::<LazyUser>().unwrap();
    let blob_key = LazyUserBlobKeys::Bio { 
        owner: user_id.clone(),
        chunk_index: 0
    };
    
    // Case 1: Read all chunks (classic method)
    group.bench_function("read_all_chunks", |b| {
        b.iter(|| {
            let chunks = LazyUser::read_blob_items(&blob_key, &tables).unwrap();
            assert_eq!(chunks.len(), num_chunks + 1); // +1 for leftover/metadata
        })
    });
    
    // Case 2: Read specific chunks (filtering)
    // Read 5 scattered chunks: 0, 25, 50, 75, 99
    let indices = vec![0, 25, 50, 75, 99];
    
    group.bench_function("read_filtered_chunks", |b| {
        b.iter(|| {
            let chunks = LazyUser::read_blob_chunks(&blob_key, &indices, &tables).unwrap();
            assert_eq!(chunks.len(), 5);
        })
    });
    
    group.finish();
}

criterion_group!(benches, bench_blob_chunks);
criterion_main!(benches);
