//! AST visitors for extracting metadata from macro inputs.
//!
//! Visitors walk the Rust AST and extract the information needed for code
//! generation. This separation of concerns keeps the macro logic clean
//! and testable.
//!
//! # Module Structure
//!
//! - [`model`]: Extracts model fields, keys, relations, and attributes
//! - [`definition`]: Extracts definition modules and their contained models
//! - [`repository`]: Extracts repository structure and isolation rules
//! - [`global`]: Extracts global registration information
//!
//! # Visitor Pattern
//!
//! Each visitor:
//! 1. Takes a `syn` AST node as input
//! 2. Walks the tree to find relevant information
//! 3. Returns a structured data type with extracted metadata
//!
//! # Example Flow
//!
//! ```text
//! TokenStream -> syn::parse() -> Visitor::visit() -> VisitorOutput -> Generator
//! ```
//!
//! # Error Handling
//!
//! Visitors report errors via `syn::Error` which the macro framework
//! converts to compiler diagnostics with source locations.

pub mod definition;
pub mod global;
pub mod model;
pub mod repository;
