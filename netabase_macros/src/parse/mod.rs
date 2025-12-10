//! Parsing infrastructure for netabase macros
//!
//! This module contains the code for parsing module and model structures
//! using syn's visitor pattern. It provides a clean separation between
//! AST parsing and code generation.
//!
//! # Architecture
//!
//! The parsing phase consists of several layers:
//!
//! 1. **Metadata Structures** (`metadata.rs`) - Data structures that hold
//!    all parsed information
//! 2. **Attribute Parsing** (`attributes.rs`) - Uses darling to parse
//!    field and model attributes
//! 3. **Model Visitor** (`model.rs`) - Visits struct definitions and
//!    extracts model metadata
//! 4. **Module Visitor** (`module.rs`) - Visits module definitions and
//!    orchestrates model parsing
//!
//! # Example Flow
//!
//! ```text
//! User's Code
//!     ↓
//! ModuleVisitor (parse module structure)
//!     ↓
//! ModelVisitor (parse each model)
//!     ↓
//! FieldAttributes (parse field attributes with darling)
//!     ↓
//! ModuleMetadata (complete parsed representation)
//!     ↓
//! Code Generation (next phase)
//! ```

pub mod metadata;
pub mod attributes;
pub mod model;
pub mod module;

// Re-export commonly used types
pub use metadata::{ModuleMetadata, ModelMetadata, FieldMetadata, ErrorCollector, MetadataValidator};
pub use attributes::{ModuleAttributes, ModelAttributes, FieldAttributes};
pub use model::ModelVisitor;
pub use module::ModuleVisitor;
