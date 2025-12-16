// Permission module - contains backend-agnostic permission structures and traits

pub mod traits;
pub mod model;
pub mod definition;

// Re-export commonly used types
pub use traits::{AccessType, NetabaseRelationalPermission, NetabasePermissionTicket, NetabasePermissionRegistry, NetabasePermissionTree};
pub use model::{ModelPermissions, AccessLevel, CrossAccessLevel};
pub use definition::{DefinitionPermissions, ModelAccessLevel};
