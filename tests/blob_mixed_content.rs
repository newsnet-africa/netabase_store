//! Integration tests for mixed content blob types (chunked and unchunked fields).

use netabase_store::prelude::*;
use netabase_store::databases::redb::transaction::RedbModelCrud;
use netabase_store::blob::NetabaseBlobItem;
use serde::{Deserialize, Serialize};

#[netabase_macros::netabase_definition(MixedContentDef)]
pub mod mixed_content_def {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Default, netabase_macros::NetabaseBlobItem)]
    pub struct MixedFile {
        #[blob]
        pub large_data: Vec<u8>, // Should be chunked
        
        pub metadata: String, // Should be stored whole (index 0)
        
        pub tags: Vec<String>, // Should be stored whole (index 0)
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
    pub struct MixedModel {
        #[primary_key]
        pub id: String,

        #[blob]
        pub content: MixedFile,
    }
}

use mixed_content_def::*;

#[test]
fn test_mixed_blob_content() -> Result<(), Box<dyn std::error::Error>> {
    let (store, _temp) = RedbStore::<MixedContentDef>::new_temporary()?;

    // Setup data
    // Large data: 150KB -> 3 chunks (0, 1, 2)
    let large_data = vec![0xDD; 150_000];
    let metadata = "This is a metadata string that should be stored whole".to_string();
    let tags = vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()];
    
    let file = MixedFile {
        large_data: large_data.clone(),
        metadata: metadata.clone(),
        tags: tags.clone(),
    };

    let model = MixedModel {
        id: MixedModelID("mixed_1".into()),
        content: file.clone(),
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
        let tables = txn.prepare_model::<MixedModel>()?;
        
        // Check content
        let blob_key = MixedModelBlobKeys::Content { owner: MixedModelID("mixed_1".into()) };
        
        // 1. Fetch Indices
        // Should have:
        // - LargeData: 0, 1, 2
        // - Metadata: 0
        // - Tags: 0
        // Note: The indices are per-variant. The macro generates an enum with variants for each field.
        // LargeData(u8, Vec<u8>)
        // Metadata(Vec<u8>)
        // Tags(Vec<u8>)
        
        let all_blobs = MixedModel::read_blob_items(&blob_key, &tables)?;
        
        // We expect:
        // 3 items for large_data
        // 1 item for metadata
        // 1 item for tags
        // Total = 5 items
        assert_eq!(all_blobs.len(), 5);
        
        // Verify reconstruction
        // The read_blob_items returns wrappers (MixedModelBlobItem).
        // We need to extract the inner MixedFileBlobs and reconstruct.
        
        let unwrapped_items: Vec<MixedFileBlobs> = all_blobs.into_iter().filter_map(|wrapper| {
             match wrapper {
                 mixed_content_def::MixedModelBlobItem::Content(inner) => Some(inner),
                 // _ => None, // Only one blob field in model, so this is exhaustive effectively
             }
        }).collect();
        
        let reconstructed = MixedFile::reconstruct_from_blobs(unwrapped_items);
        assert_eq!(reconstructed, file);
        
        // Verify individual fields
        assert_eq!(reconstructed.large_data.len(), 150_000);
        assert_eq!(reconstructed.metadata, metadata);
        assert_eq!(reconstructed.tags, tags);
    }

    Ok(())
}
