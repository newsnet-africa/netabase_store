use std::fmt::Debug;
use rewrite::traits::structural::blob::{NetabaseBlobItem, ChunkSize, BlobItemChunk};
use rewrite::results::{NetabaseError, BlobReconstructionError, NetabaseResult};

// 1. Partial Struct with #[chunk_size] on fields
#[derive(Debug, Clone, PartialEq, Eq, rewrite::NetabaseBlob)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
struct PartialFieldBlob {
    #[chunk_size(64)]
    header: String,
    #[chunk_size(256)]
    payload: Vec<u8>,
}

// 2. Partial Struct with #[blob_field]
#[derive(Debug, Clone, PartialEq, Eq, rewrite::NetabaseBlob)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
struct NestedBlob {
    id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, rewrite::NetabaseBlob)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
struct ParentBlob {
    #[blob_field(chunk_size(128))]
    child: NestedBlob,
}

// 3. Full Enum Blob
#[derive(Debug, Clone, PartialEq, Eq, rewrite::NetabaseBlob)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
enum EnumBlob {
    VariantA(String),
    VariantB { x: i32, y: i32 },
}

// 4. Simple Blob for streaming tests
#[derive(Debug, Clone, PartialEq, Eq, rewrite::NetabaseBlob)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
struct SimpleStreamingBlob {
    data: Vec<u8>,
}

// 5. Generic Blob
#[derive(Debug, Clone, PartialEq, Eq, rewrite::NetabaseBlob)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
struct GenericBlob<T> 
where 
    T: rkyv::Archive + Clone + Debug + PartialEq + 'static,
{
    data: T,
}

// 6. Partial Enum
#[derive(Debug, Clone, PartialEq, Eq, rewrite::NetabaseBlob)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
enum PartialComplexEnum {
    Full(String),
    #[blob_field(chunk_size(64))]
    Partial {
        #[chunk_size(32)]
        meta: String,
        payload: Vec<u8>,
    },
}

// 7. Strategy Toggles
#[derive(Debug, Clone, PartialEq, Eq, rewrite::NetabaseBlob)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[blob(strategy = "full")]
struct ForcedFull {
    #[chunk_size(64)]
    field1: String,
}

#[derive(Debug, Clone, PartialEq, Eq, rewrite::NetabaseBlob)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[blob(strategy = "partial")]
struct ForcedPartial {
    field1: String,
}

#[derive(Debug, Clone, PartialEq, Eq, rewrite::NetabaseBlob)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[strategy("partial")]
struct StandaloneStrategy {
    field1: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partial_field_blob() {
        let blob = PartialFieldBlob {
            header: "Short Header".to_string(),
            payload: vec![0u8; 500], // Will be split into 2 chunks of 256
        };

        let chunks: Vec<_> = blob.clone().into_chunks(ChunkSize::Default).collect();
        
        // We expect:
        // - 1 chunk for header (index 0)
        // - 2 chunks for payload (index 0, 1)
        // Total 3 chunks
        assert_eq!(chunks.len(), 3);

        // Verify variant types
        let mut header_chunks = 0;
        let mut payload_chunks = 0;
        for chunk in &chunks {
            match chunk {
                PartialFieldBlobChunk::Header(_) => header_chunks += 1,
                PartialFieldBlobChunk::Payload(_) => payload_chunks += 1,
                _ => panic!("Unexpected chunk variant"),
            }

        }
        assert_eq!(header_chunks, 1);
        assert_eq!(payload_chunks, 2);

        // Reconstruction
        let reconstructed = PartialFieldBlob::try_from_chunks(chunks.into_iter(), ChunkSize::Default)
            .expect("Failed to reconstruct PartialFieldBlob");
        assert_eq!(blob, reconstructed);
    }

    #[test]
    fn test_nested_blob_partial() {
        let blob = ParentBlob {
            child: NestedBlob { id: 12345 },
        };

        let chunks: Vec<_> = blob.clone().into_chunks(ChunkSize::Default).collect();
        assert!(!chunks.is_empty());

        let reconstructed = ParentBlob::try_from_chunks(chunks.into_iter(), ChunkSize::Default)
            .expect("Failed to reconstruct ParentBlob");
        assert_eq!(blob, reconstructed);
    }

