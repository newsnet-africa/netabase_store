//! Code generation for netabase macros
//!
//! This module contains all the code generators that transform parsed metadata
//! into actual Rust code. The generation is organized hierarchically:
//!
//! - **model/** - Per-model code generation (wrappers, enums, traits)
//! - **definition/** - Per-definition code generation (coming in Phase 5)
//!
//! # Architecture
//!
//! Code generation is separated into distinct modules, each responsible for
//! generating a specific piece of the boilerplate:
//!
//! 1. Primary key wrappers (`UserId`, `PostId`, etc.)
//! 2. Secondary key wrappers and enums
//! 3. Relational key enums
//! 4. Subscription enums
//! 5. Trait implementations
//! 6. Definition-level structures
//!
//! Each generator takes metadata from the parsing phase and produces
//! `proc_macro2::TokenStream` that can be combined into the final output.

pub mod model;
pub mod definition;
pub mod tree_naming;

// Re-export commonly used generators
pub use model::*;
pub use definition::*;
pub use tree_naming::*;
