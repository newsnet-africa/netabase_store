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
        let now: NetabaseDateTime = Utc::now();
        let now_chrono = Utc::now();

        // Should be within a reasonable time range
        let diff = (now.timestamp() - now_chrono.timestamp()).abs();
        assert!(diff < 2, "Timestamps should be within 2 seconds");
    }

    #[test]
    fn test_netabase_datetime_trait() {
        let now = DateTime::<Utc>::netabase_now();
        let now_chrono = Utc::now();

        // Should be within a reasonable time range
        let diff = (now.timestamp() - now_chrono.timestamp()).abs();
        assert!(diff < 2, "Timestamps should be within 2 seconds");
    }

    #[test]
    fn test_netabase_datetime_methods() {
        let dt: NetabaseDateTime = Utc::now();

        // Test that we can call DateTime methods directly
        let _timestamp = dt.timestamp();
        let _rfc3339 = dt.to_rfc3339();
    }

    #[test]
    fn test_netabase_datetime_type_alias() {
        // Test that NetabaseDateTime is a proper type alias for DateTime<Utc>
        let dt: NetabaseDateTime = Utc::now();
        let dt2: DateTime<Utc> = dt;

        // Both types should be identical
        assert_eq!(dt.timestamp(), dt2.timestamp());
    }
}