    #[test]
    fn test_full_enum_blob() {
        let blob = EnumBlob::VariantB { x: 1, y: 2 };

        let chunks: Vec<_> = blob.clone().into_chunks(ChunkSize::Size(128)).collect();
        assert!(!chunks.is_empty());

        let reconstructed = EnumBlob::try_from_chunks(chunks.into_iter(), ChunkSize::Size(128))
            .expect("Failed to reconstruct EnumBlob");
        assert_eq!(blob, reconstructed);
    }

    #[test]
    fn test_error_corrupted_chunk_size() {
        #[derive(Debug, Clone, PartialEq, Eq, rewrite::NetabaseBlob)]
        #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
        struct Simple { data: Vec<u8> }

        let blob = Simple { data: vec![1, 2, 3] };
        let mut chunks: Vec<_> = blob.into_chunks(ChunkSize::Size(64)).collect();
        
        // Corrupt a chunk by making it larger than expected
        chunks[0].data.extend(vec![0u8; 100]); 

        let result = Simple::try_from_chunks(chunks.into_iter(), ChunkSize::Size(64));
        assert!(matches!(result, Err(NetabaseError::BlobReconstruction(BlobReconstructionError::InvalidChunkData(s))) if s.contains("Corrupted chunk detected")));
    }

    #[test]
    fn test_error_partial_chunk_in_middle() {
        #[derive(Debug, Clone, PartialEq, Eq, rewrite::NetabaseBlob)]
        #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
        struct Large { data: Vec<u8> }

        let blob = Large { data: vec![0u8; 200] };
        let mut chunks: Vec<_> = blob.into_chunks(ChunkSize::Size(64)).collect();
        
        // chunks[0] is 64 bytes (Full)
        // chunks[1] is 64 bytes (Full)
        // chunks[2] is 64 bytes (Full)
        // chunks[3] is 8 bytes (Partial)
        assert!(chunks.len() >= 4);

        // Make an early chunk partial
        chunks[0].data.truncate(32);

        let result = Large::try_from_chunks(chunks.into_iter(), ChunkSize::Size(64));
        assert!(matches!(result, Err(NetabaseError::BlobReconstruction(BlobReconstructionError::InvalidChunkData(s))) if s.contains("Unexpected partial chunk in middle of stream")));
    }

    #[test]
    fn test_error_truncated_stream() {
        #[derive(Debug, Clone, PartialEq, Eq, rewrite::NetabaseBlob)]
        #[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
        struct Trunc { data: Vec<u8> }

        // Use enough data to have many chunks
        let blob = Trunc { data: vec![0u8; 1000] };
        let mut chunks: Vec<_> = blob.into_chunks(ChunkSize::Size(64)).collect();
        let n = chunks.len();
        
        chunks.truncate(n - 2);
        chunks.remove(1); 
        
        let result = Trunc::try_from_chunks(chunks.into_iter(), ChunkSize::Size(64));
        if let Err(NetabaseError::BlobReconstruction(BlobReconstructionError::InvalidChunkData(ref s))) = result {
            assert!(s.contains("Stream truncated"), "Error message did not contain 'Stream truncated': {}", s);
        } else {
            panic!("Expected InvalidChunkData error, got: {:?}", result);
        }
    }

    #[test]
    fn test_partial_struct_missing_field_chunks() {
        let blob = PartialFieldBlob {
            header: "H".to_string(),
            payload: vec![1, 2, 3],
        };

        let chunks: Vec<_> = blob.into_chunks(ChunkSize::Default).collect();
        
        // Remove Payload chunk
        let filtered_chunks = chunks.into_iter().filter(|c| matches!(c, PartialFieldBlobChunk::Header(_)));


        let result = PartialFieldBlob::try_from_chunks(filtered_chunks, ChunkSize::Default);
        assert!(matches!(result, Err(NetabaseError::BlobReconstruction(BlobReconstructionError::MissingChunks))));
    }

    #[test]
    fn test_simple_streaming() {
        let blob = SimpleStreamingBlob {
            data: vec![1, 2, 3, 4, 5],
        };

        // Test into_chunks_iter
        let iter = blob.clone().into_chunks_iter(ChunkSize::Size(2));
        let results: Vec<NetabaseResult<_>> = iter.collect();

        assert!(results.len() >= 3); 
        for res in &results {
            assert!(res.is_ok());
        }

        // Reconstruction from results
        let chunks = results.into_iter().map(|r| r.unwrap());
        let reconstructed = SimpleStreamingBlob::try_from_chunks(chunks, ChunkSize::Size(2))
            .expect("Failed to reconstruct");
        
        assert_eq!(blob, reconstructed);
    }

