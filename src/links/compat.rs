//! Backward compatibility layer for RelationalLink types
//!
//! Since we simplified RelationalLink to only use D and M parameters,
//! this module now simply re-exports the main type.

/// Re-export the main RelationalLink type for backward compatibility
pub use crate::links::RelationalLink;
