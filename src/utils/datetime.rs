//! DateTime utilities for bincode serialization support
//!
//! This module provides a simple type alias for DateTime<Utc> that works
//! with both serde and bincode serialization.

use chrono::{DateTime, Utc};

/// Type alias for DateTime<Utc> used throughout netabase_store
///
/// This is simply chrono::DateTime<Utc>. When used in structs that derive
/// bincode::Encode and bincode::Decode, you should also derive serde traits
/// for maximum compatibility:
///
/// ```rust,ignore
/// #[derive(
///     bincode::Encode,
///     bincode::Decode,
///     serde::Serialize,
///     serde::Deserialize
/// )]
/// struct MyModel {
///     created_at: NetabaseDateTime,
/// }
/// ```
pub type NetabaseDateTime = DateTime<Utc>;

/// Helper trait for creating NetabaseDateTime instances
pub trait NetabaseDateTimeExt {
    /// Create a new datetime with the current UTC time
    fn netabase_now() -> Self;
}

impl NetabaseDateTimeExt for DateTime<Utc> {
    fn netabase_now() -> Self {
        Utc::now()
    }
}

// Re-export chrono for convenience
pub use chrono;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_netabase_datetime_creation() {
        let now = NetabaseDateTime::now();
        let now_chrono = Utc::now();

        // Should be within a reasonable time range
        let diff = (now.timestamp() - now_chrono.timestamp()).abs();
        assert!(diff < 2, "Timestamps should be within 2 seconds");
    }

    #[test]
    fn test_netabase_datetime_conversions() {
        let original = Utc::now();
        let wrapper = NetabaseDateTime::from_datetime(original);
        let converted_back = wrapper.into_inner();

        assert_eq!(original, converted_back);
    }

    #[test]
    fn test_netabase_datetime_deref() {
        let wrapper = NetabaseDateTime::now();

        // Test that we can call DateTime methods directly
        let _timestamp = wrapper.timestamp();
        let _rfc3339 = wrapper.to_rfc3339();
    }

    #[test]
    fn test_netabase_datetime_bincode() {
        let original = NetabaseDateTime::now();

        // Test bincode serialization with serde
        let config = bincode::config::standard();
        let encoded = bincode::encode_to_vec(&original, config).unwrap();
        let decoded: NetabaseDateTime = bincode::decode_from_slice(&encoded, config).unwrap().0;

        assert_eq!(original, decoded);
    }
}