    #[test]
    fn test_partial_blob_streaming() {
        let blob = PartialFieldBlob {
            header: "Hello".to_string(),
            payload: vec![0u8; 100],
        };

        let iter = blob.clone().into_chunks_iter(ChunkSize::Default);
        let results: Vec<NetabaseResult<_>> = iter.collect();

        // header: 1 chunk (64 bytes)
        // payload: 2 chunks (64 + 36 bytes) - wait, chunk_size(256) on payload?
        // Let's re-verify: PartialFieldBlob has header chunk_size(64) and payload chunk_size(256).
        // If we use Default, it uses these. 
        // 100 bytes of payload fits in one 256-byte chunk.
        // So total 2 chunks.
        assert_eq!(results.len(), 2);
        for res in &results {
            assert!(res.is_ok());
        }

        let chunks = results.into_iter().map(|r| r.unwrap());
        let reconstructed = PartialFieldBlob::try_from_chunks(chunks, ChunkSize::Default)
            .expect("Failed to reconstruct");
        
        assert_eq!(blob, reconstructed);
    }
    #[test]
    fn test_into_iterator() {
        let blob = SimpleStreamingBlob {
            data: vec![1, 2, 3, 4, 5],
        };

        // Test IntoIterator (standard for loop)
        let mut results = Vec::new();
        for res in blob.clone() {
            results.push(res);
        }

        assert!(results.len() >= 1);
        for res in &results {
            assert!(res.is_ok());
        }

        // Reconstruction
        let chunks = results.into_iter().map(|r| r.unwrap());
        let reconstructed = SimpleStreamingBlob::try_from_chunks(chunks, ChunkSize::Default)
            .expect("Failed to reconstruct from IntoIterator");
        
        assert_eq!(blob, reconstructed);
    }

    #[test]
    fn test_generic_blob() {
        let blob = GenericBlob {
            data: "Generic Data".to_string(),
        };

        let chunks: Vec<_> = blob.clone().into_chunks(ChunkSize::Size(4)).collect();
        assert!(chunks.len() >= 1);

        let reconstructed = GenericBlob::<String>::try_from_chunks(chunks.into_iter(), ChunkSize::Size(4))
            .expect("Failed to reconstruct GenericBlob");
        
        assert_eq!(blob, reconstructed);
    }

    #[test]
    fn test_partial_enum_full() {
        let blob = PartialComplexEnum::Full("Hello Full".to_string());

        let chunks: Vec<_> = blob.clone().into_chunks(ChunkSize::Default).collect();
        assert!(chunks.len() >= 1);

        let reconstructed = PartialComplexEnum::try_from_chunks(chunks.into_iter(), ChunkSize::Default)
            .expect("Failed to reconstruct PartialComplexEnum::Full");
        
        assert_eq!(blob, reconstructed);
    }

    #[test]
    fn test_partial_enum_partial() {
        let blob = PartialComplexEnum::Partial {
            meta: "Metadata".to_string(),
            payload: vec![0u8; 100],
        };

        let chunks: Vec<_> = blob.clone().into_chunks(ChunkSize::Default).collect();
        // meta: 1 chunk (32 bytes)
        // payload: 2 chunks (64 + 36 bytes)
        // total 3 chunks
        assert_eq!(chunks.len(), 3);

        let reconstructed = PartialComplexEnum::try_from_chunks(chunks.into_iter(), ChunkSize::Default)
            .expect("Failed to reconstruct PartialComplexEnum::Partial");
        
        assert_eq!(blob, reconstructed);
    }

    #[test]
    fn test_forced_full() {
        let blob = ForcedFull { field1: "Hello".to_string() };
        let chunks: Vec<_> = blob.into_chunks(ChunkSize::Size(1024)).collect();
        assert_eq!(chunks.len(), 1);
    }

    #[test]
    fn test_forced_partial() {
        let blob = ForcedPartial { field1: "Hello".to_string() };
        let chunks: Vec<_> = blob.into_chunks(ChunkSize::Default).collect();
        assert!(chunks.len() >= 1);
    }

    #[test]
    fn test_standalone_strategy() {
        let blob = StandaloneStrategy { field1: "Hello".to_string() };
        let chunks: Vec<_> = blob.into_chunks(ChunkSize::Default).collect();
        assert!(chunks.len() >= 1);
    }
}
