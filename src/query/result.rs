//! Query result types.

/// Result of a query operation.
///
/// Can represent a single value, multiple values, or a count.
#[derive(Debug, Clone)]
pub enum QueryResult<T> {
    /// A single optional value.
    Single(Option<T>),
    /// Multiple values.
    Multiple(Vec<T>),
    /// Just a count (no data fetched).
    Count(u64),
}

impl<T> QueryResult<T> {
    /// Convert the result into a vector.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryResult;
    ///
    /// let single = QueryResult::Single(Some(42));
    /// assert_eq!(single.into_vec(), vec![42]);
    ///
    /// let multiple = QueryResult::Multiple(vec![1, 2, 3]);
    /// assert_eq!(multiple.into_vec(), vec![1, 2, 3]);
    ///
    /// let count: QueryResult<i32> = QueryResult::Count(5);
    /// assert_eq!(count.into_vec(), Vec::<i32>::new());
    /// ```
    pub fn into_vec(self) -> Vec<T> {
        match self {
            QueryResult::Multiple(vec) => vec,
            QueryResult::Single(Some(item)) => vec![item],
            _ => Vec::new(),
        }
    }

    /// Get the count if this is a count result.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryResult;
    ///
    /// let count: QueryResult<i32> = QueryResult::Count(42);
    /// assert_eq!(count.count(), Some(42));
    ///
    /// let single = QueryResult::Single(Some(1));
    /// assert_eq!(single.count(), None);
    /// ```
    pub fn count(&self) -> Option<u64> {
        match self {
            QueryResult::Count(c) => Some(*c),
            _ => None,
        }
    }

    /// Check if the result is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryResult;
    ///
    /// let empty: QueryResult<i32> = QueryResult::Single(None);
    /// assert!(empty.is_empty());
    ///
    /// let not_empty = QueryResult::Single(Some(42));
    /// assert!(!not_empty.is_empty());
    ///
    /// let multiple_empty: QueryResult<i32> = QueryResult::Multiple(vec![]);
    /// assert!(multiple_empty.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        match self {
            QueryResult::Single(None) => true,
            QueryResult::Multiple(vec) => vec.is_empty(),
            QueryResult::Count(0) => true,
            _ => false,
        }
    }

    /// Get the number of items in the result.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryResult;
    ///
    /// let single = QueryResult::Single(Some(42));
    /// assert_eq!(single.len(), 1);
    ///
    /// let multiple = QueryResult::Multiple(vec![1, 2, 3]);
    /// assert_eq!(multiple.len(), 3);
    ///
    /// let count: QueryResult<i32> = QueryResult::Count(100);
    /// assert_eq!(count.len(), 100);
    /// ```
    pub fn len(&self) -> usize {
        match self {
            QueryResult::Single(Some(_)) => 1,
            QueryResult::Single(None) => 0,
            QueryResult::Multiple(vec) => vec.len(),
            QueryResult::Count(c) => *c as usize,
        }
    }

    /// Unwrap a single result, panicking if None.
    ///
    /// # Panics
    ///
    /// Panics if the result is not a Single variant or if it contains None.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryResult;
    ///
    /// let result = QueryResult::Single(Some(42));
    /// assert_eq!(result.unwrap_single(), 42);
    /// ```
    pub fn unwrap_single(self) -> T {
        match self {
            QueryResult::Single(Some(val)) => val,
            QueryResult::Single(None) => {
                panic!("called `QueryResult::unwrap_single()` on a `None` value")
            }
            _ => panic!("called `QueryResult::unwrap_single()` on a non-Single variant"),
        }
    }

    /// Unwrap a single result with a custom panic message.
    ///
    /// # Panics
    ///
    /// Panics with the given message if the result is not a Single variant or contains None.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryResult;
    ///
    /// let result = QueryResult::Single(Some(42));
    /// assert_eq!(result.expect_single("should have value"), 42);
    /// ```
    pub fn expect_single(self, msg: &str) -> T {
        match self {
            QueryResult::Single(Some(val)) => val,
            _ => panic!("{}", msg),
        }
    }

    /// Get a reference to the single value if present.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryResult;
    ///
    /// let result = QueryResult::Single(Some(42));
    /// assert_eq!(result.as_single(), Some(&42));
    ///
    /// let empty: QueryResult<i32> = QueryResult::Single(None);
    /// assert_eq!(empty.as_single(), None);
    /// ```
    pub fn as_single(&self) -> Option<&T> {
        match self {
            QueryResult::Single(Some(val)) => Some(val),
            _ => None,
        }
    }

    /// Get a reference to the multiple values if present.
    ///
    /// # Example
    ///
    /// ```
    /// use netabase_store::query::QueryResult;
    ///
    /// let result = QueryResult::Multiple(vec![1, 2, 3]);
    /// assert_eq!(result.as_multiple(), Some(&vec![1, 2, 3]));
    ///
    /// let single = QueryResult::Single(Some(42));
    /// assert_eq!(single.as_multiple(), None);
    /// ```
    pub fn as_multiple(&self) -> Option<&Vec<T>> {
        match self {
            QueryResult::Multiple(vec) => Some(vec),
            _ => None,
        }
    }
}
