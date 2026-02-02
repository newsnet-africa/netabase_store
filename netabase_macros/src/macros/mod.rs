//! Procedural macro entry points.
//!
//! This module contains the implementation of all procedural macros exported
//! by the crate. Each submodule handles one macro.
//!
//! # Available Macros
//!
//! ## Core Macros
//!
//! - [`netabase_model`]: `#[derive(NetabaseModel)]` - Model trait derivation
//! - [`netabase_definition`]: `#[netabase_definition(Name)]` - Definition creation
//! - [`netabase_repository`]: `#[netabase_repository(Name)]` - Repository grouping
//! - [`netabase`]: `#[netabase]` - Convenience wrapper combining definition + models
//!
//! ## Supporting Macros
//!
//! - [`netabase_blob_item`]: `#[derive(NetabaseBlobItem)]` - Blob field serialization
//! - [`netabase_libp2p`]: libp2p integration macros
//! - [`netabase_networking`]: P2P networking macros
//! - [`netabase_cli`]: CLI generation (experimental)
//!
//! # Implementation Pattern
//!
//! Each macro follows the same pattern:
//!
//! 1. Parse input tokens with `syn`
//! 2. Extract metadata using a visitor
//! 3. Generate code using a generator
//! 4. Return `TokenStream` to compiler
//!
//! # Error Handling
//!
//! Macros return `syn::Error` for compile-time diagnostics with source locations.

pub mod netabase;
pub mod netabase_blob_item;
pub mod netabase_cli;
pub mod netabase_definition;
pub mod netabase_libp2p;
pub mod netabase_model;
pub mod netabase_networking;
pub mod netabase_repository;