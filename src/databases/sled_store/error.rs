//! Error handling for sled backend
//!
//! This module provides error types specific to the sled backend implementation.
//! Errors are converted to the general `NetabaseError` type for consistent error
//! handling across different backends.

use std::fmt;

/// Sled-specific error wrapper
///
/// This type wraps various error conditions that can occur when using sled
/// as the storage backend. All errors are eventually converted to `NetabaseError`
/// for consistent error handling across the application.
#[derive(Debug)]
pub enum SledError {
    /// Sled database error
    Sled(sled::Error),

    /// Serialization error from bincode
    Encode(bincode::error::EncodeError),

    /// Deserialization error from bincode
    Decode(bincode::error::DecodeError),

    /// Tree not found error
    TreeNotFound(String),

    /// Transaction conflict - retry required
    Conflict,

    /// Custom error with message
    Custom(String),
}

impl fmt::Display for SledError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SledError::Sled(e) => write!(f, "Sled database error: {}", e),
            SledError::Encode(e) => write!(f, "Serialization error: {}", e),
            SledError::Decode(e) => write!(f, "Deserialization error: {}", e),
            SledError::TreeNotFound(name) => write!(f, "Tree not found: {}", name),
            SledError::Conflict => write!(f, "Transaction conflict - retry required"),
            SledError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for SledError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SledError::Sled(e) => Some(e),
            SledError::Encode(e) => Some(e),
            SledError::Decode(e) => Some(e),
            _ => None,
        }
    }
}

impl From<sled::Error> for SledError {
    fn from(err: sled::Error) -> Self {
        SledError::Sled(err)
    }
}

impl From<bincode::error::EncodeError> for SledError {
    fn from(err: bincode::error::EncodeError) -> Self {
        SledError::Encode(err)
    }
}

impl From<bincode::error::DecodeError> for SledError {
    fn from(err: bincode::error::DecodeError) -> Self {
        SledError::Decode(err)
    }
}
