pub mod blob;
pub mod key;
pub mod model;

/// Extension trait for Plan types to be used with the proc_macro_flow framework.
/// This is mostly just a marker now as we use TryFrom.
pub trait Plan {
    type Visitor;
}
