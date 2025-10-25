pub mod databases;
pub mod error;
pub mod traits;

// Re-export netabase_deps for users of the macros
pub use netabase_deps;
pub use netabase_deps::*;

// Re-export macros for convenience
pub use netabase_macros::*;
pub use traits::*;

// Conditional Send + Sync bounds for WASM compatibility
#[cfg(not(target_arch = "wasm32"))]
pub trait MaybeSend: Send {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Send> MaybeSend for T {}

#[cfg(target_arch = "wasm32")]
pub trait MaybeSend {}
#[cfg(target_arch = "wasm32")]
impl<T> MaybeSend for T {}

#[cfg(not(target_arch = "wasm32"))]
pub trait MaybeSync: Sync {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Sync> MaybeSync for T {}

#[cfg(target_arch = "wasm32")]
pub trait MaybeSync {}
#[cfg(target_arch = "wasm32")]
impl<T> MaybeSync for T {}

// Helper trait to bundle all discriminant requirements
pub trait DiscriminantBounds:
    AsRef<str>
    + Clone
    + Copy
    + std::fmt::Debug
    + std::fmt::Display
    + PartialEq
    + Eq
    + std::hash::Hash
    + strum::IntoEnumIterator
    + MaybeSend
    + MaybeSync
    + 'static
    + std::str::FromStr
{
}

// Blanket implementation
impl<T> DiscriminantBounds for T where
    T: AsRef<str>
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + PartialEq
        + Eq
        + std::hash::Hash
        + strum::IntoEnumIterator
        + MaybeSend
        + MaybeSync
        + 'static
        + std::str::FromStr
{
}
