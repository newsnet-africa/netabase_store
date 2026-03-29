use thiserror::Error;

/// A common result type for Netabase operations.
pub type NetabaseResult<T> = std::result::Result<T, NetabaseError>;

/// Core error types for the Netabase system.
#[derive(Debug, Error)]
pub enum NetabaseError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Schema inconsistency: {0}")]
    InconsistentSchema(String),

    #[error("Migration failed: {0}")]
    Migration(#[from] MigrationError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Blob reconstruction error: {0:?}")]
    BlobReconstruction(#[from] BlobReconstructionError),
}

/// Detailed reasons for migration failures.
#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("Version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: u32, found: u32 },

    #[error("Missing model: {0}")]
    MissingModel(String),

    #[error("Missing table: {0}")]
    MissingTable(String),

    #[error("Incompatible version jump: {0}")]
    IncompatibleVersion(String),

    #[error("Custom migration error: {0}")]
    Custom(String),
}
/// Errors that can occur during the reconstruction of a blob item from its chunks.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BlobReconstructionError {
    /// One or more required chunks are missing.
    #[error("One or more required chunks are missing.")]
    MissingChunks,
    /// The provided chunks contain invalid or corrupted data.
    #[error("Invalid chunk data: {0}")]
    InvalidChunkData(String),
    /// A chunk was provided that does not belong to this item.
    #[error("Unexpected chunk provided.")]
    UnexpectedChunk,
}
