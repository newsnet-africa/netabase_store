// Blob type definitions for User and Heavy models
// NetabaseBlobItem will be implemented automatically by the model macro
use bincode::{Encode, Decode};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct LargeUserFile {
    pub data: Vec<u8>,
    pub metadata: String,
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct AnotherLargeUserFile(pub Vec<u8>);

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct HeavyAttachment {
    pub mime_type: String,
    pub data: Vec<u8>,
}
