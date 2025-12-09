/// Backend abstraction layer for Netabase Store
///
/// This module defines traits that any key-value store backend must implement.
/// It provides a clean separation between the Netabase API and specific storage implementations.

pub mod traits;
pub mod error;

pub use traits::{
    BackendKey, BackendValue, BackendStore, BackendReadTransaction, BackendWriteTransaction,
    BackendTable, BackendReadableTable, BackendWritableTable, BoxedIterator,
};
pub use error::BackendError;
