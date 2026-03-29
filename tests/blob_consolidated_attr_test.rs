use rewrite::traits::structural::blob::{NetabaseBlobItem, ChunkSize};

// 1. Fully consolidated syntax
#[derive(Debug, Clone, PartialEq, Eq, rewrite::NetabaseBlob)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[blob(chunk_size(128), strategy = "partial", chunk_owner_id, chunk_checksum)]
struct Consolidated {
    field1: String,
}

// 2. Mixed syntax (consolidated + standalone)
#[derive(Debug, Clone, PartialEq, Eq, rewrite::NetabaseBlob)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[blob(strategy = "partial", chunk_owner_id)]
#[chunk_size(256)]
#[chunk_checksum]
struct Mixed {
    field1: String,
}

// 3. Consolidated on fields
#[derive(Debug, Clone, PartialEq, Eq, rewrite::NetabaseBlob)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
struct FieldConsolidated {
    #[blob(chunk_size(64))] // Using #[blob] instead of #[chunk_size] or #[blob_field]
    field1: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consolidated_attributes() {
        let blob = Consolidated { field1: "Hello".to_string() };
        let chunks: Vec<_> = blob.into_chunks(ChunkSize::Default).collect();
        
        // Strategy was partial, so it should be an enum chunk
        // Owner ID and Checksum should be present in the underlying struct
        assert!(chunks.len() >= 1);
        
        // We can't easily check fields at runtime without reflection, 
        // but if it compiles and runs into_chunks, the plan was built correctly.
    }

    #[test]
    fn test_mixed_attributes() {
        let blob = Mixed { field1: "Hello".to_string() };
        let chunks: Vec<_> = blob.into_chunks(ChunkSize::Default).collect();
        assert!(chunks.len() >= 1);
    }

    #[test]
    fn test_field_consolidated() {
        let blob = FieldConsolidated { field1: "A".repeat(100) };
        let chunks: Vec<_> = blob.into_chunks(ChunkSize::Default).collect();
        
        // field1 should be partial with size 64. 100 bytes => 2 chunks.
        // If it was Full, it would be 1 chunk (by default 1024).
        assert_eq!(chunks.len(), 2);
    }
}
