#[derive(Debug, thiserror::Error)]
pub enum NetabaseError {
    #[error("There was a decoding Error")]
    DecodeError(#[from] bincode::error::DecodeError),
    #[error("There was a encode Error")]
    EncodeError(#[from] bincode::error::EncodeError),
    #[error("There was an error with the Sled database")]
    DatabaseError(#[from] sled::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum EncodingDecodingError {
    #[error("There was an error encoding type")]
    Encoding(#[from] bincode::error::EncodeError),
    #[error("There was an error decoding type")]
    Decoding(#[from] bincode::error::DecodeError),
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("There was an error opening the store tree")]
    OpenTreeError,
    #[error("There was an error opening the store DB")]
    OpenDBError,
}
