//! Code generation support for TOML-based schema definitions
//!
//! This module provides the infrastructure for parsing TOML schema definitions
//! and generating corresponding Rust code.

pub mod toml_parser;
pub mod toml_types;
pub mod validator;
pub mod generator;

pub use toml_parser::*;
pub use toml_types::*;
pub use validator::*;
pub use generator::*;