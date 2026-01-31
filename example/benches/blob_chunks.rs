use criterion::{criterion_group, criterion_main, Criterion};
use netabase_store::prelude::*;
use netabase_store::databases::redb::transaction::RedbModelCrud;
use serde::{Deserialize, Serialize};

#[netabase_macros::netabase_definition(BlobBenchDef)]
pub mod blob_bench_def {
    use super::*;
    use netabase_store::blob::NetabaseBlobItem;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Default, netabase_macros::NetabaseBlobItem)]
    pub struct BlobFile {
        pub data: Vec<u8>,
        pub metadata: String,
    }

    #[derive(
        netabase_macros::NetabaseModel,
        Debug,
        Clone,
        Serialize,
        Deserialize,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
    )]
    pub struct BlobUser {
        #[primary_key]
        pub id: String,
        
        pub name: String,
        
        #[blob]
        pub bio: BlobFile,
    }
}

use blob_bench_def::*;

fn bench_blob_chunks(c: &mut Criterion) {
    let mut group = c.benchmark_group("blob_chunks");
    
    // Setup DB and data
    // 100 chunks of 60KB = 6MB total
    let chunk_size = 60_000;
    let num_chunks = 100;
    let data_size = chunk_size * num_chunks;
    let large_data = vec![1u8; data_size];
    
    // Use new_temporary which returns (store, temp_dir)
    let (store, _temp_dir) = RedbStore::<BlobBenchDef>::new_temporary().unwrap();
    
    let user_id = BlobUserID("bench_user".to_string());
    let user = BlobUser {
        id: user_id.clone(),
        name: "Bench User".to_string(),
        bio: BlobFile {
            data: large_data,
            metadata: "Benchmark Data".to_string(),
        },
    };
    
    {
        let txn = store.begin_write().unwrap();
        txn.create(&user).unwrap();
        txn.commit().unwrap();
    }
    
    // Benchmark reading all chunks vs specific chunks
    let txn = store.begin_read().unwrap();
    let tables = txn.prepare_model::<BlobUser>().unwrap();
    let blob_key = BlobUserBlobKeys::Bio { 
        owner: user_id.clone(),
    };
    
    // Case 1: Read all chunks (classic method)
    group.bench_function("read_all_chunks", |b| {
        b.iter(|| {
            let chunks = BlobUser::read_blob_items(&blob_key, &tables).unwrap();
            // 100 chunks + 1 for metadata/overhead if applicable, or exactly 100.
            // Actually postcard serialization of BlobFile might add overhead.
            // Split into 60KB.
            // If total size > 0, at least 1 chunk.
            assert!(chunks.len() >= num_chunks); 
        })
    });
    
    // Case 2: Read specific chunks (filtering)
    // Read 5 scattered chunks: 0, 25, 50, 75, 99
    let indices = vec![0, 25, 50, 75, 99];
    
    group.bench_function("read_filtered_chunks", |b| {
        b.iter(|| {
            let chunks = BlobUser::read_blob_chunks(&blob_key, &indices, &tables).unwrap();
            assert_eq!(chunks.len(), 5);
        })
    });
    
    group.finish();
}

criterion_group!(benches, bench_blob_chunks);
criterion_main!(benches);