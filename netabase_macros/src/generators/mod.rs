//! Code generation modules for Netabase macros.
//!
//! This module contains the code generators that transform macro inputs
//! into Rust code. Each generator is responsible for a specific aspect
//! of the generated output.
//!
//! # Module Structure
//!
//! - [`model`]: Generates model trait implementations and key enums
//! - [`definition`]: Generates definition enums and trait implementations  
//! - [`repository`]: Generates repository isolation code
//! - [`structure`]: Generates struct modifications and derives
//! - [`global`]: Generates global registration code
//! - [`cli`]: Generates CLI bindings
//! - [`nu_test`]: Generates Nushell test scripts for CLI testing
//!
//! # Code Generation Flow
//!
//! 1. Macros parse input using visitors (see [`crate::visitors`])
//! 2. Visitors extract metadata into structured data
//! 3. Generators use metadata to produce `TokenStream` output
//! 4. Output is combined and returned to the compiler
//!
//! # Adding New Generators
//!
//! Each generator module should:
//! - Define a struct implementing the generation logic
//! - Take visitor output as input
//! - Return `proc_macro2::TokenStream`
//! - Be deterministic (same input = same output)

pub mod cli;
pub mod definition;
pub mod global;
pub mod model;
pub mod nu_test;
pub mod repository;
pub mod structure;
