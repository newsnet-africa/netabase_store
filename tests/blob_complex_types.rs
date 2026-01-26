//! Integration tests for complex blob types with multiple internal blob fields.

use netabase_store::prelude::*;
use netabase_store::databases::redb::transaction::RedbModelCrud;
use netabase_store::blob::NetabaseBlobItem;
use serde::{Deserialize, Serialize};

#[netabase_macros::netabase_definition(ComplexBlobDef)]
pub mod complex_blob_def {
    use super::*;
    use netabase_store::blob::NetabaseBlobItem;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Default, netabase_macros::NetabaseBlobItem)]
    pub struct MultiPartFile {
        #[blob]
        pub part_a: Vec<u8>, // Chunked
        
        #[blob]
        pub part_b: Vec<u8>, // Chunked
        
        pub description: String, // Single chunk (index 0)
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
    pub struct MultiBlobModel {
        #[primary_key]
        pub id: String,

        #[blob]
        pub file_1: MultiPartFile,

        #[blob]
        pub file_2: MultiPartFile,
    }
}

use complex_blob_def::*;

#[test]
fn test_complex_blob_multiple_fields() -> Result<(), Box<dyn std::error::Error>> {
    let (store, _temp) = RedbStore::<ComplexBlobDef>::new_temporary()?;

    // Setup data
    // Part A: 150KB -> 3 chunks (0, 1, 2)
    let data_a = vec![0xAA; 150_000];
    // Part B: 70KB -> 2 chunks (0, 1)
    let data_b = vec![0xBB; 70_000];
    
    let file = MultiPartFile {
        part_a: data_a.clone(),
        part_b: data_b.clone(),
        description: "Complex Multi-Part File".into(),
    };

    let model = MultiBlobModel {
        id: MultiBlobModelID("complex_1".into()),
        file_1: file.clone(),
        file_2: file.clone(),
    };

    // Write
    {
        let txn = store.begin_write()?;
        txn.create(&model)?;
        txn.commit()?;
    }

    // Read & Verify
    {
        let txn = store.begin_read()?;
        let tables = txn.prepare_model::<MultiBlobModel>()?;
        
        // Check file_1
        let blob_key_1 = MultiBlobModelBlobKeys::File1 { owner: MultiBlobModelID("complex_1".into()) };
        
        // 1. Fetch Indices
        // Should have 0, 1, 2.
        // Index 0: part_a(0), part_b(0), description
        // Index 1: part_a(1), part_b(1)
        // Index 2: part_a(2)
        let indices = MultiBlobModel::fetch_blob_indices(&blob_key_1, &tables)?;
        println!("Indices 1: {:?}", indices);
        assert_eq!(indices.len(), 3);
        assert!(indices.contains(&0));
        assert!(indices.contains(&1));
        assert!(indices.contains(&2));

        // 2. Read Chunk 1
        // Should return part_a chunk 1 and part_b chunk 1
        let chunk1_items = MultiBlobModel::read_blob_chunks(&blob_key_1, &[1], &tables)?;
        assert_eq!(chunk1_items.len(), 2);
        
        // 3. Read Chunk 2
        // Should return only part_a chunk 2 (part_b is shorter)
        let chunk2_items = MultiBlobModel::read_blob_chunks(&blob_key_1, &[2], &tables)?;
        assert_eq!(chunk2_items.len(), 1);

        // 4. Full Reconstruction
        let all_items = MultiBlobModel::read_blob_items(&blob_key_1, &tables)?;
        
        // Unwrap logic needed here to test reconstruction
        let unwrapped_items: Vec<MultiPartFileBlobs> = all_items.into_iter().filter_map(|wrapper| {
             match wrapper {
                 complex_blob_def::MultiBlobModelBlobItem::File1(inner) => Some(inner),
                 _ => None,
             }
        }).collect();
        
        let reconstructed = MultiPartFile::reconstruct_from_blobs(unwrapped_items);
        assert_eq!(reconstructed, file);
        
        // Verify file_2 as well (ensure separate keys work)
        let blob_key_2 = MultiBlobModelBlobKeys::File2 { owner: MultiBlobModelID("complex_1".into()) };
        let indices_2 = MultiBlobModel::fetch_blob_indices(&blob_key_2, &tables)?;
        assert_eq!(indices_2.len(), 3);
        
        // Ensure no cross-contamination (reading file 2 key should get file 2 data)
        let items_2 = MultiBlobModel::read_blob_items(&blob_key_2, &tables)?;
        let unwrapped_items_2: Vec<MultiPartFileBlobs> = items_2.into_iter().filter_map(|wrapper| {
             match wrapper {
                 complex_blob_def::MultiBlobModelBlobItem::File2(inner) => Some(inner),
                 _ => None,
             }
        }).collect();
        let reconstructed_2 = MultiPartFile::reconstruct_from_blobs(unwrapped_items_2);
        assert_eq!(reconstructed_2, file);
    }

    Ok(())
}
