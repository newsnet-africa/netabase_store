//! Per-model code generation
//!
//! This module contains generators for all the boilerplate code needed
//! for each model in a netabase definition.
//!
//! # Generated Code Per Model
//!
//! For a model named `User`:
//! 1. **Primary Key Wrapper**: `UserId(inner_type)`
//! 2. **Secondary Key Wrappers**: `UserEmail(String)`, `UserName(String)`, etc.
//! 3. **Secondary Keys Enum**: `UserSecondaryKeys { Email(UserEmail), Name(UserName) }`
//! 4. **Relational Keys Enum**: `UserRelationalKeys { Posts(Vec<PostId>) }`
//! 5. **Subscription Enum**: `UserSubscriptions { Updates, Premium }`
//! 6. **Value Implementations**: `redb::Value` and `redb::Key` for all wrappers
//! 7. **TryFrom Implementations**: Bincode conversions for Sled backend
//!
//! Each generator is in its own file for maintainability.

pub mod primary_key;
pub mod secondary_keys;
pub mod relational_keys;
pub mod subscription_keys;
pub mod model_trait;
pub mod complete;

// Re-exports
pub use primary_key::*;
pub use secondary_keys::*;
pub use relational_keys::*;
pub use subscription_keys::*;
pub use model_trait::*;
pub use complete::*;
