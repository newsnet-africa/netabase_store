/// Backend error trait that all storage backends must implement
///
/// This allows Netabase to work with any backend error type without
/// hardcoding specific backend error types in the core library.

use std::fmt::{Debug, Display};
use std::error::Error;

/// Generic backend error trait
///
/// All backend-specific errors must implement this trait to be compatible
/// with Netabase Store. This provides a type-erased interface for error handling.
pub trait BackendError: Error + Debug + Display + Send + Sync + 'static {
    /// Returns true if this error indicates a missing table/tree
    fn is_table_not_found(&self) -> bool;

    /// Returns true if this error indicates a transaction conflict
    fn is_transaction_conflict(&self) -> bool;

    /// Returns true if this error is retryable
    fn is_retryable(&self) -> bool {
        self.is_transaction_conflict()
    }

    /// Convert to a boxed error for type erasure
    fn into_box(self) -> Box<dyn BackendError>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

/// Type alias for results using backend errors
pub type BackendResult<T> = Result<T, Box<dyn BackendError>>;
