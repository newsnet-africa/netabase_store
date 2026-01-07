//! Utility functions for serialization and common operations.
//!
//! This module provides helper functions for common tasks, primarily
//! around serialization of types that need special handling.
//!
//! # Modules
//!
//! - `serde_system_time_option`: Serialize/deserialize `Option<SystemTime>` as nanoseconds since UNIX epoch

use serde::{Deserialize, Deserializer, Serializer};
use std::time::SystemTime;

/// Serde helpers for `Option<SystemTime>`.
///
/// SystemTime is serialized as nanoseconds since UNIX epoch for compatibility
/// with postcard binary serialization.
///
/// # Example
///
/// ```rust
/// use serde::{Serialize, Deserialize};
/// use std::time::SystemTime;
///
/// #[derive(Serialize, Deserialize)]
/// struct Event {
///     name: String,
///     #[serde(with = "netabase_store::utils::serde_system_time_option")]
///     timestamp: Option<SystemTime>,
/// }
/// ```
pub mod serde_system_time_option {
    use super::*;

    /// Serialize `Option<SystemTime>` as nanoseconds since UNIX epoch.
    pub fn serialize<S>(value: &Option<SystemTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(time) => {
                let duration = time.duration_since(SystemTime::UNIX_EPOCH).map_err(|e| {
                    serde::ser::Error::custom(format!("SystemTime before UNIX EPOCH: {}", e))
                })?;
                serializer.serialize_some(&duration.as_nanos())
            }
            None => serializer.serialize_none(),
        }
    }

    /// Deserialize `Option<SystemTime>` from nanoseconds since UNIX epoch.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<SystemTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let nanos: Option<u128> = Option::deserialize(deserializer)?;
        match nanos {
            Some(n) => {
                let secs = (n / 1_000_000_000) as u64;
                let subsec = (n % 1_000_000_000) as u32;
                Ok(Some(SystemTime::UNIX_EPOCH + std::time::Duration::new(secs, subsec)))
            }
            None => Ok(None),
        }
    }
}
