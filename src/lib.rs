#![feature(associated_type_defaults)]
pub mod results;
pub mod traits;

// Re-export derive macros from rewrite_macros
pub use rewrite_macros::{NetabaseBlob, NetabaseModel, NetabaseKey, NetabaseRepository};

// Re-export flow derive macros from proc_macro_flow
pub use proc_macro_flow::{FlowVisitor, FlowPlan, FlowGenerator};

pub use traits::structural::blob::ChunkSize;
