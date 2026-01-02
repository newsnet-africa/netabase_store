// Boilerplate library - now fully powered by macros!
pub mod boilerplate_lib;

// Re-export everything from the macro-based implementation
pub use boilerplate_lib::*;

#[macro_export]
macro_rules! import_netabase_schema {
    ($path:literal) => {
        netabase_helper_macros::infer_netabase_definition!($path);
    };
    ($path:literal, $mod_name:ident) => {
        netabase_helper_macros::infer_netabase_definition!($path, $mod_name);
    };
}
