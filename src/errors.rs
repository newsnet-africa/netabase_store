use thiserror::Error;

pub type NetabaseResult<T> = Result<T, NetabaseError>;

#[derive(Error, Debug)]
pub enum NetabaseError {
    #[error("Redb General Error: {0}")]
    RedbError(#[from] redb::Error),

    #[error("Redb Transaction Error: {0}")]
    RedbTransactionError(#[from] redb::TransactionError),

    #[error("Redb Storage Error: {0}")]
    RedbStorageError(#[from] redb::StorageError),

    #[error("Redb Database Error: {0}")]
    RedbDatabaseError(#[from] redb::DatabaseError),

    #[error("Redb Table Error: {0}")]
    RedbTableError(#[from] redb::TableError),

    #[error("Redb Commit Error: {0}")]
    RedbCommitError(#[from] redb::CommitError),

    #[error("Redb Compaction Error: {0}")]
    RedbCompactionError(#[from] redb::CompactionError),

    #[error("Redb Savepoint Error: {0}")]
    RedbSavepointError(#[from] redb::SavepointError),

    #[error("Redb Set Durability Error: {0}")]
    RedbSetDurabilityError(#[from] redb::SetDurabilityError),

    #[error("Migration Error: {0}")]
    MigrationError(String),

    #[error("Schema Version Mismatch: expected {expected}, found {found}")]
    SchemaVersionMismatch { expected: u32, found: u32 },

    #[error("Schema Conflict: {0}")]
    SchemaConflict(String),

    #[error("IO Error: {0}")]
    IoError(String),

    #[error("Transaction Error: {0}")]
    TransactionError(String),

    #[error("Definition Not Found: {0}")]
    DefinitionNotFound(String),

    #[error("Unknown Error")]
    Other,
}
