// Blob type definitions for User and Heavy models
use bincode::{Encode, Decode};
use serde::{Serialize, Deserialize};
use netabase_macros::NetabaseBlobItem;

#[derive(NetabaseBlobItem, Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct LargeUserFile {
    pub data: Vec<u8>,
    pub metadata: String,
}

#[derive(NetabaseBlobItem, Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct AnotherLargeUserFile(pub Vec<u8>);

#[derive(NetabaseBlobItem, Debug, Clone, Encode, Decode, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct HeavyBlob {
    pub data: Vec<u8>,
}
