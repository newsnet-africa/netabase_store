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

    #[error("Permission denied: model {source_model} cannot {operation} on {target_model}")]
    PermissionDenied {
        source_model: String,  // formatted from discriminant
        target_model: String,  // formatted from discriminant
        operation: &'static str,
    },

    #[error("Cross-definition access denied: {source_def} -> {target_def}")]
    CrossDefinitionAccessDenied {
        source_def: String,  // formatted from GlobalKeys
        target_def: String,  // formatted from GlobalKeys
    },

    #[error("Required Permissions not found")]
    Permission,

    #[error("Unknown Error")]
    Other,
}
