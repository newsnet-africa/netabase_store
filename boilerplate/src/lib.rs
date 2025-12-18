pub mod boilerplate_lib;

// Re-export everything from boilerplate_lib to satisfy `crate::Type` usages in submodules
// This mimics the structure where these types were expected to be at crate root (or accessible via crate::)
pub use boilerplate_lib::*;
