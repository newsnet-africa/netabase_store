//! Core transaction types and bounds.
//!
//! This module provides the fundamental types and trait bounds used throughout
//! the transaction system. By centralizing these, we reduce code duplication
//! and make the trait bounds more manageable.

mod bounds;
mod types;

pub use bounds::*;
pub use types::*;
