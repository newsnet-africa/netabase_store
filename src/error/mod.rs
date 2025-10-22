#[derive(Debug, thiserror::Error)]
pub enum NetabaseError {
    #[error("There was a conversion Error")]
    Conversion(#[from] EncodingDecodingError),
    #[cfg(feature = "native")]
    #[error("There was an error with the Sled database")]
    SledDatabaseError(#[from] sled::Error),
    #[error("There was an error with the Redb database")]
    RedbDatabaseError(#[from] redb::DatabaseError),
    #[error("There was an error with the Redb database")]
    RedbCompactionError(#[from] redb::CompactionError),
    #[error("There was an error with the Redb database")]
    RedbCommitError(#[from] redb::CommitError),
    #[error("There was an error with the Redb transaction")]
    RedbTransactionError(#[from] redb::TransactionError),
    #[error("There was an error with the Redb table")]
    RedbTableError(#[from] redb::TableError),
    #[error("There was an error with the Redb storage")]
    RedbStorageError(#[from] redb::StorageError),
    #[error("Storage error: {0}")]
    Storage(String),
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
