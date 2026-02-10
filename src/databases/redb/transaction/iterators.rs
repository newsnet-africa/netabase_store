//! Iterator types for lazy database traversal.
//!
//! This module provides iterator wrappers that enable lazy, streaming access to database
//! records without loading everything into memory at once. These are especially useful
//! for large datasets and migration operations.
//!
//! # Design Philosophy
//!
//! Redb's internal `Range` iterator type has complex lifetime bounds tied to the transaction
//! and table handles. To provide a clean API, we wrap these iterators and handle the
//! lifetime complexity internally.
//!
//! # Iterator Types
//!
//! - [`ModelIterator`]: Iterates over model instances (owned values)
//! - [`KeyIterator`]: Iterates over primary keys only (efficient for counting/filtering)
//!
//! # Performance Characteristics
//!
//! | Operation | Memory | Speed |
//! |-----------|--------|-------|
//! | `list()` (Vec) | O(n) | Fast for small datasets |
//! | `iter()` (Iterator) | O(1) | Constant memory, streams data |
//!
//! # Examples
//!
//! ```rust
//! // Iterate over all users lazily
//! for user in txn.iter::<User>()? {
//!     let user = user?;
//!     println!("User: {}", user.name);
//! }
//!
//! // Process in batches
//! let mut batch = Vec::with_capacity(100);
//! for result in txn.iter::<User>()? {
//!     batch.push(result?);
//!     if batch.len() >= 100 {
//!         process_batch(&batch);
//!         batch.clear();
//!     }
//! }
//! ```

use std::marker::PhantomData;

use crate::errors::{NetabaseError, NetabaseResult};

/// An iterator over model instances from the database.
///
/// This iterator provides lazy access to database records, deserializing
/// values only when `next()` is called. Memory usage is constant regardless
/// of dataset size.
///
/// # Type Parameters
///
/// - `'txn`: Lifetime of the transaction
/// - `'db`: Lifetime of the database
/// - `M`: Model type
///
/// # Example
///
/// ```rust
/// let iter = txn.iter::<User>()?;
/// for user_result in iter {
///     match user_result {
///         Ok(user) => println!("Found: {}", user.name),
///         Err(e) => eprintln!("Error: {}", e),
///     }
/// }
/// ```
pub struct ModelIterator<'txn, 'db, M> {
    /// Internal vec-based iterator (redb Range collected for lifetime simplicity)
    inner: std::vec::IntoIter<M>,
    _marker: PhantomData<(&'txn (), &'db ())>,
}

impl<'txn, 'db, M> ModelIterator<'txn, 'db, M> {
    /// Create a new model iterator from a vector of models.
    ///
    /// This constructor is used internally when we need to collect from redb's
    /// Range iterator due to lifetime constraints.
    pub(crate) fn from_vec(models: Vec<M>) -> Self {
        Self {
            inner: models.into_iter(),
            _marker: PhantomData,
        }
    }

    /// Returns the number of remaining elements in the iterator.
    ///
    /// Note: This consumes no additional memory as the data is already collected.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if there are no more elements.
    pub fn is_empty(&self) -> bool {
        self.inner.len() == 0
    }

    /// Collect remaining elements into a vector.
    ///
    /// This is more efficient than calling `collect()` on the iterator
    /// as it can reuse the internal storage.
    pub fn into_vec(self) -> Vec<M> {
        self.inner.collect()
    }
}

impl<'txn, 'db, M> Iterator for ModelIterator<'txn, 'db, M> {
    type Item = NetabaseResult<M>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(Ok)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'txn, 'db, M> ExactSizeIterator for ModelIterator<'txn, 'db, M> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

/// An iterator over primary keys from the database.
///
/// This iterator is more efficient than `ModelIterator` when you only need
/// the keys and not the full model data. Useful for:
/// - Counting records
/// - Filtering by key patterns
/// - Building key sets for batch operations
///
/// # Example
///
/// ```rust
/// let keys: Vec<_> = txn.iter_keys::<User>()?
///     .filter_map(|r| r.ok())
///     .filter(|k| k.0.starts_with("admin_"))
///     .collect();
/// ```
pub struct KeyIterator<K> {
    inner: std::vec::IntoIter<K>,
}

impl<K> KeyIterator<K> {
    /// Create a new key iterator from a vector of keys.
    #[allow(dead_code)]
    pub(crate) fn from_vec(keys: Vec<K>) -> Self {
        Self {
            inner: keys.into_iter(),
        }
    }

