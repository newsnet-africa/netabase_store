// Boilerplate library with manual implementations (baseline)
pub mod boilerplate_lib;

// Macro-based library - 1:1 parity with boilerplate_lib
pub mod boilerplate_lib_macros;

// Re-export manual version by default
pub use boilerplate_lib::*;
