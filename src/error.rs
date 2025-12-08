use thiserror::Error;

pub type NetabaseResult<T> = Result<T, NetabaseError>;

#[derive(Error, Debug)]
pub enum NetabaseError {
    #[error(transparent)]
    RedbError(#[from] RedbError),
    #[error(transparent)]
    DecodeError(#[from] bincode::error::DecodeError),
    #[error(transparent)]
    EncodeError(#[from] bincode::error::EncodeError),
    #[error("{0}")]
    Other(String),
}

#[derive(Error, Debug)]
pub enum RedbError {
    #[error(transparent)]
    DatabaseError(#[from] redb::DatabaseError),
    #[error(transparent)]
    TransactionError(#[from] redb::TransactionError),
    #[error(transparent)]
    TableError(#[from] redb::TableError),
    #[error(transparent)]
    CommitError(#[from] redb::CommitError),
    #[error(transparent)]
    StorageError(#[from] redb::StorageError),
    #[error("Compaction error")]
    CompactionError,
}

macro_rules! impl_from_redb {
    ($($err:ty => $variant:ident),*) => {
        $(
            impl From<$err> for NetabaseError {
                fn from(err: $err) -> Self {
                    NetabaseError::RedbError(RedbError::$variant(err))
                }
            }
        )*
    };
}

impl_from_redb!(
    redb::DatabaseError => DatabaseError,
    redb::TransactionError => TransactionError,
    redb::TableError => TableError,
    redb::CommitError => CommitError,
    redb::StorageError => StorageError
);
