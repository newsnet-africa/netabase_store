use rewrite::results::NetabaseResult;
use rewrite::traits::structural::blob::{BlobItemChunk, ChunkSize, NetabaseBlobItem};

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    rewrite::NetabaseBlob,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
struct SimpleBlob {
    id: u64,
    name: String,
    data: Vec<u8>,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    rewrite::NetabaseBlob,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[chunk_size(512)]
struct SizedBlob {
    content: String,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    rewrite::NetabaseBlob,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[chunk_derives(serde::Serialize, serde::Deserialize)]
struct SerializableBlob {
    value: i32,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    rewrite::NetabaseBlob,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
#[chunk_owner_id]
#[chunk_checksum]
struct TrackedBlob {
    payload: String,
}

fn custom_serialize(blob: &CustomSerBlob) -> NetabaseResult<Vec<u8>> {
    Ok(format!("{}:{}", blob.field1, blob.field2).into_bytes())
}

fn custom_deserialize(data: &[u8]) -> NetabaseResult<CustomSerBlob> {
    let s = String::from_utf8(data.to_vec())
        .map_err(|e| rewrite::results::NetabaseError::Serialization(e.to_string()))?;
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return Err(rewrite::results::NetabaseError::Serialization(
            "Invalid format".to_string(),
        ));
    }
    Ok(CustomSerBlob {
        field1: parts[0].to_string(),
        field2: parts[1].parse().map_err(|e: std::num::ParseIntError| {
            rewrite::results::NetabaseError::Serialization(e.to_string())
        })?,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, rewrite::NetabaseBlob)]
#[chunk_serialize(custom_serialize)]
#[chunk_deserialize(custom_deserialize)]
struct CustomSerBlob {
    field1: String,
    field2: u32,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    rewrite::NetabaseBlob,
    rkyv::Archive,
    rkyv::Serialize,
    rkyv::Deserialize,
)]
struct PartialBlob {
    #[chunk_size(256)]
    small_field: String,
    #[blob_field(chunk_size(1024))]
    large_field: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_blob_default_serialization() {
        let blob = SimpleBlob {
            id: 42,
            name: "test".to_string(),
            data: vec![1, 2, 3, 4, 5],
        };

        let chunks: Vec<_> = blob.clone().into_chunks(ChunkSize::Size(128)).collect();
        assert!(!chunks.is_empty());

        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.index, i);
            assert!(!chunk.data.is_empty());
        }

        let reconstructed = SimpleBlob::try_from_chunks(chunks.into_iter(), ChunkSize::Size(128))
            .expect("Failed to reconstruct blob");
        assert_eq!(blob, reconstructed);
    }

    #[test]
    fn test_sized_blob_respects_chunk_size() {
        let content = "a".repeat(2000);
        let blob = SizedBlob { content };

        let chunks: Vec<_> = blob.clone().into_chunks(ChunkSize::Size(512)).collect();

        assert!(
            chunks.len() > 1,
            "Expected multiple chunks for 2000 bytes with 512 byte chunks"
        );

        for (i, chunk) in chunks.iter().enumerate() {
            if i < chunks.len() - 1 {
                assert!(
                    chunk.data.len() <= 512,
                    "Chunk {} is too large: {} bytes",
                    i,
                    chunk.data.len()
                );
            }
        }

        let reconstructed = SizedBlob::try_from_chunks(chunks.into_iter(), ChunkSize::Size(512))
            .expect("Failed to reconstruct sized blob");
        assert_eq!(blob, reconstructed);
    }

    #[test]
    fn test_chunk_struct_has_required_fields() {
        let blob = SimpleBlob {
            id: 1,
            name: "test".to_string(),
            data: vec![],
        };

        let chunks: Vec<_> = blob.into_chunks(ChunkSize::Size(64)).collect();
        let chunk = &chunks[0];

        assert_eq!(chunk.index, 0);
        assert!(!chunk.data.is_empty());
    }

    #[test]
    fn test_tracked_blob_has_owner_and_checksum_fields() {
        let blob = TrackedBlob {
            payload: "test".to_string(),
        };

        let chunks: Vec<_> = blob.into_chunks(ChunkSize::Size(64)).collect();

        if let Some(chunk) = chunks.first() {
            let _ = chunk.owner_id;
            let _ = chunk.checksum;
        }
    }

    #[test]
    fn test_custom_serialization() {
        let blob = CustomSerBlob {
            field1: "hello".to_string(),
            field2: 123,
        };

        let chunks: Vec<_> = blob.clone().into_chunks(ChunkSize::Size(64)).collect();
        assert!(!chunks.is_empty());

        let reconstructed = CustomSerBlob::try_from_chunks(chunks.into_iter(), ChunkSize::Size(64))
            .expect("Failed to reconstruct with custom deserializer");
        assert_eq!(blob, reconstructed);
    }

    #[test]
    fn test_missing_chunks_error() {
        let empty_chunks: Vec<SimpleBlobChunk> = vec![];

        let result = SimpleBlob::try_from_chunks(empty_chunks.into_iter(), ChunkSize::Size(64));
        assert!(result.is_err(), "Expected error for empty chunks");

        match result {
            Err(rewrite::results::NetabaseError::BlobReconstruction(
                rewrite::results::BlobReconstructionError::MissingChunks,
            )) => {}
            _ => panic!("Expected MissingChunks error"),
        }
    }

    #[test]
    fn test_non_contiguous_chunks_error() {
        let blob = SimpleBlob {
            id: 1,
            name: "test".to_string(),
            data: vec![1, 2, 3],
        };

        let mut chunks: Vec<_> = blob.into_chunks(ChunkSize::Size(64)).collect();

        if chunks.len() > 1 {
            chunks.remove(1);

            let result = SimpleBlob::try_from_chunks(chunks.into_iter(), ChunkSize::Size(64));
            assert!(result.is_err(), "Expected error for non-contiguous chunks");
        }
    }

    #[test]
    fn test_blob_item_chunk_trait() {
        let blob = SimpleBlob {
            id: 1,
            name: "test".to_string(),
            data: vec![],
        };

        let chunks: Vec<_> = blob.into_chunks(ChunkSize::Size(64)).collect();
        let chunk = &chunks[0];

        let index = chunk.get_index();
        assert_eq!(*index, 0);
    }
}
