//! Definition-level code generation
//!
//! This module generates code at the definition level - structures that
//! encompass all models in a definition.

pub mod definition_enum;
pub mod keys_enum;
pub mod associated_types;
pub mod associated_types_ext;
pub mod backend_extensions;
pub mod tree_manager;
pub mod definition_trait;
pub mod permissions;
pub mod complete;

// Re-exports
pub use definition_enum::*;
pub use keys_enum::*;
pub use associated_types::*;
pub use associated_types_ext::*;
pub use backend_extensions::*;
pub use tree_manager::*;
pub use definition_trait::*;
pub use permissions::*;
pub use complete::*;
