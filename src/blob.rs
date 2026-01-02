use bincode::{Encode, Decode};

pub enum BlobLink<T: NetabaseBlobItem> {
    Complete(T),
    Blobs(Vec<T::Blobs>),
}

pub trait NetabaseBlobItem: Sized + Encode + Decode<()> {
    type Blobs;

    /// Wrap a chunk of data into the specific Blob enum variant
    fn wrap_blob(index: u8, data: Vec<u8>) -> Self::Blobs;

    /// Extract the index and data from a Blob enum variant
    fn unwrap_blob(blob: &Self::Blobs) -> Option<(u8, Vec<u8>)>;

    fn split_into_blobs(&self) -> Vec<Self::Blobs> {
        let serialized = bincode::encode_to_vec(self, bincode::config::standard()).unwrap();
        
        if serialized.is_empty() {
             return Vec::new();
        }

        serialized
            .chunks(60000)
            .enumerate()
            .map(|(i, chunk)| Self::wrap_blob(i as u8, chunk.to_vec()))
            .collect()
    }

    fn reconstruct_from_blobs(blobs: Vec<Self::Blobs>) -> Self {
        if blobs.is_empty() {
             // Handle empty case: try to decode from empty bytes
             // This assumes that empty struct encodes to empty bytes and vice versa
             return bincode::decode_from_slice(&[], bincode::config::standard()).unwrap().0;
        }

        let mut parts: Vec<(u8, Vec<u8>)> = blobs
            .iter()
            .filter_map(|b| Self::unwrap_blob(b))
            .collect();
        parts.sort_by_key(|(i, _)| *i);
        let mut result = Vec::new();
        for (_, part) in parts {
            result.extend(part);
        }
        bincode::decode_from_slice(&result, bincode::config::standard()).unwrap().0
    }
}