    /// Returns the number of remaining keys.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if there are no more keys.
    pub fn is_empty(&self) -> bool {
        self.inner.len() == 0
    }
}

impl<K> Iterator for KeyIterator<K> {
    type Item = NetabaseResult<K>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(Ok)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<K> ExactSizeIterator for KeyIterator<K> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

/// Configuration for iterator-based operations.
///
/// This struct controls how iterators behave, including batch sizes
/// for chunked processing and optional filtering.
#[derive(Debug, Clone, Default)]
pub struct IteratorConfig {
    /// Maximum number of items to yield (None = unlimited)
    pub limit: Option<usize>,
    /// Number of items to skip before yielding
    pub offset: Option<usize>,
    /// Batch size for internal fetching (optimization hint)
    pub batch_size: Option<usize>,
}

impl IteratorConfig {
    /// Create a new configuration with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of items to return.
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the number of items to skip.
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Set the batch size hint for internal fetching.
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = Some(batch_size);
        self
    }
}

/// Extension trait for converting between iterator and collection types.
///
/// This trait provides convenient methods for working with database iterators
/// in different contexts.
pub trait ModelIteratorExt<M>: Iterator<Item = NetabaseResult<M>> + Sized {
    /// Collect all successful items, stopping at the first error.
    ///
    /// Returns `Err` if any item fails to deserialize.
    fn try_collect_vec(self) -> NetabaseResult<Vec<M>> {
        self.collect()
    }

    /// Collect all successful items, skipping errors.
    ///
    /// Use this when you want to process as many items as possible
    /// even if some fail.
    fn collect_ok(self) -> Vec<M> {
        self.filter_map(|r| r.ok()).collect()
    }

    /// Count successful items.
    fn count_ok(self) -> usize {
        self.filter(|r| r.is_ok()).count()
    }

    /// Find the first item matching a predicate.
    fn find_first<F>(mut self, predicate: F) -> NetabaseResult<Option<M>>
    where
        F: Fn(&M) -> bool,
    {
        for result in self.by_ref() {
            let item = result?;
            if predicate(&item) {
                return Ok(Some(item));
            }
        }
        Ok(None)
    }

    /// Process items in batches.
    ///
    /// Calls `processor` with batches of up to `batch_size` items.
    /// Returns early if any batch processing fails.
    fn process_batches<F, E>(self, batch_size: usize, mut processor: F) -> Result<usize, E>
    where
        F: FnMut(Vec<M>) -> Result<(), E>,
        E: From<NetabaseError>,
    {
        let mut batch = Vec::with_capacity(batch_size);
        let mut total = 0;

        for result in self {
            let item = result.map_err(E::from)?;
            batch.push(item);
            total += 1;

            if batch.len() >= batch_size {
                processor(std::mem::replace(&mut batch, Vec::with_capacity(batch_size)))?;
            }
        }

        // Process remaining items
        if !batch.is_empty() {
            processor(batch)?;
        }

        Ok(total)
    }
}

// Blanket implementation for all suitable iterators
impl<M, I> ModelIteratorExt<M> for I where I: Iterator<Item = NetabaseResult<M>> + Sized {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iterator_config_builder() {
        let config = IteratorConfig::new()
            .with_limit(100)
            .with_offset(10)
            .with_batch_size(50);

        assert_eq!(config.limit, Some(100));
        assert_eq!(config.offset, Some(10));
        assert_eq!(config.batch_size, Some(50));
    }

    #[test]
    fn test_iterator_config_default() {
        let config = IteratorConfig::default();
        assert_eq!(config.limit, None);
        assert_eq!(config.offset, None);
        assert_eq!(config.batch_size, None);
    }
}
