//! Query configuration and result types.
//!
//! This module provides types for configuring queries with pagination,
//! fetch options, and result modes. All queries use a builder pattern
//! for convenient configuration.
//!
//! # Query Modes
//!
//! - **Fetch**: Return the actual data (default)
//! - **Count**: Return only the count of matching records
//!
//! # Query Configuration
//!
//! Queries are configured using `QueryConfig` which supports:
//! - Range-based queries (e.g., `0..100`)
//! - Pagination with limit/offset
//! - Reversed iteration order
//! - Custom fetch options
//!
//! # Common Patterns
//!
//! ## Basic Queries
//!
//! Query operations use `QueryConfig` to control pagination and limits.
//! Models are queried via transaction methods:
//!
//! ```rust,no_run
//! use netabase_store::prelude::*;
//! use netabase_store::query::QueryConfig;
//!
//! // Open tables and use model-level query methods
//! let table_defs = User::table_definitions();
//! let tables = txn.open_model_tables(table_defs, None)?;
//!
//! // Fetch entries with pagination
//! let entries = User::list_entries(&tables, CrudOptions::default().with_limit(10))?;
//!
//! // Count entries
//! let count = User::count_entries(&tables)?;
//! ```
//!
//! See [tests/integration_list.rs](../tests/integration_list.rs) for complete examples.
//!
//! ## Pagination
//!
//! ```rust,no_run
//! use netabase_store::query::QueryConfig;
//!
//! // Page 1: items 0-9
//! let page1 = QueryConfig::default()
//!     .with_limit(10)
//!     .with_offset(0);
//!
//! // Page 2: items 10-19
//! let page2 = QueryConfig::default()
//!     .with_limit(10)
//!     .with_offset(10);
//! ```
//!
//! ## Range Queries
//!
//! Range queries allow filtering by primary key ranges:
//!
//! ```rust,no_run
//! use netabase_store::query::QueryConfig;
//!
//! // Fetch items with keys in range
//! let config = QueryConfig::new(0u32..100u32);
//! ```
//!
//! ## Reversed Iteration
//!
//! ```rust,no_run
//! use netabase_store::query::QueryConfig;
//!
//! // Get most recent items first
//! let recent = QueryConfig::default()
//!     .reversed()
//!     .with_limit(10);
//! ```
//!
//! # Performance Considerations
//!
//! - Use `count_only()` when you only need the count
//! - Use `with_limit()` to prevent loading too much data
//! - Range queries are more efficient than full scans
//! - Reversed iteration has same performance as forward

mod config;
mod options;
mod result;

pub use config::QueryConfig;
pub use options::{FetchOptions, Pagination, QueryMode};
pub use result::QueryResult;

#[cfg(test)]
mod tests;
